# Run history
> Run History is always active. The History rail stays closed until you open it with the history button at the right end of the title bar, and the project remembers the state you leave it in, so a rail you closed stays closed when the project reopens. Closing the rail changes only the view: runs keep recording and parameter sweeps stay available.

The History rail appears on the right edge and lists a project's run records and sweep groups, newest first. A record pairs a project snapshot with the results and provenance of one run, so you can overlay a past run on the current one, restore its state, or compare two of them.

## What a record holds

Every record carries three things:

- **The exact project state** exported for the solver: the geometry, stackup, settings, and resolved variable values, minus pure view state. Editing the open project while a run continues does not change the state recorded for that run. Snapshots are content-addressed, so two identical states share one snapshot on disk.
- **The result bundle** the run produced, stored per record. This includes S-parameters, per-port data, and radiation summaries or patterns when present. On-disk field frames and surface-current files stay in the run's `_sim_result` folder.
- **The provenance** of the run: the backend (CPU or CUDA), remote host and selected GPU when applicable, the mesh cell count and step count, the runtime, the resolved variable values, and the timestamps. A record also carries the scalar metrics min |S11| and its frequency, the -10 dB bandwidth, and the resonant frequency.

A rail row shows the record's trace-style chip, its name, one short metadata line, and a glyph only for a state that changes what the record can be used for: running, failed, and the caveat marker a run kept from an early stop carries. Hover the marker for the note, or expand the row to read it first. Clicking a row expands it to the values the run was made with and its cell and step count, then, when the project has changed since, how many primitives and settings moved and whether the mode switched, and last the machine it ran on. The full list of changes is in the compare dialog. A record with no stored results has nothing to overlay, so its eye is disabled and says so.

## Where records come from

Four actions add a record in the desktop app:

- **A finished run.** Every completed local or remote run adds a record named for its run number and time. A run you stop early and choose to keep records the same way, with a stored caveat that it was stopped early and the step it reached. A mesh convergence study you stop and keep records the pass it kept and that pass's mesh.
- **A save point.** The Save Snapshot button in the rail title bar snapshots the current project as a save-point record. It attaches the current results only when they are present and not stale, so a save taken after an edit records the state without pairing it to results it no longer matches.
- **A sweep point.** Each point of a parameter sweep is a record inside its sweep group. See [Parameter sweeps](docs-sweeps.html).
- **A reference import.** A Touchstone or CSV overlay imported on the [S-parameters tab](docs-s-parameters.html), or a Set current trace as reference capture, becomes a reference record shown as an overlay.

A normal `machshev run` always writes its run manifest but does not add a History record. Add `--record-history`, or set `record_history: true` in a JobSpec, when that run should appear in the desktop rail. The flag does not save or modify the project file. These project commands are also opt-in:

```
machshev project save-point --project patch.rfsim
machshev project import-reference --project patch.rfsim --file measured.s1p
```

A CSV reference needs an explicit value representation and column mapping. Use `machshev project import-reference --help` for the mapping flags. The CLI and GUI use the same conversion and validation path, and both copy the imported data into the record so the source file can be removed afterward.

## The history sidecar

The records live in a folder named `<stem>_history` next to the `.rfsim`, so a project saved as `patch.rfsim` keeps its history in `patch_history`. The folder holds the record index, the content-addressed snapshots, the embedded-bitmap payloads, and the per-record results. History is separate from the latest result bundle: the latest result stays inside the project by default, while the multi-run archive stays in this folder. Adding History never changes the `.rfsim` itself.

When moving files in the filesystem, move the `.rfsim` and its history folder together. A rename must rename both stems: renaming `patch.rfsim` to `antenna.rfsim` also requires renaming `patch_history` to `antenna_history`. In-app Save As binds the saved copy to a fresh history folder. It does not copy the source project's history. Saving over another project's file moves that file's history folder aside, renamed `<stem>_history.replaced-<date>-<time>`, so the two projects' runs never mix. The Console names the new folder.

Records exist only for a saved project. An unsaved project has no history store, so record-producing runs, sweeps, and save points require Save As first. A reference import still embeds its data in an unsaved project. History indexes a copy after the project is saved and bound to a history folder.

Deleting a project-backed reference row removes only its History copy and keeps that copy dismissed on later opens. The project-owned reference remains available from the S-parameters tab, where each trace can be hidden or the whole reference can be removed from the project. A reference record with no matching project source is deleted in full. Removing a project reference leaves an existing History copy archived but turns its overlay off. This explicit removal can update an already-existing history folder. It never creates one for an unsaved project.

## Overlaying a record

The eye toggle on a row overlays that record's traces on the S-parameter panels, in the record's own color and dash and labeled by the record name. A sweep group's header carries its own eye, which moves every point of that sweep in one click: it reads as on only when every point that has results is overlaid, and it leaves a point with no results alone. A project-owned reference remains controlled by the S-parameters tab when its indexed History copy is hidden. The magnitude plot draws every overlay trace. The Smith chart and the impedance panel draw only reflection traces, which carry the complex impedance those panels need, so a transmission-only overlay draws on the magnitude plot alone. An overlay is a view: toggling the eye never changes the record or the project.

## Restoring a snapshot

Double-click a record to restore its snapshot into the open project. When the project has unsaved edits, a prompt confirms the replacement first. The restore pushes the current state onto the project undo stack, so Ctrl+Z reverses it from any tab. Pure view preferences, including layer visibility and the close-time field cleanup choice, stay as they were in the open project. Project-owned S-parameter references also stay in place as comparison state. A reference record has no project snapshot and cannot be restored.

## Comparing two records

Select one record, Ctrl+click a second, then choose Compare selected from the right-click menu. The compare window reads A to B: the settings delta, the geometry delta (primitives added, removed, and changed), and a scene render of each snapshot side by side.

## Pinning and clearing

History keeps every record until you delete it. Right-click a record to Pin it, Rename it, restyle it (color and dash), or Delete it. Clear in the rail title bar deletes every run and sweep that is not pinned, not a project reference, and not running, after a prompt that names the count and the size on disk. A sweep with a pinned or running point stays whole. A running point or sweep cannot be deleted until it stops. Delete and Clear are permanent and cannot be undone, unlike a restore, which the undo stack reverses. When the history grows past 1 GB on disk, the rail says so once.

## Imported references become records

Opening a project with embedded reference overlays copies each one into the rail as a reference record, once, while retaining the project-owned source. The conversion is idempotent, so reopening the project adds nothing further. If an imported reference is malformed or the history store cannot accept it, a copyable error gives the exact reason and the project copies remain unchanged for repair and retry. The archived copy retains the selected trace names, so removing the project source does not expose traces that were hidden at migration.

## The rail in live mode

Live mode hides the rail while it owns the window and restores it on exit. You can also close the rail with the X in its title bar or reopen it from the title-bar history button. Its visibility and width persist between sessions.
