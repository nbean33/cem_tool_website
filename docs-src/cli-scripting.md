# Scripting and specs
> A project spec describes authored geometry. A board-import spec describes how Gerber, Excellon, DXF, KiCad, or openEMS and CSXCAD sources enter a project. `machshev project build` and `machshev project import` produce `.rfsim` files, `machshev project export-spec` turns authored geometry back into a project spec, and one JobSpec can drive a whole run. The loop these specs sit inside is [Driving Machshev Labs with an agent](docs-cli-agent-driving.html).

## The project-as-spec model

`machshev schema --what spec` prints the JSON Schema for project specs. The spec has these top-level keys:

- **domain**: the editor rectangle, in one of two modes. With `auto_fit` true (the default) the rectangle is derived from the geometry plus the padding keys `pad_mode` (percent or mm), `pad_percent`, and `pad_mm`, both when the spec is built and again at run time. A rectangle key (`x_mm`, `y_mm`, `w_mm`, `h_mm`) beside `auto_fit` true is refused, since it could never reach the run. With `auto_fit` false the rectangle keys size the box verbatim, `w_mm` and `h_mm` are required, and a padding key beside them is refused. `export-spec` writes whichever set the project uses, so build reproduces the same box. Unknown domain keys are rejected.
- **layers**: the stackup, each conductor, dielectric, or air, top to bottom. An air layer is a free-space spacer that sets the gap between conductors through its `thickness_mm` and emits no material entry, so it renders as air. `z_mm` is recomputed from stack order, so omit it. Unknown layer keys are rejected, so `eps_r` in place of `epsilon_r` is a clear error. A conductor with `conductivity_S_m` 0 is PEC. A positive conductivity activates finite sheet loss and requires a positive `conductor_thickness_um`. That thickness affects electrical loss but does not change stackup Z.
- **shapes**: geometry, each with a `kind` and a `layer`. A hole or cutout takes its span from `start_layer` and `end_layer`, so its layer can be left out, and a layer given with one must name the first layer it cuts. See [CLI reference](docs-cli-reference.html) and `machshev schema --what shapes|ports` for the per-kind parameters.
- **variables, bindings, constraints, generators**: the parametric model. A `bindings` entry maps a param path to an expression over variables. Path prefixes are `primitive:ID:param` (a poly vertex is `primitive:ID:points.INDEX.x`), `layer:NAME:field`, `domain:field`, `fdtd:field`, and `generator:ID:param`. A `domain:w_mm` or `domain:h_mm` binding belongs to an explicit box and is refused with `auto_fit` true, the same as the rectangle keys. Auto-fit padding accepts `domain:padding_percent` and `domain:padding_mm` bindings with values of zero or greater. Polygon vertex bindings use local coordinates relative to the shape origin.
- **settings**: any FDTDSettings field, set verbatim. `machshev schema --what settings` lists every field with its type, default, units, and description. A spec that sets `export_vdb` or `adv_export_vdb` builds with the value read as `field_format` or `adv_field_format` (true is vdb, false is vtk), and a warning on stderr names the change.

Only `layers` and `shapes` are required. Any setting not present takes the project default.

!note Mesh realization resolves the air margin from `air_mode` on every path, so a headless run of a spec-built project realizes the same margins the GUI shows. The modes are `cells`, `mm`, and `auto`. Every mode is resolved at export on every path: cells from `air_cells` x the face's own cell size, auto from `auto_air_*_factor` x the wavelength at f_min. In those two modes the `air_*_mm` fields are display state. Only `air_mode=mm` reads them. `mesh explain` reports the realized value.

`machshev project build --spec FILE --out p.rfsim` writes the project. On a spec error or a failed resolve (an undefined variable, an unknown layer) it prints one line to stderr and exits 2.

