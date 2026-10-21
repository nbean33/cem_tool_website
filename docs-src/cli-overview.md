# CLI overview
> The `rayrf` command line builds, inspects, validates, runs, and post-processes `.rfsim` projects headless, driving the same solver as the desktop app. This page covers invocation, licensing, the verbs, an end-to-end workflow, and the exit codes and machine-readable output an agent depends on. If you are pointing a coding agent at RayRF, start with [Driving RayRF with an agent](docs-cli-agent-driving.html).

## Invocation

The CLI exports a case and launches the backend for you. You never call the solver directly.

```
rayrf run --project patch.rfsim --quality medium    bare form, latest CLI version
rayrf --cli-v1 run --project patch.rfsim            pin the CLI version
```

The bare `rayrf <verb>` form routes to the latest CLI version. `rayrf --cli-v1 <verb>` pins the version so a script keeps working across releases. From a source checkout the entry point is `python app.py`, so the same commands read `python app.py --cli-v1 run ...`. Every example here uses the packaged `rayrf` form.

On Windows the install folder (`%LOCALAPPDATA%\Programs\RayRF Studio`) ships the console launcher `rayrf.com` next to the windowed `RayRF.exe`. A shell resolves `rayrf` to the launcher, which waits and streams output. On Linux the AppImage itself is the command: `./RayRF-Studio-<version>-x86_64.AppImage run ...`. A launch with no arguments opens the desktop app, and an argument that is not a verb or a project file prints the unknown command and exits 2.

## Licensing

Headless runs need a Pro-tier license, which carries the CLI capability (`CAP_CLI`). The `auth`, `schema`, `guide`, and `calc` verbs work with no license, and `remote describe-schema` does too, so an agent can read the contracts and size a design before activating. Every other verb, including `validate`, `estimate`, and `fields`, refuses without `CAP_CLI` and exits 2 with a short message on stderr that names the reason and the auth command to run.

```
rayrf auth --license-key KEY    activate this machine (shared with desktop sign-in)
rayrf auth --status             show cached state and re-check with the server
rayrf auth --deactivate         release this machine's seat
```

Activation binds the key to this machine and stores the result where the desktop app reads it, so signing in from either surface covers both.

## Verbs

The local verbs plus the serve and remote group. Each `--help` lists its flags, and [CLI reference](docs-cli-reference.html) documents every one.

Verb || Purpose
`rayrf benchmark` || Run CPU and/or CUDA benchmark pass.
`rayrf auth` || Manage the local license cache.
`rayrf run` || Run one project or all projects in a folder (sequential).
`rayrf sweep` || Run the same strict sweep definition and sequential engine as the desktop app.
`rayrf network` || Inspect, preview, or report port matching networks.
`rayrf reprocess` || Recompute S-parameter CSV/PNG from embedded port time-series without running the backend.
`rayrf project` || Build, import, inspect, or mutate a project.
`rayrf fields` || Inspect, convert, or view 3D field frames.
`rayrf schema` || Print the JSON Schema / reference for a file format or contract.
`rayrf guide` || Built-in reference: orientation and topics (fields, workflow, geometry, ports, networks, boundaries, meshing, settings, diagnostics, remote, sweeps, pitfalls).
`rayrf validate` || Validate a project, spec, or job without running the backend.
`rayrf estimate` || Derive the auto-mode mesh and print cell count, VRAM, boundary sizing, and runtime cost without running the backend.
`rayrf mesh` || Explain, print, or compare the exact realized mesh.
`rayrf calc` || RF transmission-line and antenna calculators (microstrip, patch with inset feed, quarter-wave, Smith, cpw, ...).
`rayrf serve` || Run the remote-runner daemon.
`rayrf remote` || Drive a remote runner.

## End-to-end workflow

Size the geometry with a calculator, build or import the project, inspect and check it, run it, then re-post-process without re-running.

