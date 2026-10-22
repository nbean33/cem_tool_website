# Benchmarking
> The performance benchmark measures CPU and GPU solver throughput with bundled, scalable geometry. It does not read or change the open project. The same plan, memory gates, result format, and sharing controls apply to this computer and to configured runners.

## Open a benchmark

Settings then Benchmark lists this computer and every configured runner. Choose Benchmark this computer for a local run, or a named runner for a remote run. In Configure remotes, right-click a runner and choose Benchmark this runner to open the same dialog for that machine.

Benchmarking this computer opens the dialog immediately. The dialog shows its own busy state while it reads local hardware, prepares a plan, or runs a case, and the window behind it stays visible. Opening a remote benchmark refreshes that runner's hardware first, behind a busy overlay, and then opens the same dialog. A target with more than one GPU requires an explicit device choice.

A local benchmark cannot overlap Live mode, a local simulation, a parameter sweep, a convergence study, or mesh preview. A remote benchmark is a durable runner job. Closing its dialog offers to detach and leave it running. The job then remains available to inspect or cancel in Configure remotes.

## Preview the plan

The dialog selects CPU, GPU, or both, then maps quality to a fixed solver step count.

Quality || Solver steps per case
Approximate || 100
Standard || 1,000
Precise || 10,000

Quality changes the number of solver steps. It does not select a different memory rung. The normal plan builds an ascending ladder from these target peak memory sizes: 100MB, 1GB, 3GB, 10GB, 20GB, 40GB, 80GB, 100GB, 150GB, and 200GB. The bundled geometry is scaled until its structural peak estimate is within 5 percent of the rung target. CPU admission applies a 1.20x bound to that estimate before the device-headroom gate.

The planner applies a live headroom gate before admitting each rung. A CPU case may use 85 percent of the currently available system RAM minus 0.5 GiB. A GPU case may use 85 percent of the currently available VRAM minus 0.6 GiB for the driver and GPU context.

The planner refuses a target whose required memory cannot be measured. An ascending ladder stops before the first rung that does not fit that backend.

Run one size replaces the ladder with the selected rung. It still passes the same fit gate and is refused when that rung does not fit. The runner reads live memory again before preparing each case and after exporting the realized mesh. A rung is never resized after preview.

The plan table lists the backend, realized grid estimate, update cells, peak memory, steps, and estimated time for every case, and follows the options as they change. The first plan uses a preliminary rate scaled from the target hardware. A completed result stores a measured rate for that target, which calibrates later plans. Run benchmark stays disabled until the plan matches the current options.

## Run and cancel

Each rung runs the bundled geometry through the normal export and solver path. The timed loop includes the full per-step S-parameter and NF2FF capture workload. Surface-current capture is not enabled, because it is an opt-in extra rather than part of a normal run. The final far-field post-pass is skipped because it is outside steady-state throughput. Every result records the workload it was measured under, and a stored result from a different workload does not calibrate a later plan.

The busy overlay names the active case and step count. Cancel requests a graceful stop from an active solver and prevents later cases from starting. Cancelled, failed, and skipped points remain in the result with their reason, but they do not publish a settled throughput value. A live memory refusal stops later rungs for that backend and is recorded in the warnings.

## Results and settled throughput

The result page names the hardware, plots update cells against settled throughput for each backend, and tables the case, backend, realized grid, completed steps, elapsed time, settled GCell/s, peak process RAM, and peak GPU memory. Drag to pan, scroll to zoom, and Home refits the plot. A case that did not complete states its reason under the table. Its actions are Copy result JSON, Share benchmark with Machshev Labs developers, and Close. Reopen the benchmark to plan a different configuration.

!fig Benchmark results with CPU and GPU curves, hardware summary, measured points, and result actions

Settled throughput uses the solver's native settled window when present. The fallback computes a rate from step telemetry after the warmup interval and before the final hold interval. A point with no valid settled window fails instead of substituting the full-run average.

Copy result JSON copies the complete `machshev-benchmark-result/v1` document. Its top level contains `schema`, `app_version`, `timestamp`, `hardware`, `backend`, `device_index`, `quality`, `single_size`, `completed`, `cancelled`, `error`, `points`, `aggregate`, `annotations`, and `warnings`. Each point keeps its realized grid and cell count, requested and completed steps, elapsed time, settled and full-run rates, throughput source, estimated and observed memory, status, error, and annotations. Copying this JSON does not share it.

## Headless benchmark

`machshev benchmark` runs the same bundled workload without the GUI.

```
machshev benchmark --backend both --quality standard
machshev benchmark --backend cuda --device-index 1 --single-size 10GB
```

`--backend` accepts `cpu`, `cuda`, or `both`. `--quality` accepts `approximate`, `standard`, or `precise`. `--single-size`, also accepted as `--size`, selects one ladder label. Progress and plan warnings go to stderr. Stdout contains one compact `machshev-benchmark-result/v1` JSON document and no progress text. Exit status 0 means complete, 130 means cancelled, 2 means the plan was refused, and 1 means another failure. See the [CLI reference](docs-cli-reference.html) for the complete generated syntax.

## Opt-in sharing and privacy

No benchmark data is sent automatically. Share benchmark with Machshev Labs developers sends the request below, and nothing else, when it is pressed. Copy result JSON puts the full result on the clipboard without sending it, so the request can be read first.

The share request contains exactly these fields:

Field || Contents
`hardware` || CPU model, physical core count, total RAM, operating system, GPU model, and total VRAM or null
`points` || Up to 64 completed points with update cells, settled GCell/s, observed peak RAM, observed peak VRAM, and backend
`curve_image` || The displayed curve as a base64 PNG with media type, width, and height
`app_version` || The Machshev Labs version that produced the result
`timestamp` || The benchmark result time with its UTC offset

The decoded PNG is limited to 1 MiB. The complete request is limited to 1.5 MiB. Incomplete points, full result annotations, warnings, timing estimates, and raw step telemetry are not included.

The JSON body contains no project data, geometry, simulation settings, ports, file names, file paths, host name, account identifier, email address, IP address, or user-agent value. The receiver stores the listed hardware and point fields in a benchmark record and stores a sanitized copy of the curve PNG. It adds a record ID and submission time, but does not store the request IP or user agent. The full result JSON copied by Copy result JSON is separate from this smaller sharing request.