`machshev project export-spec --project FILE --out FILE` writes the spec for an authored project, emitting the verbatim params form so a rebuild reproduces it. Projects containing bitmap, DXF, or imported board artwork shapes refuse because a project spec cannot preserve their embedded payload. Keep the original board-import spec as the reproducible source for an imported board.

## Exporting DXF drawings

The DXF exporter reads one saved project and writes its current resolved 2D geometry:

```
machshev project export-dxf --project FILE --out FILE [--replace] [--json]
```

The output is an R2013 DXF with $INSUNITS set to millimeters. Every project layer is represented, including empty layers, and hidden layers are off. The built-in layer 0 keeps its name. Defpoints, invalid names, and collisions receive legal unique names listed in approximated. The rectangular simulation domain is written on Board Outline because the project does not retain a separate shaped fabrication outline. Cutouts are written on Board Outline too, and a cutout that reaches the domain edge notches it.

Variables, bindings, constraints, script shapes, and generator instances are resolved on a project copy before export. The drawing contains the resolved primitive and generator geometry, not the parametric definitions. Bitmap and imported DXF shapes are vectorized from their stored non-transparent pixels with their position, scale, rotation, and flips applied. Ports become visible footprints on Ports. A via writes a filled pad on each conductor layer it spans and its drill circle on Drill. Holes and slots go on Drill PTH when plated and Drill NPTH otherwise.

Filled polygons, discs, rings, and arc bands have a solid HATCH plus an ordinary visible tagged boundary. Native wide polylines represent unclipped script paths and line ports. Machshev Labs suppresses a tagged duplicate boundary on re-import only while its geometry still matches the companion HATCH boundary. If another editor removes the HATCH, the boundary remains and imports as geometry. If the boundary is edited independently, Machshev Labs retains both and reports the difference. The simulation-domain Board Outline is boundary-only.

The command prints three complete report categories: exported, approximated, and omitted. Without `--json`, approximated and omitted entries go to stderr. JSON output uses `machshev-dxf-export-report/v1`:

```
{
  "schema": "machshev-dxf-export-report/v1",
  "project_file": "/work/board.rfsim",
  "output_path": "/work/board.dxf",
  "entity_count": 42,
  "report": {
    "exported": [],
    "approximated": [],
    "omitted": []
  }
}
```

An existing drawing refuses unless `--replace` is set. The `.dxf` suffix is added when absent, and parent directories are created as needed. A completed file passes serialized readback and a structural audit before installation. A destination that appears during a no-replace export is left untouched.

The drawing does not encode stackup order and Z, layer material properties, simulation and mesh settings, editor state, run data, or parametric definitions. Port and via electrical behavior is also absent, a hole or cutout that does not run through the whole board keeps no layer span, and a plated cutout keeps no plating. Those losses are listed in omitted, never hidden. Naming changes and flattened script curves are listed in approximated with their effective name or tolerance. The exporter refuses corrupt embedded pixels, failed resolution, invalid geometry, a hole or cutout the run would refuse, or a structural DXF error. Unsupported, missing-layer, and degenerate shapes also refuse. They are never successful omissions. With domain auto-fit enabled, the Board Outline rectangle is recomputed from resolved geometry and the stored padding settings.

The full mapping is in [Exporting DXF drawings](docs-export-dxf.html).

## Exporting openEMS XML

The openEMS exporter resolves one saved project into a standalone FDTD and CSXCAD XML setup:

```
machshev project export-openems --project FILE --out FILE [--replace] [--json]
```

The XML includes one resolved Gaussian pulse, boundary faces, mesh lines, stackup materials, supported geometry, vias, holes and cutouts, and supported lumped ports. Exactly one port must be active. Each port needs an explicit number, reference impedance, and Boolean active value. Point and rectangular ports need both terminal layers. A planar port needs its one conductor layer, and a present `excitation_mode` must be `planar`. Rectangular or planar ports must align with a Cartesian axis. Continuous-wave setup, de-embedding, unsupported electrical geometry, geometry outside the resolved domain, and fuzzy load-time layer remaps refuse the whole export. Variables, bindings, constraints, and generators are resolved on a copy. Uniform grids use the solver's whole-cell board, gap, absorber, and stackup quantization. Any resulting padded edge, expanded gap, or shifted stackup plane is listed as approximated. Graded grids use the solver's nearest explicit Z line for every stackup edge. Every completed report states that the XML is a translated setup and that the target runtime can produce a different mesh and different results. Parametric definitions, editor state, stored results, Run History, and reference traces are listed as omitted.

