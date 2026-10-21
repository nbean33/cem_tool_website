# Remote running
> Submit a simulation to a machine you own and stream the results back to your desktop. The case runs on the selected runner, including a rented machine, and the results return to your project folder. Setting up the runner box is a separate walkthrough: [Runner setup](docs-runner-setup.html).

Remote running is a Pro feature. Founding accounts are Pro.

## What it is

A runner is a machine you own that accepts RayRF jobs. When you run remotely, the case uploads to the runner, the solver runs there, telemetry streams back live, and the results download to your project folder.

Jobs move over a direct TLS 1.3 link with certificate pinning, straight from your desktop to the runner's address. The website never carries the case files, the results, or the session keys. It brokers discovery metadata only: a runner registers its reachable address and identity fingerprint under your account, and the desktop resolves that to connect directly.

## The Remote runners manager

On the Simulate tab, check Run remotely, then click Configure remotes.... Your account's current runners appear automatically after a verified connection. The list refreshes while the manager is open and removes runners that stop advertising or cannot be reached.

Select a runner to see its jobs. Right-click a runner or job for its actions. The "Use this computer as a runner" button closes the editor and opens the runner window. CPU and available GPUs are enabled by default.

For a direct LAN or fixed-address connection, open Advanced, click Connect by address..., and enter its host and port. Remember this connection saves it. Account discovery does not save a machine list. A current account advertisement takes precedence when its address also appears in a saved connection.

## Submitting a run

On the Simulate tab, check Run remotely and pick a runner from the selector below it. With the checkbox on and no runner selected, the run is blocked with a message rather than falling back to your local machine.

The Backend selector under the runner lists what that runner exposes: its CPU and each GPU it offers, plus Auto, the default, which takes the first free device with GPUs first. A named device queues the job until that device is free rather than switching to another. The list comes from the runner itself on connect, so a runner that exposes only its GPUs, or only its CPU, shows exactly that. Preflight sizes the run against the chosen device's reported memory (for Auto, the engine the runner says an idle auto job lands on), the submitted job carries the device id, and Run History records the device with the completed run. The local Use CPU backend checkbox hides while Run remotely is on, because it describes this machine, not the runner.

Pull Snapshot works on a remote run the way it does on a local one. The runner asks its backend for the snapshot, keeps each completed snapshot as a numbered set, and the desktop fetches that set while the job keeps running, verified file by file. The S-Parameters and Radiation tabs then show the partial results marked with the snapshot step.

On submit, the case uploads to the runner, which queues and runs it. Only the solver-input files move: the geometry description (`files.json`) and its PNG bitmasks. The license token is never sent over the wire. Telemetry streams back to the same progress panel a local run uses, so the ring-down plot and the run status update as the solve advances.

When the run finishes, the small results, the CSV and JSON tables, download automatically into the project's `_sim_result` folder, and the result tabs open as they do for a local run. Heavy artifacts are fetched on demand: the raw field frames, surface currents, and mesh geometry stay on the runner until you ask for them. The Radiation and Field Viewer tabs show a Download banner when a view needs an artifact that the automatic bundle skipped, and the jobs column lets you download any tier with its on-disk size shown.

Because the job lives on the runner's queue, closing the app detaches from it. The run keeps going, and you collect the results later from the manager.

## Managing jobs

Right-click a job in the jobs column to download its results or cancel it while it is queued or running. Right-click a machine to refresh it, benchmark it, or clear its job history. Manually added machines can also be renamed or removed. Clearing keeps a job that is currently running and any job that finished in the last minute.

The runner's queue is durable. Jobs survive a runner restart, and a running job whose backend is interrupted by a restart or power loss is re-queued and run again from the start.

## Identity and trust

Account discovery supplies the current runner identity. The desktop verifies that identity over TLS before listing the runner as ready. Simulation files and results travel directly between the two machines.

Direct connections ask you to confirm the fingerprint on first use. Compare it with Details in the runner window. A different fingerprint requires a new confirmation. It is never accepted from the network alone.

## Roles

The runner grants control to the account that licenses it. A signed-in desktop presents the signed token of its own license when it connects, and a runner licensed by the same account lets that session submit, cancel, and clear jobs. The typed account email is only the name the runner shows for the session. Any other session connects view-only: it can watch jobs and download results, but cannot submit or control. Submitting also needs your own Pro entitlement on the connecting machine. A runner started with `--open-owner` on a trusted LAN is the one case that grants control without a token, and it says so at start.

## From the command line

The `rayrf serve` and `rayrf remote` verbs drive runners headless, for scripted submission and for agents. See the [CLI reference](docs-cli-reference.html) for the full verb list and [Runner setup](docs-runner-setup.html) for the box-side commands.
