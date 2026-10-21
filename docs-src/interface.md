# The interface
> RayRF Studio is a full-GUI desktop app (with CLI capability too). This section covers all key sections of the main window: the title bar, menus, bottom panel, and 5 tabs (Edit, Simulate, S-Parameters, Radiation Pattern, and Field Viewer).

## Title bar

The menu bar provides dropdowns for project saving/loading, settings, and help. It also shows account and license information, as well as dynamic simulation run info.

- **Menus.** `File` (new, open, open recent, welcome panel, save, save as, the `Export` and `Import` submenus, and exit), `Edit` (undo, redo), `Window` (shown only in Live mode, customize the layout), `Help` (about), and `Settings` (usage telemetry, check for updates, project settings, and the `Benchmark` submenu).
- **Simulation status.** During simulation runs a `RUNNING` indicator shows on the right-hand side with the elapsed time and throughput in GCell/s. It reads `PREPARING RESULTS` during postprocessing, then `COMPLETE` when a converged run's results are ready. The final solve time covers the FDTD loop and not export, launch, or result loading.
- **Update indicator.** When a new release (which has not been skipped) is available, an `Update <version> available` indication appears at the top, click it to open the update dialog.
- **Account.** Shows your email and plan status when signed in, or `Sign in` when not. Click to view the options: manage account (go to dashboard), refresh plan status (if you changed plans and don't want to restart to see the change), deactivate machine (remove from used seats), and sign out.
- **Project name.** The file label shows the current `.rfsim` file name, or `Untitled` for an unsaved project. The window title carries the same name and a trailing `*` while there are unsaved changes.

## The 5 tabs

The tab bar runs left to right in a fixed order. Edit is the workspace. The four to its right hold results.

- **Edit** is the geometry editor: the stackup and layers, shapes, ports, and the 2D canvas. See [Stackup and layers](docs-stackup-and-layers.html).
- **Simulate** holds the mesh, boundary conditions, and run controls, and shows live progress during a run. See [Auto mode](docs-auto-mode.html).
- **S-Parameters** plots S11 and the Smith chart from the last run. See [S-parameters](docs-s-parameters.html).
- **Radiation Pattern** shows the far-field pattern when NF2FF output is enabled. See [Radiation pattern](docs-radiation-pattern.html).
- **Field Viewer** shows the 3D fields and surface currents when those exports are enabled. See [Field viewer](docs-field-viewer.html).

When a run finishes, a dot prefixes each result tab that received new output, so you can see which views changed without opening them. S-Parameters and Simulate always dot. Radiation Pattern dots only when NF2FF was enabled, Field Viewer only when field or surface-current export was enabled. Opening a dotted tab clears its dot.

## Live mode

The `LIVE` button is at the right end of the tab bar. Live mode re-runs the simulation continuously as you edit the geometry: it docks the result views into one surface and updates them in lockstep with each change, so S-parameters and fields track the edit. Two consecutive failed runs turn it off. The [Running simulations](docs-running.html) page covers it in full.

## Bottom panel

A two-tab panel is below the tabs, split from the main view by a draggable divider.

- **Console** is the timestamped run log: solver output, status lines, and app messages.
- **Notices** lists what the current project has raised, at three levels. An error blocks a run until it is fixed. A warning asks before the run starts, and can be dismissed from its right-click menu, either the one specific warning or all warnings of that type. Dismissals persist with the project, and a dismissal is keyed to the value that triggered it, so the same warning re-surfaces when that value changes materially. A note is context, never a problem: a reference frequency quoted outside the simulated band reads as one. Clicking Run or Preview Mesh completes the list and brings this tab forward once, so nothing needs a dialog of its own. Dismissed warnings are hidden from the list. Check Show dismissed to list them, and Reset Dismissals clears every dismissal so those warnings show again.

A seat-decision banner appears above the tabs when your plan allows fewer machines than are activated. Its button opens the account page to choose which machines to keep.

## Undo and redo

Undo and redo are per tab. The Edit tab keeps up to 50 geometry states. Ctrl+Z and Ctrl+Y apply to whichever tab has focus, and each result tab manages its own history for the edits made in it.

## Project files

A project is one `.rfsim` JSON file that holds the whole design: stackup, geometry, ports, settings, view state, and the last run's results. There is no autosave. Edits set the dirty marker, the `*` in the title, and you save explicitly with Ctrl+S. An older release refuses to open a project saved by a newer one. A project can be configured to write its results to a separate `.rfsimout` sidecar next to the `.rfsim`, so the project can be committed to version control while the results stay out of it. See [File formats](docs-file-formats.html).

## The included examples

The Welcome Panel that opens on launch offers four included examples alongside a blank project and your recent files: a patch antenna, a Vivaldi antenna, a grounded coplanar line, and a bandpass filter. Each opens with its S-parameter results, and running it fills the radiation and surface-current views. They are the fastest way to open a complete design and see every tab populated. The filter and the coplanar line size their geometry from variables and constraints, and the Vivaldi draws its arms with script shapes.