An existing output refuses unless `--replace` is set. The `.xml` suffix is added when absent. The writer performs XML readback before installing the destination, so a refusal does not replace an existing file. JSON output uses `machshev-openems-export-report/v1` and carries `entity_count` and the exported, approximated, and omitted lists. The complete mapping is in [Exporting openEMS XML](docs-export-openems.html).

## Board import specs

`machshev schema --what board-import` prints the authoritative JSON Schema. Board-import source paths are relative to the import spec. Gerber accepts a set of artwork, drill, and job files. DXF, KiCad, and openEMS accept one source file. A Gerber job file must be listed in `sources` if it should control roles or stackup. An unlisted job file beside the artwork cannot affect a CLI import.

This Gerber example explicitly accepts detected roles, supplies the one dielectric gap, and adds the settings and port needed for headless validation and simulation:

```
{
  "schema": "machshev-board-import/v1",
  "format": "gerber",
  "sources": [
    "board-F_Cu.gbr",
    "board-B_Cu.gbr",
    "board-Edge_Cuts.gbr",
    "board-PTH.drl"
  ],
  "role_source": "detected",
  "stackup": {
    "source": "explicit",
    "gaps": [
      {"name": "Core", "thickness_mm": 1.6, "epsilon_r": 4.4}
    ]
  },
  "project": {
    "settings": {"freq_min_hz": 1.0e9, "freq_max_hz": 2.0e9},
    "shapes": [
      {"kind": "port", "layer": "Top Copper", "name": "Feed",
       "port_number": 1, "x_mm": 10.0, "y_mm": 6.0, "active": true}
    ]
  }
}
```

`role_source: detected` accepts the effective job-file, X2, filename, DXF layer, KiCad layer, or CSXCAD Z-plane detection that the editor displays before import. An `assignments` entry can replace any detected row. `role_source: explicit` instead requires one entry for every parsed file or layer, including rows that are excluded. Gerber rows use `source`. DXF, KiCad, and openEMS rows use `layer`. Included copper positions must be exactly 1 through the included copper count, and at most one outline can be included.

Gerber copper assignments accept `"polarity": "auto"`, `"positive"`, or `"negative"`. `auto` is the default and reads file and job polarity declarations. An explicit choice overrides them. Negative copper requires an included closed board outline to define the plane boundary. The JSON import report records `polarity` and `effective_polarity` for each copper row, and `span` and `drill_counts` for each drill row.

The copper raster samples the artwork's smallest feature at four pixels. The import report names the feature and its source.

Every drill hit and slot imports as a hole through its span. A non-plated one is cut through the copper and dielectric, and a plated one is a solid conductor through its drill between the copper layers at its ends. A drill file that declares plating per tool (KiCad writes that attribute) keeps each tool's plating unless its assignment sets `"plated"`, which then applies to every hole in the file and is named in the report. Any other drill uses its X2, job-file, or filename plating declaration, and when none is available its assignment needs `"plated": true` or `"plated": false`. A file whose X2 FileFunction contradicts its per-tool plating refuses until `"plated"` is set. A mixed-plating file without per-tool declarations refuses, since the importer cannot tell the holes apart. Plated holes need at least two included copper layers. A Gerber spec can list at most one `.gbrjob` file.