```
# 1. Size a 50 ohm microstrip feed on an FR4 stackup.
rayrf calc microstrip --freq 3.0 --er 4.4 --h 1.6 --tand 0.02 --z0 50 --json

# 2. Build the project from a spec (schema --what spec is the format).
rayrf project build --spec patch.json --out patch.rfsim

# An imported board uses a board-import spec instead.
rayrf project import --spec board.import.json --out board.rfsim

# 3. Inspect one output. Substitute patch.rfsim for the authored example.
rayrf project query --project board.rfsim
rayrf project dump-settings --project board.rfsim --changed-only

# 4. Validate the project and estimate the mesh and memory.
rayrf validate --project board.rfsim --json
rayrf estimate --project board.rfsim --quality medium

# 5. Run with a quality preset and the exports you want.
rayrf run --project board.rfsim --quality medium \
  --export-sparams all --export-touchstone --save-after-run

# Add --record-history only when this run should enter the desktop History rail.

# 6. Preview and run a repeatable parameter sweep.
rayrf sweep preview --project board.rfsim --axis traceW=1.0,1.1,1.2
rayrf sweep run --project board.rfsim --axis traceW=1.0,1.1,1.2

# 7. Re-post-process the embedded time series over a new band.
rayrf reprocess --project board.rfsim --fmin 4000 --fmax 7000
```

For this imported-board path, `board.import.json` includes a `project` block with an active port and frequency band. Without that optional block the command still imports the exact board geometry, and validation reports the missing simulation configuration.

`rayrf run --dry-run` applies overrides, validates, exports the case, and prints the exact mesh dimensions, cell count, timestep, and memory without launching the backend. `rayrf guide workflow` prints this recipe offline. A project saved with `--save-after-run` or `--save-as` opens in the desktop app with the S-parameters and radiation pattern tabs populated from the embedded results.

## Quality grammar

The `--quality` flag on `run` and `estimate` takes a name, a midpoint, or a number. The names are `low`, `medium` (alias `med`), `high`, and `very-high`, mapping to 0, 1, 2, and 3. The three midpoints are `low-med` (0.5), `med-high` (1.5), and `high-vhigh` (2.5), and any decimal in [0, 3] sets the value directly. `run --quality-frac` instead takes a slider fraction in [0, 1], multiplied by 3. A value outside the range is refused, never clamped.

## Exit codes

Scripts branch on the code. The stderr message is short and names the cause. `RAYRF_CLI_TRACEBACK=1` adds the full traceback.

Code || Meaning
0 || Success.
2 || Usage, spec, validation, or license error the caller can fix.
1 || The run started but failed at runtime.
130 || Interrupted (Ctrl+C).

A missing or sub-Pro license, a bad flag, an unknown `--setting`, or an out-of-range `--quality` all exit 2 before the backend launches.

A solver failure exits 1, and the message carries the solver's own exit code and its reason, for example `Backend exited with code 8` when the fields go non-finite. A run that goes numerically unstable writes no results. When the solver refuses the case input, the message names the geometry, port, or setting it cannot realize.

## Machine-readable output

`--json` emits machine-readable output from `validate`, `estimate`, `calc`, `sweep`, `network`, `mesh`, `project import`, `project import-openems`, `project export-dxf`, `project export-openems`, `project save-point`, `project import-reference`, `project query`, and `project dump-settings`. `fields inspect` and `fields convert` always print JSON, and `rayrf schema --what fields` is its contract. Sweep commands emit one JSON event per preview, progress, point, failure, and completion. The `remote` subcommands print JSON to stdout and human text to stderr, so a caller reads stdout and logs stderr. Every run writes `run_manifest.json` into the out dir with the resolved parameters, the auto-derived mesh and its reasoning, performance figures, and the artifact list. [Scripting and specs](docs-cli-scripting.html) documents the manifest.

## Environment variables

Variable || Effect
`RAYRF_STATE_DIR` || Directory for remote-runner identity and queue state.
`RAYRF_LICENSE_KEY` || License key a runner resolves when no `--license-key` is given.
`RAYRF_CLI_SHOW_TELEM=1` || Echo the backend's per-step telemetry stream to stdout.
`RAYRF_CLI_TRACEBACK=1` || Add the full traceback to the message on a runtime error.

## Driving the CLI from an agent

`rayrf guide` and its topics are bundled with the executable as offline orientation, and `rayrf schema --what all` prints the machine-readable contract for every file format. The [CLI reference](docs-cli-reference.html) lists every verb and flag, [Scripting and specs](docs-cli-scripting.html) covers the spec and JobSpec formats, and the [docs landing page](docs.html) offers the whole reference as a single-file download to load into an agent's context.
