# Parameter sweeps
> A parameter sweep runs one project across a range of variable values and records every point as its own run in the History rail. Arm it with the Parameter sweep checkbox under the run controls on the Simulate tab.

While the sweep is armed a Sweep Options button appears below the checkbox and opens the definition dialog, and the run button reads Run Simulation Sweep and launches it. An armed sweep with no saved definition blocks the run and outlines the button in red until you configure it, with the reason on hover. A sweep needs a saved project, because its records land beside the project file.

Sweeps use [Run History](docs-run-history.html), which is always active, because every point is stored as a record. The `machshev sweep` commands own their own state directory and record into project history only when you ask, as described below.

## Defining the axes

A sweep spans up to two axes. Each axis binds one project variable (from Variable Studio, see [Variables, constraints, and generators](docs-parametric.html)) to the values it takes. Picking a variable prefills the range from the variable's min and max when they are set.

Each axis has a spacing mode:

- **List:** the values you type, comma or space separated.
- **Linspace:** N points evenly spaced from a start to a stop, both included.
- **Log:** N points spaced logarithmically from a start to a stop, both included and both positive.

Two axes combine as a grid: every combination of the two value lists (the cartesian product). The `machshev sweep` commands also accept a zip combine, which pairs the two axes element by element and needs both the same length.

Drive every port (full S-matrix) sits at the top of the dialog. It is its own kind of sweep: while it is on the variable axes are hidden and do not run, and they keep their values for when it is turned off again. With it on, the sweep runs a single point: the project as authored, one solve per port, assembled into a full S-matrix.

Sweeps use the Simulate tab's run controls: Use CPU backend selects the local backend, and Run remotely with the Run on and Backend lists selects the runner and its device. These selections apply when you launch the sweep.

## The cost estimate

The estimate table refreshes after you stop editing, without running the solver. Estimates for older definitions are discarded. The same memory check runs when you launch the sweep and reports why any points cannot fit.

The estimate detaches one base snapshot from the open project, applies each point's variable overrides to a copy, derives the mesh the run would build, and sizes memory the way the run gate does. It never launches the solver and never changes the open project. The launch snapshots the project at that moment and stores that exact base with the sweep, so resume and retry cannot drift to later edits. The table reports one row per point:

Column || Shows
Point || the point index
Variables || the variable assignment for that point
Cells || the mesh cell count the run would build
Est GB || the estimated run memory on the chosen backend
Valid || OK, OOM, or Error, with the full reason on hover

The dialog does not show a runtime estimate. The CLI JSON preview includes one, as described under Limits below.

Device memory can be unknown, for example when a local GPU probe or a remote runner's cached hardware has no usable capacity. Fit cannot be verified in that state, so every point is refused and the launch refuses with that reason. A remote preview uses only the selected runner's advertised CPU or selected GPU memory. It never substitutes this computer's memory. A remote CUDA run records the selected physical device index with its run provenance.

Refused points are listed and skipped, and the remaining points still run. When every point is refused, the launch refuses.

## Running a sweep

Points run one at a time, on every backend and target. As each point starts it appears in the rail as its own record with a running glyph, then fills in when it completes. The next point launches only when the previous one finishes, so records appear point by point, never batched at the end. Even on CPU the points stay sequential: one solve already spreads across the machine's cores, so a second concurrent solve would contend for those cores rather than add throughput.

A remote sweep sends every point to one runner and runs the points one at a time. Each point exports its own case, submits with a memory estimate so the runner's pre-launch fit check fires, streams telemetry to the rail, and downloads the automatic result tier.

## Sweeps from the CLI

The CLI uses the same definition, cost estimator, sequential engine, result records, resume logic, and retry logic as the desktop app. Preview is a separate read-only command, and run prints that same preview before it launches:

```
machshev sweep preview --project patch.rfsim --axis patchW=15,16,17
machshev sweep run --project patch.rfsim --axis patchW=15,16,17
```

Use a strict JSON or YAML definition for linear or logarithmic spans, two-axis grid or zip combinations, all-port runs, and remote targets:

```
{
  "axes": [
    {"name": "patchW", "mode": "linspace", "start": 15, "stop": 17, "n": 5}
  ],
  "combine": "grid",
  "all_ports": false,
  "backend": "cuda",
  "target": "remote:runner.example:9443",
  "device": "cuda:1"
}
```