Each included drill row in the JSON import report records `span`, with `start_layer`, `end_layer`, and `through`, and `drill_counts`, with `plated_holes`, `plated_slots`, `non_plated_holes`, `non_plated_slots`, and `routed_islands` (areas that closed routes cut free). The text report prints the same span on the row, and `plated=per_tool` for a row that keeps its per-tool plating.

This unitless DXF example supplies its unit and enumerates every layer:

```
{
  "schema": "machshev-board-import/v1",
  "format": "dxf",
  "sources": ["board.dxf"],
  "drawing_unit": "mm",
  "role_source": "explicit",
  "assignments": [
    {"layer": "TopCopper", "include": true,
     "role": "copper", "position": 1},
    {"layer": "BottomCopper", "include": true,
     "role": "copper", "position": 2},
    {"layer": "BoardOutline", "include": true, "role": "outline"},
    {"layer": "Drills", "include": true, "role": "drill",
     "plated": true},
    {"layer": "Notes", "include": false, "role": "unknown"}
  ],
  "stackup": {
    "source": "explicit",
    "gaps": [
      {"name": "Core", "thickness_mm": 1.6, "epsilon_r": 4.4}
    ]
  }
}
```

Omit `drawing_unit` to use a DXF file's $INSUNITS. A unitless drawing requires `mm` or `inch` in the spec. A declared drawing refuses the field, so its authored unit cannot be silently replaced.

A KiCad import requires an explicit destination. Detected stackup values come from the parsed board:

```
{
  "schema": "machshev-board-import/v1",
  "format": "kicad",
  "sources": ["board.kicad_pcb"],
  "destination": {"mode": "new"},
  "role_source": "detected",
  "stackup": {"source": "detected"}
}
```

An openEMS merge names its saved destination project and makes the frequency choice explicit:

```
{
  "schema": "machshev-board-import/v1",
  "format": "openems",
  "sources": ["setup.xml"],
  "destination": {
    "mode": "current",
    "project": "current.rfsim",
    "bring_frequency_settings": false
  },
  "role_source": "detected",
  "stackup": {"source": "detected"}
}
```

`machshev project import-openems` accepts the openEMS spec as a strict alias for `machshev project import`.

`stackup.source` has format-specific explicit choices:

- `explicit` requires exactly one gap fewer than the included copper count.
- `job` requires a listed Gerber job file with every gap thickness and permittivity present.
- `detected` requires a complete KiCad or CSXCAD material stack.
- `defaults` selects the editor's nominal 1.6 mm FR-4 gap distribution and 4.4 relative permittivity.

Thickness must be finite and greater than zero. Relative permittivity must be finite and at least 1. Values outside the usual ranges print a warning and are kept exactly.

The optional `project` block applies settings and authored shapes after the board geometry is built and before it is saved. Settings use the same value gate as project specs and the GUI. Shapes use the same syntax and port defaults as `project build`, but cannot set an `id`. Imported geometry owns the existing ids. Bitmap and DXF shapes are also refused in this block because `sources` owns the imported artwork. Omitting `project` adds no authored ports or setting overrides. An openEMS new-project import still carries its supported source frequency band. Other formats retain project defaults unless the `project` block supplies settings. Explicit `project.settings` are applied after the openEMS Auto initialization and define the resulting controls. If the block sets either frequency edge, those explicit settings define the final band.

DXF always creates a fresh project and rejects `destination`. Gerber takes an optional `destination` and creates a fresh project without one. KiCad and openEMS require `destination.mode` to be `new` or `current`. Current mode requires a saved `destination.project` and merges geometry into it. openEMS current mode also requires `bring_frequency_settings` to be true or false. Choosing true refuses the import when the XML has no supported positive Gaussian band. The `--out` path always names the result. Writing back to the destination path also requires `--replace`.

```
machshev project import --spec board.import.json --out board.rfsim
machshev project import --spec board.import.json --out board.rfsim --json
machshev validate --project board.rfsim --json
machshev run --project board.rfsim --dry-run
```

