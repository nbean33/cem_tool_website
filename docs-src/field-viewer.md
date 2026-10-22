# Field viewer
> The Field Viewer tab shows what a run wrote inside the domain: surface current on a conductor layer in 2D, and the 3D E, H, and energy volumes. Two mode buttons at the top of the toolbar switch between them.

Both views share one toolbar. A slider with prev and next buttons scrubs frames, and the colormap and geometry-overlay controls are on the second row. A corner badge warns when the frames come from an earlier run than the latest one, when the project changed since the run that produced them, or when that run stopped before its ring-down target.

Max FPS beside Play caps playback for E/H fields and time-domain surface currents. It defaults to 30 and accepts any finite positive rate, including fractional values. Loading and rendering can reduce the rate. Each displayed frame starts the next interval, so a slow frame does not cause a burst of skipped or rushed frames. The limit is saved with the project and does not limit camera movement.

## Surface currents

Surface Currents is the default mode. It draws the current-density magnitude |J| on one conductor layer as a heatmap, with a colorbar in A/m and the layer name in the status bar. The conductor outline is overlaid as a white contour.

!fig Surface current magnitude on a conductor layer

Frequency and Time switch the domain. Frequency shows the steady-state current at each DFT point, labeled in GHz, and is the default. Time steps through the recorded time-domain frames, labeled in picoseconds, and Play auto-advances them. Play is hidden in the frequency domain, which is a static sweep. Each domain keeps its own scrub position.

Range sets how the color scale is chosen: Local rescales to the current frame, Global spans every frame in the domain, and Freq-based or Frame-based pins the scale to one reference frequency or frame that you pick. Color Style selects the colormap. Drag to pan, scroll to zoom. Home fits the whole surface-current plot.

To record surface currents, turn on `enable_surface_currents` and name the conductor layer in `surface_current_layer`. The run refuses if that layer is not a conductor. Time-domain frames are written every `surface_current_interval` steps, or at an automatic stride of about ten samples per period at f_max when the auto interval is on. The frequency-domain DFT accumulates every step regardless of the frame interval, over `surface_current_num_freqs` points (default 63).

## E/H fields

E/H Fields renders the 3D field as a volume: a translucent glow over the whole domain with a lit hot core where the field is strongest, drawn over the geometry mesh. It is a time-domain view that plays through the exported frames. It needs a graphics driver with OpenGL 4.3 and compute shaders that reports free GPU memory. Without one, the Field Viewer says what is missing.

!fig Volumetric E-field with the hot core over the geometry

Each selected quantity writes one file per captured frame:

Source || Setting || Unit
E magnitude || `export_e_mag` || V/m
H magnitude || `export_h_mag` || A/m
Total energy density || `export_energy` || J/m^3
Ex, Ey, Ez || `export_ex`, `export_ey`, `export_ez` || V/m
Hx, Hy, Hz || `export_hx`, `export_hy`, `export_hz` || A/m

`export_e_mag` is the default content. H is time-centered to the E level before the magnitude, and energy density combines the aligned E and H at each cell. The Source selector lists the quantities recorded by the run. Components use a symmetric signed color range.

Range sets the color scale: Local for the current frame, Global across all frames, or Frame-based pinned to a reference frame you pick. Global uses the recorded frame statistics or scans the files in the background. The requested frame appears once its range is available. Color Style selects the colormap, Geometry toggles the structure overlay, and the opacity slider sets its transparency. Geometry edges always use 4x multisample anti-aliasing in the E/H viewer. Photorealistic lights matte cream dielectric and rough copper with soft studio lights. Copper highlights move as you orbit, and nearby visible fields cast colored light onto the geometry. Subtle contact shading makes the copper relief easier to see. Field illumination follows the displayed frame, color style, range, and field opacity. The preference affects the E/H display only and is saved with the project. The geometry opacity slider also applies in this mode.

