# Driving RayRF with an agent
> The canonical approach for using RayRF with a coding agent such as Claude or Codex to build, edit, and run a RayRF project is the `rayrf` command line. This page provides guidance on best practices, exit code meanings, and interpreting results.

## The short version

Read `rayrf guide` first. Author or edit a project through the CLI verbs, never by editing the `.rfsim` file directly. After every mutation, read back what the program says it now has, and run `validate` and `estimate` before `run`.

## Why not edit the project file

Use CLI commands to edit `.rfsim` projects. The CLI checks value bounds, accepted values, settings used by the active mode, mesh feasibility, port layer assignments, and feature validation status. Direct JSON edits bypass these checks.

The project schema is versioned. RayRF migrates supported older formats on load and rejects invalid documents with an error. A project saved by a newer release refuses to open. After editing a `.rfsim` file directly, run `rayrf validate --project FILE` and review the result.

## Discover the contract

These verbs need no license and change nothing, so an agent can read the whole contract before it authors a line.

```
rayrf guide                      orientation and the topic list
rayrf guide workflow             the end-to-end recipe with commands
rayrf guide pitfalls             the mistakes that silently invalidate a run
rayrf schema                     every file format as JSON Schema
rayrf schema --what spec         the project-spec format you author and build
rayrf schema --what settings     every setting: type, default, units, bounds
rayrf schema --what shapes       shape kinds, parameters, anchors
rayrf schema --what ports        port types and their parameters
rayrf calc microstrip --help     the RF calculators, for sizing before drawing
```

`rayrf guide <topic>` covers fields, workflow, geometry, ports, networks, boundaries, meshing, settings, diagnostics, remote, sweeps, and pitfalls.

## The loop

1. **Author.** Write a project spec and build it, or start from an existing project.

```
rayrf project build --spec patch.json --out patch.rfsim
```

To edit an existing project as text, round-trip it: `project export-spec` writes the spec that rebuilds it, and `project build` turns the edited spec back into a project. A project holding bitmap, DXF, or imported board artwork refuses `export-spec`, because embedded artwork has no spec representation.

2. **Inspect the project.** Read back the geometry, settings, and mesh before running.

```
rayrf project query --project patch.rfsim --json            shapes, variables, settings
rayrf project dump-settings --project patch.rfsim           every setting with its meaning
rayrf mesh explain --project patch.rfsim                    the mesh as it will be realized
rayrf mesh lines --project patch.rfsim --axis z             the exact line positions
```

`dump-settings` tags a setting this configuration does not read as inactive. A project legitimately stores values for the mode it is not in, so an inactive value is not an error, but it is also not what the run will use. Read the tag before concluding a cap or a criterion is in force.

3. **Validate and estimate.** Both commands read the project without launching the solver.

```
rayrf validate --project patch.rfsim --json
rayrf estimate --project patch.rfsim --json
```

`validate` reports blocking errors, advisory warnings, and notes. `estimate` reports the grid, cell count, and timestep the solver will use, plus modeled CPU and GPU memory. Actual memory use varies with allocator behavior.

4. **Run.**

```
rayrf run --project patch.rfsim --quality medium --export-sparams all --save-after-run
rayrf run --project patch.rfsim --dry-run
```

`--dry-run` validates, exports the case, prints the same realization, and stops before the backend. Use it as the last check on anything expensive. Every run writes a manifest recording the settings that produced it.

5. **Read the results.** S-parameter CSV and PNG land under `_sim_result`, and a run with `--save-after-run` (or `--save-as`) embeds the results in the project file. `rayrf reprocess` recomputes the post-processing from the stored time series without re-running the solver.

## What the program tells you, and when

Read stdout for command output and stderr for errors. Exit code 0 means success, 2 indicates a usage, spec, or validation error, and 1 indicates a runtime failure.

You did this || The program says
Passed a value outside its bounds || Refuses, exit 2, naming the field, the value, and the limit.
Passed a value the active mode does not read || Refuses, exit 2, naming the field and how to make it live.
Passed a setting name that no longer exists || Refuses, exit 2, naming the setting that replaced it. Any other unknown name points to `rayrf schema --what settings`.
Stored a value for the other mode || Nothing at validate or run time. It is a draft. `dump-settings` tags it inactive.
Asked for a mesh transition its region cannot hold at the maximum ratio || Refuses before the backend launches, with the axis, the region, the maximum you set and the gentlest ratio that would fit.
Asked for a mesh that cannot be realized || Refuses before the backend launches, with the reason.
Drew a via, hole, or cutout the mesh does not resolve as drawn || Warns and runs. `validate` and `run` print Via not resolved by the mesh, Opening not resolved by the mesh, or Clearance under a cell, with its size in cells and what the mesh builds instead.
Asked for more memory than the machine has || Refuses before the backend launches, with the numbers.

## Pitfalls worth reading before the first run

- **Warnings do not block.** A hard mesh transition or an unusual value produces a warning. The run proceeds with the requested values.
- **Auto and Advanced differ.** Each mode reads its own step-cap field. `auto_max_steps` is the Auto-mode cap. Advanced mode reads `adv_max_steps`, and only when `adv_max_steps_enabled` is true. `run --max-steps N` caps one run in either mode without touching the stored setting. A cap hit before ring-down cuts the time record short, so the S-parameters and radiation are taken over an unfinished signal.
- **Inspect the mesh.** Use `mesh explain` to review the mesh before a run. Use `mesh diff` to compare a project with an exported case or another project.
- **Coordinates are scene millimeters.** Y is up, and a shape's `x_mm`/`y_mm` is its center. The domain rectangle is the exception: its `x_mm` and `y_mm` are the minimum corner, with `w_mm` and `h_mm` running from there. Rotation is clockwise-positive degrees, expression trigonometry is in degrees, spec frequencies are in Hz, and the run flags take MHz.
- **Headless runs need Pro.** `auth`, `schema`, `guide`, and `calc` work without a license, so the whole discovery half of this page is free.