The JSON report contains the effective unit, destination, every effective assignment, every gap, applied setting names, authored shape identities, the output path, and the imported, approximated, and skipped report lists. An existing output project or same-stem `.rfsimout` refuses. `--replace` explicitly replaces the project and removes the old result sidecar.

## A worked example

A single rectangular patch over a ground plane on FR4, with one vertical feed port and one variable driving the patch width. Built with `machshev project build` and checked with `machshev validate`.

```
{
  "schema": "machshev-project-spec/v1",
  "domain": {"auto_fit": true},
  "layers": [
    {"name": "Top", "type": "conductor"},
    {"name": "Core", "type": "dielectric", "thickness_mm": 1.6, "epsilon_r": 4.3,
     "loss_tangent": 0.02},
    {"name": "Bottom", "type": "conductor"}
  ],
  "variables": [
    {"name": "patchW", "value": "16", "min": 12, "max": 20, "group": "Geometry"}
  ],
  "shapes": [
    {"kind": "rect", "layer": "Top", "name": "Patch",
     "x_mm": 0, "y_mm": 0, "w_mm": "patchW", "h_mm": 12.3},
    {"kind": "rect", "layer": "Core", "name": "Substrate",
     "x_mm": 0, "y_mm": 0, "w_mm": 22, "h_mm": 22},
    {"kind": "rect", "layer": "Bottom", "name": "Ground",
     "x_mm": 0, "y_mm": 0, "w_mm": 22, "h_mm": 22},
    {"kind": "port", "layer": "Top", "name": "Feed",
     "x_mm": 0, "y_mm": -4, "port_number": 1, "impedance_ohm": 50,
     "active": true, "bottom_layer": "Bottom", "top_layer": "Top"}
  ],
  "settings": {"freq_min_hz": 4.5e9, "freq_max_hz": 7.0e9, "bc_all": "PML"}
}
```

```
machshev project build --spec patch.json --out patch.rfsim
machshev validate --project patch.rfsim
```

A `rect` on a numeric field takes a plain number (`"h_mm": 12.3`) or, when the value is a non-numeric string (`"w_mm": "patchW"`), an expression binding on that field. The feed spans vertically from `bottom_layer` to `top_layer`, so the layer order matters: a port whose two conductors land on the same mesh cell injects nothing and validate reports it. The ground and substrate rects paint the Bottom and Core layers so the dielectric is real, not air.

## Inspecting a project

`machshev project query --project FILE --json` prints a machine-readable readout: every shape (id, kind, layer, name, params), every variable (source value, resolved value, range, group, where-used count), constraints, generators, bindings, settings, the domain, a geometry readout (per-shape center, bounds, anchors, resolved boundary faces, domain clearance), and a last-results summary. Use it to inspect a project you did not build. A shape that cannot be measured carries its own error in the readout, and the project-wide `bounds_xy` is null whenever any shape is unreadable, since an extent over the readable rest would be smaller than the project's.

`machshev project dump-settings --project FILE` prints every setting with its value, default, units, and description, grouped by category. Flags narrow the output:

- `--changed-only` lists only settings that differ from their default.
- `--category NAME` limits to one category (an unknown name exits 2).
- `--advanced` keeps only advanced settings, `--no-advanced` only the others.
- `--json` emits a machine-readable list instead of the grouped text.

## Mutating a project

`machshev project set --project FILE --var name=value [--out FILE]` sets one or more variables (repeatable `--var`), re-resolves bindings and constraints, and saves the project in place or to `--out`. A value may be a number or an expression over other variables. An unknown reference or a bad expression exits 2 with the field message and leaves the geometry unchanged.

Two run-time override forms change values for a single run without editing the project:

- `machshev run --set name=value` overrides a variable, resolved before export.
- `machshev run --setting name=value` sets any FDTDSettings field. It goes through the same value gate as the GUI: an unknown name, a bad type or enum, or a hard-bound violation exits 2 before the run starts, a soft-bound value prints one warning and runs with the value exactly as set. Both flags repeat.

