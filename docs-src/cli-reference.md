# CLI reference
> List of all verbs and flags for `rayrf` command line use. Workflows and examples are on [CLI overview](docs-cli-overview.html), spec details on [Scripting and specs](docs-cli-scripting.html).

## Local verbs

### `rayrf benchmark`

Run CPU and/or CUDA benchmark pass. The command uses bundled geometry and prints one JSON result to stdout.

Flag || Value || Default || Meaning
`--backend` || cpu / cuda / both || both || Backend ladder to run (default: both available backends).
`--device-index` || DEVICE_INDEX || || Reported GPU index for CUDA cases.
`--quality` || approximate / standard / precise || standard || Step count: approximate 100, standard 1000, precise 10000.
`--single-size`, `--size` || SIZE || || Run only one memory rung: 100MB, 1GB, 3GB, 10GB, 20GB, 40GB, 80GB, 100GB, 150GB, or 200GB.

### `rayrf auth`

Manage the local license cache. Activation binds the key + machine id pair on the RayRF licensing service and persists the result for both the desktop UI and CLI. The auth subcommand does not require Pro.

Flag || Value || Default || Meaning
`--license-key` || LICENSE_KEY || || License key to activate on this machine. Format: the same string you received by email after subscribing. Case is normalized to upper, surrounding whitespace stripped.
`--machine-name` || MACHINE_NAME || || Optional human-readable label for this machine (shown in the account dashboard's seat list). Defaults to the host name.
`--status` || || || Print the cached license state and perform a fresh server check. Default action when no other flag is given.
`--deactivate` || || || Release this machine's seat on the licensing service and clear the local cache.

### `rayrf run`

Run one project or all projects in a folder (sequential). Exit codes on both forms: 0 all projects ran clean, 1 at least one failed at runtime, 2 no runtime failures but at least one was refused by validation (the same fixable-invocation code validate and `--dry-run` use). A folder batch keeps going past a bad project and reports the worst outcome.

Flag || Value || Default || Meaning
`--project`, `-p` || PROJECT || || Path to a single .rfsim/.emflux project file.
`--folder`, `-f` || FOLDER || || Path to a folder containing .rfsim/.emflux projects.
`--job` || JOB_FILE || || Path to a YAML/JSON JobSpec.
`--recursive` || || || When using `--folder`, search recursively.
`--backend` || BACKEND || || Backend to run: cpu, cuda, or a path to a backend executable. Overrides the job file's backend field. If omitted, auto-detected next to the app (CUDA preferred).
`--air-around`, `--air-around-mm` || AIR_AROUND_MM || || Override FDTD air around (mm).
`--air-above`, `--air-above-mm` || AIR_ABOVE_MM || || Override FDTD air above (mm).
`--air-below`, `--air-below-mm` || AIR_BELOW_MM || || Override FDTD air below (mm).
`--dx`, `--dx-mm` || DX_MM || || Override dx (mm).
`--dy`, `--dy-mm` || DY_MM || || Override dy (mm).
`--dz`, `--dz-mm` || DZ_MM || || Override dz (mm).
`--mesh` || uniform / graded || || Mesh-style override for this run. Both choices explicitly select Ratio strategy while retaining any Custom profile as an inactive draft. 'uniform' sets x/y/z multipliers to 1, so dx/dy/dz are literal everywhere. 'graded' uses the saved per-axis Ratio multipliers. Default: use the project's saved strategy.
`--freq-min`, `--freq-min-mhz` || FREQ_MIN_MHZ || || Override frequency band minimum in MHz.
`--freq-max`, `--freq-max-mhz` || FREQ_MAX_MHZ || || Override frequency band maximum in MHz.
`--boundary-condition` || BC_ALL || || Override boundary condition (e.g. PML8).
`--eps-r` || EPS_R || || Override dielectric eps_r.
`--thickness`, `--thickness-mm` || THICKNESS_MM || || Override dielectric thickness (mm).
`--loss-tangent` || LOSS_TANGENT || || Override every dielectric layer's loss tangent (tan d). 0 is lossless. FR4 is typically ~0.02, low-loss substrates ~0.001-0.005.
`--loss-tangent-at` || LOSS_TANGENT_REF_HZ || || Override every dielectric layer's loss reference frequency, in Hz: the frequency its loss tangent is exact at. Required alongside a positive `--loss-tangent` on a stackup that carries no reference. It may lie outside the simulated band.
`--fft-length-multiplier`, `--fft-mult` || FFT_LENGTH_MULTIPLIER || || Force override FFT length multiplier used for S-parameter post-processing (zero-padding). Must be >= 1.
`--quality` || QUALITY || || Auto-mode quality. Name ("low"/"medium"/"high"/"very-high"/"med-high") or float in [0,3]. The effective project mode must be auto.
`--quality-frac` || QUALITY_FRAC || || Auto-mode quality as a slider fraction in [0, 1] (multiplied by 3).
`--sim-mode` || auto / advanced || || Force simulation mode. Use auto with `--quality`. Advanced keeps the explicit settings.
`--convergence`, `--no-convergence` || || || Enable or disable the mesh-convergence study. When enabled the run repeats up to `conv_max_passes` backend passes, refining dx/dy/dz by `conv_mesh_refine_factor` per pass, until the deepest S-parameter dips stabilize within the `conv_*` thresholds. Omit to use the project's saved setting.
`--perf-mode` || || || Performance-benchmark mode: keep mesh/domain at the chosen quality but loosen ringdown to a fixed -20 dB (1e-2 energy ratio) and disable the convergence study so a high-quality CPU run finishes in minutes instead of 30+ minutes.
`--max-steps` || MAX_STEPS || || Hard cap on this run's backend steps in either simulation mode. It does not change the stored `auto_max_steps` or `adv_max_steps` setting. If the cap is hit before ring-down, the time record is truncated before the fields settle, so the S-parameter and radiation results are not converged.
`--ringdown-min-steps` || RINGDOWN_MIN_STEPS || || Optional minimum step before energy ringdown may stop the run, above the built-in floor of two excitation lengths every run keeps. Zero adds nothing above that floor. Omit to use the project value, which defaults to 0.
`--vram-budget-mb` || VRAM_BUDGET_MB || || Post-export memory budget in MB. The CLI reads the realized GPU VRAM or CPU RAM estimate for the selected backend and refuses before launch when it exceeds this value (writes a manifest with status='vram_overflow_predicted'). Applies in auto and advanced modes. Default: no budget check.
`--no-surface-currents` || || || Force-disable surface current capture regardless of the project's saved setting.
`--no-nf2ff` || || || Force-disable NF2FF far-field capture regardless of the project's saved setting. The far-field DFT is VRAM-resident and scales with the Huygens surface x frequency count, so skipping it on an S-parameter-only run avoids large VRAM use for results that are not needed.
`--no-raw-fields` || || || Force-disable all 3D field frame exports regardless of the project's saved setting.
`--export-port-csv` || || || Also write the raw port time-series CSVs (`port_signals_raw.csv` and the per-port voltage CSVs), for debugging and external post-processing. Off by default. The S-parameters are stored from the backend's `sparams.csv` either way.
`--skip-geometry-export` || || || Do not write geometry.vtk/geometry.vdb when NF2FF or 3D field export would normally request it. Useful for very large domains when the mesh is not needed.
`--dry-run` || || || Apply overrides, validate, export the case, and print its exact mesh dimensions, cell count, timestep, and memory, then stop before launching the backend. Exit 2 if validation finds a blocking error.
`--per-face-bc` || FACE=TYPE[:MM] || [] || Set a per-face boundary condition (repeatable). FACE is one of x_lo, x_hi, y_lo, y_hi, z_lo, z_hi. TYPE is PML, PEC, or PMC. An optional :MM sets a fixed gap from the geometry to that face, realized exactly as given and overriding the air margins there. Faces without :MM follow the air margins. Example: `--per-face-bc z_lo=PEC:2.0`. Any per-face setting forces bc_all to Custom. Always give PEC/PMC faces an explicit gap.
`--mesh-preview-size` || MESH_PREVIEW_SIZE || || Pixel size of the mesh-preview PNG written by `--export-mesh`.
`--save-after-run` || || || After the run, save the updated project (in-place unless `--save-as` is given).
`--save-as` || SAVE_AS || || Path to save the project as after the run (implies `--save-after-run`).
`--record-history` || || || Append the completed run to the project's GUI Run History rail. Off by default and does not imply `--save-after-run`.
`--externalize-results` || project / run / off || || Where to store run results when the project is saved. 'project' writes results to a sidecar .rfsimout next to the .rfsim and makes that the project's saved default (so the .rfsim can be committed and the .rfsimout gitignored). 'run' does the same for this run only without changing the saved default. 'off' keeps results inside the .rfsim. Omit to use the project's own setting.
`--out-dir` || OUT_DIR || || Directory to write exports + run_manifest.json. Default: `<project_dir>/_sim_result/<case>`. With `--folder`, each project gets its own subdirectory here so no two runs share a manifest.
`--export-sparams` || all / s11 / s21 / none || all || S-parameter CSV/PNG export selection.
`--export-touchstone` || || || Also write Touchstone (.s1p/.s2p).
`--export-smith` || || || Write Smith chart PNG.
`--export-polar` || || || Write polar radiation pattern PNG (E-plane + H-plane).
`--polar-freq-mhz` || POLAR_FREQ_MHZ || || Target frequency (MHz) for the polar plot. The nearest recorded pattern frequency is used. Default: the centre of the simulated band.
`--export-rad3d` || || || Write 3D radiation pattern PNG (PyVista off-screen).
`--rad3d-freq-mhz` || RAD3D_FREQ_MHZ || || Target frequency (MHz) for the 3D radiation plot. The nearest recorded pattern frequency is used. Default: the centre of the simulated band.
`--export-rad3d-vtk` || || || Also write radiation pattern as a VTK/VTP file.
`--export-rad-pattern` || || || Write a flat radiation-pattern CSV (freq_hz,theta,phi,gain) from NF2FF data. Not a 3D render, so it runs under `--no-3d-renders`.
`--export-mesh` || || || Write mesh preview PNG (PyVista off-screen).
`--export-mesh-vtk` || || || Also copy geometry.vtk into the out dir.
`--export-currents` || || || Write surface-currents heatmap PNG at nearest frequency.
`--currents-freq-mhz` || CURRENTS_FREQ_MHZ || || Target frequency (MHz) for surface currents. The nearest recorded frequency is used. Default: the centre of the simulated band.
`--export-all` || || || Convenience: enable every export artifact.
`--no-3d-renders` || || || Skip PyVista off-screen exports (mesh preview, rad3d).
`--set` || VAR_SET || [] || Variable override name=value applied before the run (repeatable, parametric-studio).
`--setting` || NAME=VALUE || [] || Set any FDTD setting by name (repeatable), gated by the same value gate as the GUI. An unknown name, bad type/enum, or hard-bound value exits 2. A soft-bound value prints one warning and runs as set. See `rayrf schema --what settings` for the field list.

### `rayrf sweep`

Run the same strict sweep definition and sequential engine as the desktop app. By default state is kept under `_sim_result` and does not enter the GUI History rail. `--record-history` explicitly uses the project's History store.

The flags shared by every sweep subcommand:

Flag || Value || Default || Meaning
`--project`, `-p` || PROJECT || || Saved project (.rfsim).
`--account` || ACCOUNT || || Remote runner account. Defaults to saved config.
`--fingerprint` || FINGERPRINT || || Expected remote runner identity fingerprint.
`--json` || || || Emit machine-readable JSON events.

#### `rayrf sweep preview` and `rayrf sweep run`

`preview` validates and prints the mandatory per-point cost preview. `run` previews, persists state, and runs every sweep point. Both take the definition flags. `run` also takes the last three rows.

Flag || Value || Default || Meaning
`--definition` || DEFINITION || || Strict SweepDefinition JSON/YAML file.
`--axis` || NAME=V1,V2,... || [] || Inline list axis (repeat for a second axis).
`--combine` || grid / zip || || Two-axis combination (default: grid).
`--all-ports` || || || Drive every port at each point.
`--backend` || cpu / cuda || || Sweep backend (default: cpu).
`--target` || TARGET || || local or remote:HOST:PORT (default: local).
`--device` || DEVICE || || Runner device for a remote target: auto (default), cpu, or cuda:INDEX.
`--name` (run only) || NAME || || Sweep name (default: project stem plus timestamp).
`--state-dir` (run only) || STATE_DIR || || Durable CLI sweep-state directory. Default: project `_sim_result`.
`--record-history` (run only) || || || Write the sweep into the project's GUI Run History rail.

#### `rayrf sweep resume` and `rayrf sweep retry`

`resume` resumes failed or missing points from an exact saved base. `retry` retries one failed point from an exact saved base.

Flag || Value || Default || Meaning
`--sweep-id` || SWEEP_ID || || Persisted sweep id, such as s0001.
`--state-dir` || STATE_DIR || || CLI sweep-state directory used by the original run.
`--record-history` || || || Read the sweep from the project's GUI Run History rail.
`--point-index` (retry only) || POINT_INDEX || || Zero-based failed point index to run again.

### `rayrf network`

Inspect, preview, or report port matching networks.

Subcommand || Flags || Meaning
`rayrf network show` || `--project`, `--port`, `--json` || The resolved ladder per networked port, with the compiled companion at the project timestep. `--port` limits to one 1-based port number.
`rayrf network preview` || `--project`, `--port`, `--json` || The ladder's continuous and dt-companion two-port response, plus the predicted matched S11 when recorded results exist for the port.
`rayrf network report` || `--project`, `--json` || Recorded per-part loss, totals, and the energy-balance residual from the last run.

`--json` emits a machine-readable `rayrf-network/v1` envelope.

### `rayrf reprocess`

Recompute S-parameter CSV/PNG from embedded port time-series without running the backend. Updates the project file with refreshed embedded outputs.

Flag || Value || Default || Meaning
`--project`, `-p` || PROJECT || || Path to a single .rfsim/.emflux project file.
`--folder`, `-f` || FOLDER || || Path to a folder containing .rfsim/.emflux projects.
`--recursive` || || || When using `--folder`, search recursively.
`--fmin`, `--fmin-mhz` || FMIN_MHZ || || Override fmin (MHz).
`--fmax`, `--fmax-mhz` || FMAX_MHZ || || Override fmax (MHz).
`--fft-length-multiplier`, `--fft-mult` || FFT_LENGTH_MULTIPLIER || || Override FFT length multiplier used for reprocessing (>= 1).

### `rayrf project`

Build, import, inspect, or mutate a project.

#### `rayrf project build`

Build a project from a spec file.

Flag || Value || Meaning
`--spec` || SPEC || Spec file (.json/.yaml).
`--out` || OUT || Output project (.rfsim).

#### `rayrf project import` and `rayrf project import-openems`

`import` builds or extends a project from a strict board-import JSON spec. The spec explicitly selects detected or enumerated row roles, Gerber copper polarity, drill plating, the dielectric source, any undeclared DXF unit, the KiCad or openEMS destination, and optional project settings and authored shapes. Existing project and result-sidecar paths refuse unless `--replace` is set. `import-openems` imports openEMS / CSXCAD XML through the same strict assignment and destination contract, and requires format openems in the spec.

Flag || Value || Meaning
`--spec` || SPEC || Board-import spec JSON (`schema --what board-import`).
`--out` || OUT || Output project (.rfsim).
`--replace` || || Replace an existing output project and its result sidecar.
`--json` || || Emit the effective assignments and import report as JSON.

#### `rayrf project export-dxf`

Export selected project layers and explicitly selected utility geometry to one DXF or one file per selection. Existing targets refuse unless `--replace` is set.

Flag || Value || Default || Meaning
`--project` || PROJECT || || Saved project (.rfsim).
`--out` || OUT || || Output DXF drawing (.dxf).
`--replace` || || || Replace an existing DXF drawing.
`--json` || || || Emit the structured DXF export report as JSON.
`--layer` || LAYER || [] || Project layer name to export, repeatable. Defaults to all layers.
`--separate` || || || Write filename-[layer].dxf for each selected layer/utility.
`--no-outline` || || || Do not export the Board Outline utility layer or the board cutouts.
`--no-ports` || || || Do not export the Ports utility layer.
`--no-drills` || || || Do not export via drills or holes.

#### `rayrf project export-openems`

Export supported project geometry, materials, vias, holes and cutouts, one active axis-aligned lumped port, boundaries, mesh, and a Gaussian band to openEMS XML. Unsupported electrical content refuses the whole export. Text output is one outcome plus actionable warnings, and `--json` retains structured details. An existing XML file refuses unless `--replace` is set.

Flag || Value || Meaning
`--project` || PROJECT || Saved project (.rfsim).
`--out` || OUT || Output openEMS XML file (.xml).
`--replace` || || Replace an existing openEMS XML file.
`--json` || || Emit the complete openEMS export report as JSON.

#### `rayrf project save-point`

Append a restorable project save point to Run History.

Flag || Value || Meaning
`--project` || PROJECT || Saved project (.rfsim).
`--json` || || Emit the created record as JSON.

#### `rayrf project import-reference`

Import Touchstone or explicitly mapped CSV data into Run History.

Flag || Value || Default || Meaning
`--project` || PROJECT || || Saved project (.rfsim).
`--file` || FILE || || Reference data file.
`--format` || auto / touchstone / csv || auto || Input format. Auto recognizes .sNp, .csv, and .txt.
`--name` || NAME || || History record name. Default: input filename.
`--csv-kind` || db-phase / real-imag / impedance || || CSV value representation (required for CSV).
`--freq-column` || HEADER_OR_INDEX || || CSV frequency column name or zero-based index.
`--value-column` || HEADER_OR_INDEX || || CSV magnitude, real-S, or real-Z column.
`--second-column` || HEADER_OR_INDEX || || Optional CSV phase, imaginary-S, or imaginary-Z column.
`--freq-unit` || auto / ghz / mhz / khz / hz || auto || CSV frequency unit. Auto reads the header first.
`--parameter` || PARAMETER || S11 || CSV S-parameter label, such as S11 or S10,2.
`--z0-ohms` || Z0_OHMS || 50 || CSV reference impedance in ohm (default: 50).
`--accept-guessed-unit` || || || Accept Auto's magnitude-based frequency unit when no header names one.
`--allow-many-skipped` || || || Import when more mapped CSV rows are skipped than used.
`--json` || || || Emit the created record and effective mapping as JSON.

#### Other project subcommands

Subcommand || Flags || Meaning
`rayrf project export-spec` || `--project`, `--out` || Export a project back to a spec (.json/.yaml).
`rayrf project query` || `--project`, `--json` || Project readout: shapes, variables, and settings. `--json` emits the full `rayrf-query/v1` JSON document instead of the short human summary.
`rayrf project dump-settings` || `--project`, `--json`, `--changed-only`, `--category`, `--advanced` / `--no-advanced` || Print every FDTD setting with its value, default, units, and description. `--changed-only` shows only settings that differ from their default, `--category` limits to one category (e.g. boundary, mesh, frequency, field_export), and `--advanced` or `--no-advanced` restricts to advanced-only or non-advanced settings.
`rayrf project set` || `--project`, `--var name=value` (repeatable), `--out` || Set variable values (validated). `--out` defaults to in-place.
`rayrf project transfer-to-advanced` || `--project`, `--out` || Materialize auto-derived values as editable Advanced settings. `--out` defaults to in-place.

### `rayrf fields`

Inspect, convert, or view 3D field frames.

Subcommand || Arguments and flags || Meaning
`rayrf fields inspect` || `PATH` || Print the RFF field archive metadata as JSON.
`rayrf fields convert` || `PATH`, `--out OUT`, `--source` (E_mag / H_mag / U / Ex / Ey / Ez / Hx / Hy / Hz, default E_mag) || Write a compressed field archive from a VTK or VDB frame. Existing output files are refused.
`rayrf fields view` || `PATH`, `--photorealistic-geometry` / `--no-photorealistic-geometry` (default on), `--show-meshing` / `--no-show-meshing`, `--max-fps` (default 30) || Open a result directory containing frames.json in the standalone field viewer. Photorealistic uses studio lighting, cream dielectric, rough copper, field illumination, and contact shading with visual copper bevels. Loading and rendering can reduce the playback rate.

### `rayrf schema`

Print the JSON Schema / reference for a file format or contract.

Flag || Value || Default || Meaning
`--what` || fields / spec / board-import / query / settings / shapes / ports / boundaries / mesh / manifest / all || all || Which contract to print.

### `rayrf guide`

Built-in reference: orientation and topics (fields, workflow, geometry, ports, networks, boundaries, meshing, settings, diagnostics, remote, sweeps, pitfalls).

Argument || Meaning
`TOPIC` || Topic to print. Omit it for the orientation and the topic list.

### `rayrf validate`

Validate a project, spec, or job without running the backend. Prints blocking errors and advisory warnings.

Flag || Value || Meaning
`--project`, `-p` || PROJECT || Project (.rfsim) to validate.
`--spec` || SPEC || Project spec (.json/.yaml) to build and validate.
`--job` || JOB || JobSpec (.json/.yaml) to validate.
`--json` || || Emit a machine-readable JSON report.

### `rayrf estimate`

Derive the auto-mode mesh and print cell count, VRAM, boundary sizing, and runtime cost without running the backend.

Flag || Value || Meaning
`--project`, `-p` || PROJECT || Project (.rfsim).
`--quality` || QUALITY || Quality name or 0..3 to estimate at (default: the project's saved quality).
`--json` || || Emit a machine-readable JSON report.

### `rayrf mesh`

Explain, print, or compare the exact realized mesh. Every subcommand takes `--project` / `-p`, `--quality` (a quality name or 0..3 to realize at, omitted uses saved settings), and `--json`.

Subcommand || Extra flags || Meaning
`rayrf mesh explain` || || Explain mesh regions, faces, cost, and timestep ownership. `--quality` reports an auto-derived override even for an advanced project. `--json` emits a `rayrf-mesh/v1` report.
`rayrf mesh lines` || `--axis` x / y / z || Print the exact mesh-line arrays a run will consume. `--axis` limits output to one axis. `--json` emits a `rayrf-mesh-lines/v1` document.
`rayrf mesh diff` || `--against AGAINST` || Compare mesh lines with a project, mesh-lines JSON, files.json, or case dir. Per-line drift up to 1e-12 of the smaller adjacent local pitch is treated as serialization noise. The report keeps the actual maximum delta. Fixed JSON and case lines are unchanged by `--quality`. `--json` emits a `rayrf-mesh-diff/v1` report.

### `rayrf calc`

RF transmission-line and antenna calculators (microstrip, patch with inset feed, quarter-wave, Smith, cpw, ...). Frequencies in GHz, lengths in mm, impedances in ohm.

Flag || Meaning
`calc_kind` || Which calculator to run: microstrip / stripline / cpw / gcpw / diff / embedded / patch / lambda / lambda-diel / qwt / smith.
`--freq` || Frequency (GHz).
`--er` || Substrate relative permittivity.
`--h` || Substrate height (mm).
`--w` || Strip width (mm). Omit it to solve from `--z0`.
`--z0` || Target/reference impedance (ohm). The patch feed and smith use 50 when it is omitted.
`--s` || Gap/spacing (mm) for cpw/gcpw/diff.
`--b` || Plate spacing (mm) for stripline.
`--t` || Copper thickness (mm) for stripline. Omitted, the result uses and prints 0.035 mm (1 oz). 0 is a strip of no thickness.
`--h2` || Cover height (mm) for embedded microstrip.
`--er2` || Cover permittivity for embedded microstrip.
`--tand` || Dielectric loss tangent. Given, the line calculators report dielectric loss. Omitted, the model runs lossless and prints no loss field.
`--z1` || Source impedance (ohm) for qwt.
`--z2` || Load impedance (ohm) for qwt.
`--r` || Resistance (ohm) for smith.
`--x` || Reactance (ohm) for smith.
`--json` || Emit a machine-readable JSON report.

## Remote verbs

The serve and remote verb group runs and drives remote runner boxes. Setup walkthrough: [Runner setup](docs-runner-setup.html).

### `rayrf serve`

Run the remote-runner daemon.

Flag || Value || Meaning
`--state-dir` || STATE_DIR || Runner state directory.
`--name` || NAME || Runner name (default: computer name).
`--owner` || OWNER || Owner account email. Only this account gets control, everyone else connects view-only.
`--host` || HOST || Bind address (default 0.0.0.0).
`--port` || PORT || TCP port (default 8787).
`--slots` || SLOTS || At most this many jobs at once, on top of one job per device (default: one per exposed device).
`--devices` || DEVICES || The devices jobs may run on, as a comma-separated list of cpu and cuda:<index> (for example cpu,cuda:0). Omitted, the runner exposes every device this box can run on: the CPU when the CPU backend is present and each NVIDIA GPU when the CUDA backend answers. A job names one of them or auto, which takes the first free one, GPUs first. `--install` persists it to runner.json.
`--backend-dir` || BACKEND_DIR || Directory containing the backend binary (searched first).
`--advertise` || ADVERTISE || auto (default), explicit host:port, or off.
`--license-key` || LICENSE_KEY || Account license key (or set `RAYRF_LICENSE_KEY`). A Pro account key licenses the runner with no seat consumed: the website signs a machine-bound token per job, and the same key registers with the broker for 'discover by account'. `--install` persists it to runner.json. The key is never sent to a peer.
`--allow-unlicensed` || || Development option for a trusted LAN: run jobs without the backend license gate on a runner with no activated license. Without this flag an unlicensed runner refuses to serve. The mode is reported to every connecting client in the handshake.
`--open-owner` || || Trusted-LAN option: with no `--owner` configured, grant owner control to any connecting peer instead of the default view-only role.
`--install` || || Generate identity + config, print fingerprint.
`--autostart` || || Also emit a run-on-startup service unit.
`--health` || || Print status JSON and exit.
`--servers` || || Print this box's hardware and current load, then exit.
`--window` || || Run the runner with a status window instead of the console. The editor's "Use this computer as a runner" button starts this.

### `rayrf remote`

Drive a remote runner. Every subcommand except `describe-schema` takes the connection flags `--host HOST`, `--port PORT` (default 8787), `--fingerprint FINGERPRINT` (pinned runner SPKI fingerprint, hex), and `--account ACCOUNT`.

Subcommand || Extra arguments and flags || Meaning
`rayrf remote ls-jobs` || || List jobs on a runner.
`rayrf remote status` || `JOB_ID` || Job status.
`rayrf remote info` || || The runner's advertised hardware and version.
`rayrf remote submit` || `--case-dir CASE_DIR`, `--spec SPEC`, `--meta META`, `--device DEVICE`, `--wait` || Submit a pre-exported case dir. `--spec` is simulation runner JSON with only job_kind and device, `--meta` is metadata JSON, and `--device` is the runner device to run on: auto (a free device, GPUs first), cpu, or cuda:<index>. The runner queues the job until that device is free. `--wait` attaches and streams to completion.
`rayrf remote poll` || `JOB_ID`, `--interval` (default 5), `--timeout` || Poll status until terminal or timeout. `--interval` is seconds between status checks, `--timeout` gives up after this many seconds (0 = no limit).
`rayrf remote list-files` || `JOB_ID` || List a finished job's result files.
`rayrf remote fetch` || `JOB_ID`, `--dest DEST`, `--artifacts` auto / ports / pattern / mesh / currents / fields / all || Download results.
`rayrf remote cancel` || `JOB_ID` || Cancel a job.
`rayrf remote describe-schema` || || Print the JobSpec / ExportSpec / telemetry contract (no connection).