Show meshing draws mesh cell outlines independently of Photorealistic. Enable both for lit materials with mesh outlines, or disable both for material colors without outlines. Material colors keep the dielectric translucent. New projects start with Photorealistic on and Show meshing off in the E/H viewer. Each viewer saves its own pair of choices in the project.

The geometry display follows the recorded conductor and dielectric footprints on the realized mesh. Copper uses its recorded thickness: outer copper extends outward from the substrate, and buried copper occupies its recorded layer. This display does not change the solver mesh or fields.

A result that recorded no layer description takes it from a matching saved Run History snapshot. When no recorded layer description can be verified, the viewer uses the recorded solver-material geometry. Isolated PEC edges remain visible as narrow translucent copper.

The volume uses the recorded grid coordinates, including graded cells. Drag with the left mouse button to orbit with an upright view, with the right or middle button or Shift plus left button to pan, and use the wheel to zoom toward the point under the pointer. Each orbit starts around the closest board surface at the center of the view. That pivot stays fixed throughout the drag. A miss keeps the last valid pivot, starting at the board or field-domain center. Home fits the full scene. The same 3D controls apply in Simulate and Radiation.

The cursor is hidden during a drag and returns to where the drag started. A drag keeps going past the edge of the viewport. Release the button or press Esc to end it. Orbit sensitivity is 0.25 degrees per logical pixel at every viewport size. Frame reads and compression run in the background. While frames load, a status line under the viewport names the step, and Cancel stops loading and keeps the frame on screen. Playback advances after each frame is presented. CPU and GPU caches retain compressed frames within their memory budgets.

## Export cost and cleanup

In Auto mode, Export 3D Fields on the Options card turns field export on and shows the format list beside it. In Advanced mode, Export full fields on the Field Export card turns it on, with the quantities, capture interval, format, and step window under it.

Field format selects Machshev Labs compressed (`rff`), VTK (`vtk`), or OpenVDB (`vdb`). Machshev Labs compressed is the default. It stores signed logarithmic samples in compressed bricks without threshold pruning. VTK stores fp32 samples. OpenVDB can prune samples below `vdb_threshold_frac` times the frame's peak magnitude. OpenVDB needs a uniform mesh, and the run refuses it on a graded one. Two settings bound the volume of data:

- The capture interval, dense for smooth playback or sparse to save disk.
- The step-range window (`field_export_range_enabled` with a start and end step), which drops warm-up frames. The solver still runs every step, only the disk write outside the window is skipped.

[FDTD tips](docs-fdtd-tips.html) covers when the fields are worth exporting. The on-disk frame formats are on [File formats and outputs](docs-file-formats.html).

The viewer reads all three formats. A voxel pruned from an OpenVDB file shows as zero. The archive inspection command reports the error bound and storage size of a Machshev Labs compressed frame.

## Standalone viewer and archive commands

Open a recorded sequence without opening the editor:

```
machshev fields view PATH_TO_RESULT_DIRECTORY
```

The standalone viewer opens with Photorealistic on. Add `--no-photorealistic-geometry` for material colors, `--show-meshing` for mesh cell outlines, or `--max-fps` to change the playback cap (default 30).

The directory must contain `frames.json` and its frame files. Inspect an archive or convert one VTK or OpenVDB quantity to a new archive:

```
machshev fields inspect step_0000.rff
machshev fields convert step_0000.vtk --source E_mag --out converted.rff
machshev schema --what fields
```

Inspection returns JSON under `machshev.fields.v1`. Conversion retains the input file's coordinates and refuses an existing output file. Errors go to stderr as JSON. The command exits 2 for a refused path or input and 1 for a failed read or write. When moving a result sequence, include `files.json` to retain its position in the project. `machshev guide fields` documents these commands.

!note When you close a project whose field frames, geometry mesh, and surface-current data together exceed 100 MB on disk, Machshev Labs asks which of those categories to delete. Removed data cannot be viewed again without re-running.