!note A setting fixed in auto mode is derived and overwritten at run time. The run prints the discarded derived value so an override never loses silently.

## Explicit Run History operations

Automation stays history-free unless it opts in. A normal run always writes its `run_manifest.json` but does not add a record to the desktop History rail. Add `--record-history` when a completed run should be recorded. This does not imply `--save-after-run` and does not modify the `.rfsim`.

Two project commands expose the matching desktop actions without changing the project file:

```
machshev project save-point --project patch.rfsim --json
machshev project import-reference --project patch.rfsim \
  --file measured.s1p --json
```

For CSV data, select the representation and columns explicitly. This example imports frequency in MHz and complex impedance, converting it to S11 at 50 ohm:

```
machshev project import-reference --project patch.rfsim --file measured.csv \
  --format csv --csv-kind impedance --freq-column frequency_mhz \
  --value-column resistance --second-column reactance --freq-unit mhz \
  --parameter S11 --z0-ohms 50 --json
```

`db-phase` maps magnitude in dB plus optional phase in degrees. `real-imag` maps real and optional imaginary S values. `impedance` maps resistance and optional reactance, then applies the selected reference impedance. When Auto must infer a frequency unit from value magnitude, the command refuses until `--accept-guessed-unit` is explicit. It also refuses a mapping that skips more rows than it imports unless `--allow-many-skipped` is explicit.

## Parameter sweep definitions

The inline form accepts one or two explicit value lists:

```
machshev sweep preview --project patch.rfsim --axis patchW=15,16,17
machshev sweep run --project patch.rfsim --axis patchW=15,16,17
```

Use `--definition FILE` for the full strict JSON or YAML structure. Each axis has `name`, `mode`, and either `values` for list mode or `start`, `stop`, and `n` for linspace or log. A definition contains at most two axes plus `combine` (grid or zip), `all_ports`, `backend` (cpu or cuda), `target` (local or remote:HOST:PORT), and an optional `device` for a remote target: auto, cpu, or cuda:INDEX, which the runner must expose. Unknown fields refuse.

```
{
  "axes": [
    {"name": "patchW", "mode": "linspace", "start": 15, "stop": 17, "n": 5}
  ],
  "combine": "grid",
  "all_ports": false,
  "backend": "cpu",
  "target": "local"
}
```

Every run prints the mandatory point-by-point cost preview before launch. The default durable state store is `_sim_result/<project-stem>_cli_sweeps`. `--state-dir` selects another. Add `--record-history` instead when the sweep should populate the desktop rail. Resume and retry need the original store choice and the reported sweep id:

```
machshev sweep resume --project patch.rfsim --sweep-id s0001
machshev sweep retry --project patch.rfsim --sweep-id s0001 --point-index 2
```

## Variables and expressions

A numeric field accepts an expression over the project variables: the operators `+ - * / // % **`, parentheses, the trig and math functions, and the constants `pi` and `e`. Trig is in degrees. The full function list is in `machshev schema --what spec` and `machshev guide geometry`.

## JobSpec: one payload per run

`machshev run --job FILE` reads a YAML or JSON JobSpec that carries the project, the overrides, the save behavior, the output directory, and the artifact selection. The job file owns the run request: combining `--job` with another run flag is refused rather than silently discarded, except the flags that layer onto a loaded job (`--backend`, `--dry-run`, `--per-face-bc`, `--set`, `--externalize-results`). An unknown key in the file is refused, so a typo cannot silently revert to a default. Relative `project`, `save_as`, and `out_dir` paths resolve from the directory that contains the job file.

```
project: patch.rfsim
quality: medium
out_dir: _job_out
record_history: true
settings:
  freq_max_hz: 6.5e9
exports:
  sparams: all
  sparams_touchstone: true
  smith: true
```