Pass it with `--definition sweep.json`. A remote target takes a device the runner exposes: `auto` lands each point on the first free device with GPUs first, `cpu` names its CPU, and `cuda:INDEX` names one card. The preview sizes the points against that device (for `auto` on a CUDA sweep, the smallest card), and the live hardware report must contain it with usable memory or the preview refuses.

By default, CLI sweep state is command-owned and lives under `_sim_result/<project-stem>_cli_sweeps`. It does not populate the desktop History rail. Use `--state-dir DIR` to choose another durable state directory, or use `--record-history` to put the group and its point records in the project's History rail. These choices are exclusive. Resume and retry reopen the same store and exact saved base snapshot:

```
machshev sweep resume --project patch.rfsim --sweep-id s0001
machshev sweep retry --project patch.rfsim --sweep-id s0001 --point-index 2
```

Add the original `--state-dir` to either command when the run used one, or add `--record-history` when the group was written to project History. `--json` emits one JSON event per preview, progress, point, failure, and completion record.

The engine handles interruption point by point:

- **Resume.** A sweep left interrupted (cancelled, or the app closed mid-run) resumes from the rail and skips the points already complete.
- **Retry.** Right-click a failed sweep point and choose Retry this point to re-run only that point. The retry updates that point's durable record and increments its attempt count instead of adding a duplicate point.
- **Cancel.** Cancelling stops launching new points and shows Cancelling while the in-flight point settles. The group becomes interrupted only after that point has stopped.
- A failed point records its reason and the sweep moves on to the next point.

Preparing a resume or retry can derive many meshes, so it runs in the background behind a visible busy overlay. A second resume click is refused while that work is active. If the process ends unexpectedly during a point, reopening the project changes the orphaned running group to interrupted and offers Resume sweep while preserving the point's last attempt.

While a sweep is preparing, running, or cancelling, Machshev Labs refuses Save, Save As, new project, open project, and application close. Normal Run, mesh preview, Live mode, and a second sweep are also unavailable. These gates keep the active runner and history sink bound to one saved project until they settle.

!note A sweep never changes the project you have open. Every point runs from the sweep's saved base snapshot, so later open-project edits do not change a resumed or retried point. A sweep group without a saved base snapshot cannot be resumed.

## Multi-port sweeps and the S-matrix

With Drive every port per point on, each point runs once per port, driving exactly that port each time, then assembles the full S-matrix from the driven-port columns. The assembled matrix is written as a Touchstone file (`.s1p`, `.s2p`, or `.sNp` by port count) into that point's run directory under `_sim_result`. The per-port runs are kept under the one point record.

## Metrics and the sweep plots

Open a sweep's plot from its group row in the rail. The plot reads one figure of merit across the sweep's points:

- min |S11| and its frequency.
- The -10 dB bandwidth around that minimum. It is blank when there is no threshold dip, there are fewer than two samples, or the minimum is ambiguous or non-unique. A threshold region clipped at a sampled band edge reports a lower-bound width.
- The resonant frequency from a detected |S11| minimum, blank when a resonance cannot be established.
- Re(Z) and Im(Z) at a frequency you pick.
- |S21| at a frequency you pick.

The two frequency-dependent readouts are computed on demand from each point's stored S-parameters at the frequency you enter. The plot shape follows the definition: a one-axis sweep draws the metric against the swept variable, a two-axis grid draws a heatmap, and a zipped pair or a ports-only sweep draws the metric against point index. A point that failed or has no value for the metric is drawn as a gap, never a fabricated value, and named in a footnote. Clicking a point or a heatmap cell selects that point's record, the same as clicking it in the rail. Drag to pan, scroll to zoom, and Home refits the plot. The metric and the two frequencies persist per sweep.

## Limits

- The CLI JSON runtime estimate uses the mesh cell count, step cap, saved typical throughput, and port count. Actual runtime depends on throughput and when the run ends.
- Points run one at a time, on every backend and target.
- A remote sweep needs a runner configured in Configure remotes, and all points go to that one runner and the selected backend.
- A sweep never changes the open project.