The `settings` block passes through the same value gate as `--setting`. The `overrides` block sets advanced numeric overrides (cell sizes, air margins, the frequency band, and material fields). The dielectric permittivity key is `epsilon_r`, with `eps_r` accepted as an alias. Setting both to different values is an error, not a last-wins pick. JobSpec booleans require JSON or YAML booleans, enums require a listed value, and numeric controls refuse non-finite or out-of-range values. `quality` and `quality_frac` cannot appear together. `record_history` defaults to false and is the JobSpec equivalent of `run --record-history`. `machshev validate --job FILE` applies the job to a detached project through the same pre-run preparation as run, then reports findings without saving the project. The JobSpec fields:

Field || Type || Default
`project` || Path || required
`save_after_run` || bool || false
`save_as` || Optional[Path] || none
`record_history` || bool || false
`quality` || Optional[str] || none
`quality_frac` || Optional[float] || none
`sim_mode` || Optional[str] || none
`mesh_style` || Optional[str] || none
`vram_budget_mb` || Optional[float] || none
`backend` || Optional[str] || none
`overrides` || Dict[str, Any] || factory
`settings` || Dict[str, Any] || factory
`convergence` || Optional[bool] || none
`perf_mode` || bool || false
`max_steps` || Optional[int] || none
`ringdown_min_steps` || Optional[int] || none
`sparam_stop` || bool || false
`sparam_stop_db` || float || -50
`sparam_stop_periods` || float || 2
`sparam_stop_min_periods` || float || 8
`sparam_stop_consec` || int || 3
`force_disable_surface_currents` || bool || false
`force_disable_nf2ff` || bool || false
`force_disable_save_vtk` || bool || false
`force_skip_geometry_export` || bool || false
`export_port_timeseries` || bool || false
`out_dir` || Optional[Path] || none
`exports` || ExportSpec || factory

The `exports` block selects what is written after the run:

Field || Type || Default
`sparams` || str || all
`sparams_touchstone` || bool || false
`smith` || bool || false
`polar` || bool || false
`polar_freq_mhz` || Optional[float] || none
`rad3d` || bool || false
`rad3d_freq_mhz` || Optional[float] || none
`rad3d_vtk` || bool || false
`rad_pattern_csv` || bool || false
`mesh_preview` || bool || false
`mesh_preview_vtk` || bool || false
`mesh_preview_size` || Optional[int] || none
`surface_currents` || bool || false
`surface_currents_freq_mhz` || Optional[float] || none
`no_3d_renders` || bool || false

## The run manifest

Every run writes `run_manifest.json` into its output directory. It is the machine-readable record of the run, and a failed write is fatal so a missing manifest is never mistaken for success. `status` is one of:

- `ok`: the run completed.
- `preflight_refused`: a pre-flight gate refused before launch.
- `vram_overflow_predicted`: the `--vram-budget-mb` estimate exceeded the budget.
- `error`: a runtime failure.

The manifest records the realized sim params (cell sizes, runtime and ringdown values, frequency band, boundaries), the auto-derived mesh with its reasoning, performance (duration, steps, cells, GCell/s, VRAM, ringdown, convergence history), and the list of artifacts written. The pml/air mm fields inside `sim_params` are the authored request, and the per-face realization lives in `mesh_realization.faces`. `overrides_requested` and `overrides_honored` are both present, and `overrides_honored` reads back the values the exported case actually carried, so a request the run could not realize is visible. The full contract is at `machshev schema --what manifest`.

## Batch running

`machshev run --folder DIR` runs every project in a folder in sequence, `--recursive` to descend into subfolders. One project that fails does not abandon the rest: the batch continues to the next project and exits 1 if any project failed at runtime, 2 if none failed at runtime but at least one was refused, or 0 if all succeeded. Each failure is appended to `OUT_DIR/batch_failures.jsonl` when `--out-dir` is set, or `DIR/batch_failures.jsonl` otherwise. If that log cannot be written, the runner reports the logging error on stderr without changing the project's batch outcome.
