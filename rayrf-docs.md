# RayRF documentation

# Install and first simulation

Download RayRF Studio, sign in, and run the included 5.8 GHz patch antenna from the Welcome Panel. Follow the video or the written steps below.


## Download and install

RayRF Studio runs on Windows and Linux. The 3D field view needs a graphics driver with OpenGL 4.3 and compute shaders that reports free GPU memory. Sign in at [rayrf.com/dashboard/downloads](pricing.html) and download the build for your platform.

- **Windows.** Run `RayRF-Studio-<version>-Setup.exe` to install the app and add RayRF Studio to the Start menu.
- **Linux.** Download the `.AppImage`, make it executable (`chmod +x`), and run it. It needs no installer.

The desktop app has an automatic update checker that runs on startup to let you know when a new version is available.

## Sign in

On the first launch with no license saved, the sign-in dialog presents three options: sign in with browser (recommended, fastest), enter license key manually (get it from your dashboard), or continue as viewer-only. Viewer mode is always free, so you can view projects and results even after your trial or subscription expires.

Once signed in, the Welcome Panel is shown. You can open this menu at any time using `File` then `Welcome Panel`.

- **Blank Project** create an empty project with no layers, no geometry, and default settings.
- **Examples** open one of the included projects: the 5.8 GHz patch antenna, a 2 to 11 GHz Vivaldi antenna, a grounded coplanar line with a via fence, and a 2.4 GHz bandpass filter. Each opens with its S-parameter results, and none of them need more than a Hobby plan.
- **Recent projects** lists your recently opened `.rfsim` files.

The Welcome Panel can be closed with the Escape key or `x` in the top right. If you don't have a project open, a blank one will open, and if you already have a non-blank project open there is no effect.

## Activate a license

Activation assigns a machine to your account.

- **Sign in through RayRF** opens your browser to approve the sign-in. This requires logging into your RayRF account in this browser, which is convenient if it's your primary machine, but if it's a secondary/server machine you may want to use the license key method instead.
- **Enter license key** uses the key from your dashboard, copy it in and click Activate.

If sign-in or activation can't reach rayrf.com, the dialog says why (DNS, a timeout, a proxy, or the certificate, for example). If it keeps happening, click Copy details and send the details to [support@rayrf.com](mailto:support@rayrf.com). `rayrf auth --license-key` prints the same details.

The CLI uses the same license as the UI. `rayrf auth --license-key KEY` performs key-based activation, `rayrf auth --status` displays the current license state, and `rayrf auth --deactivate` releases the seat. Browser sign-in cannot be used with the CLI. For related information see the [CLI overview](docs-cli-overview.html).

## Patch antenna example

Select Patch Antenna from the Welcome Panel. The example is a 5.8 GHz inset-feed patch antenna on an FR-4 stackup with reasonable sim settings. Use the `Edit` tab to explore the stackup and the geometry.

Open the `Simulate` tab and click Run Simulation. RayRF will ask you to save the project before running.

When a simulation is running you can use Stop Early to conclude results early or discard the run. The Energy Ringdown plot indicates net energy remaining in the system. Auto mode targets an energy decay between -45 dB at Low quality and -60 dB at Very High, and a smaller target means a longer, more converged run. See [FDTD tips](docs-fdtd-tips.html). When the ringdown target is reached, timestepping stops and the title bar reads PREPARING RESULTS. The Simulate and result tabs show what is being processed until the results are ready. See [Running simulations](docs-running.html).

Open the `S-Parameters` tab to inspect return loss (S11 only in this example) with the resonant dip near 5.8 GHz. See [S-parameters](docs-s-parameters.html). Open the `Radiation Pattern` to inspect directionality in full 3D and 2D polar E/H slices. See [Radiation pattern](docs-radiation-pattern.html).

## Telemetry and updates

RayRF sends anonymous usage telemetry about which features you use. We never send any information about your designs, sim settings, results, or any text you input to the program. For full transparency you can view the exact raw telemetry data by going to `Settings` then `Anonymous Usage Telemetry` then `View raw events sent in this session` to see a json-format breakdown of the information sent. For example, the information includes time spent on each tab (40% on simulate, 30% on settings, 30% inspecting results, for example), simulation size (bucketed) such as <1M cells, 1M-10M cells, 10M-100M cells, >100M cells. It also includes hardware specifications, runtime, and, if the program crashed, the name of the module it crashed in, nothing more. Raw crash logs can carry a lot of identifying information about a user, and depending on the nature of the crash they can leak information about the project, so we never send crash logs, only the section of the program that crashed. For example "frontend_python", or "backend_cpu" or "backend_gpu". Anonymous usage telemetry is on by default everywhere except the EU and UK to comply with local regulations. You can turn it off at any time under `Settings`, and the setting takes effect immediately for the rest of the session.

The update checker runs at startup and offers Update now if a newer version is available. Update now does four things: Download the installer for the new version, verify its checksum, launch the new installer (you must click through the typical install steps), and close RayRF for you (if a project is open with unsaved changes or a running sim you will need to cancel or confirm closing). RayRF cannot be updated while it's running, so save your projects and finish simulation runs before updating. If you prefer to not install the update, use Remind me later or Skip this version. `Update <version> available` is shown in the title bar if the update was deferred, and you can click there to begin the update when convenient. `Settings` then Check for updates lets you check for a new version manually without restarting RayRF.

---

# The interface

RayRF Studio is a full-GUI desktop app (with CLI capability too). This section covers all key sections of the main window: the title bar, menus, bottom panel, and 5 tabs (Edit, Simulate, S-Parameters, Radiation Pattern, and Field Viewer).


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

---

# Stackup and materials

The stackup is the ordered set of layers that gives your board its vertical structure and its material properties. This page covers the layer model, the dielectric and conductor properties, and where to edit them. Drawing copper and placing ports is covered in [Drawing geometry](docs-geometry.html).


## The layer model

A layer is one of three types:

- **Conductor** carries the copper you draw on it. It is a zero-thickness sheet at a single Z, not a solid slab. The drawn shapes on that layer become the metal.
- **Dielectric** is a substrate slab with a thickness, a relative permittivity, and a loss tangent. It fills the gap between the conductors above and below it.
- **Air** is a free-space spacer. It has a thickness so it sets a gap, but no permittivity or loss, and the gap simulates as air.

Layers are ordered top to bottom in the Stackup list, topmost first. Each layer's Z is derived, never typed: the bottom layer sits at z = 0 and every layer above it stacks up by the dielectric and air thicknesses below it. A conductor's own thickness does not move any Z, only dielectric and air thicknesses do. Because Z is computed, the z field in the properties panel is read only. Change a dielectric thickness and every layer above it shifts.

A new blank project starts with no layers. Build the stackup by adding layers, or open one of the examples to start from a working one.

!fig The Stackup section with a dielectric layer selected, showing the layer list, the New Layer and Delete Selected buttons, and the Mesh View and Calculators buttons

## Adding, removing, and ordering layers

The Stackup section holds the layer list, the Mesh View and Calculators buttons (see [Mesh regions](docs-mesh-regions.html) and [Calculators](docs-calculators.html)), and two buttons that edit the stack:

- **New Layer** opens a dialog. Pick the type (Conductor, Dielectric, or Air), give it a unique name, and for a dielectric enter the thickness and relative permittivity. The new layer lands at the top of the stack.
- **Delete Selected** removes the selected layer. Del does the same while the layer list has focus.

Drag a row in the list to reorder it. Right-click a row for Properties, Rename, Copy Layer Above, Copy Layer Below, and Delete. Double-click a row to rename it.

The active layer is the one new geometry is drawn on. Click a row to make it active, or press a digit key 1 to 9 on the Edit tab to select that row by position.

The Viewport view-mode selector sets which layers are drawn on the canvas. All Layers shows everything, Focused highlights the active layer and dims the rest, and Active Only shows only the active layer's geometry. Shift+S cycles the three modes.

## Dielectric properties

Select a layer, with nothing else selected, to see the Layer Properties panel: the name, the derived z, and the material fields for that layer type. The right-click Properties dialog holds the same fields and works even when another selection hides the panel.

A dielectric layer carries a thickness in mm, a relative permittivity epsilon_r (at least 1), and a loss tangent (0 for lossless). Higher permittivity slows propagation and shortens the guided wavelength, so it sets the electrical length of traces and the resonant frequencies.

A positive loss tangent also carries Loss tangent at, the frequency that tangent is exact at. The solver realizes a loss tangent as one fixed conductivity, so the loss tangent the run models is exact at that frequency and falls as 1/f above it and rises below it. Nothing about the simulation band or the excitation changes it, and it may sit outside the band, which is where a datasheet number usually belongs. The field appears as soon as the loss tangent goes positive, filled once with the frequency this project's excitation sits at (the center of the band, or the CW tone in a CW project), and the number is then the layer's own.

A preset fills these values:

Preset || epsilon_r || Loss tangent
Ideal FR4 || 4.4 || 0.0
Real FR4 || 4.3 || 0.02
Custom || (your values) || (your values)

Editing any value switches the preset to Custom.

## Conductors

A conductor layer carries the copper you draw on it as a zero-thickness sheet at that layer's Z. The material preset fills its conductivity and physical conductor thickness:

Preset || Conductivity || Conductor thickness
Perfect Conductor || 0 S/m || 35 um
Copper || 5.8e7 S/m || 35 um
Custom || (your value) || (your value)

A conductivity of 0 exports as PEC. A positive conductivity activates finite sheet loss and requires a positive conductor thickness. Higher conductivity means less resistive loss. Conductor thickness is an electrical material value. It does not move the layer's Z plane or add stackup height. The geometry stays a zero-thickness sheet, but the loss model reads that thickness as a conductor carrying field on one face with an opaque interior, which is what a ground plane, a trace, or a cavity wall is. Editing either value switches the preset to Custom.

## Paint every dielectric, or it simulates as air

A dielectric layer with no geometry painted on it is treated as air and its permittivity is ignored. `validate` and running a simulation both warn on this, since running a layer as air is sometimes intended. If the layer is a real substrate, paint a board-sized rectangle across it so it carries its permittivity. If it is left over from an earlier stackup, delete it. A layer at epsilon_r = 1 is not flagged, since painting it would change nothing.

## Layer limit on Hobby

The Hobby plan allows a stackup of up to 3 layers total, counting conductors and dielectrics together. Pro and Founding plans have no layer limit.

---

# Drawing geometry

The Edit tab is a 2D drafting canvas: draw copper and dielectric shapes on the active layer, then place vias and ports through the stack. This page covers the coordinate model, the draw tools, and every shape primitive.


## Coordinate model

The scene is in scene millimeters with Y up, so a shape moved toward the top of the canvas gains Y. Every center-based shape stores `x_mm` and `y_mm` as the center of the shape, not a corner, and rotation is clockwise-positive degrees. The ruler and the property spins read and write these same scene millimeters.

Snap and the grid step are in the Viewport section of the left panel. With Snap on (the default), a clicked point rounds to the nearest grid multiple in both X and Y. The grid step defaults to 1 mm. Typing directly in a shape's property fields bypasses the grid, so snap constrains the pointer, not the stored value. M starts the ruler: a two-click transient measurement that draws a line with a distance readout in mm and places nothing permanent.

## Draw tools

!fig The Draw Tools panel on the Edit tab

Each tool arms a single drawing mode. Its shortcut fires only on the Edit tab and only when a text field does not have focus.

Tool || Key || Draws
Select || `Esc` || Pick, move, and edit existing shapes
Rectangle || `R` || Drag a box, or click for a grid-sized default
Circle || `C` || Drag out a radius from the center
Arc || || A curved-trace band (unkeyed)
Polygon || `P` || Click vertices, close to finish
Via || `F` || Drop a plated via through two conductor layers
Hole || `O` || Menu: Hole (click for a round hole, drag for a slot), Cutout
Point Port || `Q` || Single-cell vertical feed
More Ports || || Menu: Area, Planar Line, Planar Bar port
Ruler || `M` || Measure a distance, transient
Import || || Menu: PNG, Gerber, DXF, KiCad, and openEMS importers
Script Shape || || Author a Python-defined shape

Esc cancels the active tool and returns to Select. Import and Script Shape open a menu or a dialog rather than arming a canvas mode. The area, line, and bar ports live under More Ports, with the rectangular Area port also on A. Cutout lives under Hole.

Select a shape to see its handles: a rectangle gets corner and edge-midpoint handles that resize it, a circle gets outer and inner rim handles for its diameters, an arc gets rim and angle handles, and a polygon or cutout gets one handle per vertex. Drag the body to move. Space rotates the selection 90 degrees clockwise about its centroid, X mirrors it horizontally, and Y mirrors it vertically. Ctrl+C, Ctrl+X, and Ctrl+V copy, cut, and paste the selection, Delete removes it. Full key list on [Keyboard shortcuts](docs-keyboard-shortcuts.html).

## Shape primitives

!fig The shape primitives on the canvas

The primitive kinds and their parameters, straight from the geometry registry:

### `rect`

Axis-aligned rectangle (center-based) with optional rotation.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the rectangle center.
`y_mm` || number || mm || 0 || yes || Y of the rectangle center.
`w_mm` || number || mm || none || yes || Full width (local X extent).
`h_mm` || number || mm || none || yes || Full height (local Y extent).
`rotation` || number || deg || 0 || || Rotation, clockwise-positive.

Anchors: `center`, `corner_ne`, `corner_nw`, `corner_sw`, `corner_se`, `edge_n`, `edge_s`, `edge_e`, `edge_w`

### `circle`

Filled disc, or an annular ring when `inner_d_mm` > 0.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the circle center.
`y_mm` || number || mm || 0 || yes || Y of the circle center.
`d_mm` || number || mm || none || yes || Outer diameter.
`inner_d_mm` || number || mm || 0 || || Inner diameter. 0 is a solid disc, >0 is a ring.

Anchors: `center`, `quad_0`, `quad_90`, `quad_180`, `quad_270`

### `arc`

Curved trace as an annular band sector (centerline radius plus width).

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the arc center.
`y_mm` || number || mm || 0 || yes || Y of the arc center.
`r_mm` || number || mm || none || yes || Centerline radius.
`width_mm` || number || mm || none || yes || Band thickness (radial width).
`a0_deg` || number || deg || 0 || || Start angle, counter-clockwise from +X.
`a1_deg` || number || deg || none || yes || End angle, counter-clockwise from +X.

Anchors: `center`, `start`, `end`, `mid`, `start_outer`, `start_inner`, `end_outer`, `end_inner`

### `poly`

Polygon whose vertices are local offsets from (`x_mm`, `y_mm`). Rotation is baked into the points.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X offset of the polygon frame.
`y_mm` || number || mm || 0 || yes || Y offset of the polygon frame.
`points` || array || || [] || || Vertices as `[[lx, ly], ...]` local offsets.

Anchors: `center`, `vertex_<i>`, `edgemid_<i>`

### `via`

Plated via: a copper pad on every conductor layer it spans, joined by a conductor barrel of the drill diameter. The barrel is the same whether the via is filled, capped or open.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the via center.
`y_mm` || number || mm || 0 || yes || Y of the via center.
`annular_mm` || number || mm || none || yes || Pad outer diameter, the copper the mesh realizes on each spanned layer. Must be at least `drill_mm`.
`drill_mm` || number || mm || 0.3 || || Drill diameter, the barrel that joins the pads. Greater than zero and at most `annular_mm`.
`start_layer` || string || || || || Top conductor layer name.
`end_layer` || string || || || || Bottom conductor layer name.

Anchors: `center`

### `hole`

Drilled hole or routed slot through a span of layers. An unplated hole removes the dielectric and copper inside it, except a copper layer it ends on with more of the stack beyond it, which stays as its floor. A plated hole removes nothing: it is a conductor barrel through the tool path that joins the copper layers in its span, the same conductor a via's drill is.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the hole center. For a slot, the midpoint of the tool path.
`y_mm` || number || mm || 0 || yes || Y of the hole center. For a slot, the midpoint of the tool path.
`d_mm` || number || mm || none || yes || Tool diameter, the width of the opening. Greater than zero.
`length_mm` || number || mm || 0 || || Tool travel of a routed slot, center to center. 0 is a round hole. The opening is `length_mm + d_mm` long.
`angle_deg` || number || deg || 0 || || Slot direction, counter-clockwise from +X.
`plated` || bool || || false || || True makes the hole a conductor barrel through its tool path that removes nothing, from the lowest to the highest copper layer in its span, a copper floor included. A copper end with more of the stack beyond it is a floor: an inner plane, or outer copper under a coating such as a solder mask layer.
`start_layer` || string || || || || First layer of the hole's span. Empty is the top conductor or dielectric layer. A copper layer here stays as the hole's floor when more of the stack lies beyond it: an inner plane, or outer copper under a coating such as a solder mask layer. To cut that copper, name the layer beyond it.
`end_layer` || string || || || || Last layer of the hole's span. Empty is the bottom conductor or dielectric layer. A copper layer here stays as the hole's floor when more of the stack lies beyond it. To cut that copper, name the layer beyond it.

Anchors: `center`, `start`, `end`

### `cutout`

Board cutout: a polygon opening through a span of layers, with vertices as local offsets from (`x_mm`, `y_mm`). The dielectric and copper inside it are removed, except a copper layer it ends on with more of the stack beyond it, which stays as its floor. A plated cutout adds a conductor wall on its edge.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X offset of the polygon frame.
`y_mm` || number || mm || 0 || yes || Y offset of the polygon frame.
`points` || array || || none || yes || Vertices as `[[lx, ly], ...]` local offsets.
`plated` || bool || || false || || True puts a conductor wall on the edge, from the lowest to the highest copper layer in the cutout's span, a copper floor included.
`start_layer` || string || || || || First layer of the cutout's span. Empty is the top conductor or dielectric layer. A copper layer here stays as the cutout's floor when more of the stack lies beyond it. To cut that copper, name the layer beyond it.
`end_layer` || string || || || || Last layer of the cutout's span. Empty is the bottom conductor or dielectric layer. A copper layer here stays as the cutout's floor when more of the stack lies beyond it. To cut that copper, name the layer beyond it.

Anchors: `center`, `vertex_<i>`, `edgemid_<i>`

### `port`

Excitation or measurement port. Set `params['port_type']` to `point`, `rect`, `line`, or `bar`. See [Ports and excitation](docs-ports-and-excitation.html).

Param || Type || Units || Default || Required || Meaning
`port_type` || string || || point || || Port subtype: point, rect, line, or bar.
`port_number` || int || || 1 || || Port index used to name the S-matrix column.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || True drives the port. False makes it a passive load or measurement.
`visual_diameter_mm` || number || mm || 2.1 || || Diameter of the marker the editor draws for this port and the DXF export emits for it. Drawing size only: the solver feeds the port from its own footprint.

Anchors: `center`

### `script`

Python-defined shape: the source draws geometry and declares named point anchors.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X origin offset for the script frame.
`y_mm` || number || mm || 0 || yes || Y origin offset for the script frame.
`source` || string || || || || Python source using the shape DSL (path, point, ...).
`xmin_mm` || number || mm || none || || Clip box minimum X, relative to the script frame. Every drawn op is bounded by this box, and it is the shape's measured extent. Omit it and that side is clipped at the domain edge instead.
`xmax_mm` || number || mm || none || || Clip box maximum X, relative to the script frame. Omitted means the domain edge.
`ymin_mm` || number || mm || none || || Clip box minimum Y, relative to the script frame. Omitted means the domain edge.
`ymax_mm` || number || mm || none || || Clip box maximum Y, relative to the script frame. Omitted means the domain edge.

Anchors: `center`, `point:<name>`

A few points the table does not state:

- A circle with `inner_d_mm` greater than zero is a ring, not a disc. Use it for annular pads and washers.
- An arc is a curved trace: a band of `width_mm` swept along a centerline of radius `r_mm` between two angles. Angles are counter-clockwise from +X, which is the drafting convention and independent of the clockwise-positive rotation used by the box-based shapes.
- A poly bakes its rotation into the point list, so it has no rotation parameter. Its vertices are local offsets from the frame origin.
- A bitmap carries an imported raster rasterized as a conductor pattern. See [Importing artwork](docs-import-png.html).
- A script is drawn by Python source using the shape DSL and can declare named point anchors. Author it in the Script Shape dialog below.

## Vias

!fig The via type dialog

A via runs from `start_layer` to `end_layer`, two conductor layers in the stack. The via dialog sets the drill and annular-ring diameters and picks the two layers. The via is a solid conductor through its drill, and its pad at the annular-ring diameter is copper on every conductor layer in its span. The drill must be greater than zero and the annular ring at least the drill diameter.

A via cuts nothing, so a plane in its span joins it unless the plane's drawing keeps a clearance around the pad. The mesh holds a cleared plane at least one cell edge from the via, so a clearance narrower than a cell does not short. The drill is realized on the mesh nodes within its radius grown by a fixed fraction of the local cell, which puts the realized conductor surface on the drill. A drill under 2 cells across still conducts, on as few as one node, and the pre-run check and estimate report it. See [Auto mode](docs-auto-mode.html).

## Holes and cutouts

A hole is a drilled round hole or a routed slot, and a cutout is a polygon opening such as a board cutout. An unplated hole and every cutout remove the dielectric and copper inside them from `start_layer` to `end_layer`. A copper end with more of the stack beyond it stays as the floor of the opening: an inner plane, or outer copper under a coating such as a solder mask layer. To cut it too, end on the layer beyond it, which is then removed as well. A new hole or cutout runs through the whole stack, from the first to the last conductor or dielectric layer, and is not plated.

With the Hole tool, click to place a round hole or drag to place a slot along the drag. It uses the diameter of the newest hole in the project, or one snap grid step for the first, and you can change it in the properties panel. Cutout draws like a polygon: click the vertices, then click the first one again.

Check Plated to join the copper layers in the opening's span, a copper floor included. A plated hole cuts nothing. The hole or slot is a solid conductor, realized on the mesh the way a via's drill is. A plated cutout keeps its cut and gets a conductor wall around it. A plated opening needs at least two conductor layers in its span. A plated opening is drawn with a copper rim. An opening a run would refuse is drawn with a red dashed outline, and the properties panel says why.

## Script shapes

!fig The script shape dialog

A script shape is a small Python program that draws a 2D shape, for the geometry a rectangle or polygon cannot express: a spiral, a taper, a mathematically swept curve. The Script Shape dialog is a code editor over the drawing calls with a live preview, and a lint or runtime error is drawn in the preview with its line number. A new shape opens on a spiral template that runs as is.

The program draws in a local millimeter frame whose origin is the shape's `x_mm`/`y_mm`, so you author around (0, 0) and move the shape on the canvas afterward. Four calls are available:

Call || Draws
`path(points, width=0.2)` || a stroked open polyline, width in mm
`polygon(points)` || a filled polygon, three or more points
`disc(cx, cy, r)` || a filled circle
`point(name, x, y)` || a named anchor, not geometry

`points` is a list of (x, y) pairs in local mm. A `point` call declares a named anchor that appears on the shape for constraints and dimensions to attach to, the same way a rectangle exposes its corners.

The program runs with a math environment and ordinary Python control flow, and no imports. Available names include `pi`, `e`, `tau`, the trig and `sqrt`/`exp`/`log` family, `hypot`, `atan2`, `floor`, `ceil`, `degrees`, `radians`, and `linspace`, plus the plain builtins `range`, `len`, `min`, `max`, `sum`, `enumerate`, `zip`, and the `list`, `dict`, and `set` constructors. Trig here works in **radians**, unlike the expression fields elsewhere in the editor, which use degrees: write `cos(2 * pi * t)`, not `cos(360 * t)`.

A bare name that matches a Variable Studio variable is injected as its current value, so a script shape is parametric: reference `turns` or `pad_w` in the source and the shape redraws when that variable changes. Assigning a name inside the script uses the local value instead. The starter template sets its inputs as local assignments and notes which line to delete to drive one from a variable.

Local-mm bounds clip the drawn geometry, and Auto-fit from geometry sets the bounds to the current extent. A lint blocks imports and attribute escapes before the code runs. A script that raises reports the error and its line in the status bar rather than writing a partial shape into the project.

## Anchors

Every shape exposes named anchors derived from its parameters: a rectangle's corners and edge midpoints, a circle's quadrant points, an arc's ends and rim points, a hole's center and slot ends, a cutout's vertices and edge midpoints, a script shape's declared points. Anchors are computed from the shape, never stored, so they move with it. Constraints and dimensions attach to anchors to lock geometry together. See [Parametric constraints](docs-parametric.html).

## Layer targeting

A new shape lands on the active layer, the row selected in the stack panel. Digits 1 through 9 select a layer by position, and drawing with no layer selected is refused with a prompt to add one. Vias, holes and cutouts are the exception: they name their own start and end layers and span the stack.

The active layer's type decides what a shape paints. On a conductor layer, a shape is copper, and a via adds its pad to every conductor layer it spans. On a dielectric layer, the same shape paints that dielectric region using the same renderer, so a dielectric accepts every shape kind. A dielectric layer left unpainted renders as air and is flagged before a run rather than filled with a guessed slab. Build and order the layers first on [Stackup and layers](docs-stackup-and-layers.html).

---

# Variables, constraints, and generators

The parametric system drives geometry by relationships instead of fixed numbers: named variables, a constraint solver on the canvas, and generators that stamp copies of a master shape. One change propagates everywhere it is referenced.


## Variables

Variable Studio holds the project's named variables. Each row is a name, a value, and an optional min and max.

!fig Variable Studio panel

A value is a literal number or an expression over other variables. The Value cell tooltip shows the resolved number, and the Name cell tooltip shows how many fields use the variable and its group. A name is letters, digits, and underscore, does not start with a digit, and is case sensitive. `pi`, `e`, and the function names are reserved. Min and Max are an optional allowed range. Once both are filled, the only check is that min does not exceed max. Set a variable's group from the row context menu.

Any numeric field of a shape can bind to an expression. To bind a field, type a variable name or an expression straight into its box, or right-click the box and pick a variable from the Variable menu. A polygon vertex binds the same way in the Vertices table, and its expression is measured from the shape's origin. A bound field reads its value from the expression each time the project resolves rather than storing a fixed number. Categorical fields (layer names, port subtype, feed direction) never bind. A binding addresses one field: a shape param, a layer property, a domain or FDTD setting, or a generator param.

The expression grammar: numbers, variable names, `+ - * / // % **` with unary sign and parentheses, the functions `sin cos tan asin acos atan atan2 sqrt abs min max floor ceil round exp log log10`, and the constants `pi` and `e`. Trig is in degrees, the CAD convention, so sin, cos, and tan take degrees and the inverse functions return degrees. An expression evaluates in the field's own unit with no unit suffix. A dependency cycle, an undefined name, or a result of infinity or NaN is reported with the offending token named, never applied.

Set values three ways: edit the cell in Variable Studio, or headless with `rayrf project set --var name=value --project p.rfsim`, or per run with `rayrf run --set name=value`. Both CLI paths re-resolve the whole parametric system (variables, bindings, then constraints) before the project is used. `--set` applies at run start.

## Constraints

The constraint solver holds geometric relationships as you draw and drag. Add a relation and the solver drives it to satisfaction in a least-squares sense, keeping every enabled constraint true. Dimensions and relation glyphs draw on the canvas next to the shapes they bind.

Each constraint type:

Constraint || Meaning
`coincident` || Anchor onto an anchor, an edge (point-on-line), or a circle rim.
`point_on_line` || An anchor lies on an edge segment.
`point_on_circle` || An anchor lies on a circle or arc rim.
`horizontal` || Two anchors share a Y coordinate, or an anchor shares Y with a Y-axis mesh pin.
`vertical` || Two anchors share an X coordinate, or an anchor shares X with an X-axis mesh pin.
`distance` || Distance between two anchors or an edge length (aligned, horizontal, or vertical). Horizontal mode also spans an anchor or another pin to an X-axis mesh pin, vertical mode to a Y-axis pin. The value is a number or expression.
`angle` || Angle between two edges, in degrees.
`radius` || Radius of a circle, arc, or via ring.
`diameter` || Diameter of a circle, arc, or via ring.
`equal` || Two refs measure the same: edge lengths, circle/arc radii, or anchor-pair distances (four-point form).
`tangent` || A rect/poly edge tangent to a circle/arc, or circle tangent to circle.
`fix` || Pin an anchor, a whole shape, or a mesh pin in place.

A dimensional constraint (distance, angle, radius, diameter) carries a target value that is a number or an expression over variables, so a dimension can be driven by a variable. A redundant but consistent dimension is downgraded to a driven reference dimension: it reports its measured value and is excluded from the solve. An equal constraint ties two edge lengths or two circle radii together.

Arm a tool from the keyboard, then click the anchors, edges, or rims it relates. Each key toggles its tool, and it stays armed until Escape, another tool, or Select disarms it.

Key || Tool
`D` || Dimension
`I` || Coincident
`H` || Horizontal
`V` || Vertical
`T` || Tangent
`N` || Angle
`K` || Fix
`E` || Equal

Dragging is constrained. When you move a shape that carries constraints, the solver runs on release with the dragged shape pinned toward where you dropped it and its constrained partners moved the least amount that keeps the hard constraints satisfied. An infeasible drop lands at the closest feasible pose instead of reverting the whole move. A shape held by fix does not move.

## Generators

A generator stamps copies of one or more master shapes. Select the masters, then create an array from the Generators panel.

!fig Generators panel

The stored state is the master shapes plus the generator parameters. Copies (instances) are never written into the saved shape list. They materialize at render time so the canvas shows them, and again at export time for the solver, always from the current masters. Editing a master updates its copies live: a width change on the source resizes every copy at once, and dragging the source moves the copies with it, holding their relative offsets.

The two generator types:

Generator || Params || Meaning
`array` || `nx, ny, nx_minus, ny_minus, dx_mm, dy_mm, mode, lock_spacing` || Rectangular grid of copies of one or more master shapes. The master sits at index (0, 0) and the grid grows both ways along each axis.
`linked_clone` || `dx_mm, dy_mm` || A single offset copy of a master shape (the degenerate array case).

There is no instance limit. A grid is bounded by the machine's memory, and a build that runs out says so.

`nx`, `ny`, `dx_mm`, and `dy_mm` each accept a number or an expression, so a variable can drive the copy count or the pitch.

!fig EBG array on the canvas

A generator has no limit on its copy count. A large array draws on the canvas behind a loading overlay. Cancel stops the drawing and removes the copies drawn so far, and the Console says so. The run and the exports still build every copy.

---

# Importing artwork

Trace a copper layout from a PNG image. The import dialog calibrates the image scale, masks one color as conductor, and places the result on a layer as a bitmap primitive that rasterizes into the mesh like any other shape.


## Opening the import dialog

Two entry points open the same dialog:

- `File` then `Import` then `Import PNG...` in the menu bar.
- Import PNG in the editor's Import dropdown beside the other artwork and project importers.

Both prompt for a `.png` file, then open the scale-and-mask dialog on that image. An image whose decoded RGBA buffer is at least 256 MB first shows a size warning with the estimated peak RAM and added project-file cost, and lets you cancel before loading.

## The steps

The image fills the canvas on the left. Wheel zooms, right-drag pans, and Fit View frames the whole image. The side panel walks the steps in order. OK stays disabled until both a mask color and a ruler length are set.

- **Crop.** Drag the yellow handles on the image to trim the region. Pixels outside the yellow rectangle are ignored. The default rectangle is the whole image.
- **Pick mask color.** Toggle Pick Mask Color, then click a pixel. That pixel's RGB becomes the mask color. Only pixels matching it are kept as conductor.
- **Adjust tolerance.** A per-channel RGB match window, 0 to 255. 0 matches the mask color exactly, higher values include more nearby colors. The overlay updates live: matched pixels fill purple, unmatched pixels drop out.
- **Enter real-world ruler length (mm).** Drag the two ruler endpoints across a feature whose true length you know, then type that length in mm. The dialog divides it by the measured pixel span to get mm-per-pixel. Snap to pixel edges locks the endpoints to pixel boundaries.
- **Choose target layer.** The layer the bitmap is placed on, defaulting to the active layer. A bitmap is a conductor pattern, so target the conductor layer the artwork belongs to.

The Ruler readout shows the current ruler span in pixels. The dialog warns if the mask selects zero pixels or nearly the whole image, both signs the color or tolerance is off.

!note **Scale accuracy.** Measure the longest feature you can. Each ruler endpoint is uncertain by about a pixel, so a short ruler carries far more scale error than a long one. Start from the highest-resolution image you have.

## What mask color to pick

The mask is a single color plus a tolerance, so the image should be close to monochrome artwork: one flat color for copper against a contrasting background. The picked color is the copper, whether it is the dark or the light region. A wider tolerance accepts colors farther from the picked one, absorbing the pixel-to-pixel variation of a noisy or JPEG-compressed image.

## Where the bitmap is placed

On OK, the masked pixels become a `bitmap` primitive on the chosen layer, recolored to that layer's editor color. The image is embedded in the project as pixel data, so it saves inside the `.rfsim` and needs no sidecar file. The bitmap is placed past the right edge of the existing geometry (or at the origin on an empty canvas), snapped to the grid, and selected. Its mm-per-pixel scale sets its physical size. Move, rotate, or flip it afterward like any other primitive.

At simulation export the bitmap rasterizes onto its layer's plane: every kept pixel becomes conductor at the layer's Z, the same conductor mask a drawn polygon would produce. A bitmap that has lost its pixels is a hard export error naming the primitive, never a silently blank layer.

## Reference-trace overlays

Importing a measured sweep to plot against a simulated result is a separate flow. Touchstone and CSV traces load on the results view, not here. See [S-parameters](docs-s-parameters.html).

---

# Importing Gerber boards

The Gerber importer turns a fabrication file set into RayRF geometry: stackup layers from the copper files and the dielectric gaps you assign, copper artwork on those layers, holes and slots from the drill files, and the substrate and domain from the board outline. It lives in the editor's Import dropdown and in File, Import.


The destination is a choice. Add to current project merges the board into the open project and is preselected. Create a new project follows the normal save prompt before replacing the open project. The CLI takes the same choice in the spec's optional `destination` and writes a new project without one.

## Selecting and assigning files

Choose Files or ZIP... takes the board's files or one ZIP archive, and Choose Folder... searches a folder and its subfolders. When the source holds more than one job folder, choose the one to import.

Select the files of one board (Gerber layers plus Excellon drill files). If a `.gbrjob` job file is among them or sits next to them, its declared layer roles and stackup are used. Files without a job entry are classified by the X2 FileFunction attribute inside the file when present, then by the filename conventions of the common CAD exporters (KiCad, Altium and Protel extensions, Eagle).

The assignment dialog then lists every file with its guessed role and a small preview. Per row you can include or exclude the file, override its role, and set a copper layer's stackup position. Copper, drill, and outline files start included. Soldermask, silkscreen, and paste files start excluded and cannot be included under those roles, because they have no effect on the EM model. Override the role if a file was misclassified. A file that cannot be parsed shows its named reason and stays excluded. A row that can't be included is greyed out, and hovering its checkbox, name, or role says why.

Included copper positions must be exactly 1 through the included copper count. At most one outline can be included. A drill row also shows its `Plating` and its `Span`, the copper layers its holes run between (or through the board). The span follows the copper rows you include, so it updates as you change them. X2, job-file, and recognized filename declarations prefill the plating. An undeclared drill requires an explicit plated or non-plated choice, because a plated hole joins the copper layers of its span. Plated holes need at least two included copper layers.

A drill file that declares plating per tool, as KiCad's drill export does, starts at `Per tool (declared in the file)` and every hole keeps its tool's plating, whatever the file name or file-level declaration says. Choosing `Plated` or `Non-plated` instead applies to every hole in the file, and the report names that choice. A file whose X2 FileFunction declares one plating while its tools declare the other is refused until you choose one. A mixed-plating file without per-tool declarations is refused, since the importer cannot tell the holes apart.

Excellon `;TYPE=PLATED` and `;TYPE=NON_PLATED` comments declare plating for the following tool definitions. A tool's X2 plating declaration takes precedence.

Each copper row has a `Polarity` choice: `Auto`, `Positive`, or `Negative`. `Auto` reads image polarity from the file and material polarity from the file or job metadata. An explicit choice overrides those declarations. Negative copper fills the included closed board outline and removes the plotted clearances. Without a valid closed outline the import is refused, since the copper extent cannot define the plane boundary. The preview shows the selected polarity.

Each gap between two included copper layers gets an editable dielectric thickness and relative permittivity. Values declared by the job file are prefilled. Missing values remain required.

Click Import to check the assignment and build the board. If the check finds a problem, the dialog stays open with the reason next to the button and on the row it names, and focus moves to the setting to change.

## Importing from the CLI

`rayrf project import` reads a JSON board-import spec and calls the same parser, assignment resolver and project builder as the editor.

```
rayrf schema --what board-import
rayrf project import --spec board.import.json --out board.rfsim
```

The spec lists every source file and at most one `.gbrjob`. Job paths resolve relative to that job file and apply only to the exact selected file. An adjacent `.gbrjob` that is not listed cannot change the CLI import. `role_source` is `detected` to accept the same job-file, X2, and filename detection shown by the editor, or `explicit` to require one assignment for every parsed file. `stackup.source` is `explicit`, `job`, or `defaults`, so nominal dielectric values never enter a CLI project without an explicit choice. The complete format and examples are in [Scripting and specs](docs-cli-scripting.html).

An optional `project` block adds simulation settings and authored shapes after the board is built. Use it to add an active port and frequency band when the result must validate and run without opening the editor. It uses the same shape defaults and settings value gate as `project build`.

The output must not already exist. `--replace` explicitly replaces an existing project and removes its same-stem `.rfsimout`, so results from older geometry cannot attach to the imported board. For a merge, set `destination.mode` to `current` and name the saved input in `destination.project`. `--out` always names the result. Writing back to the same path also needs `--replace`.

## What the import produces

A new-project import installs a normal unsaved project, marked dirty and without a file path. A merge into the current project is one undoable edit and reuses or adds layers as a KiCad merge does. See [Importing KiCad boards](docs-import-kicad.html). The CLI saves the result to its required `--out` path. The imported board contains:

- One conductor layer per included copper file, top first, with one native artwork object retaining its paths, apertures, regions, and holes. Board files carry no conductor material, so each layer is a perfect conductor 35 um thick. Change it per layer on the Edit tab.
- One dielectric layer per gap, carrying a substrate polygon from the outer board outline or a rectangle from the copper extent when no outline is given.
- One cutout per inner contour of the board outline, cut through the whole board. A contour that is exactly a circle or a slot becomes a hole of that exact size.
- One hole per drill hit and one slot per routed segment, plated or not, at the drill diameter. Each spans the drill file's declared copper pair or, when undeclared, the whole board. An X2 FileFunction span is mapped to those exact copper indices regardless of endpoint order. A same-layer or out-of-range span is refused rather than adjusted. A non-plated hole or slot is cut through the copper and dielectric of its span, and a plated one is a solid conductor through its drill between the copper layers at the ends of its span. The pads around a hole are the copper artwork.
- Routed slots that enclose an area free the board inside it, so that area is cut out as well, in whatever order the routes were cut. Routes that stop short of each other still close the area when their cuts overlap. An enclosed area the cut already removes, such as the strip between two passes of a slot wider than its tool, adds no cutout.
- The domain rectangle from the board outline. Without an outline file the copper extent is used and domain auto-fit stays on.

The report gives each drill file one line with its plated and non-plated holes and slots and the layers they run between, and one line with the conductor the copper layers were given. It also lists geometry approximations and omitted material or geometry data. The full transfer record is written to the Console. A clean import needs no popup.

Advanced, Geometry, Raster selects compact binary masks. Their pixel size derives from the smallest source aperture. The CLI uses `"geometry": "raster"` for the same option. Native geometry is the default.

## Format coverage

Gerber RS-274X: standard and macro apertures (including macro variables and arithmetic), draws and validated arcs in single and multi quadrant modes, G36/G37 region fills, dark and clear polarity, and step-repeat blocks. Leading and trailing zero suppression follow the active %FS declaration, including a coordinate-format change within the file. Units must come from %MO or G70/G71. M02 is required, and unterminated word or extended-command blocks and commands after it are refused.

Excellon drill files: metric and inch declarations, including M71 and M72, tool definitions with diameter, feed and spindle fields, drill hits, straight G01 routes, and G85 slots. A headerless file is accepted only when its units are declared. Fixed-point coordinates need an explicit coordinate format, from a unit declaration or a recognized format comment. They also need LZ or TZ zero suppression unless every coordinate is written at the full declared width, because a shorter coordinate reads two ways without it. Unit and tool changes take effect where they occur. M30 is required, and malformed coordinate content or non-comment commands after it are refused. Curved G02 and G03 routes are refused with the named code because the project model cannot preserve them. Both readers accept ASCII-compatible UTF-8 and Latin-1 text and strip an optional UTF-8 BOM. UTF-16 and UTF-32 artwork files are refused with the required export encodings named.

A construct the importer cannot represent fails with a named reason instead of importing a wrong board: incremental coordinates and axis transform blocks are refused. Legacy `IPPOS` and `IPNEG` image polarity and X2 `FilePolarity` declarations are read by `Auto`. Missing units, ambiguous fixed-point drill coordinates, missing end-of-file codes, and trailing command content are refused rather than guessed or ignored.

A drill file whose closed routes enclose the whole board outline, touch it, or run outside it routes the board edge rather than an opening, so it is refused by name until you exclude it. Without an outline, routes that enclose all the copper are refused the same way.

## Limitations

- Copper imports as per-layer artwork, not editable pads and traces. Drawn geometry can be added on top, and the artwork can be moved, rotated, or mirrored.
- Outline arcs that are not an exact circle or slot use 64 segments per circle, and the report names that approximation.
- Straight G01 and G85 slot endpoints and diameters are read exactly. Curved routes are refused.
- Soldermask, silkscreen, and paste artwork is not imported.

---

# Importing DXF drawings

The DXF importer turns a CAD drawing into a new RayRF project the same way the [Gerber importer](docs-import-gerber.html) turns a fabrication set into one: assign roles, get stackup layers, artwork, holes, and the domain rectangle. It lives in the editor's Import dropdown and in File, Import. Parsing uses the ezdxf library (MIT licensed).


Import creates a fresh project. The editor asks about unsaved work before replacing its current project. The CLI writes a new project from an explicit board-import spec. Neither surface merges the drawing into existing geometry.

## Selecting and assigning layers

Select one `.dxf` file. Its drawing unit comes from the file's declared $INSUNITS. All defined DXF length-unit codes 1-24 are converted to millimeters. An unknown unit code is refused. A drawing that declares no unit is asked whether its coordinates are millimeters or inches, never guessed. No unit is preselected, so the import cannot proceed on a default.

The assignment dialog then lists every DXF layer the same way the Gerber dialog lists files: a guessed role from the layer name, a small preview, include or exclude per row, role override, and stackup position for copper layers. Layers that were frozen or off in the source CAD stay selectable but never default into the board. Dielectric gaps between included copper layers work exactly as in the Gerber import.

Included copper positions must be exactly 1 through the included copper count, and at most one outline can be included. A layer assigned the drill role needs an explicit plated or non-plated choice when its name did not declare one. A `PTH` or `NPTH` layer name supplies that declaration. A mixed-plating layer is refused because the importer cannot retain plating separately for each hole. A drill-role layer may contain circles, slot outlines, and block placements of them. A slot outline is a closed polyline with two straight sides and two half-circle ends, or the same outline drawn as two lines and two half-circle arcs. The polyline closes by its closed flag or by repeating its first vertex, and it reads as a slot when no part of it strays from one by more than 1% of the slot radius, the same rule an outline cutout follows. If the layer also contains other supported artwork, import refuses and names the layer, entity types, and source file because the drill model cannot retain that artwork. A polyline drawn like a slot that misses one by more than 1% is named as such.

## Importing from the CLI

`rayrf project import` reads a JSON board-import spec and calls the same DXF reader, assignment resolver and project builder as the editor.

```
rayrf schema --what board-import
rayrf project import --spec drawing.import.json --out drawing.rfsim
```

The spec lists one DXF source. `role_source` is `detected` to accept the same layer-name detection shown by the editor, or `explicit` to require one entry for every DXF layer. A drawing without `$INSUNITS` requires `drawing_unit` set to `mm` or `inch`. A declared drawing uses its own unit and refuses a `drawing_unit` override. `stackup.source` records whether gaps are explicit or use the nominal defaults. The complete format and examples are in [Scripting and specs](docs-cli-scripting.html).

An optional `project` block adds settings and authored shapes after the drawing is imported. A headless workflow can add an active port and frequency band there, using the same port defaults and settings value gate as `project build`.

The output must not already exist. `--replace` explicitly replaces an existing project and removes its same-stem `.rfsimout`. Import creates a fresh project and has no merge flag.

## What the import produces

The editor installs the same fresh unsaved project a Gerber new-project import produces. The CLI saves that content to `--out`. It contains copper layers as native artwork objects, dielectric layers carrying the substrate (the outline polygon, or a rectangle from the copper extent without an outline), the domain from the outline role, and holes. On a layer assigned the drill role, every circle becomes a round hole of the circle diameter and every slot outline a slot of its width, plated or not as the layer's plating says, running from the top to the bottom copper layer. A closed contour inside the board outline becomes a cutout, or a hole of its exact size when it is a circle or a slot outline. The report lists approximations and omitted geometry. The full transfer record is written to the Console. A clean import needs no popup.

## Entity coverage

Converted to artwork: LWPOLYLINE, POLYLINE, LINE, ARC, CIRCLE, ELLIPSE, SPLINE, HATCH (including hole loops), SOLID, TRACE, and INSERT block references with their full transforms, nested up to 16 levels. MINSERT arrays expand every declared placement, and nested translation, rotation, scaling, mirroring, and entity coordinate-system transforms are applied to the artwork. A block cycle or deeper nesting refuses the file by name. A supported entity that cannot be transformed or converted also refuses the file rather than leaving partial artwork.

A closed zero-width path fills. A width-defined polyline becomes its actual trace polygon, including varying widths, so closing it leaves the center hollow instead of filling the enclosed area. Native import refuses open zero-width paths. Set their width in the source drawing or exclude their layer. Raster import omits these paths and lists them in the import report. Arcs, circles, ellipses, splines, bulged polylines, and curved HATCH boundaries are flattened to line segments using a tolerance local to each entity, so distant large geometry cannot erase a small curve. For a width-defined bulged polyline, the stated tolerance bounds the outer trace edge rather than only its centerline. The report states the tolerance or tolerance range used for each approximated entity kind on each layer. A circle or slot outline that becomes a hole keeps its exact size and is not reported as flattened. On a copper layer it is flattened artwork and is reported.

Text, dimensions, points, images, and other non-artwork entities are counted and listed in the report as skipped, never silently dropped. Degenerate artwork with no extent or closed area is also named in the skipped section. A file with structural errors is retried through a recovery reader. Safe repairs and a recoverable missing EOF marker are reported. Unrepaired structural errors and files truncated before any modelspace entity can be recovered are refused with a named reason. UTF-16 and UTF-32 text drawings are refused with the required export encodings named.

## Limitations

- Each layer is one artwork object. Its position, rotation, and mirroring remain editable. Individual source entities remain grouped.
- Geometry is read in the XY plane. A 3D drawing imports as its XY projection.
- Closed zero-width outlines fill. Width-defined closed paths remain hollow traces. A shape drawn as disconnected open segments does not close into a filled area.
- Only circles and slot outlines become holes, on layers assigned the drill role or inside the board outline.
- Import budgets: a drawing that expands past 500,000 entities, including MINSERT placements, or would generate more than 2,000,000 output segments is refused with a named reason rather than stalling the editor.
- Advanced, Geometry, Raster selects compact binary masks. That mode samples the smallest feature at four pixels and reports millimeters per pixel.

---

# Importing KiCad boards

The KiCad importer reads one `.kicad_pcb` layout and sends it through the same layer assignment and project builder as the Gerber and DXF importers. It is available in the editor's Import dropdown and under File, Import.


The destination is always a choice. Add to current project merges the layout with the open project. Create new project follows the normal save prompt before replacing the open project with a fresh unsaved one. Neither choice is preselected.

## Assigning the board

After the file is read, the assignment dialog lists each parsed KiCad layer. Copper, drill rows, and `Edge.Cuts` start included. Other layers start excluded. Holes are grouped into one drill row per plating and span, and each row shows the copper layers its holes run between. Exact dielectric thickness and permittivity from the saved stackup fill the editable gap fields. Missing values remain required.

Rows that can't be included are greyed out, and hovering one says why. Click Import to check the assignment and build the board, as in the [Gerber import](docs-import-gerber.html).

Repeated dielectric sublayers become one editable gap. Their thicknesses are summed. Equal relative-permittivity values remain exact. Differing values use a thickness-weighted mean, and the report names that approximation.

The report lists geometry approximations and omitted material or geometry data. The full transfer record is written to the Console. A clean import needs no popup.

## Current and new projects

A new-project import creates an unsaved project with the imported layer stack, native copper artwork, holes, board cutouts, and domain. It does not invent a port or frequency band.

Every imported project records the source basename, format epoch, and SHA-256 digest. The absolute source path is not saved. This provenance survives project save and reload, Run History snapshots, undo and redo, and current-project merges.

A current-project import is one undoable edit. An imported layer reuses an existing layer only when its name and material values are equivalent. A name collision with different values receives a stable `KiCad import` suffix and is listed in the report. New layers are added below the current stack in imported order. Where the current stack continues past the imported board's outer copper, that copper stays inside the board's holes and cutouts, and the report names the layer and the count. Existing results stay attached but are marked stale.

## Geometry

Native geometry is the default. Each copper layer retains its paths, apertures, regions, and holes in one artwork object. Moving, rotating, and mirroring it preserve the source geometry. The mesh samples it directly at the requested cell size.

Advanced, Geometry, Raster selects embedded binary masks instead. The CLI uses the optional `"geometry": "raster"` field for the same choice.

## Importing from the CLI

`rayrf project import` reads the same file, detected rows, assignment rules, and project builder from a strict JSON spec. The destination remains explicit.

```
{
  "schema": "rayrf-board-import/v1",
  "format": "kicad",
  "sources": ["board.kicad_pcb"],
  "destination": {"mode": "new"},
  "role_source": "detected",
  "stackup": {"source": "detected"}
}
```

```
rayrf project import --spec board.import.json --out board.rfsim
```

For a merge, set `destination.mode` to `current` and name the saved input in `destination.project`. `--out` always names the result. Writing back to the same path also needs `--replace`. A different output keeps the input project unchanged.

## Geometry coverage

The importer reads board and footprint lines, rectangles, circles, arcs, polygons, and Bezier curves. It applies saved footprint translation and rotation. Bottom-side child coordinates are read as stored in the board. Tracks, track arcs, filled zones, vias, pad holes, and oval pad slots are read, plated and non-plated. Each via and pad hole becomes a hole or slot at its exact drill size. A non-plated one is cut through the copper and dielectric it passes, and a plated one is a solid conductor through its drill between the copper layers at its ends. The copper around a hole is the pad and via artwork, on exactly the layers KiCad declares. A non-plated round or oval pad centered on its drill and no larger than it on either axis lies inside the hole and is not drawn. Pad holes go through the board. Blind and buried vias keep their physical copper endpoints.

Filled graphics on copper include their declared outline stroke. On `Edge.Cuts` every shape counts by its centerline, whatever its stroke width or fill, so a zero-width or filled shape still cuts the board, once. The outer closed contour becomes the substrate polygon and domain extent. Each inner contour becomes a cutout through the board, and one that is exactly a circle or slot, such as a mounting hole, becomes a hole of that exact size. Other outline arcs are flattened at 64 segments per full circle and are named in the report. Open outlines and disconnected outer contours refuse.

Supported pad shapes are circle, rectangle, oval, rounded rectangle, trapezoid, and custom pads with supported anchor and graphic shapes. Zone contours use even-odd filling so holes remain holes. Copper artwork remains one native object per included layer. Source curves that require approximation use bounded segments and are named in the import report.

## Skipped content and limits

- Text, dimensions, targets, images, and other unsupported objects on copper refuse with the source file and object context. The same object on a non-copper layer is listed as skipped.
- Unfilled copper zones, copper knockout artwork, chamfered pads, custom pad stacks, backdrills, and unsupported conductor graphics refuse instead of disappearing from the imported model. A keepout rule area carries no copper on any layer, so it is omitted with a notice naming the zone.
- Removal of unused pad layers requires connectivity information and refuses.
- Soldermask, paste, and silkscreen do not enter the model unless the user assigns a modeled role.
- Copper thickness, material names, and dielectric loss tangent are not carried by the shared board stackup builder. Available thickness and relative permittivity remain visible during assignment.
- The parser accepts an explicit allowlist of KiCad 6 through 10 format epochs from `20211014` through `20260206`. An older, newer, missing, malformed, or unknown interior epoch refuses with the source file and value named.
- One board is limited to 256 MiB and 2,000,000 output geometry operations. The parsed-node budget scales from 4,000,000 to 12,000,000 with source size.

The reader is a format parser. It does not invoke or link KiCad code and adds no runtime dependency.

---

# Importing openEMS and CSXCAD XML

The openEMS importer reads a complete openEMS XML setup or a geometry-only CSXCAD `ContinuousStructure`. It is available in the editor's Import dropdown and under File, Import.


After parsing, a dialog asks for the destination. Create new project follows the normal save prompt and replaces the open project. Merge into current project keeps the open project and offers Import frequency band when the file declares a supported band. Neither destination is preselected. Every approximation and every item that does not transfer is listed in the import report and the Console.

Ports, boundary conditions, and domain spacing are not imported. The source mesh lines are kept as an explicit grid that follows the reviewed stack, and RayRF sets boundaries, air, and PML to Auto mode. Add and check ports before running.

## Reviewing layers and materials

The shared assignment dialog groups conductor properties on each CSXCAD Z plane. Each plane can be included, excluded, assigned a role, and placed in the RayRF copper order. Vias, drawn as vertical metal cylinders or round extruded polygons, appear as one plated-drill row. The row offers only plated, because the vias are metal in the source, and a CLI spec that sets it non-plated is refused. Dielectric gaps show the source spacing and scalar relative permittivity.

A source dielectric region that only partly covers a conductor gap is refused. Two lateral material regions in the same gap must have the same permittivity and loss tangent. This prevents the single reviewed gap value from hiding a Z subdivision or unlike electrical materials. Free space is the exception, see Holes and free space below. Free space over part of a gap reads as empty space there, and an empty gap takes its reviewed material over the imported geometry bounds.

The reviewed assignment is authoritative. Excluded conductor properties and vias are listed as skipped. A reassigned outline row sets the initial XY extent. Every material substitution or combined region is listed in approximated.

## Holes and free space

A `Material` with relative permittivity 1 and no conductivity is free space. Where it outranks the substrate it removes it, so it imports as an unplated opening. A vertical `Cylinder` becomes a round hole, and a `Box` or `LinPoly` with height becomes a cutout with the same outline. The import report counts the free-space regions per material and names the layers their openings run between. Openings are not rows in the review. They import with the board and can be edited or deleted afterward like any hole or cutout.

Priority decides what an opening cuts, by the CSXCAD rule: the highest priority wins, and a primitive's face counts as inside it. On each conductor plane the free space reaches, the copper is cut where the free space outranks it and kept where the copper outranks it, so a mounting hole drawn as air above the copper priority clears the copper inside its outline. Copper that lies under a higher shape inside the outline, such as a trace under a pad, is covered there and not cut. Copper drawn with thickness is cut only where the free space spans its whole thickness, and free space that ends on its face, or inside copper hanging from a plane, leaves it in place. CSXCAD does not define which of two equal priorities wins, so at a tie the substrate or copper is kept and the report says so. Free space drawn as stacked pieces of one outline at one priority counts as one region, even when the pieces come from different materials.

What free space removes is decided in source coordinates with the reviewed copper rows, so excluding a copper row changes it the way the same edit to the source would, and every message quotes the Z values in the file. Excluding the vias leaves their bores and fills out with them. A reordered stack keeps an opening as long as the copper rows it runs through stay next to each other, in either direction. Material past the outer kept planes is listed as skipped, and free space that outranks only that material removes nothing.

Free space that fills every gap it touches from plane to plane, as the only material there, and removes nothing imports as an ordinary dielectric layer with permittivity 1, such as an air spacer under a patch. Free space that the board material outranks everywhere, such as a whole-domain air box at a priority below the substrate, removes nothing and is listed as skipped, with the reason. So is free space beside the substrate that overlaps no board material, and free space that only touches the face of thick copper.

Free space that outranks a whole via over its height removes the via. Free space inside a via over the via's height is its bore and is not imported, and a dielectric fill inside a via is not imported either. A part drawn past the via into the open space above or below the board does not change that. A fill that runs past its via into the board is a dielectric insert beyond it, which the one-material gap rule refuses.

A RayRF opening runs through whole layers and cuts its whole outline. Free space that cannot be expressed that way refuses the file, with the primitive named:

- It removes the substrate over only part of its thickness, or copper drawn with thickness over only part of that thickness.
- It ends on inner copper that it removes, with material beyond that copper. When the next gap holds no material under it, the opening runs on through that gap to the next plane instead.
- It clears copper without the dielectric beside it. Cut that clearance into the copper shapes instead.
- It outranks one material it overlaps but not another, so it would cut only part of its outline.
- It cuts into a via, runs past the via it sits in, or becomes an opening around or across a via it does not outrank.
- Inside a via, it holds another via, which is a coaxial structure, or reaches into a second via.
- The reviewed copper order separates the planes it runs through.
- It is one of several pieces of one outline stacked at different priorities, and that piece alone cannot be expressed. Pieces at one priority are one region.

## Settings after import

A new project receives a positive Gaussian band when the XML declares one. Other excitation types are reported and leave the band unset, which must be set before running. A merged project brings the band only when its frequency-setting checkbox is checked.

Both destinations record the source basename, XML root and declared version when present, and SHA-256 digest in the project import provenance. A source with no version attribute is recorded as version undeclared rather than assigned a guessed version.

Both destinations set these controls to Auto mode:

- Simulation mode.
- Air and PML spacing.
- All six boundary faces.

A new project keeps the source mesh lines as an explicit grid. The Z lines move with the reviewed stack: each kept conductor plane lands on its reviewed plane, a line between two planes scales with their gap, and a line past the outer planes keeps its distance from the nearest one. When the reviewed copper order does not follow the source Z order, or only one conductor plane is kept, the Z lines are dropped, Z is meshed automatically, and the report says so. When that grid has X and Y lines, the XY domain follows the retained lines exactly and domain fit is off. A merge applies the source grid to the combined project, refuses when the current project already has a mesh profile, and turns domain fit back on.

Source ports are never converted. A new project has no ports. A merge keeps the current project's ports, marks existing results stale, and reports that the ports must be checked against the combined geometry.

An imported stack merges only when all equivalent named layers can be reused, or when the complete imported stack can be appended as one block. A partial layer match is refused because reusing part of the stack and appending the rest would change its Z order.

## Importing from the CLI

The generic import verb and the strict openEMS alias use the same spec and backend:

```
{
  "schema": "rayrf-board-import/v1",
  "format": "openems",
  "sources": ["setup.xml"],
  "destination": {"mode": "new"},
  "role_source": "detected",
  "stackup": {"source": "detected"}
}
```

```
rayrf project import-openems --spec setup.import.json --out setup.rfsim
```

For a merge, `destination.project` names the saved input and `destination.bring_frequency_settings` must be `true` or `false`. `--out` always names the result. The generic `rayrf project import` command accepts the same spec. A `true` frequency choice refuses the import when the XML has no supported positive Gaussian band.

The CLI prints the same imported, approximated, and skipped categories after a successful write. Any fatal XML or representation error is reported before the output project is created or replaced.

## XML coverage

The reader accepts unnamespaced Cartesian XML with exactly one `RectilinearGrid`, one `Properties` container, and one element for each grid axis. `DeltaUnit` is expressed in meters per drawing unit. When it is omitted, the CSXCAD default of 1 meter per drawing unit is applied and disclosed. Converted coordinates must remain finite and nonzero values must not underflow.

Scalar `Material`, `Metal`, and `ConductingSheet` properties are read. Supported primitives are planar or extruded `Box`, Z `Polygon`, Z `LinPoly`, and vertical `Cylinder`. Each primitive needs its integer `Priority`. Priority decides what free space removes. Elsewhere project layer order replaces it, and the report says so. Polygons with duplicate adjacent vertices, zero area, or self-intersections are refused.

Finite-thickness conductor boxes and extrusions flatten to their top plane and are listed as approximated. A vertical metal cylinder becomes a via, with the drill and the pad at the cylinder diameter, only when its actual Z interval intersects at least two distinct conductor planes. A conductor `LinPoly` whose six or more corners go all the way round one circle, no two neighbors more than 60 degrees apart, and whose Z interval crosses two conductor planes, is a via drawn as a polygon and becomes a via at that circle. An arc closed by a chord spanning more than 60 degrees, such as a castellated half hole, and a chamfered square are flattened like other copper. The importer does not snap cylinder endpoints to nearby planes. `ConductingSheet` conductivity and thickness become active conductor-layer material fields. Its planar geometry uses the finite-loss sheet model. Material conductivity can become loss tangent when a positive Gaussian band supplies the conversion frequency. The conversion uses the Gaussian center frequency and is listed as approximated. That frequency becomes the imported layer's loss reference frequency, so the run realizes the conductivity the source stated. A positive material conductivity without that band refuses the file rather than becoming zero loss.

The parser reads boundary values, supported port components, and FDTD runtime attributes only to report what was present and not transferred. Lumped-port inventory recognizes Python and MATLAB probe names, optional name prefixes, and passive ports without an excitation property. Duplicate, misaligned, incomplete, short-circuit, and open-circuit component groups are named in the import report and are not counted as complete finite-impedance ports.

## Refused and untransferred content

- Non-Cartesian coordinate systems refuse the file.
- XML namespaces, duplicate singleton containers, missing complete-setup FDTD, non-default background material, and unit overflow or underflow refuse the file.
- Anisotropic materials, magnetic material values, spatial material weights, transformed primitives, non-Z polygons, non-vertical cylinders, invalid polygon topology, unknown geometry fields, and unsupported property or primitive types refuse the whole file before any project is changed.
- Source ports, recording boxes, boundary conditions, runtime stop criteria, and domain spacing are listed as skipped.
- Material regions above or below the outermost conductor planes, and the part of a region that runs past them, are listed as skipped.
- A file with no conductor is refused, because its material regions have no planes to sit between.
- Holes, cutouts, and plating that a RayRF export wrote as `RayRF openings` and `RayRF opening walls` are counted and listed as not imported.
- XML with a DTD or entity declaration is refused in every supported text encoding.
- One file is limited to 64 MiB, 1,000,000 XML elements, and 250,000 CSXCAD primitives.
- A grid is limited to 500,000 lines per axis and 1,000,000 lines in total.

The importer uses the Python standard library. It does not invoke or link openEMS or CSXCAD code.

---

# Exporting DXF drawings

The DXF exporter writes the current project's resolved 2D geometry as a millimeter DXF drawing with the ezdxf library (MIT licensed). It is available under File, Export and as a headless project command.


## Exporting from the editor

Select File, Export, Export DXF drawing. The export dialog lists every project layer, all checked, and three utility layers under Include: Board outline, Port footprints, and Drill holes. A utility the project has nothing for is greyed out. File layout writes a Single file or a Separate file per layer, named `name-[layer].dxf`. Click Export and choose the output file. The export runs behind a loading overlay, and Cancel stops it without leaving a partial file.

The completed export shows the entity count and any warnings, and the Console prints the outcome. The full exported, approximated, and omitted report is embedded in the drawing as leading 999 comment lines, so the file carries its own record. A failed export names the shape, input, or file operation that prevented a complete drawing.

## Exporting from the CLI

The CLI reads one saved project and writes one drawing, or one per layer:

```
rayrf project export-dxf --project FILE --out FILE [--replace] [--json]
    [--layer NAME]... [--separate] [--no-outline] [--no-ports] [--no-drills]
```

`--layer` names one project layer to export and can repeat. Without it every layer is exported. `--separate` writes `FILE-[layer].dxf` for each selected layer and utility layer. `--no-outline` leaves out Board Outline and the board cutouts, `--no-ports` leaves out Ports, and `--no-drills` leaves out via drills and holes.

The `.dxf` suffix is added when `--out` does not have it. An existing output refuses unless `--replace` is set. Parent directories are created as needed. The output is prepared in the destination directory and installed only after it passes a serialized DXF readback and structural audit.

Without `--json`, the outcome line goes to stdout and each warning to stderr. `--json` writes this schema to stdout:

```
{
  "schema": "rayrf-dxf-export-report/v1",
  "project_file": "/work/board.rfsim",
  "output_path": "/work/board.dxf",
  "output_paths": ["/work/board.dxf"],
  "output_family": "",
  "entity_count": 42,
  "layout": "single",
  "selected_layers": ["Top Conductor", "Bottom Conductor"],
  "report": {
    "exported": [],
    "approximated": [],
    "omitted": [],
    "warnings": []
  }
}
```

## Drawing contract

The writer produces an R2013 DXF with $INSUNITS set to millimeters. Project coordinates and dimensions are written at their stored millimeter scale in the XY plane.

Every selected project layer is represented in the DXF layer table, including an empty layer. A hidden project layer is off. The built-in DXF layer 0 keeps that name. Any other valid, unique project layer name is preserved. Defpoints, a utility-layer collision, invalid characters, an empty or over-length name, or a case-insensitive project-layer collision produce a legal unique DXF name and an approximated report entry. Defpoints is renamed because DXF gives it non-plot semantics. Exact duplicate project layer names refuse the export because they do not identify one unambiguous source layer.

The project does not retain a separate shaped fabrication outline. The exporter writes the rectangular simulation domain as a closed polyline on the Board Outline layer and records that substitution as approximated. When domain auto-fit is enabled, the exporter recomputes the rectangle from the same resolved geometry and padding settings used by simulation export. A manual domain is written verbatim. A cutout that reaches the rectangle's edge notches it.

Before geometry is written, variables, bindings, constraints, and script shapes are resolved on a copy of the project. Authored primitives and current generator instances are included. The parametric definitions and script source are not embedded in the DXF.

## Geometry coverage

RayRF geometry || DXF representation
Rectangle and polygon || Filled hatch plus a closed lightweight-polyline boundary
Circle and annulus || Filled hatch plus one or two circle boundaries
Arc band || Filled hatch plus arc and line boundaries
Script polygon and disc || Evaluated fill and boundary entities
Script path || Native wide polyline and endpoint discs, or filled polygons when clipping is needed, and the path needs a positive width
Bitmap and imported DXF shape || Filled rectangles vectorized from stored non-transparent pixels, each with a closed boundary
Via || Filled pad and boundary entities on each conductor in its span, plus a circle of the drill diameter on Drill
Hole || A circle for a round hole, or a closed polyline with two straight sides and two true arc ends for a slot. Plated holes go on Drill PTH, the rest on Drill NPTH. A hole writes no pad
Cutout || A closed lightweight polyline on Board Outline. Cutouts that overlap, share an edge, nest, or reach the board edge are written as the board they leave, one contour for their union and a notch in the board outline where they reach its edge. A cutout off the board is not written
Point, line, bar, and rectangle port || Visible footprint on Ports, with a hairline for the mesh-thin line port and fill plus boundaries for filled regions

Filled polygons, discs, rings, and arc bands include a solid HATCH and an ordinary visible boundary tagged as its companion. RayRF suppresses that duplicate boundary on re-import only while its geometry still matches the HATCH boundary. If another editor removes the HATCH, the ordinary boundary remains and imports as geometry. If the boundary is edited independently, RayRF retains both and reports the difference. Board Outline is boundary-only.

Raster-backed shapes keep their stored position, pixel scale, rotation, and flips. Non-transparent pixel runs become closed rectangles at the image's own resolution.

Script geometry is clipped to the script shape's bounds. A round clipped edge that must be flattened uses a relative chord tolerance of 1e-4, and the approximation is reported.

## Export report

Every completed export records the same three categories, and the approximated and omitted entries are its warnings:

Category || Meaning
Exported || Layers, entities, resolved generator instances, and raster-backed shapes written to the drawing
Approximated || Geometry or naming represented with a stated substitution or tolerance
Omitted || Non-geometric project information that DXF cannot encode and that does not change the written 2D geometry

The omitted section names non-geometric stackup data, simulation and mesh settings, editor state, results, run history, and imported or reference traces. For parametric projects it also names the definitions that are absent while confirming that current resolved geometry was written.

Port footprints do not carry port number, excitation, impedance, layer connections, or de-embedding. Via footprints do not carry barrel connectivity, plating, or nets. A hole keeps its plating through the layer it is written on, but neither a hole nor a cutout keeps its layer span. The omitted section counts the holes and cutouts that do not run from the top to the bottom copper layer, and the plated cutouts, because Board Outline has no plating. It also counts the cutouts written as the outline they cut together, which re-import as that outline, and the cutouts off the board.

Unsupported primitive kinds, missing layers, missing required coordinates, invalid topology, degenerate geometry, and values that would need repair refuse the whole export. The refusal names the source project, destination drawing, shape ID, and kind when those values are available. A completed report never uses omitted to excuse missing geometry.

## Re-importing the drawing

DXF re-import preserves millimeter coordinates, layer names, and the 2D artwork represented by the drawing, but it is not a project-file round trip. Included copper artwork returns as one native artwork object per assigned conductor layer, so separate primitives on one layer coalesce. Native arcs, circles, curved HATCH boundaries, and bulged polylines become line segments. The import report states the local flattening tolerance for each curve kind. Stackup material properties and port electrical behavior do not return. Via pads return as part of the conductor artwork. Circles and slot outlines on a layer assigned the drill role return as holes through the board. Drill PTH and Drill NPTH name their plating, and the import asks for the plating of Drill. Cutouts return as board cutouts. The Ports utility layer is not assigned to the EM model by default.

## Limits and refusal cases

- DXF output is a 2D XY drawing. Stackup order, Z positions, thicknesses, layer types, materials, permittivity, loss, and conductivity are not encoded.
- Missing or corrupt stored raster data refuses the export instead of leaving a blank shape.
- A project with no geometry, a zero-width arc or script path, an invalid annulus, or geometry too small or large for its coordinate magnitude refuses instead of writing a changed or empty shape.
- A via outer diameter smaller than its drill refuses instead of enlarging the pad.
- A hole or cutout a RayRF run refuses also refuses the export, by shape, where the export writes it: no width or area, a parameter its kind does not have, a span that does not end on a conductor or dielectric layer of the stack, or a via that reaches into it.
- Cutouts that cover the board, cut it into separate pieces (a ring of cutouts around an island of board does), or meet the board edge or each other at a single point refuse by shape, because Board Outline carries one board with separate cutouts inside it.
- A failed parametric resolve, invalid shape value, or structural DXF error refuses the output with a named reason.

---

# Exporting openEMS XML

The openEMS exporter writes the current resolved project as a standalone XML setup containing an openEMS FDTD section and a CSXCAD ContinuousStructure. It is visible under File, Export and as a headless project command.


## Exporting from the editor

Select Export openEMS XML under File, Export, then choose an XML file. Preparation and writing run behind a loading overlay, and Cancel stops the export without leaving a partial file. The completed export shows the entity count and any warnings, and the Console prints the outcome. The XML starts with a comment naming the format, the entity count, and the warning count. The CLI's `--json` output carries the full exported, approximated, and omitted report. Unsupported electrical content refuses before the destination is replaced, with the source project, destination XML, and shape, via, port, or layer named in the error.

## Exporting from the CLI

```
rayrf project export-openems --project FILE --out FILE [--replace] [--json]
```

The `.xml` suffix is added when absent. An existing output refuses unless `--replace` is set. Without `--json`, the outcome line goes to stdout and each warning to stderr. JSON output uses `rayrf-openems-export-report/v1` and contains the output path, `entity_count`, all three report categories, and the warnings.

## Setup contract

The writer resolves project variables, bindings, constraints, and generator instances on a copy. Auto-mode simulation values are derived before export, and the report lists the resolution details that affect the setup. When project domain auto-fit is enabled, export recomputes the domain from the resolved geometry with the saved padding, matching the simulation export path. The XML uses millimeters and includes one resolved Gaussian pulse, boundary faces, mesh lines, stackup materials, supported geometry, vias, holes and cutouts, and lumped ports. When mesh grading is off, the grid uses the same whole-cell board, free-space, and absorber counts as the uniform RayRF solver. Stackup planes use the same uniform-cell quantization. The report lists any padded board edge, expanded free-space gap, or shifted stackup plane. A dielectric that collapses to no solver cells refuses by layer name. When grading is on, each stackup edge uses the same nearest explicit Z line as the RayRF solver, and a shifted edge is listed in the report.

A standalone export requires exactly one active port. Every port needs an explicit positive number, reference impedance, and Boolean active value. Point and rectangular ports need both terminal layers. A planar line or bar port needs its bottom layer. An omitted top layer means that same conductor, while an explicitly different top layer refuses. A present planar `excitation_mode` must be `planar`. Point ports excite Z. Rectangular, line, and bar ports must be unrotated and aligned with a Cartesian axis. Their boxes are aligned to the resolved mesh and the effective coordinates are listed as approximated when that changes the authored box. Line and bar polarity follows their authored director. Passive ports contain their resistance and probes without an excitation property.

A via is written in the `RayRF vias` metal as a cylinder of its drill diameter over its conductor span. Its pad is written on each conductor layer it spans as a polygon with a vertex at no more than 5-degree spacing. A via drill RayRF realizes as a post at one mesh node is written as drawn and listed as approximated.

An unplated hole or a cutout is written as vacuum in the `RayRF openings` material: a cylinder for a round hole, a rectangle and a disc at each end for a slot, and the outline polygon for a cutout. Over the copper layers it cuts, the vacuum has a priority above copper, so it removes the copper there. Where the opening ends on copper that stays, the vacuum stops at the center of the first mesh cell short of that copper. The dielectric it cuts is vacuum at a priority between the dielectric and copper. A plated cutout also gets its wall in `RayRF opening walls`: the cell faces RayRF plates on the resolved mesh, written as conductor sheets. Where the drawing keeps a copper plane clear of that wall, the plane edges RayRF clears around it are written as vacuum above copper priority on that plane. The report lists the wall and the cleared planes as approximated.

A plated hole cuts nothing. It is written in `RayRF opening walls` as metal from the lowest to the highest copper layer it joins: a cylinder for a round hole, a rectangle and a disc at each end for a slot. A plated hole RayRF realizes as a post at one mesh node is written as drawn and listed as approximated.

Zero-thickness conductor circles, annuli, and arc bands are written as polygon segments at no more than 5-degree spacing. The report names each segmented shape. Dielectric circles with positive thickness remain cylinders.

The output is prepared beside the destination and installed only after XML readback and structural checks pass. A failure leaves an existing destination unchanged.

Every emitted geometry bound must remain inside both the resolved project domain and the resolved mesh. Geometry that would be clipped by either boundary refuses instead of producing a different electrical model. Script clip ranges must increase on both axes. A missing required shape, via, or port coordinate refuses by shape id instead of placing content at the origin.

## Report contract

Every completed export has three sections:

Category || Meaning
Exported || Setup and geometry represented directly in the XML.
Approximated || Content represented with the stated flattening, segmentation, or substitution.
Omitted || Project content the XML does not carry or a supported export deliberately leaves out.

The target runtime realizes the geometry and materials itself, so its results are not bit-identical to a RayRF run.

Parametric definitions, editor view state, stored results, Run History, and reference traces are not embedded. The current resolved geometry is embedded. Unsupported or degenerate electrical geometry refuses the whole export instead of producing a partial case. A failed export leaves an existing destination unchanged. Dielectric loss tangent is written as conductivity at the layer's own loss reference frequency and listed as approximated. Enabled NF2FF, surface-current, full-field, field-transform, and early-stop requests are named as omitted when the XML does not include them.

RayRF re-import preserves the supported planar geometry, stackup, materials, units, and frequency band. A via returns as a via over the same span with its drill and pad at the drill diameter, and its pads return as polygons on the conductor layers. Re-import inventories ports rather than creating RayRF ports, lists holes, cutouts and their plating as not imported, and rebuilds boundary faces, mesh lines, domain spacing, and automatic run controls through the import workflow.

## Limits

- Resolved maximum steps must be from 1 through 4294967295, the range carried by the target runtime.
- The exporter supports a positive Gaussian band. Continuous-wave run-to-cap stop semantics refuse because a zero ringdown criterion would not retain the same runtime behavior.
- A missing frequency band, invalid mesh, incomplete boundary definition, bad geometry value, unsupported primitive, fuzzy load-time layer remap, or failed project resolve refuses the export.
- Stored raster geometry is written at the image's own resolution, one rectangle per pixel run, and the report counts the rectangles.
- A hole or cutout a RayRF run refuses also refuses the export, by shape: a parameter its kind does not have, a span the stack does not have, a plated opening with no board material around it to plate, or a via that reaches into it.
- Review every approximated and omitted entry before using the XML.

The writer uses the existing project, mesh, and geometry resolution code. It does not require an openEMS or CSXCAD installation. Its built-in checks parse the XML and re-read the supported structure, but they are not a target-runtime load test. Runtime acceptance requires testing the file with the target version.

---

# Ports and excitation

A port is where the simulation feeds energy into the structure and measures the response. This page covers the four port types, the options every port carries, the excitation waveform, and what validate checks before a run.


## What a port is

A port is a shape of kind `port`. Each port is both an excitation source and a measurement site. Its `active` flag decides which: an active port is driven, a passive port is a matched load that the run still measures.

A run drives one port and produces that port's S-matrix column: driving port n yields S1n, S2n, and so on across every measured port. S-parameter extraction requires exactly one excited port per run. Both surfaces read the same rule, so a project with several active ports, with none active, or with no ports at all is refused before the backend launches, naming the ports involved. A full matrix is built one driven port at a time, in the desktop app and in `rayrf run` alike.

!fig The four port types placed on traces in the Edit tab

## Port types

Four port types cover vertical and in-plane feeds, straight from the geometry registry. Every port type also carries the matching-network and measurement-shift keys listed under [Shared port keys](#shared-port-keys).

### `point`

Single-cell vertical probe between two conductor layers.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the feed point.
`y_mm` || number || mm || 0 || yes || Y of the feed point.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`top_layer` || string || || || yes || Positive-terminal conductor layer.
`bottom_layer` || string || || || yes || Negative-terminal conductor layer.

### `rect`

Distributed rectangular (area) feed between two conductor layers.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the footprint center.
`y_mm` || number || mm || 0 || yes || Y of the footprint center.
`w_mm` || number || mm || none || yes || Footprint width.
`h_mm` || number || mm || none || yes || Footprint height.
`port_type` || string || || rect || yes || Must be `rect`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`direction` || string || || z || || Feed axis: z (transverse), x, or y.
`top_layer` || string || || || yes || Positive-terminal conductor layer.
`bottom_layer` || string || || || yes || Negative-terminal conductor layer.

### `line`

In-plane line feed across a gap on a single conductor layer. The feed drives along its director, p0 to p1, at any angle: on an axis it is an X or Y feed, off the axes it drives both in-plane components and measures along the drawn director.

Param || Type || Units || Default || Required || Meaning
`p0_x_mm` || number || mm || 0 || yes || Endpoint 0 X (negative terminal).
`p0_y_mm` || number || mm || 0 || yes || Endpoint 0 Y (negative terminal).
`p1_x_mm` || number || mm || none || yes || Endpoint 1 X (positive terminal).
`p1_y_mm` || number || mm || none || yes || Endpoint 1 Y (positive terminal).
`width_mm` || number || mm || 0.1 || || Strip width perpendicular to the line.
`port_type` || string || || line || yes || Must be `line`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`bottom_layer` || string || || || yes || Conductor layer the feed sits on.
`top_layer` || string || || || || Same layer as `bottom_layer` for a planar feed.
`excitation_mode` || string || || planar || || Must be `planar`.
`deembed_ref_layer` || string || || || || Conductor layer the shifted recording point measures voltage against, required when this in-plane feed carries a measurement shift. It must not be the layer the feed sits on. Refused on a z-fed port, whose `bottom_layer` already names it.

### `bar`

In-plane bar feed on a single conductor layer. The feed drives along its director at any angle: on an axis it is an X or Y feed, off the axes it drives both in-plane components and measures along the drawn director.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || Bar center X.
`y_mm` || number || mm || 0 || yes || Bar center Y.
`length_mm` || number || mm || 1 || yes || Gap length along the feed director.
`width_mm` || number || mm || 4 || yes || Transverse bar width.
`angle_deg` || number || deg || 0 || || Director angle from +X, any value. The feed drives along it, and off the axes the gap is realized as a staircase on the mesh.
`port_type` || string || || bar || yes || Must be `bar`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`bottom_layer` || string || || || yes || Conductor layer the feed sits on.
`top_layer` || string || || || || Same layer as `bottom_layer` for a planar feed.
`excitation_mode` || string || || planar || || Must be `planar`.
`deembed_ref_layer` || string || || || || Conductor layer the shifted recording point measures voltage against, required when this in-plane feed carries a measurement shift. It must not be the layer the feed sits on.

### Shared port keys

These keys are the same on all four port types:

Param || Type || Units || Default || Meaning
`network` || object || || none || Matching ladder between the source and the port terminal, written system to device: `{branches: [{orient: series or shunt, parts: [{kind, value, q, q_ref_hz}]}]}`. `kind` is L, C, or R. `value` is H, F, or ohm as a number or an expression over variables. `q` is a required positive quality factor on L and C, refused on R. `q_ref_hz` is the required frequency in Hz that q is exact at. A resistor dissipates the resistance you author. Absent means none.
`deembed_mm` || number || mm || 0 || Distance the point where voltage and current are recorded moves from the feed toward the device, along `deembed_dir`. The recorder moves during the run: nothing is corrected afterwards, and the port still injects where it sits. 0 records at the feed. This is the technique RF practice calls de-embedding.
`deembed_dir` || string || || || Direction the recording point moves: one of x+, x-, y+, y-. It moves in a straight line, so the trace has to run straight through the whole shift. A landing point that crosses no metal is refused at the start of the run. A trace that curves away is not refused, and leaves the recorder reading whatever other copper is at the landing point. An in-plane feed drives the trace it is cut into, so only its own axis is legal.
`network_plane` || string || || system || Reference plane for the reported S-parameters when this port has both a matching network and a measurement shift: `system` keeps the network in the reported reflection, `deembed` reports the device alone at the shifted recording point. Ignored on a port that lacks either.

Placement notes the table does not carry:

- A **point** port is a single-cell vertical probe. It drives the gap between a `top_layer` and a `bottom_layer`, so the two must be different conductor layers. Use it as the default lumped feed on a narrow trace. The point and area placement dialogs label `top_layer` Positive layer and `bottom_layer` Negative layer.
- A **rect** port spreads the same vertical feed across a footprint you drag out. Use it when a single cell is too coarse for the conductor, such as a wide trace or a patch edge. Its `direction` sets the feed axis: z feeds vertically between `bottom_layer` and `top_layer`, x or y feeds in-plane along that axis.
- A **line** port is an in-plane feed across a gap on one layer, set by two endpoints: p0 is the negative terminal, p1 the positive, and current runs p0 to p1. Use it for a coplanar or edge feed where both terminals sit on the same layer. The endpoints come from the two-click placement, not a dialog, and snap to the nearest axis.
- A **bar** port is a fixed in-plane bar on one layer, placed with a single click. Use it when you want a set-orientation in-plane gap rather than two picked endpoints. The bar's director angle is free: on an axis the bar is an X or Y feed, off the axes the solver drives and measures along the drawn director and realizes the gap as a staircase on the mesh.

The point and line dialogs set the electrical options and the layers:

!fig The point port placement dialog

!fig The in-plane line port placement dialog

## Common options

Every port carries the same electrical options.

- `port_number` names the S-matrix column, so keep it unique across the project. Two ports sharing a number leave the S-matrix mapping undefined, which validate refuses. A new port takes the lowest unused number.
- `impedance_ohm` is the reference impedance the S-parameters normalize to, 50 by default. It must be positive: the wave math divides by its square root.
- **Measurement shift**, a checkbox in the port dialog, moves the spot where the run records this port's voltage and current, on any port type. This is what RF practice calls de-embedding, and the stored keys keep that name. Turn it on, then set `deembed_mm` to the distance and `deembed_dir` to the direction (x+, x-, y+, or y-) pointing from the feed toward the device. Off, the port records at the feed. The matching network sits behind its own checkbox at the bottom of the same dialog. Use it to leave a length of feed line out of the reported impedance and S-parameters. The distance cannot be negative, as the direction control sets the sign. On a port that also carries a matching network, the shift moves the recording spot past the network as well, so `network_plane` chooses which plane is reported.

The run samples voltage and current at the shifted position from the first timestep instead of correcting feed measurements afterward. The excitation remains at the port.

The recording spot travels down the line the feed drives, and that is what decides which directions a port accepts. A z-fed feed sits between a strip and the conductor under it, so the line it drives runs in-plane and all four directions are legal. An in-plane feed sits in a gap cut across a trace and drives that trace, so only its own axis is legal. A spot shifted across such a feed would leave the trace, and the editor does not offer it.

A measurement shift requires a straight conductor of constant width. At run start, RayRF checks the realized copper cell by cell from the feed to the recording position. The run is refused by port name if the position is on bare substrate or disconnected metal, or if the path includes a bend, taper, or junction.

An in-plane feed also has to name the conductor its shifted voltage is measured against, in `deembed_ref_layer`. It sits on one layer, so unlike a z-fed feed it carries no second terminal of its own. At the shifted spot the reported voltage is the one between the trace and the return conductor, usually the ground plane under it. It cannot be the layer the feed sits on. A z-fed port refuses the key, since its `bottom_layer` already names that conductor. With more than one conductor on offer the editor asks rather than choosing: the control starts on "Choose layer", and a port left that way is refused by name at the run decision.

The shift lands on a mesh line, so what the run records is a whole number of cells from the feed and not the distance you typed. The property panel says where it lands in its Records at row, estimate prints the same number per port under records at, and the run log and the per-port network-loss file carry it as well.

The shift rows for a line or bar port live in the Edit tab's property panel, not in its placement dialog. Those two ports get their director from the placement gesture, so while the dialog is open the port has no axis yet and a direction control there could not know what it was offering.

Ports serialize into the solver input as lumped feeds: a point port becomes a `lumped_point`, the others a `lumped_rect`, with the impedance carried as `R_ohm`. An in-plane line or bar on an axis emits as an X or Y feed. Off the axes it emits as an xy feed carrying its unit director and one footprint mask per in-plane component, and the backend drives both components along the director.

## Excitation

The source is a Gaussian pulse spanning the frequency band, from `freq_min_hz` to `freq_max_hz`. One broadband pulse excites the whole band in a single run, and the S-parameters come from the ratio of the transformed port voltages and currents.

Continuous-wave mode replaces the pulse with a steady sinusoid at one frequency. It never rings down, so the run ends at its step count. Under a single tone the S-parameters are of limited meaning, while the radiation pattern at the tone is meaningful. Both outputs stay available, and field animations are the usual reason to use it. CW is an advanced-mode feature, and an auto-mode project with CW on is refused rather than run as a pulse. See [FDTD tips](docs-fdtd-tips.html).

## What validate catches

Run `validate` before a run. For ports it reports:

- A zero-height span, as an error. A point or rect port whose top and bottom conductors land on the same realized z mesh line collapses to no vertical extent and injects nothing. Setting `top_layer` equal to `bottom_layer` lands on one line by definition, so it is refused the same way.
- No active port, as an error. With nothing driven there is no excitation and no S-parameters, and a project with no ports at all, or with more than one active port, is refused the same way.
- A port that maps to zero cells or lies outside the domain, as an error, and a non-positive impedance or a negative shift distance, as an error.
- A duplicate `port_number` or an unrecognized `deembed_dir`, as an error, and so is a shift running across an in-plane feed rather than down the trace it is cut into, or one on a feed with no extent at all, which names no axis.
- A shifted in-plane feed that names no return conductor, or names one this project has no conductor layer for, as an error.
- A port with no copper under it on one of its reference layers, as an error. Copper that a hole or cutout removes does not count.

Whether the trace actually runs under the shift is not something validate can see: it depends on the mesh, so the run checks it at the start and refuses there instead.

---

# Auto mode and quality

Auto mode derives the mesh, CFS-CPML absorber, air margins, ring-down criterion, and output-capture bins from the frequency band, quality level, stackup, and copper geometry. The derived-settings panel shows the rule used for each value.


## Auto versus advanced

Auto mode derives dx, dy, dz, PML depth, per-face air margins, ring-down, and the convergence targets from a single quality slider plus the measured geometry, and never coarsens the mesh to make a run fit: if it will not fit, the run gate refuses it with a reason. [Advanced mode](docs-advanced-mode.html) sets the same parameters by hand. The mode toggle lives at the top of the Simulate tab.

## Frequency band

The Frequency Range card takes the band two ways. F min / F max, the default, enters the two band edges directly. Centre + BW enters a center frequency and a full bandwidth. The button pair switches between them, and both write the same stored band, so a value typed one way carries over to the other.

The band is stored in Hz and shown in GHz, and the CLI flags take MHz, covered on [FDTD tips](docs-fdtd-tips.html). A non-positive or inverted band is refused downstream rather than repaired: auto derivation needs `f_max > f_min > 0` and the run gate blocks a project without a valid band. f_max sets the mesh cell size (finer at higher frequency), and f_min feeds the pulse and the absorber sizing. The Derived Settings panel reports the radiation-pattern and surface-current frequency resolution as the band span divided by one less than the number of capture points.

## The quality slider

Quality is a continuous value from 0.0 to 3.0 with four named stops: Low, Medium, High, and Very High at 0, 1, 2, 3. The value interpolates between the stops, so 1.5 is halfway between Medium and High. One value drives every accuracy knob at once:

- Cells across the narrowest measured conductor width and gap on each in-plane axis, and cells per wavelength.
- Cells through the thinnest dielectric (the dz resolution).
- CFS-CPML absorber depth, floored at a per-axis cell count.
- The energy ring-down threshold that ends the run.
- The air-margin cell counts on every face.
- Radiation-pattern, surface-current, and field-FFT frequency-bin counts.
- The convergence-study thresholds and pass count.

Raising quality refines the mesh, including narrow conductor gaps, deepens the absorber, tightens ring-down, and adds capture bins. These settings do not depend on which outputs are enabled. Toggling radiation-pattern export does not change the physics setup or move a resonance.

## Per-axis derivation

Each in-plane axis is sized on its own measured copper. For an axis, the cell is the finest of: the guided wavelength at f_max over the cells-per-wavelength count (guided by the largest stackup permittivity, `lambda / sqrt(eps_r)`), the narrowest measured width on that axis over the width-cell count, the narrowest measured gap over the gap-cell count, and edge fringing from the thinnest dielectric height. A cross-axis feature guard then bounds an axis that carries no measured feature of its own by the finest cell any measured feature on the other axis demands, so a notch invisible to a scan along one axis still resolves. dz is the through-substrate rule: the thinnest dielectric over the dielectric-cell count, capped by the free-space cell. The estimate and derived panels show which rule set each axis.

Unplated holes, slots and cutouts are cut out of the copper before it is measured, so a web of copper between two holes is measured like any other copper. A plated hole's pad is measured as drawn. The size of a hole, a cutout or a via drill never sets the mesh. The derived-settings summary reports how the mesh realizes them instead.

## Minimum feature per axis

The Min feature X/Y/Z fields set a resolution floor per axis, each independent. On X and Y the value is both a floor and a measurement threshold: set it, and the mesh refines that axis to resolve it, while measured copper below the value does not drive the mesh and is reported as a warning naming the feature, its location, and the consequence. Left at 0, an in-plane axis takes no feature-size constraint: measured copper only warns, and the cell stays at the cells-per-wavelength and dielectric baseline. Z is a pure floor with no threshold behavior, and stackup layers are always resolved regardless. Below the fields, an Auto-detected minimum line reports the narrowest measured feature per axis, and Use suggested copies the values in: X and Y from the measured copper, Z from the thinnest dielectric layer.

## The derived-settings summary

The Derived Settings panel reports the full result: the per-axis base cell and what drove it, the resolved mesh dimensions with cell count and VRAM, the per-face air margins, the PML depth in cells, the ring-down threshold and runtime estimate, and the enabled outputs with their bin counts. A Constraint Traceability block lists one line per derived value naming the rule behind it, with the mesh-scale driver marked. When the probe measured features the mesh does not resolve, a Mesh Feature Findings block gives their count, then one line per feature, finest first: size, kind, axis, cells across against the target, layer, and location. The first five show and the rest fold under a toggle. Hover a line for the full finding. A feature too small for any mesh of the domain is listed last and marked that way, and Use suggested skips it. A Per-term Mesh Cost block reports the cell count the mesh would have with each measured term excluded.

An Opening Findings block lists the holes, slots, cutouts and via drills the mesh realizes differently from the drawing, one line per group of identical ones. A via drill or plated hole under 2 cells across still conducts, as a coarse barrel of mesh nodes or a single conductor post. An unplated hole under 2 cells across opens a few cells, and a cutout whose cells do not join is joined by extra cells along its outline. The block also counts plated openings and via pads that clear a copper plane by less than a cell, which the mesh holds one cell edge from that plane. Hover a line to see what resolves it: the min feature and the cells and VRAM that costs, the cell size alone where explicit mesh lines set the cells, or why no cell size does. The pre-run check and `estimate` report the same findings. When the export cannot realize an opening or a via, the block names the reason, and `validate` refuses the project with it.

The last line of the Quality & Geometry card reads the grid, the cell count, and the memory the run needs on the selected backend: VRAM on the GPU, or RAM when Use CPU backend is checked. With Run remotely on, it follows the Backend chosen for the runner. The CLI mirrors it:

```
rayrf estimate --project patch.rfsim --quality medium
```

`estimate` derives the same mesh and prints the cell count, memory, boundary sizing, ring-down, runtime cost, the measured per-axis suggestion, and the same advisory warnings and notes, without launching the solver. `--quality` accepts a name or a number in 0 to 3.

## Convergence study

The Perform Convergence Study checkbox, which writes `auto_convergence_enabled`, runs the solver several times with an increasingly fine mesh until the tracked resonance dips stabilize, instead of a single pass. The frequency-shift and depth thresholds follow the quality level, and each pass refines the mesh by a fixed factor up to a pass cap. This costs several full runs. The threshold, refine-factor, and pass-cap fields are in the [settings reference](docs-settings.html).

---

# Advanced mode

Advanced mode hands you the mesh, air margins, PML depth, run length, and convergence criteria that [Auto mode](docs-auto-mode.html) derives for you. Boundary face types live on [Boundaries](docs-boundaries.html). This page covers everything else the Advanced toggle exposes.


## Switching to Advanced

Select Advanced on the Simulate tab to set `dx_mm`, `dy_mm`, `dz_mm`, air margins, PML depth, and run length manually. The run uses these values without snapping or re-deriving them.

Advanced values persist in the project independently of the auto values. Every Advanced field has an `adv_*` shadow, so a project saved in Auto mode keeps its Advanced entries and reopening in Advanced mode restores them. Transfer to Advanced Mode, on the Derived Settings card in Auto mode, copies the current auto-derived mesh, margins, PML, and boundary preset into the Advanced fields and turns on Max steps at the auto step cap. You edit from there.

## Manual mesh

The Mesh / Cell Size card sets the three base cell sizes.

`dx_mm`, `dy_mm`, and `dz_mm` are the Yee cell dimensions in millimeters, default `0.2` each. The run uses them exactly. [FDTD tips](docs-fdtd-tips.html) covers picking cell sizes.

If `dz` does not evenly divide the conductor stackup height, the substrate lands between grid lines. `validate` warns (`dz does not divide the substrate height`) and reports the closest fitting dz without changing your value. See [FDTD tips](docs-fdtd-tips.html) for choosing dz.

## Mesh grading

Grading keeps fine cells in the board region and lets them grow toward the air and PML, cutting total cells while holding resolution where the fields are. It is what the Ratio grading strategy does, and it is the default. The controls:

- **Refine margin** (`mesh_grading_margin_mm`, default `1.0`): how far the fine-cell region extends past the board in every direction.
- **Per-axis multiplier** (`mesh_grading_{x,y,z}_mult`, default `1.0`): the board-region refinement factor, `1.0` meaning no refinement.
- **Per-axis max ratio** (`mesh_grading_{x,y,z}_max_ratio`, default `1.3`): the largest cell-to-cell size step allowed in that axis' transition. Must exceed `1.0`, or no transition can satisfy it.

## Air margins

The air gap between the geometry and the boundary has three modes, selected by the Air Margins mode combo.

- **Manual (cell count)** (`air_cells`, default `6`): the same number of air cells on every face. Keyed to the mesh, so the physical gap follows the cell size.
- **Manual (mm)** (`air_around_mm`, `air_above_mm`, `air_below_mm`, default `10.0`): each face set in mm directly.
- **Manual (frequency)** (`auto_air_sides_factor`, `auto_air_above_factor`, `auto_air_below_factor`, default `0.25`): each face sized as a fraction of the wavelength at `f_min`.

PML depth has its own three modes (`pml_cells_manual` default `8`, `pml_thickness_mm` default `4.0`, `auto_pml_factor` default `0.10`), set on the Boundary Conditions card and covered under [Boundaries](docs-boundaries.html).

## Run length

The Energy Ringdown card sets when the run stops.

The ring-down criterion is the remaining field energy relative to peak at which the solver stops. It is edited in dB and stored as a ratio (`ringdown_end_crit`), default `-50 dB` (ratio 1e-5). More negative is a longer, more converged run. The floor is `-120 dB`, since the control is logarithmic and zero has no dB value. The criterion is checked every 250 steps by default. Every run keeps a built-in floor of two excitation lengths before the criterion may stop it: one length is the source injection itself, and the one that follows lets the source transient leave the port before the energy is judged. The Minimum steps field shows that floor greyed, worked out from the band and the timestep, while its checkbox is off. Tick Minimum steps to raise the floor when the model needs a longer observation window. Then `ringdown_min_steps` carries your value, and a value below the built-in floor has no effect.

Max steps adds a hard timestep cap (`adv_max_steps`). Leave it off and the run has no cap and stops on ring-down alone. If the cap is hit before ring-down, the time record is truncated before the fields settle, so the S-parameter and radiation results are not converged. See [FDTD tips](docs-fdtd-tips.html).

S-param early stop (`sparam_early_stop`, off by default) stops the run once the S-parameters stop changing by more than `sparam_delta_crit_db` (default `-40 dB`) between checks. The threshold must be negative and does not enable or disable the criterion. `sparam_early_stop` is its sole enable. `sparam_min_steps` (default `10000`) is a floor so it cannot fire during the source excitation transient.

## Mesh convergence

The Mesh Convergence Study checkbox on the Mesh / Cell Size card, which writes `auto_mesh_enabled`, runs a multi-pass study that refines the cell size until the tracked resonances stabilize. It starts from the card's cell sizes and refines only the mesh each pass. The study settings are on the same card, and the Mesh Convergence plot beside the runtime stats tracks each pass.

- **Source** (`conv_source`, default `S11`) and **resonance count** (`conv_num_resonances`, default `1`): which S-parameter and how many of its deepest dips to track, matched between passes by rank.
- **Freq shift** (`conv_freq_threshold_db`, default `-20 dB`) and **depth diff** (`conv_depth_threshold_db`, default `-20 dB`): the per-resonance thresholds a pass must meet to count as converged. `-20 dB` is about 1 percent.
- **Mesh step** (`conv_mesh_refine_factor`, default `1.26`): the division factor each pass. 1.26 is about cbrt(2), roughly doubling total cells.
- **Max passes** (`conv_max_passes`, default `8`): the hard iteration limit.

## CLI equivalent

Every field above is a project setting reachable headless: override one with `--setting <name>=<value>` or carry the whole set in a JobSpec. See [Scripting and specs](docs-cli-scripting.html). The full field list with units and bounds is on [Settings reference](docs-settings.html).

---

# Mesh regions

A mesh region is a span of one axis with its own cell size, used to refine selected areas and coarsen the surrounding mesh. RayRF reports cell count, time step, and estimated memory before the run.


## Two ways to grade a mesh

The `Mesh strategy` control on the Simulate tab selects either Ratio grading or Custom profile for the project.

- **Ratio grading** is the default and needs no drawing. You give a per-axis refinement multiplier for the board region and a maximum cell-to-cell ratio for the transition out to the air, and RayRF builds the regions for you.
- **Custom profile** is the drawn mesh. You place mesh lines by hand in Mesh View, give each span between them a multiplier, and RayRF realizes exactly what you drew.

Use Ratio grading when the geometry is one board and the interesting fields are on it. Use a Custom profile when you know where the fine cells belong: a coupled edge, a narrow gap, a via barrel, the layers of a stack that differ by an order of magnitude in thickness.

## Ratio grading

The Mesh / Cell Size card in Advanced mode carries the whole control set. In Auto mode the Quality & Geometry card has the Mesh strategy selector only.

- **Refine margin** (`mesh_grading_margin_mm`, default `1.0`): how far the fine-cell region extends past the board in every direction.
- **Per-axis multiplier** (`mesh_grading_x_mult`, `_y_mult`, `_z_mult`, default `1.0`): the board-region refinement factor. `1.0` means no refinement, so a fresh project is uniform until you raise one.
- **Per-axis maximum ratio** (`mesh_grading_x_max_ratio`, `_y_max_ratio`, `_z_max_ratio`, default `1.3`): the largest step in cell size allowed between neighboring cells in that axis' transition. It must exceed `1.0` or no transition can satisfy it.

The max-ratio row for an axis, and the refine margin, appear once at least one axis carries a multiplier above `1.0`. An axis at `1.0` has no transition, so its ratio has nothing to bound.

## Custom profiles and Mesh View

Mesh View overlays the Custom profile on the geometry in the Edit tab. Open it from either location:

- **Open Mesh View** on either Simulate mode card, shown while the Custom profile is the selected strategy, because Mesh View edits the Custom profile and nothing else.
- **Mesh View** under the layer list in the Edit tab's Stackup section. If Ratio grading is selected, confirm the switch to Custom profile to continue.

Inside Mesh View you work on the two rails at the edges of the viewport. Click a rail to place a mesh line, drag a line to move it, and click the badge on a span to set that span's multiplier. A line can be constrained to a primitive, and then it follows when the primitive moves, in the editor and through the CLI alike: right-click the pin for Constrain, or arm Vertical, Horizontal, Dimension, or Fix from the Constraints panel and click the pin, on its rail or anywhere along its line, as one of the participants.

The Mesh Controls section is where Z lives. While Mesh View is on it takes the place of the layer list in the left panel: it holds the stackup drawn against its realized mesh lines, the per-layer multipliers in physical order from the air above the stack to the air below it, the per-axis max ratios, and a boundary row for every face. Clicking a layer bar selects that layer, dragging a bar reorders the stack, and New Layer and Delete Selected sit under the bars. Right-click a layer bar for the usual layer actions, double-click to rename it, or press Del while the bar is selected to remove it. Click inside its multiplier box to edit the multiplier. Dielectric and air-gap layer multipliers both affect the realized Z mesh and cell count. A boundary label in the viewport opens the same type and spacing fields on a double-click. The free-space rows and the dielectric rows report the cells the mesh actually realized, and a boundary edited here is the same setting the Simulate tab's Boundary Condition matrix edits. A spacing left blank on a checked face turns its box red. The footer button returns to the mesh card on the Simulate tab. When the profile cannot be realized, the last mesh that could stays on screen, the reason sits in red under the cell readout, and the pin or multiplier it names is painted red. A clicked pin deletes with Del or Backspace.

## What the numbers mean

The Mesh View heads-up display shows four facts, and `rayrf mesh explain` prints them:

```
grid           : 104 x 104 x 27 = 292,032 cells
cell range     : 0.304 mm minimum, driven by X region 0
timestep       : 6.02592683e-13 s
memory         : 0.36 GB VRAM, 0.11 GB RAM
```

Both Simulate mode cards show the grid, the cell count, and the memory for the selected backend.

The grid, the cell count, and the time step are exact. They are not a forecast of the mesh: they are the mesh, generated by the same code the exporter runs, so the number on the card is the number the solver receives. `mesh diff` proves it by comparing a project against the case that was exported from it.

Memory is the one number that is not exact. It starts from the exact padded grid and adds mesh storage, the CPML shell, whatever captures the run requested, a fixed allowance, and headroom, because allocator behavior is not one fixed count.

The cell range names which region owns the smallest cell, which is the region that sets the time step. Halving one span's cells doubles the run length of the entire simulation, so know which span you are paying for.

## Where to put a mesh line

Every pin becomes a mesh line, so pinning a feature edge is how you make the solver see that edge exactly where you drew it. Pin freely for that.

What matters is where the cell size *changes*. A pin between two regions with the same multiplier is only a line, and putting it on a conductor edge is fine. A pin between two different multipliers is a transition, and a transition sitting on a conductor edge biases the result, because the cells on one side of that edge are a different size from the cells on the other. RayRF warns when a primitive edge lands within half a local cell of a multiplier change, and names the primitive and the position. The fix is to extend the fine region past the feature so the step happens in empty space.

## The maximum ratio is a maximum

RayRF grades each transition at or below the configured maximum cell-to-cell ratio. If a region is too short, the run is refused with the axis, region, and reason. When a higher ratio would fit, the error gives that ratio. Increase the maximum to the reported ratio, widen the region, or adjust its multiplier to reduce the cell-size change.

Refusals appear in the Notices tab beside the console in the desktop app, and on stdout from `validate` and `run`.

## From the CLI

The whole feature is authorable and inspectable headless. Mesh lines, multipliers, and constraints are part of the project spec, so `project build` and `project export-spec` round-trip them like any other authored content.

```
rayrf mesh explain --project patch.rfsim              regions, cost, timestep owner
rayrf mesh lines --project patch.rfsim --axis z       the exact line positions
rayrf mesh diff --project patch.rfsim --against DIR   project against exported case
rayrf estimate --project patch.rfsim --json           exact cells, memory
```

`mesh explain` reports every region on every axis with its span, multiplier, target cell size, realized minimum and maximum cell, and cell count, so a profile can be checked without opening the desktop app. `mesh diff` tolerates only serialization-scale drift, so it answers whether two meshes are the same mesh rather than whether they look similar.

## Mesh regions and convergence

A convergence study refines the mesh each pass and re-solves until the S-parameters stop moving. It starts from the mesh you built, Ratio or Custom, so a profile is the starting resolution rather than something the study replaces. See [Advanced mode](docs-advanced-mode.html) for the study controls.

---

# Boundary conditions

The simulation sits in a box of six faces, and each face carries one of three boundary conditions. This page covers the types, the presets, the per-face custom matrix, the spacing between the geometry and each face, and the PML depth controls.


## The three types

Type || Meaning
PML || Perfectly Matched Layer (CFS-CPML), the only absorbing boundary. Use it for radiating or open problems.
PEC || Perfect Electric Conductor hard wall (tangential E = 0), a closed metal box.
PMC || Perfect Magnetic Conductor hard wall (tangential H = 0), the dual of PEC.
!caption Presets: PML, PEC, PMC, Custom. Faces: x_lo, x_hi, y_lo, y_hi, z_lo, z_hi.

PML, a CFS-CPML absorber, is the only absorbing boundary. Outgoing waves enter the layer and decay instead of reflecting, which makes it the boundary for antennas and any radiating or open problem. PEC is a hard electric wall (tangential E forced to zero): a closed metal box that reflects everything reaching it. PMC is the dual hard wall (tangential H forced to zero) and serves as a symmetry plane. [FDTD tips](docs-fdtd-tips.html) covers giving each hard-wall face an explicit spacing.

Radiation capture (NF2FF) works with any face mix that leaves at least one PML face. A box with no PML face, or a PEC or PMC pair on both faces of one axis, is refused. See [Radiation pattern](docs-radiation-pattern.html).

## Presets and per-face control

The Type selector on the Simulate tab sets all six faces at once: Open Air (PML), PEC box, PMC box, or Custom.

!fig Boundary condition panel with the Open Air preset

Custom reveals a six-row matrix, one row per face (+Z, -Z, +X, -X, +Y, -Y), each with its own type and its own spacing field.

!fig Custom per-face matrix mixing PMC, PEC, and PML faces

From the CLI the same control is the repeatable `--per-face-bc` flag. FACE is one of `x_lo`, `x_hi`, `y_lo`, `y_hi`, `z_lo`, `z_hi`, and any per-face setting forces the preset to Custom:

```
rayrf run --project p.rfsim --per-face-bc z_lo=PEC:0 --per-face-bc x_lo=PMC:4
```

## Spacing between geometry and each face

On the CLI, set a hard wall and its fixed gap together on the run line:

```
rayrf run --project p.rfsim --per-face-bc z_lo=PEC:2.0
```

The PEC and PMC box presets keep the derived margins on every face. Every face has an air gap between the bounding box of the geometry and the wall. Two regimes exist per face:

- **Derived:** with the spacing checkbox unchecked, the face inherits the air margins (auto mode sizes them by cell count with a substrate-height floor). The field shows the derived gap grayed out as a placeholder, never as an editable value. This sizing exists to give a PML face room to absorb.
- **Fixed:** with the checkbox set, the gap realizes exactly as entered, in mm, overriding the margins on that face. A fixed gap of 0 puts the wall flush against the geometry, which turns a PEC bottom face into an infinite ground plane.

A PEC or PMC face in the Custom matrix requires an explicit spacing. A hard wall that inherits the PML-sized margin can end up against, or inside, the geometry, reflecting energy back into the design and corrupting the S-parameters. The GUI does not let that happen: for a PEC or PMC face the spacing checkbox locks on and the field stays marked required until you enter a value. A Custom-mode case that reaches export with a hard-wall face and no spacing is refused with an error naming the face, so the run never starts on an unspecified gap. The PEC box and PMC box presets are the exception: they keep the derived margins on every face.

## PML depth

`pml_thickness_mm` is the authoritative absorber depth. The panel offers three ways to set it: a manual cell count (default 8 cells), a manual thickness in mm, or an automatic fraction of the wavelength at f_min (default 0.10 λ). A resolved cell count of 0 derives the count from the thickness at export.

The PML is added outside the air margin. It extends the domain rather than consuming the gap, so it never overlaps the geometry. Deeper PML only increases total cell count. Air margins themselves are covered under [Advanced mode](docs-advanced-mode.html), and every boundary and spacing field is listed in the [settings reference](docs-settings.html).

---

# Running simulations

Run Simulation exports the case and launches the solver. This page covers the run path and its pre-flight gate, choosing the CPU or GPU engine, the live telemetry a run shows, where results land when it finishes, and Live mode.


## Run Simulation and Preview Mesh

The two actions are in the left rail under the mode buttons. Both need a saved project, and both save a dirty project before they start. Benchmarks for this computer and configured remote runners are under Settings then Benchmark. They use bundled geometry instead of the open project. See [Benchmarking](docs-benchmarking.html).

**Run Simulation** exports the project to a solver case and launches the backend. Before anything launches, the same pre-flight gate the CLI uses (`sim.preflight`) validates the project. A blocking error refuses the run and names the reason: a port too small for the mesh, a port whose vertical span collapses to zero cells, a collapsed shape that would rasterize blank, an unknown boundary type, a negative air margin, a radiation pattern with no open face. Nothing is silently clamped or recomputed. A warning does not block: it is shown before the run so you decide. The gate sizes the run from the exact grid, then models what it will actually allocate (the fields, the absorber, whatever captures you turned on, and headroom), and refuses a GPU run whose modeled VRAM exceeds the detected card, or a CPU run whose modeled footprint exceeds free RAM.

**Preview Mesh** validates the project and displays the 3D FDTD mesh without running the solver. While the preview is open, the left rail's LAST RUN block reads MESH INFO and shows the preview status and the cell count of the generated mesh. Hover the count to see the grid dimensions and time step. With a convergence study on, a note says the preview shows the first mesh pass, which the run then refines. Back to simulate returns the run controls and the last run's details. Previews of 20 million cells or more require confirmation because they can take minutes to build and render. If the mesh cannot be realized, the preview reports the reason and stops.

The 3D preview uses the same upright navigation as Field Viewer and Radiation. Left drag orbits. Right drag, middle drag or Shift plus left drag pans. The wheel zooms toward the point under the pointer, which stays under it. Each orbit starts around the closest board surface at the center of the view and keeps that pivot for the drag. A miss keeps the last valid pivot, starting at the board or scene center. Orbit sensitivity is 0.25 degrees per logical pixel. The cursor is hidden during a drag and returns to where the drag started. A drag keeps going past the edge of the viewport. Release the button or press Esc to end it.

Show meshing controls the cell outlines, and Photorealistic switches between material colors and lit cream dielectric with rough copper. The two checkboxes work independently. New projects start with mesh outlines on and photorealistic shading off, keeping the dielectric translucent. The project saves these choices separately from Field Viewer and Radiation.

## GPU and CPU engines

The CUDA GPU backend is the default. At startup the app probes for a compatible NVIDIA GPU. When one is found, its name and VRAM print to the console. When none is found, the run falls back to the CPU backend (OpenMP): the Use CPU backend checkbox locks on for the session and a banner explains the fallback. The same fp32 solver runs on either engine and produces equivalent results to numerical precision. The CPU is significantly slower.

At CPU startup, short segments of the actual simulation measure candidate schedules. The search has a fixed step budget and locks the selected schedule for the rest of the run. These steps count toward the simulation.

Both engines step in an internal unit scale: lengths in 2^k metres and times in 2^k seconds, with k chosen so the mesh's finest cell lands in the one range the solver is validated at (0.12 to 0.24 mm). Every run takes that path, so a structure a million times smaller or larger than a circuit board runs with the same fp32 magnitudes a board does, through the same code. Because the scale is a power of two it moves float exponents only, and results agree across exponents to fp32 rounding. The run manifest records the exponent as `unit_scale_exp`.

Check Use CPU backend yourself to run on the CPU with a compatible GPU present. If startup has not reached a definitive hardware verdict when Run is clicked, the run gate offers the CPU backend and can remember that choice until the GPU hardware changes. A remembered choice is keyed to the GPU fingerprint, so a driver or card change re-prompts.

Selecting a specific GPU is a remote concern: a run on a runner names one of the devices the runner exposes (see [Remote runners](docs-remote-overview.html)). A local run uses the machine's own GPU, or the CPU when the checkbox says so.

## Live telemetry during a run

While the solver runs, the title bar shows elapsed time and throughput in GCell/s, and the runtime panel updates in place.

- **Energy Ringdown** plots the energy ratio against time step on a log axis, with the stop threshold drawn as a dashed line. The run ends when the ratio crosses it.
- **S-Parameter Convergence** overlays the S-parameter estimate from recent passes so you can watch it settle.
- **Runtime Stats** reads out elapsed, time step, GCell/s, cell size, ringdown, cell count, RAM, VRAM, and disk. A VRAM overflow raises a warning here.

Two buttons are below the stats. **Pull Snapshot** exports the results computed so far from a local or remote run without stopping: the solver pauses briefly to write S-parameters and enabled radiation data, then resumes. Remote snapshots are downloaded while the job continues. Field-frame snapshots are not supported. The loaded snapshot is marked partial on the ringdown plot. **Stop Early** ends the run before the energy criterion, and asks what to do with the work so far: Stop and Keep Results flushes a partial run and keeps it, or Discard All Results throws it away. A kept partial run is recorded as stopped, never as a clean completion.

## When a run finishes

After timestepping finishes, the title bar reads PREPARING RESULTS. The Simulate and result tabs show the current stage: extracting S-parameters, exporting fields, computing surface currents or radiation patterns, saving, or loading results. Radiation processing also shows completed and total frequency counts. The status stays visible until the results are ready.

A run ends when the energy ringdown crosses its threshold or hits the configured step and ring-down caps, and a capped run is not converged ([FDTD tips](docs-fdtd-tips.html)). Results land in the project: the S-Parameters and Simulate tabs always update, and the Radiation Pattern and Field Viewer tabs update when their outputs were enabled. Each tab that got new output shows a dot until you open it. When the project has result externalization on, the heavy run output is written to a `.rfsimout` sidecar next to the project instead of inside the `.rfsim`.

The result tabs show a badge only when the data on screen needs a warning: it comes from an earlier run than the latest one, the geometry or the config changed since the run that produced it, or that run stopped before its ring-down target. Switching between auto and advanced mode keeps the existing results, and the badge appears only when the effective settings differ from the run.

## Live mode

Live mode reruns the simulation automatically as you edit, for fast iteration on small designs. Enter it with the LIVE button in the tab row. The tabs reparent from the tab bar into a 2x2 dock grid: Edit above Simulate on the left, S-Parameters above the Field Viewer on the right, with Radiation tabbed onto the Field Viewer. You can rearrange the grid and the layout persists with the project. Markers placed on the S-parameter plot keep tracking across reruns.

Every geometry or settings edit schedules a rerun after a 120 ms debounce, so a burst of edits fires one run, not one per keystroke. An edit that arrives while a run is in flight supersedes it: the running solver is stopped and relaunched on the newer input, and the superseded run is discarded rather than counted. Live mode auto-runs with no Run dialog, so it falls back to the CPU backend when no GPU was found, and it disables itself after two consecutive failed runs. Leave Live mode with the same LIVE button.

## From the command line

`rayrf run` runs the same solver headless, and `rayrf serve` and `rayrf remote` drive runs on another machine. On the Simulate tab, Run remotely shows the Run on runner list and the Backend list under it, and the run goes to that runner. Configure remotes under the lists opens the runner manager. See [Remote overview](docs-remote-overview.html). The full verb and flag list is in the [CLI reference](docs-cli-reference.html).

---

# Benchmarking

The performance benchmark measures CPU and GPU solver throughput with bundled, scalable geometry. It does not read or change the open project. The same plan, memory gates, result format, and sharing controls apply to this computer and to configured runners.


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

The result page names the hardware, plots update cells against settled throughput for each backend, and tables the case, backend, realized grid, completed steps, elapsed time, settled GCell/s, peak process RAM, and peak GPU memory. Drag to pan, scroll to zoom, and Home refits the plot. A case that did not complete states its reason under the table. Its actions are Copy result JSON, Share benchmark with RayRF developers, and Close. Reopen the benchmark to plan a different configuration.

!fig Benchmark results with CPU and GPU curves, hardware summary, measured points, and result actions

Settled throughput uses the solver's native settled window when present. The fallback computes a rate from step telemetry after the warmup interval and before the final hold interval. A point with no valid settled window fails instead of substituting the full-run average.

Copy result JSON copies the complete `rayrf-benchmark-result/v1` document. Its top level contains `schema`, `app_version`, `timestamp`, `hardware`, `backend`, `device_index`, `quality`, `single_size`, `completed`, `cancelled`, `error`, `points`, `aggregate`, `annotations`, and `warnings`. Each point keeps its realized grid and cell count, requested and completed steps, elapsed time, settled and full-run rates, throughput source, estimated and observed memory, status, error, and annotations. Copying this JSON does not share it.

## Headless benchmark

`rayrf benchmark` runs the same bundled workload without the GUI.

```
rayrf benchmark --backend both --quality standard
rayrf benchmark --backend cuda --device-index 1 --single-size 10GB
```

`--backend` accepts `cpu`, `cuda`, or `both`. `--quality` accepts `approximate`, `standard`, or `precise`. `--single-size`, also accepted as `--size`, selects one ladder label. Progress and plan warnings go to stderr. Stdout contains one compact `rayrf-benchmark-result/v1` JSON document and no progress text. Exit status 0 means complete, 130 means cancelled, 2 means the plan was refused, and 1 means another failure. See the [CLI reference](docs-cli-reference.html) for the complete generated syntax.

## Opt-in sharing and privacy

No benchmark data is sent automatically. Share benchmark with RayRF developers sends the request below, and nothing else, when it is pressed. Copy result JSON puts the full result on the clipboard without sending it, so the request can be read first.

The share request contains exactly these fields:

Field || Contents
`hardware` || CPU model, physical core count, total RAM, operating system, GPU model, and total VRAM or null
`points` || Up to 64 completed points with update cells, settled GCell/s, observed peak RAM, observed peak VRAM, and backend
`curve_image` || The displayed curve as a base64 PNG with media type, width, and height
`app_version` || The RayRF version that produced the result
`timestamp` || The benchmark result time with its UTC offset

The decoded PNG is limited to 1 MiB. The complete request is limited to 1.5 MiB. Incomplete points, full result annotations, warnings, timing estimates, and raw step telemetry are not included.

The JSON body contains no project data, geometry, simulation settings, ports, file names, file paths, host name, account identifier, email address, IP address, or user-agent value. The receiver stores the listed hardware and point fields in a benchmark record and stores a sanitized copy of the curve PNG. It adds a record ID and submission time, but does not store the request IP or user agent. The full result JSON copied by Copy result JSON is separate from this smaller sharing request.

---

# S-parameters and Smith chart

The S-Parameters tab plots the network response of a finished run: magnitude in dB against frequency, the same data on a Smith chart, and the port input impedance against frequency. It also carries the markers, reference overlays, imports, and exports for that data.


After a run, each active port contributes a column of traces. The tab shows the reflection Sii for every active port, and both Sij and Sji for every active pair. The reflection trace of the port that was driven this run is the one that carries impedance, so it also appears on the Smith chart.

Trace labels follow the port identities recorded by the run, including sparse result sets. A result for port 2 stays S22 even when no port 1 result is present. It uses port 2's recorded reference impedance rather than its position in the result file. Labels through port 9 use compact forms such as S21. If either port number has two digits, a comma keeps the identities unambiguous, as in S10,1, S1,10, and S10,10.

## Traces and parameter chips

The toolbar carries a chip for S11 and S21 and a More Parameters popover. A chip toggles that trace on the plot and colors itself to match it. More Parameters lists every remaining Sij for the port count, plus S12 and S22 even for a one-port run, and holds one checkbox per imported overlay. Each trace keeps a fixed color so a port reads as the same hue across the plot, the chart, and the editor canvas.

The active ports determine which traces are initially visible. Changing the active ports resets the trace selection. Chips remain enabled when data is missing. A selected trace with no data means the run did not produce that result.

Once you edit the project after a run, a badge in the corner says so ("Geometry changed since this run", "Config changed since this run", or both). A run that stopped before its ring-down target also gets a badge that says how it stopped. A stale badge means re-run before trusting the numbers.

## The panels

Panels is a dropdown of checkable entries, one per panel: Magnitude (dB), Smith chart, and Impedance. Any subset can be checked, and the visible panels sit side by side in that fixed order, divided by draggable splitters. At least one panel is always visible: unchecking the last one is refused. A new project starts with the magnitude plot and the Smith chart. The selection and the splitter widths are stored in the project.

When a run recorded matching-network loss, the Panels list adds Matching-network loss. It plots each network part's dissipated power and the total, as a percent of the power sent toward the port, against frequency. With more than one matched port, each curve is named by its port.

The dB plot has frequency in GHz on the x-axis and magnitude in dB on the y-axis. Drag to pan, scroll to zoom both axes, Ctrl+scroll to zoom frequency only, Shift+scroll to zoom magnitude only.

The Smith chart is normalized to 50 Ω. Constant-resistance circles and constant-reactance arcs are labeled in ohms, the center is the matched load, the left real-axis endpoint is a short and the right is an open, and inductive (+jX) reactance is the upper half. It stays square and aspect-locked, so it never stretches. Only reflection traces carry the complex impedance the chart needs, so a magnitude-only trace does not draw here. An exact short stays at the left endpoint and reads 0 ohm. An exact open stays at the right endpoint and reads infinite ohm rather than a clipped finite value.

The impedance panel plots the port input impedance Zin = Z0 (1 + S) / (1 - S) of every visible reflection trace, using that port's reference impedance recorded with the run for Z0. Editing the live port after a run does not change the plotted impedance. Recorded reference-impedance metadata must contain finite positive values through the highest recorded backend port. A partial record is refused rather than padded with 50 ohms. Each trace contributes a solid Re Z curve and a dash-dot Im Z curve in the trace color, against frequency, with the axis in ohms. Reference and imported overlays draw here the same way they do on the magnitude plot, dashed. Transmission and magnitude-only traces carry no input impedance and do not appear. The hover readout shows the same R + jX value the magnitude plot's marker readout shows at that frequency. At S = +1 the readout is infinite ohm, and at S = -1 it is 0 ohm.

## Ports with matching networks

A port that carries a matching network is plotted at the system plane: its reflection is referenced at the ideal source behind the network, against the port's reference impedance. That is what an instrument connected through the same network would read, and every export (CSV, Touchstone, sweep output) uses the same plane. The run keeps recording the port's other recorded plane too, so the input impedance is available at both planes from one run: the port terminal when there is no measurement shift, the de-embedded plane when there is. The per-port network-loss CSV carries both, under headers that name which plane each one is.

A measurement shift on a port that carries a matching network is the one case where the reference plane is a choice. The shift moves the spot where the run records voltage and current, and the network sits between the ideal source and the port footprint, so the shifted spot is past the network as well as past the feed, and the two planes give genuinely different answers. The editor's Reported plane control, `network_plane` in a spec file, decides which one is reported. `system`, the default, keeps the plotted and exported response at the system plane through the network, and the de-embedded input impedance is recorded beside it in the per-port loss CSV. `deembed` reports the de-embedded plane instead, where the network is absent from the reflection by construction, and the system-plane impedance is recorded beside it. Either way the dissipated-power columns are referred to the power sent toward the port and do not change with the choice, the Smith chart's other-plane overlay is labeled with the plane it draws, and the exported Touchstone names its reference plane in a comment line.

The loss CSV reports one baseline and only one: each column is that circuit part's dissipated power as a share of the power sent toward the port, meaning the incident wave at the port reference resistance. The reflection follows from `zin_sys` and that same reference resistance, so the share of accepted power is derivable from the file rather than duplicated in it. The file opens with comment lines starting with `#` that state this and name the reference locations its impedance columns describe, and a frequency where a quantity is undefined is written as an empty cell, never as a number. The run manifest names both planes per port: `loss_reference_plane` for the loss columns and `reported_plane` for the S-parameters.

Every inductor and capacitor in a matching network requires a positive Q and the frequency that Q is exact at. The loss resistance it realizes is computed once at that frequency and is constant over frequency, so nothing about the simulation band or the excitation changes a component value. A resistor takes no Q: it dissipates the resistance you entered. The continuous preview, the stepped simulation, and the loss report all use the same realized values.

When a run recorded matching-network results, a checkbox for the port's other recorded plane appears in the controls row, labeled Device plane or De-embedded plane for the plane the run actually recorded. It overlays each matched port's other recorded input impedance on the Smith chart, dimmer, beside its system-plane trace, so the transformation the network applies is visible as the movement between the two loci.

A matching network attaches across the whole port footprint: it drives the footprint with a uniform current density and loads the area-averaged voltage over the same footprint. That pair is the network's terminal, and it is the plane the device-plane numbers of a networked port describe, on every footprint. A bare port's device plane is the center-line voltage over the same Ampere loop, which is a different functional. On a single-column footprint the two coincide, and on a wide footprint they do not. Removing the network from a wide port therefore moves the reported device plane, by design.

Because both planes of a networked port are read off one terminal, the system-plane impedance minus the device-plane impedance is the authored ladder, on any footprint, to the run's own discretization error. What is left over is that discretization and nothing else. The plotted response is still the co-simulated one, not a hand cascade: the port loads the network with whatever the structure presents, and the two only agree to the extent that the cascade is a good model of the geometry.

## Range and Home

Range opens a popover to set the frequency and magnitude axis limits. Leave a bound on Auto to fit the data. The popover reflects the plot's true current extent: a manual bound shows its value, and an Auto bound shows the value it resolves to in the gray caption beneath. Home clears every bound back to Auto and refits every visible panel. Range applies to the dB plot only.

## Markers

Marker (shortcut M) arms placement. Click the plot to drop a marker on the nearest trace point, and placement disarms after one marker. A marker reads out its frequency and magnitude, and the impedance where the trace carries it. On the Smith chart the readout is frequency, |Gamma|, angle of Gamma, and impedance.

Drag a marker to slide it along the trace, or to another trace. Esc cancels placement, Delete removes the selected marker, and Ctrl+Z undoes the last marker edit.

Right-click a marker for Rename, Hide, Delete, and, on the dB plot, Properties. The properties dialog:

- Sets the source trace and renames the marker.
- Follows a chosen dip. Pick the rank N and the ordering (deepest, shallowest, lowest frequency, highest frequency). A tracked marker re-snaps to that dip on every replot, including each Live-mode rerun, so a linked [surface-current or radiation](docs-field-viewer.html) viewer stays on the resonance as it moves. If the dip is gone the marker shows (no match) and holds its last frequency.
- Draws a vertical line at the center frequency, shades the bandwidth band below a dB threshold (default -10 dB, reflection traces only), and shows a center-frequency and bandwidth readout on the plot.

## Set as Reference

Set as Reference freezes the traces currently on screen as gray dashed overlays, so you can change the design and watch each trace shift against the frozen one. The baseline is stored in the project, and one is kept at a time: setting a new reference replaces the old one.

The reference is view-neutral. Reflection traces keep their complex impedance and phase, so the baseline shows on both the dB plot and the Smith chart, while transmission traces keep magnitude and show on the dB plot only. Setting a reference does not move the axes. In live mode the button moves into the More Parameters popover.

## Importing reference traces

Import a measured or external sweep to overlay it on the run. Both live in the Import / Export dropdown, render as dashed traces in distinct colors, and are stored in the project so the source file can be deleted afterward. Importing requires a signed-in license.

- **Touchstone** reads `.s1p` through `.s8p` and `.sNp`. Each S-parameter in the file becomes one overlay trace.
- **CSV** opens a column-mapping dialog, because a plain CSV is not self-describing. Choose the frequency column and one of three data types: an S-parameter as dB magnitude plus phase in degrees, an S-parameter as real plus imaginary, or an impedance Z as real plus imaginary in ohms. The second value column is optional. The frequency unit is taken from an explicit choice, then from the column header, then guessed from the values, with the guess confirmed before import. The reference Z0 you set converts an imported impedance to S and is stored with the trace so the Smith chart renders it correctly.

## Exporting

Every export is in the Import / Export dropdown. Exact file layouts are on [File formats and outputs](docs-file-formats.html).

- **CSV** writes one row per frequency, with each trace interpolated onto the first parameter's frequency grid and points outside a trace's own band left blank. When both plots are visible and reflection data is present, a dialog offers the dB magnitude, the reflection coefficient, or both.
- **Touchstone** writes all S-parameters as one network file, labeled by port count (`.s2p` for two ports). Filled values are disclosed in the export dialog and in file comments. The fill rules are on [File formats and outputs](docs-file-formats.html). Touchstone export is a Pro feature and shows a lock badge otherwise.
- **PNG** saves the plot as an image. PNG export covers the dB plot and the Smith chart. When both are visible a dialog offers the dB plot alone, both as two files, or both stitched into one wide image. Marker and curve legends are drawn into the saved image.

---

# Radiation pattern

The Radiation Pattern tab shows the antenna far field as a 2D polar plot and a 3D lobe over the geometry, one pattern per simulated frequency. It fills in when a run finishes with the radiation pattern enabled.


## Where the data comes from

The solver wraps the structure in a Huygens surface and records the near field on it during the run, then transforms that near field to the far field. Turn it on with `enable_nf2ff` (the Compute Radiation Pattern output toggle). The transform runs at a set of frequencies spread across the band, so every result below is available at each of those points. In auto mode the frequency count comes from the quality preset. In advanced mode `nf2ff_num_freqs` sets it directly, default 51, up to the backend ceiling of 512 points.

The far field is defined only when the box radiates into open space, so the run needs at least one PML face. Any mix of PML, PEC, and PMC on the remaining faces works. A PEC face acts as an infinite ground plane by image theory. Two cases are refused before the solve: a box with no PML face (a closed cavity, no radiated power), and a PEC or PMC pair on both faces of one axis (that axis is closed, so the pattern is undefined). See [Boundaries](docs-boundaries.html).

## Polar view

The polar view draws the two principal cuts, the E-plane and the H-plane. The right panel holds a frequency selector, a metrics block, and toggles for the two traces.

The frequency slider and the linked GHz spinbox pick which stored pattern to show. Patterns prepare in the background and stay cached while you scrub. If a pattern is still loading, the tab prioritizes the current selection and keeps navigation available. The dB scale is fixed across the slider so the traces stay comparable as you scrub. Clip below sets the floor (default -25 dBi), and a sample below it draws at the center. The ceiling is the peak directivity of the whole band plus 2 dB, rounded up. The legend names each cut and its phi angle.

The metrics block reports:

- **Directivity max.** Peak directivity at the selected frequency, in dBi.
- **HPBW theta and HPBW phi.** Half-power beamwidth of the E-plane and H-plane cuts, in degrees, measured across the connected -3 dB region around the peak.
- **Sidelobe.** Level of the largest lobe outside the main lobe, in dB relative to the peak, measured on the plane cut that carries the global peak.
- **F/B ratio.** Peak gain minus the gain in the opposite direction on that same cut, in dB.

## 3D view

The 3D view draws the pattern as a color-mapped lobe with the geometry overlaid at scale, so the beam direction reads against the antenna. Radius and color both follow directivity over a 40 dB window below the peak, and the scalar bar is labeled in dBi.

Two opacity sliders set the alpha blend between the lobe and the geometry, and either can be hidden with its visibility toggle. Show meshing controls the geometry cell outlines independently of Photorealistic, which lights cream dielectric and rough copper. New projects start with mesh outlines on and photorealistic shading off, keeping the dielectric translucent. The project saves this pair separately from Simulate and Field Viewer.

Position and scale each have an auto mode and a manual mode:

- **Auto position** centers the lobe on the geometry. Turn it off to set the X, Y, and Z origin directly in millimeters.
- **Auto scale** sizes the lobe so its tip-to-tip diameter matches the longest dimension of the geometry. Turn it off to set the scale directly, which holds a fixed lobe size while you step across frequencies.

Views picks which pattern panels are shown. Check both Polar Pattern and 3D Pattern to get them side by side with a draggable splitter, both driven by the one frequency slider. Check one to give it the whole workspace. At least one view stays checked.

The 3D view uses the same upright navigation as Simulate and Field Viewer. Left drag orbits. Right drag, middle drag or Shift plus left drag pans. The wheel zooms toward the point under the pointer, which stays under it. Each orbit starts around the closest board surface at the center of the view and keeps that pivot for the drag. Without a board, it uses the pattern surface. A miss keeps the last valid pivot, starting at the board or scene center. Orbit sensitivity is 0.25 degrees per logical pixel. The cursor is hidden during a drag and returns to where the drag started. A drag keeps going past the edge of the viewport. Release the button or press Esc to end it. Home fits the 3D scene and resets polar pan and zoom.

## Export to CSV

Export CSV asks what to write, then a save path. The scope dialog offers two choices:

- **E-plane and H-plane cuts** writes only the two displayed curves. Columns: `theta_deg`, `phi_deg`, `directivity_dBi`, `plane`.
- **Full 3D pattern** writes every theta/phi sample the run holds for that frequency, with no resampling. Columns: `theta_deg`, `phi_deg`, `directivity_dBi`.

Both files open with comment lines naming the frequency, the run, and the project. The export covers the frequency selected in the tab.

The CLI writes a flat table across the whole band instead, straight from the backend columns:

```
freq_hz,theta_deg,phi_deg,D_total_dBi,D_theta_dBi,D_phi_dBi
```

`D_total_dBi` is the total directivity, `D_theta_dBi` and `D_phi_dBi` the two polarization components, all in dBi.

## From the CLI

`rayrf run` writes the same patterns headless: `--export-polar` for the polar PNG, `--export-rad3d` for the 3D PNG (add `--export-rad3d-vtk` for the surface as VTP), and `--export-rad-pattern` for the flat CSV above. `--polar-freq-mhz` and `--rad3d-freq-mhz` pick the frequency, defaulting to the band center. Full flag list on the [CLI reference](docs-cli-reference.html).

---

# Field viewer

The Field Viewer tab shows what a run wrote inside the domain: surface current on a conductor layer in 2D, and the 3D E, H, and energy volumes. Two mode buttons at the top of the toolbar switch between them.


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

Field format selects RayRF compressed (`rff`), VTK (`vtk`), or OpenVDB (`vdb`). RayRF compressed is the default. It stores signed logarithmic samples in compressed bricks without threshold pruning. VTK stores fp32 samples. OpenVDB can prune samples below `vdb_threshold_frac` times the frame's peak magnitude. OpenVDB needs a uniform mesh, and the run refuses it on a graded one. Two settings bound the volume of data:

- The capture interval, dense for smooth playback or sparse to save disk.
- The step-range window (`field_export_range_enabled` with a start and end step), which drops warm-up frames. The solver still runs every step, only the disk write outside the window is skipped.

[FDTD tips](docs-fdtd-tips.html) covers when the fields are worth exporting. The on-disk frame formats are on [File formats and outputs](docs-file-formats.html).

The viewer reads all three formats. A voxel pruned from an OpenVDB file shows as zero. The archive inspection command reports the error bound and storage size of a RayRF compressed frame.

## Standalone viewer and archive commands

Open a recorded sequence without opening the editor:

```
rayrf fields view PATH_TO_RESULT_DIRECTORY
```

The standalone viewer opens with Photorealistic on. Add `--no-photorealistic-geometry` for material colors, `--show-meshing` for mesh cell outlines, or `--max-fps` to change the playback cap (default 30).

The directory must contain `frames.json` and its frame files. Inspect an archive or convert one VTK or OpenVDB quantity to a new archive:

```
rayrf fields inspect step_0000.rff
rayrf fields convert step_0000.vtk --source E_mag --out converted.rff
rayrf schema --what fields
```

Inspection returns JSON under `rayrf.fields.v1`. Conversion retains the input file's coordinates and refuses an existing output file. Errors go to stderr as JSON. The command exits 2 for a refused path or input and 1 for a failed read or write. When moving a result sequence, include `files.json` to retain its position in the project. `rayrf guide fields` documents these commands.

!note When you close a project whose field frames, geometry mesh, and surface-current data together exceed 100 MB on disk, RayRF asks which of those categories to delete. Removed data cannot be viewed again without re-running.

---

# Run history

Run History is always active. The History rail stays closed until you open it with the history button at the right end of the title bar, and the project remembers the state you leave it in, so a rail you closed stays closed when the project reopens. Closing the rail changes only the view: runs keep recording and parameter sweeps stay available.


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

A normal `rayrf run` always writes its run manifest but does not add a History record. Add `--record-history`, or set `record_history: true` in a JobSpec, when that run should appear in the desktop rail. The flag does not save or modify the project file. These project commands are also opt-in:

```
rayrf project save-point --project patch.rfsim
rayrf project import-reference --project patch.rfsim --file measured.s1p
```

A CSV reference needs an explicit value representation and column mapping. Use `rayrf project import-reference --help` for the mapping flags. The CLI and GUI use the same conversion and validation path, and both copy the imported data into the record so the source file can be removed afterward.

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

---

# Parameter sweeps

A parameter sweep runs one project across a range of variable values and records every point as its own run in the History rail. Arm it with the Parameter sweep checkbox under the run controls on the Simulate tab.


While the sweep is armed a Sweep Options button appears below the checkbox and opens the definition dialog, and the run button reads Run Simulation Sweep and launches it. An armed sweep with no saved definition blocks the run and outlines the button in red until you configure it, with the reason on hover. A sweep needs a saved project, because its records land beside the project file.

Sweeps use [Run History](docs-run-history.html), which is always active, because every point is stored as a record. The `rayrf sweep` commands own their own state directory and record into project history only when you ask, as described below.

## Defining the axes

A sweep spans up to two axes. Each axis binds one project variable (from Variable Studio, see [Variables, constraints, and generators](docs-parametric.html)) to the values it takes. Picking a variable prefills the range from the variable's min and max when they are set.

Each axis has a spacing mode:

- **List:** the values you type, comma or space separated.
- **Linspace:** N points evenly spaced from a start to a stop, both included.
- **Log:** N points spaced logarithmically from a start to a stop, both included and both positive.

Two axes combine as a grid: every combination of the two value lists (the cartesian product). The `rayrf sweep` commands also accept a zip combine, which pairs the two axes element by element and needs both the same length.

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
rayrf sweep preview --project patch.rfsim --axis patchW=15,16,17
rayrf sweep run --project patch.rfsim --axis patchW=15,16,17
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
rayrf sweep resume --project patch.rfsim --sweep-id s0001
rayrf sweep retry --project patch.rfsim --sweep-id s0001 --point-index 2
```

Add the original `--state-dir` to either command when the run used one, or add `--record-history` when the group was written to project History. `--json` emits one JSON event per preview, progress, point, failure, and completion record.

The engine handles interruption point by point:

- **Resume.** A sweep left interrupted (cancelled, or the app closed mid-run) resumes from the rail and skips the points already complete.
- **Retry.** Right-click a failed sweep point and choose Retry this point to re-run only that point. The retry updates that point's durable record and increments its attempt count instead of adding a duplicate point.
- **Cancel.** Cancelling stops launching new points and shows Cancelling while the in-flight point settles. The group becomes interrupted only after that point has stopped.
- A failed point records its reason and the sweep moves on to the next point.

Preparing a resume or retry can derive many meshes, so it runs in the background behind a visible busy overlay. A second resume click is refused while that work is active. If the process ends unexpectedly during a point, reopening the project changes the orphaned running group to interrupted and offers Resume sweep while preserving the point's last attempt.

While a sweep is preparing, running, or cancelling, RayRF refuses Save, Save As, new project, open project, and application close. Normal Run, mesh preview, Live mode, and a second sweep are also unavailable. These gates keep the active runner and history sink bound to one saved project until they settle.

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

---

# Remote running

Submit a simulation to a machine you own and stream the results back to your desktop. The case runs on the selected runner, including a rented machine, and the results return to your project folder. Setting up the runner box is a separate walkthrough: [Runner setup](docs-runner-setup.html).


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

---

# Runner setup

To set up a runner, install RayRF on the machine, license it against your account, open the runner's TCP port, and start the daemon. See [Remote running](docs-remote-overview.html) for submitting jobs from the desktop app.


Runner licensing is seatless. A Pro or Founding license key from the dashboard licenses page validates the runner against your account and signs a machine-bound token for every job, without consuming an activation seat.

## Linux

Install once, with the license key from the dashboard licenses page:

```
./RayRF-Studio-<version>-x86_64.AppImage serve --install --license-key RAYRF-XXXX-XXXX-XXXX-XXXX
```

Then start serving:

```
./RayRF-Studio-<version>-x86_64.AppImage serve
```

The runner validates the key, registers this machine's address to your account, and your other machines discover it automatically. The runner exposes every device the box can run on: the CPU when the CPU backend is present and each NVIDIA GPU when the CUDA backend answers. Pass `--devices` with `--install` to expose a subset, for example `--devices cuda:0,cuda:1` on a box whose CPU should stay out of the pool. Later starts are the same plain `serve`, because the configuration is stored, and the runner window lets you check devices on and off while it serves.

Open the runner port so clients can reach it. Jobs move over a direct link to port 8787, which a host firewall or cloud security group often blocks by default. On a box running `ufw` that is `sudo ufw allow 8787/tcp`. On a cloud VM also open the port in the provider's security group.

To keep the runner alive across reboots, pass `--autostart` with `--install`. It writes a systemd unit into the state dir and prints the install command:

```
sudo cp /var/lib/rayrf/rayrf-runner.service /etc/systemd/system/ && sudo systemctl enable --now rayrf-runner
```

## Windows

The same two steps, from the install folder (`%LOCALAPPDATA%\Programs\RayRF Studio`). `rayrf` resolves to the console launcher `rayrf.com`, which waits and streams output, and `RayRF.exe` is the windowed app.

```
.\rayrf serve --install --license-key RAYRF-XXXX-XXXX-XXXX-XXXX
```

```
.\rayrf serve
```

Open the runner port so clients can reach it, the same as on Linux. Windows Firewall blocks inbound connections by default, so a runner that registers with your account is still unreachable until port 8787 is allowed. From an Administrator PowerShell:

```
netsh advfirewall firewall add rule name="RayRF runner 8787" dir=in action=allow protocol=TCP localport=8787
```

On a cloud VM also open the port in the provider's security group. If the machine's network is set to Public, allow the rule on the Public profile (the command above covers all profiles).

To run at startup, register a Scheduled Task (or a service wrapper) that runs `RayRF.exe serve --state-dir "C:\ProgramData\RayRF\runner"` from the install folder. Passing `--autostart` with `--install` prints the serve command with the resolved state dir.

## Advertise a fixed address

By default the runner advertises its active LAN addresses and the public IP observed by the website. Clients use an address that passes the verified connection. No files or results pass through the website.

For a translated port or fixed hostname, set the reachable endpoint with `--advertise HOST:PORT` at install or serve time: on a NAT or portmapped host (a rented container, a port-forwarded home box) that is the external address and mapped port, which can differ from the listen port. Registration is metadata only. Jobs move over a direct TLS 1.3 link to the advertised address, so the port (default 8787) must be reachable through the firewall or cloud security group.

`--advertise off` skips account registration entirely. The runner then serves direct connections only, and clients add it by host, port, and fingerprint.

## What each piece does

- `--install` writes `runner.json` (name, owner, host, port, slots, devices, advertise, and the license key when one is resolved) and the identity into the state dir. Command-line flags always win over the stored values.
- The **state dir** holds the identity, the configuration, and the durable job queue. It resolves from `--state-dir`, then the `RAYRF_STATE_DIR` environment variable, then the machine default (`C:\ProgramData\RayRF\runner` on Windows, `/var/lib/rayrf` on Linux when writable, else `~/.rayrf/runner`). Keep every invocation on one state dir: a new dir mints a new identity, and clients that pinned the old fingerprint refuse the new one.
- `--devices` names the devices jobs may run on, as a comma-separated list of `cpu` and `cuda:<index>`. Omitted, every device the box can run on is exposed. A job names one of them or `auto`, which takes the first free device with GPUs first. A named device queues the job until it is free. One job runs per device at a time.
- Control belongs to the license key's account: a desktop signed in to the same account proves it with its signed token and gets submit and control, and every other connection is view-only. `--owner` sets the account the runner displays, and `--open-owner` grants control to any peer on a trusted LAN. `--allow-unlicensed` skips the license gate for closed-network development. In that mode `--owner` is trusted by the email a peer types, and the start banner says so.
- `--slots` caps how many jobs run at once below the one-per-device default.

## Verify and connect

Account discovery verifies the advertised identity automatically. For a direct connection, compare the fingerprint printed by `--install` with the client prompt, or supply it explicitly:

```
rayrf serve --health                                status JSON on the box
rayrf remote info --host HOST --fingerprint FP      from a client
```

A client that connects by CLI stores the fingerprint it first trusted in `~/.rayrf/pins.json` (`%USERPROFILE%\.rayrf\pins.json` on Windows). If a runner's key later changes (wiping its state dir mints a new identity), the next connection is refused. After verifying the new fingerprint out of band, reconnect with `--fingerprint NEWFP`, or remove that runner's `host:port` entry from the pins file and reconnect to trust it fresh. For a saved direct desktop connection, remove it and add the verified new identity. Account discovery uses the current advertisement automatically.

The runner also has a windowed form, `rayrf serve --window`, which is what the desktop app's "Use this computer as a runner" button starts: the same daemon with the runner name, discovery status, device switches, and job counts. Address, identity, and logs are under Details.

Submit work from the desktop app ([Remote running](docs-remote-overview.html)) or from the CLI with the `rayrf remote` verbs ([CLI reference](docs-cli-reference.html)).

## Updating a runner

A runner and the desktop app or CLI that connects to it must run the same RayRF release. A mismatch is refused with a message that says which end to update. To update a runner, stop `serve`, install the new release on the runner machine, and start `serve` again on the same state dir. The stored configuration and identity carry over, so clients keep their pinned fingerprint.

---

# CLI overview

The `rayrf` command line builds, inspects, validates, runs, and post-processes `.rfsim` projects headless, driving the same solver as the desktop app. This page covers invocation, licensing, the verbs, an end-to-end workflow, and the exit codes and machine-readable output an agent depends on. If you are pointing a coding agent at RayRF, start with [Driving RayRF with an agent](docs-cli-agent-driving.html).


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

---

# Driving RayRF with an agent

The canonical approach for using RayRF with a coding agent such as Claude or Codex to build, edit, and run a RayRF project is the `rayrf` command line. This page provides guidance on best practices, exit code meanings, and interpreting results.


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

---

# CLI reference

List of all verbs and flags for `rayrf` command line use. Workflows and examples are on [CLI overview](docs-cli-overview.html), spec details on [Scripting and specs](docs-cli-scripting.html).


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

---

# Scripting and specs

A project spec describes authored geometry. A board-import spec describes how Gerber, Excellon, DXF, KiCad, or openEMS and CSXCAD sources enter a project. `rayrf project build` and `rayrf project import` produce `.rfsim` files, `rayrf project export-spec` turns authored geometry back into a project spec, and one JobSpec can drive a whole run. The loop these specs sit inside is [Driving RayRF with an agent](docs-cli-agent-driving.html).


## The project-as-spec model

`rayrf schema --what spec` prints the JSON Schema for project specs. The spec has these top-level keys:

- **domain**: the editor rectangle, in one of two modes. With `auto_fit` true (the default) the rectangle is derived from the geometry plus the padding keys `pad_mode` (percent or mm), `pad_percent`, and `pad_mm`, both when the spec is built and again at run time. A rectangle key (`x_mm`, `y_mm`, `w_mm`, `h_mm`) beside `auto_fit` true is refused, since it could never reach the run. With `auto_fit` false the rectangle keys size the box verbatim, `w_mm` and `h_mm` are required, and a padding key beside them is refused. `export-spec` writes whichever set the project uses, so build reproduces the same box. Unknown domain keys are rejected.
- **layers**: the stackup, each conductor, dielectric, or air, top to bottom. An air layer is a free-space spacer that sets the gap between conductors through its `thickness_mm` and emits no material entry, so it renders as air. `z_mm` is recomputed from stack order, so omit it. Unknown layer keys are rejected, so `eps_r` in place of `epsilon_r` is a clear error. A conductor with `conductivity_S_m` 0 is PEC. A positive conductivity activates finite sheet loss and requires a positive `conductor_thickness_um`. That thickness affects electrical loss but does not change stackup Z.
- **shapes**: geometry, each with a `kind` and a `layer`. A hole or cutout takes its span from `start_layer` and `end_layer`, so its layer can be left out, and a layer given with one must name the first layer it cuts. See [CLI reference](docs-cli-reference.html) and `rayrf schema --what shapes|ports` for the per-kind parameters.
- **variables, bindings, constraints, generators**: the parametric model. A `bindings` entry maps a param path to an expression over variables. Path prefixes are `primitive:ID:param` (a poly vertex is `primitive:ID:points.INDEX.x`), `layer:NAME:field`, `domain:field`, `fdtd:field`, and `generator:ID:param`. A `domain:w_mm` or `domain:h_mm` binding belongs to an explicit box and is refused with `auto_fit` true, the same as the rectangle keys. Auto-fit padding accepts `domain:padding_percent` and `domain:padding_mm` bindings with values of zero or greater. Polygon vertex bindings use local coordinates relative to the shape origin.
- **settings**: any FDTDSettings field, set verbatim. `rayrf schema --what settings` lists every field with its type, default, units, and description. A spec that sets `export_vdb` or `adv_export_vdb` builds with the value read as `field_format` or `adv_field_format` (true is vdb, false is vtk), and a warning on stderr names the change.

Only `layers` and `shapes` are required. Any setting not present takes the project default.

!note Mesh realization resolves the air margin from `air_mode` on every path, so a headless run of a spec-built project realizes the same margins the GUI shows. The modes are `cells`, `mm`, and `auto`. Every mode is resolved at export on every path: cells from `air_cells` x the face's own cell size, auto from `auto_air_*_factor` x the wavelength at f_min. In those two modes the `air_*_mm` fields are display state. Only `air_mode=mm` reads them. `mesh explain` reports the realized value.

`rayrf project build --spec FILE --out p.rfsim` writes the project. On a spec error or a failed resolve (an undefined variable, an unknown layer) it prints one line to stderr and exits 2.

`rayrf project export-spec --project FILE --out FILE` writes the spec for an authored project, emitting the verbatim params form so a rebuild reproduces it. Projects containing bitmap, DXF, or imported board artwork shapes refuse because a project spec cannot preserve their embedded payload. Keep the original board-import spec as the reproducible source for an imported board.

## Exporting DXF drawings

The DXF exporter reads one saved project and writes its current resolved 2D geometry:

```
rayrf project export-dxf --project FILE --out FILE [--replace] [--json]
```

The output is an R2013 DXF with $INSUNITS set to millimeters. Every project layer is represented, including empty layers, and hidden layers are off. The built-in layer 0 keeps its name. Defpoints, invalid names, and collisions receive legal unique names listed in approximated. The rectangular simulation domain is written on Board Outline because the project does not retain a separate shaped fabrication outline. Cutouts are written on Board Outline too, and a cutout that reaches the domain edge notches it.

Variables, bindings, constraints, script shapes, and generator instances are resolved on a project copy before export. The drawing contains the resolved primitive and generator geometry, not the parametric definitions. Bitmap and imported DXF shapes are vectorized from their stored non-transparent pixels with their position, scale, rotation, and flips applied. Ports become visible footprints on Ports. A via writes a filled pad on each conductor layer it spans and its drill circle on Drill. Holes and slots go on Drill PTH when plated and Drill NPTH otherwise.

Filled polygons, discs, rings, and arc bands have a solid HATCH plus an ordinary visible tagged boundary. Native wide polylines represent unclipped script paths and line ports. RayRF suppresses a tagged duplicate boundary on re-import only while its geometry still matches the companion HATCH boundary. If another editor removes the HATCH, the boundary remains and imports as geometry. If the boundary is edited independently, RayRF retains both and reports the difference. The simulation-domain Board Outline is boundary-only.

The command prints three complete report categories: exported, approximated, and omitted. Without `--json`, approximated and omitted entries go to stderr. JSON output uses `rayrf-dxf-export-report/v1`:

```
{
  "schema": "rayrf-dxf-export-report/v1",
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
rayrf project export-openems --project FILE --out FILE [--replace] [--json]
```

The XML includes one resolved Gaussian pulse, boundary faces, mesh lines, stackup materials, supported geometry, vias, holes and cutouts, and supported lumped ports. Exactly one port must be active. Each port needs an explicit number, reference impedance, and Boolean active value. Point and rectangular ports need both terminal layers. A planar port needs its one conductor layer, and a present `excitation_mode` must be `planar`. Rectangular or planar ports must align with a Cartesian axis. Continuous-wave setup, de-embedding, unsupported electrical geometry, geometry outside the resolved domain, and fuzzy load-time layer remaps refuse the whole export. Variables, bindings, constraints, and generators are resolved on a copy. Uniform grids use the solver's whole-cell board, gap, absorber, and stackup quantization. Any resulting padded edge, expanded gap, or shifted stackup plane is listed as approximated. Graded grids use the solver's nearest explicit Z line for every stackup edge. Every completed report states that the XML is a translated setup and that the target runtime can produce a different mesh and different results. Parametric definitions, editor state, stored results, Run History, and reference traces are listed as omitted.

An existing output refuses unless `--replace` is set. The `.xml` suffix is added when absent. The writer performs XML readback before installing the destination, so a refusal does not replace an existing file. JSON output uses `rayrf-openems-export-report/v1` and carries `entity_count` and the exported, approximated, and omitted lists. The complete mapping is in [Exporting openEMS XML](docs-export-openems.html).

## Board import specs

`rayrf schema --what board-import` prints the authoritative JSON Schema. Board-import source paths are relative to the import spec. Gerber accepts a set of artwork, drill, and job files. DXF, KiCad, and openEMS accept one source file. A Gerber job file must be listed in `sources` if it should control roles or stackup. An unlisted job file beside the artwork cannot affect a CLI import.

This Gerber example explicitly accepts detected roles, supplies the one dielectric gap, and adds the settings and port needed for headless validation and simulation:

```
{
  "schema": "rayrf-board-import/v1",
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
  "schema": "rayrf-board-import/v1",
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
  "schema": "rayrf-board-import/v1",
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
  "schema": "rayrf-board-import/v1",
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

`rayrf project import-openems` accepts the openEMS spec as a strict alias for `rayrf project import`.

`stackup.source` has format-specific explicit choices:

- `explicit` requires exactly one gap fewer than the included copper count.
- `job` requires a listed Gerber job file with every gap thickness and permittivity present.
- `detected` requires a complete KiCad or CSXCAD material stack.
- `defaults` selects the editor's nominal 1.6 mm FR-4 gap distribution and 4.4 relative permittivity.

Thickness must be finite and greater than zero. Relative permittivity must be finite and at least 1. Values outside the usual ranges print a warning and are kept exactly.

The optional `project` block applies settings and authored shapes after the board geometry is built and before it is saved. Settings use the same value gate as project specs and the GUI. Shapes use the same syntax and port defaults as `project build`, but cannot set an `id`. Imported geometry owns the existing ids. Bitmap and DXF shapes are also refused in this block because `sources` owns the imported artwork. Omitting `project` adds no authored ports or setting overrides. An openEMS new-project import still carries its supported source frequency band. Other formats retain project defaults unless the `project` block supplies settings. Explicit `project.settings` are applied after the openEMS Auto initialization and define the resulting controls. If the block sets either frequency edge, those explicit settings define the final band.

DXF always creates a fresh project and rejects `destination`. Gerber takes an optional `destination` and creates a fresh project without one. KiCad and openEMS require `destination.mode` to be `new` or `current`. Current mode requires a saved `destination.project` and merges geometry into it. openEMS current mode also requires `bring_frequency_settings` to be true or false. Choosing true refuses the import when the XML has no supported positive Gaussian band. The `--out` path always names the result. Writing back to the destination path also requires `--replace`.

```
rayrf project import --spec board.import.json --out board.rfsim
rayrf project import --spec board.import.json --out board.rfsim --json
rayrf validate --project board.rfsim --json
rayrf run --project board.rfsim --dry-run
```

The JSON report contains the effective unit, destination, every effective assignment, every gap, applied setting names, authored shape identities, the output path, and the imported, approximated, and skipped report lists. An existing output project or same-stem `.rfsimout` refuses. `--replace` explicitly replaces the project and removes the old result sidecar.

## A worked example

A single rectangular patch over a ground plane on FR4, with one vertical feed port and one variable driving the patch width. Built with `rayrf project build` and checked with `rayrf validate`.

```
{
  "schema": "rayrf-project-spec/v1",
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
rayrf project build --spec patch.json --out patch.rfsim
rayrf validate --project patch.rfsim
```

A `rect` on a numeric field takes a plain number (`"h_mm": 12.3`) or, when the value is a non-numeric string (`"w_mm": "patchW"`), an expression binding on that field. The feed spans vertically from `bottom_layer` to `top_layer`, so the layer order matters: a port whose two conductors land on the same mesh cell injects nothing and validate reports it. The ground and substrate rects paint the Bottom and Core layers so the dielectric is real, not air.

## Inspecting a project

`rayrf project query --project FILE --json` prints a machine-readable readout: every shape (id, kind, layer, name, params), every variable (source value, resolved value, range, group, where-used count), constraints, generators, bindings, settings, the domain, a geometry readout (per-shape center, bounds, anchors, resolved boundary faces, domain clearance), and a last-results summary. Use it to inspect a project you did not build. A shape that cannot be measured carries its own error in the readout, and the project-wide `bounds_xy` is null whenever any shape is unreadable, since an extent over the readable rest would be smaller than the project's.

`rayrf project dump-settings --project FILE` prints every setting with its value, default, units, and description, grouped by category. Flags narrow the output:

- `--changed-only` lists only settings that differ from their default.
- `--category NAME` limits to one category (an unknown name exits 2).
- `--advanced` keeps only advanced settings, `--no-advanced` only the others.
- `--json` emits a machine-readable list instead of the grouped text.

## Mutating a project

`rayrf project set --project FILE --var name=value [--out FILE]` sets one or more variables (repeatable `--var`), re-resolves bindings and constraints, and saves the project in place or to `--out`. A value may be a number or an expression over other variables. An unknown reference or a bad expression exits 2 with the field message and leaves the geometry unchanged.

Two run-time override forms change values for a single run without editing the project:

- `rayrf run --set name=value` overrides a variable, resolved before export.
- `rayrf run --setting name=value` sets any FDTDSettings field. It goes through the same value gate as the GUI: an unknown name, a bad type or enum, or a hard-bound violation exits 2 before the run starts, a soft-bound value prints one warning and runs with the value exactly as set. Both flags repeat.

!note A setting fixed in auto mode is derived and overwritten at run time. The run prints the discarded derived value so an override never loses silently.

## Explicit Run History operations

Automation stays history-free unless it opts in. A normal run always writes its `run_manifest.json` but does not add a record to the desktop History rail. Add `--record-history` when a completed run should be recorded. This does not imply `--save-after-run` and does not modify the `.rfsim`.

Two project commands expose the matching desktop actions without changing the project file:

```
rayrf project save-point --project patch.rfsim --json
rayrf project import-reference --project patch.rfsim \
  --file measured.s1p --json
```

For CSV data, select the representation and columns explicitly. This example imports frequency in MHz and complex impedance, converting it to S11 at 50 ohm:

```
rayrf project import-reference --project patch.rfsim --file measured.csv \
  --format csv --csv-kind impedance --freq-column frequency_mhz \
  --value-column resistance --second-column reactance --freq-unit mhz \
  --parameter S11 --z0-ohms 50 --json
```

`db-phase` maps magnitude in dB plus optional phase in degrees. `real-imag` maps real and optional imaginary S values. `impedance` maps resistance and optional reactance, then applies the selected reference impedance. When Auto must infer a frequency unit from value magnitude, the command refuses until `--accept-guessed-unit` is explicit. It also refuses a mapping that skips more rows than it imports unless `--allow-many-skipped` is explicit.

## Parameter sweep definitions

The inline form accepts one or two explicit value lists:

```
rayrf sweep preview --project patch.rfsim --axis patchW=15,16,17
rayrf sweep run --project patch.rfsim --axis patchW=15,16,17
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
rayrf sweep resume --project patch.rfsim --sweep-id s0001
rayrf sweep retry --project patch.rfsim --sweep-id s0001 --point-index 2
```

## Variables and expressions

A numeric field accepts an expression over the project variables: the operators `+ - * / // % **`, parentheses, the trig and math functions, and the constants `pi` and `e`. Trig is in degrees. The full function list is in `rayrf schema --what spec` and `rayrf guide geometry`.

## JobSpec: one payload per run

`rayrf run --job FILE` reads a YAML or JSON JobSpec that carries the project, the overrides, the save behavior, the output directory, and the artifact selection. The job file owns the run request: combining `--job` with another run flag is refused rather than silently discarded, except the flags that layer onto a loaded job (`--backend`, `--dry-run`, `--per-face-bc`, `--set`, `--externalize-results`). An unknown key in the file is refused, so a typo cannot silently revert to a default. Relative `project`, `save_as`, and `out_dir` paths resolve from the directory that contains the job file.

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

The `settings` block passes through the same value gate as `--setting`. The `overrides` block sets advanced numeric overrides (cell sizes, air margins, the frequency band, and material fields). The dielectric permittivity key is `epsilon_r`, with `eps_r` accepted as an alias. Setting both to different values is an error, not a last-wins pick. JobSpec booleans require JSON or YAML booleans, enums require a listed value, and numeric controls refuse non-finite or out-of-range values. `quality` and `quality_frac` cannot appear together. `record_history` defaults to false and is the JobSpec equivalent of `run --record-history`. `rayrf validate --job FILE` applies the job to a detached project through the same pre-run preparation as run, then reports findings without saving the project. The JobSpec fields:

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

The manifest records the realized sim params (cell sizes, runtime and ringdown values, frequency band, boundaries), the auto-derived mesh with its reasoning, performance (duration, steps, cells, GCell/s, VRAM, ringdown, convergence history), and the list of artifacts written. The pml/air mm fields inside `sim_params` are the authored request, and the per-face realization lives in `mesh_realization.faces`. `overrides_requested` and `overrides_honored` are both present, and `overrides_honored` reads back the values the exported case actually carried, so a request the run could not realize is visible. The full contract is at `rayrf schema --what manifest`.

## Batch running

`rayrf run --folder DIR` runs every project in a folder in sequence, `--recursive` to descend into subfolders. One project that fails does not abandon the rest: the batch continues to the next project and exits 1 if any project failed at runtime, 2 if none failed at runtime but at least one was refused, or 0 if all succeeded. Each failure is appended to `OUT_DIR/batch_failures.jsonl` when `--out-dir` is set, or `DIR/batch_failures.jsonl` otherwise. If that log cannot be written, the runner reports the logging error on stderr without changing the project's batch outcome.

---

# Settings reference

All simulation settings, grouped by category. Settings are saved in the project's FDTD settings block.


## Reading and changing settings

The same values reach the solver from four surfaces:

- The Simulate tab panels edit the run settings in the GUI. Advanced-mode settings appear when Advanced mode is on. Viewer settings such as Photorealistic, Show meshing, and Max FPS are set in the viewer that uses them and do not change the run.
- `rayrf project dump-settings --project FILE` prints every setting with its value, default, units, and description, grouped by category. `--changed-only` shows only settings that differ from their default, `--category NAME` limits to one category, `--advanced` or `--no-advanced` limits to advanced-only or non-advanced settings, and `--json` emits a machine-readable list.
- `rayrf run --setting name=value` overrides a setting for one run (repeatable). It is value-gated by the same gate as the GUI. A hard bound out of range refuses with exit 2, a soft bound out of range prints one warning and runs the value exactly as set. An unknown setting refuses with exit 2, and a renamed one names its replacement.
- `rayrf schema --what settings` prints the machine contract: every field with its type, default, units, enum, and category.

## Reading the table

Type and Default are the live values from the registry. A **ui-only** setting persists viewer or editor state and does not change the physics. An **advanced** setting surfaces in Advanced mode. A **Caution** note is the registry's warning about a common mistake for that setting.

## Frequency band

Setting || Type || Default || Units || Notes || Description
`freq_min_hz` || number || 0 || Hz || || Lower edge of the excitation band, in Hz. Also sets the reference wavelength for PML and air sizing. **Caution:** This field is in Hz. The CLI run flags and the GUI use MHz/GHz, so a value of 3e9 is 3 GHz.
`freq_max_hz` || number || 0 || Hz || || Upper edge of the excitation band, in Hz. Sets the required mesh resolution (cells per wavelength at f_max). **Caution:** Must be greater than `freq_min_hz`.
`cw_enabled` || boolean || false || bool || || Inject a continuous sinusoid at `cw_freq_hz` instead of a Gaussian pulse. Advanced-mode only: export refuses an auto-mode project with CW enabled. **Caution:** Continuous wave never rings down, so the run stops only at whichever step cap is live: `auto_max_steps` in Auto mode, `adv_max_steps` in Advanced mode when `adv_max_steps_enabled` is true, otherwise at the step counter's limit. Under a single tone the S-parameters are of limited meaning, the radiation pattern at the tone is meaningful, and both outputs stay available.
`cw_freq_hz` || number || 2.4e+09 || Hz || || Continuous-wave excitation frequency, in Hz, used when `cw_enabled` is on.
`auto_freq_mode_centre` || boolean || false || bool || ui-only || UI input style for the auto-mode frequency band: f_min / f_max (False, the default) or centre plus bandwidth (True).

## Mesh / cell size

Setting || Type || Default || Units || Notes || Description
`dx_mm` || number || 0.2 || mm || || FDTD cell size in X. Smaller cells raise accuracy, memory, and runtime. **Caution:** In advanced mode this is used exactly. In auto mode it is derived from quality and overwritten.
`dy_mm` || number || 0.2 || mm || || FDTD cell size in Y.
`dz_mm` || number || 0.2 || mm || || FDTD cell size in Z. Must resolve the substrate thickness (aim for several cells per dielectric layer). **Caution:** In advanced mode dz is not snapped. If it does not divide the stackup height the substrate can land between grid lines.

## Non-uniform mesh grading

Setting || Type || Default || Units || Notes || Description
`mesh_strategy` || string || ratio || enum || enum: ratio / custom || Select the sole mesh-grading owner: board-region Ratio grading or the Custom profile edited in Mesh View.
`mesh_grading_x_mult` || number || 1 || factor || || Board-region refinement factor in X (1.0 means no refinement).
`mesh_grading_y_mult` || number || 1 || factor || || Board-region refinement factor in Y.
`mesh_grading_z_mult` || number || 1 || factor || || Board-region refinement factor in Z. **Caution:** A value of 2.0 halves the Z cell near the board, roughly doubling Z cells there and raising memory.
`mesh_grading_margin_mm` || number || 1 || mm || || How far the fine-cell region extends beyond the board in every direction.
`mesh_grading_x_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the X grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.
`mesh_grading_y_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the Y grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.
`mesh_grading_z_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the Z grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.

## Boundary conditions and PML

Setting || Type || Default || Units || Notes || Description
`bc_all` || string || PML || || enum: PML / PEC / PMC / Custom || Boundary preset applied to all six faces, or Custom to set each face individually. **Caution:** PEC and PMC are hard walls with no absorber. Give every PEC/PMC face an explicit per-face spacing, otherwise it inherits the air margin sized for PML and the geometry can sit on the wall.
`pml_thickness_mm` || number || 4 || mm || || Physical thickness of the PML absorber, the authoritative depth when `pml_mode` is manual. In the cells and auto modes the export derives the depth (`pml_cells_manual` x finest cell, or `auto_pml_factor` x wavelength) on every path and this field is display state. The PML is added outside the air margin (it extends the domain), so it never overlaps the geometry. Thickness only increases total cell count.
`pml_cells` || integer || 0 || cells || || Resolved PML cell count. 0 means derive it from `pml_thickness_mm` at export. A positive value is a per-face floor, and the axis count is raised to ceil(`pml_thickness_mm` / cell) when the thickness needs more. In advanced mode with a manual thickness the thickness alone is authoritative and this count is not read.
`auto_pml_factor` || number || 0.1 || fraction || || PML thickness as a fraction of the wavelength at f_min when auto PML is on (0.10 means lambda/10). The depth is realized as written. A thin absorber is warned about, never enlarged.
`pml_mode` || string || cells || || enum: cells / auto / manual || How PML depth is specified: a cell count (`pml_cells_manual`), auto-sized from the wavelength at f_min, or the explicit `pml_thickness_mm`.
`pml_cells_manual` || integer || 8 || cells || || PML absorber depth in cells per face when `pml_mode` is cells. The export pins every face to exactly this count on every path. A face declared PML needs at least one absorber cell to be one.
`bc_x_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -X face when `bc_all` is Custom.
`bc_x_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +X face when `bc_all` is Custom.
`bc_y_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -Y face when `bc_all` is Custom.
`bc_y_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +Y face when `bc_all` is Custom.
`bc_z_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -Z (bottom) face when `bc_all` is Custom.
`bc_z_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +Z (top) face when `bc_all` is Custom.
`bc_x_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -X gap (`bc_x_lo_spacing_mm`) instead of the auto/global air margin.
`bc_x_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +X gap (`bc_x_hi_spacing_mm`) instead of the auto/global air margin.
`bc_y_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -Y gap (`bc_y_lo_spacing_mm`) instead of the auto/global air margin.
`bc_y_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +Y gap (`bc_y_hi_spacing_mm`) instead of the auto/global air margin.
`bc_z_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -Z gap (`bc_z_lo_spacing_mm`) instead of the auto/global air margin.
`bc_z_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +Z gap (`bc_z_hi_spacing_mm`) instead of the auto/global air margin.
`bc_x_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -X boundary when its spacing override is enabled.
`bc_x_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +X boundary when its spacing override is enabled.
`bc_y_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -Y boundary when its spacing override is enabled.
`bc_y_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +Y boundary when its spacing override is enabled.
`bc_z_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -Z boundary when its spacing override is enabled. A Custom PEC/PMC face requires it (0 puts the wall flush, e.g. an infinite ground plane). PML faces fall back to the derived margin.
`bc_z_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +Z boundary when its spacing override is enabled. A Custom PEC/PMC face requires it. PML faces fall back to the derived margin.

## Air margins / domain

Setting || Type || Default || Units || Notes || Description
`air_around_mm` || number || 10 || mm || || Air gap in the XY plane from the meshed board edge to the side boundaries. A per-face spacing overrides it on that face.
`air_above_mm` || number || 10 || mm || || Air gap above the topmost stackup plane to the upper boundary. A per-face top spacing overrides it.
`air_below_mm` || number || 10 || mm || || Air gap below the bottom stackup plane to the lower boundary. A per-face bottom spacing overrides it.
`air_mode` || string || cells || || enum: cells / mm / auto || How the advanced air margin is specified: a per-face cell count, the explicit `air_*_mm` values, or auto-sized from the wavelength at f_min. **Caution:** Every mode is resolved at export on every path: cells from `air_cells` x the face's own cell size, auto from `auto_air_*_factor` x the wavelength at f_min. In those two modes the `air_*_mm` fields are display state. Only `air_mode=mm` reads them.
`air_cells` || integer || 6 || cells || || Air cells on every face when `air_mode` is cells. The export resolves the count against the face's own cell size (the coarser in-plane cell laterally, dz above and below).
`auto_air_enabled` || boolean || false || bool || || Size the air margins automatically from the wavelength at f_min instead of using the fixed `air_*_mm` values.
`auto_air_sides_factor` || number || 0.25 || fraction || || Side air margin as a fraction of the wavelength at f_min when auto air is on.
`auto_air_above_factor` || number || 0.25 || fraction || || Above air margin as a fraction of the wavelength at f_min when auto air is on.
`auto_air_below_factor` || number || 0.25 || fraction || || Below air margin as a fraction of the wavelength at f_min when auto air is on.

## Run length and telemetry

Setting || Type || Default || Units || Notes || Description
`typical_speed_gcells_per_s` || number || 10 || GCell/s || ui-only || Saved throughput estimate used only for the UI runtime prediction.
`auto_max_steps` || integer || 1000000 || steps || || Auto-mode safety cap on backend timesteps. In Advanced mode, `adv_max_steps` applies only when `adv_max_steps_enabled` is true. The max-steps CLI option forces a one-run cap in either mode. **Caution:** If the cap is hit before ringdown, the S-parameter and radiation results are not physically converged.
`ringdown_end_crit` || number || 1e-05 || ratio || || Energy ratio (relative to peak) at which the run stops. Smaller means a longer, more converged run. Auto mode derives it from quality. Advanced mode honors the value as written. The floor is -120 dB: the control is logarithmic and zero has no dB representation.
`ringdown_check_steps` || integer || 250 || steps || || How often, in steps, the ringdown energy criterion is evaluated. Also the step-telemetry window and the engines' temporal-blocking round length, so it is consumed on every run including CW.
`ringdown_min_steps` || integer || 0 || steps || || Optional user floor before the ringdown criterion may stop the run, above the built-in floor of two excitation lengths that every run keeps. 0 (the default) adds nothing above that floor.
`sparam_delta_crit_db` || number || -40 || dB || || Threshold on the change in S-parameters between ringdown checks. It must be negative when `sparam_early_stop` is on. The threshold itself never enables or disables the stop.
`sparam_delta_check_steps` || integer || 0 || steps || advanced || How often, in steps, the S-parameter early-stop delta is evaluated. 0 reuses `ringdown_check_steps`.
`sparam_delta_consecutive` || integer || 2 || count || advanced || Consecutive stable S-parameter checks required before the early stop fires.
`sparam_min_steps` || integer || 10000 || steps || || Minimum steps before the S-parameter early stop is allowed to fire (a floor so it cannot stop during source excitation).
`sparam_early_stop` || boolean || false || bool || advanced || Enable the early stop once the S-parameters stop changing by `sparam_delta_crit_db` across consecutive checks. This boolean is the sole enable for the criterion.
`telemetry_interval` || integer || 25 || steps || ui-only || Emit step telemetry every N steps. Lower values update the live charts more often at some compute cost.
`adv_max_steps` || integer || 1000000 || steps || advanced || Hard step cap applied when 'Max steps' is enabled. Honored on the GUI and CLI run paths. **Caution:** If the cap is hit before ringdown, the S-parameter and radiation results are not physically converged.
`adv_max_steps_enabled` || boolean || false || || advanced || When set, the run stops at `adv_max_steps`. Honored on the GUI and CLI run paths.

## Mesh convergence study

Setting || Type || Default || Units || Notes || Description
`auto_mesh_enabled` || boolean || false || bool || advanced || Advanced-mode toggle for the mesh convergence study.
`auto_convergence_enabled` || boolean || false || bool || || Run a multi-pass mesh convergence study, refining until the tracked resonances stabilise.
`conv_source` || string || S11 || || enum: S11 / S21, advanced || Which S-parameter the convergence study tracks.
`conv_num_resonances` || integer || 1 || count || advanced || Number of deepest resonance dips matched between convergence passes.
`conv_freq_threshold_db` || number || -20 || dB || advanced || Allowed per-resonance frequency shift between passes, as 10*log10(df/f) (-20 dB is about 1 percent).
`conv_depth_threshold_db` || number || -20 || dB || advanced || Allowed per-resonance depth change between passes, in dB of the linear-magnitude difference.
`conv_mesh_refine_factor` || number || 1.26 || factor || advanced || Mesh division factor applied each convergence pass. **Caution:** 1.26 is about cbrt(2), roughly doubling cell count per pass. Larger values explode the cell count. Below 1 the step would coarsen instead of refine.
`conv_max_passes` || integer || 8 || count || advanced || Maximum number of convergence passes before stopping.

## Radiation (NF2FF)

Setting || Type || Default || Units || Notes || Description
`enable_nf2ff` || boolean || true || bool || || Compute the 3D radiation pattern, gain, and directivity via a near-field to far-field transform.
`skip_geometry_export` || boolean || false || bool || || Skip writing geometry.vtk / geometry.vdb when a radiation or 3D-field run would normally request it. A suppression: with no geometry output requested it changes nothing and is still honored.
`nf2ff_num_freqs` || integer || 51 || count || || Number of frequency points for the radiation-pattern output. The solver's NF2FF buffers hold 512 points, a fixed backend contract.

## Surface currents

Setting || Type || Default || Units || Notes || Description
`enable_surface_currents` || boolean || false || bool || || Record surface current density on a conductor layer (time-domain frames plus a frequency-domain DFT).
`surface_current_layer` || string || || || || Conductor layer name to capture surface currents on.
`surface_current_interval_auto` || boolean || true || bool || || Pick the time-domain capture stride automatically for 10 samples per period at f_max. **Caution:** When on, this overrides the manual `surface_current_interval`.
`surface_current_interval` || integer || 50 || steps || || Save a time-domain surface-current frame every N steps when the auto stride is off.
`surface_current_num_freqs` || integer || 63 || count || || Number of frequency points for the surface-current DFT. The solver's DFT buffers hold 512 points, a fixed backend contract.
`surface_current_view_domain` || string || frequency || || enum: frequency / time, ui-only || Saved viewer preference for surface currents: frequency-domain or time-domain.

## 3D field export

Setting || Type || Default || Units || Notes || Description
`save_vtk` || boolean || false || bool || || Write full 3D field frames during the run. **Caution:** Produces very large files and is disk-bound. Enable only when you need full-volume fields.
`export_e_mag` || boolean || true || bool || || Write the E-field magnitude |E| (V/m) for each exported frame. The default field-export content. **Caution:** Turn it off only to export H or energy alone. With all three field types off there is nothing to write.
`export_h_mag` || boolean || false || bool || || Also write the H-field magnitude |H| (A/m) for each exported frame. H is time-centered to the E level before the magnitude.
`export_energy` || boolean || false || bool || || Also write the total field energy density U (J/m^3) for each exported frame, combining the aligned E and H at each cell.
`export_ex` || boolean || false || bool || || Also write the signed Ex component (V/m) for each exported frame.
`export_ey` || boolean || false || bool || || Also write the signed Ey component (V/m) for each exported frame.
`export_ez` || boolean || false || bool || || Also write the signed Ez component (V/m) for each exported frame.
`export_hx` || boolean || false || bool || || Also write the signed Hx component (A/m) for each exported frame, time-centered to the E level.
`export_hy` || boolean || false || bool || || Also write the signed Hy component (A/m) for each exported frame, time-centered to the E level.
`export_hz` || boolean || false || bool || || Also write the signed Hz component (A/m) for each exported frame, time-centered to the E level.
`vtk_dump_interval` || integer || 50 || steps || || Write a field frame every N steps when field export is on and the auto stride is off.
`vtk_dump_interval_auto` || boolean || false || bool || || Floor the field-export stride to the Nyquist-safe value derived from f_max. **Caution:** When on, this overrides the manual `vtk_dump_interval`.
`field_export_range_enabled` || boolean || false || bool || || Limit field frames to a step window. The solver still runs every step but skips writes outside it. **Caution:** Off by default, so frames are written at the export stride for the entire run when `save_vtk` is on.
`field_export_range_start` || integer || 0 || steps || || First step (inclusive) written when the field-export range is enabled.
`field_export_range_end` || integer || 10000 || steps || || Last step (inclusive) written when the field-export range is enabled.
`field_format` || string || rff || || enum: rff / vtk / vdb || 3D field storage format: RFF compresses signed logarithmic samples without threshold pruning, VTK stores fp32 samples, and VDB stores sparse volumes.
`vdb_threshold_frac` || number || 0.001 || fraction || || VDB sparsification cutoff as a fraction of each selected array's peak magnitude. Voxels below it collapse to the pruned background (0 disables). **Caution:** Very small values keep more of the wave tail and enlarge files.

## Backend selection

Setting || Type || Default || Units || Notes || Description
`use_cpu_backend` || boolean || none || bool || || Run on the CPU (OpenMP) backend. None defers to the saved app preference. The GUI auto-enables this when no CUDA GPU is found.

## Mode and quality

Setting || Type || Default || Units || Notes || Description
`sim_mode` || string || auto || || enum: auto / advanced || Simulation mode: auto derives the mesh from quality, advanced uses the explicit mesh and boundary fields.
`auto_quality` || number || 1 || level || || Continuous auto-mode quality from 0.0 to 3.0, interpolating cells per wavelength, PML, and convergence criteria.
`auto_min_feature_x_mm` || number || 0 || mm || || Smallest feature the mesh resolves in X. Set it and the mesh refines X to resolve it. Measured copper below it is reported as a warning, not silently meshed. 0 means no feature-size constraint on X, so measured copper only warns and the cell stays at the cells-per-wavelength and dielectric baseline. **Caution:** Per-axis and independent. Setting one axis does not tighten the others, which can leave an anisotropic mesh.
`auto_min_feature_y_mm` || number || 0 || mm || || Smallest feature the mesh resolves in Y. Set it and the mesh refines Y to resolve it. Measured copper below it is reported as a warning, not silently meshed. 0 means no feature-size constraint on Y, so measured copper only warns and the cell stays at the cells-per-wavelength and dielectric baseline.
`auto_min_feature_z_mm` || number || 0 || mm || || Smallest feature the mesh resolves in Z. A pure resolution floor: the mesh refines Z to resolve it. 0 means no feature-size constraint on Z, so the cell stays at the dielectric and cells-per-wavelength baseline. Stackup layers are always resolved and never excluded.

## Editor preferences

Setting || Type || Default || Units || Notes || Description
`editor_grid_mm` || number || 1 || mm || ui-only || Editor snap-grid spacing.
`snap_enabled` || boolean || true || bool || ui-only || Whether the editor snaps to the grid.

## Result-viewer UI state

Setting || Type || Default || Units || Notes || Description
`fv_viewer_mode` || string || currents || || enum: fields / currents, ui-only || Field viewer active mode.
`fv_cmap` || string || coolwarm || || ui-only || Field viewer colormap name.
`fv_log_scale` || boolean || false || bool || ui-only || Field viewer linear vs log scale (fields only).
`fv_field_scale_mode_idx` || integer || 1 || || ui-only || Field viewer scale mode index (0 frame-based, 1 local, 2 global).
`fv_field_ref_frame` || integer || 0 || || ui-only || Field viewer reference frame index for scaling.
`fv_show_geometry` || boolean || true || bool || ui-only || Field viewer geometry overlay visibility.
`fv_geom_opacity` || integer || 100 || percent || ui-only || Field viewer geometry overlay opacity in percent.
`fv_photorealistic_geometry` || boolean || true || bool || ui-only || E/H field viewer geometry uses studio lighting, matte cream dielectric, rough copper, local field illumination, and contact shading around copper edges.
`fv_show_meshing` || boolean || false || bool || ui-only || Field viewer geometry shows mesh cell outlines, independently of material lighting.
`fv_max_fps` || number || 30 || FPS || ui-only || Maximum playback rate for E/H fields and time-domain surface currents, with slower playback when loading or rendering takes longer.
`fv_field_frame_idx` || integer || 0 || || ui-only || Field viewer current frame index.
`fv_sc_freq_idx` || integer || -1 || || ui-only || Surface-current viewer frequency index (-1 defaults to centre).
`fv_sc_time_idx` || integer || 0 || || ui-only || Surface-current viewer time-frame index.
`fv_sc_scale_mode_idx` || integer || 2 || || ui-only || Surface-current viewer scale mode index (0 ref-based, 1 local, 2 global).
`fv_sc_ref_idx` || integer || 0 || || ui-only || Surface-current viewer reference index for scaling.
`fv_sc_cmap` || string || inferno || || ui-only || Surface-current viewer colormap name.
`fv_rad_freq_idx` || integer || -1 || || ui-only || Radiation tab frequency index (-1 defaults to centre).
`fv_rad_viewer_mode` || string || polar || || enum: polar / 3d, ui-only || Radiation viewer mode.
`fv_rad_show_geometry` || boolean || true || bool || ui-only || Radiation viewer geometry overlay visibility.
`fv_rad_show_pattern` || boolean || true || bool || ui-only || Radiation viewer 3D pattern visibility.
`fv_rad_geom_opacity` || integer || 100 || percent || ui-only || Radiation viewer geometry opacity in percent.
`fv_rad_show_meshing` || boolean || true || bool || ui-only || Radiation viewer geometry shows mesh cell outlines.
`fv_rad_photorealistic` || boolean || false || bool || ui-only || Radiation viewer geometry uses matte cream dielectric and rough copper with physical lighting.
`fv_rad_pattern_opacity` || integer || 85 || percent || ui-only || Radiation viewer pattern-surface opacity in percent.
`fv_rad_polar_floor_dbi` || number || -25 || dBi || ui-only || Radiation viewer polar plot floor in dBi. A sample below it is drawn at the chart center, and the floor ring is labeled with this value.
`fv_rad_auto_position` || boolean || true || bool || ui-only || Radiation viewer auto-computes the pattern origin.
`fv_rad_auto_scale` || boolean || true || bool || ui-only || Radiation viewer auto-computes the pattern scale.
`fv_rad_pos_x_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin X.
`fv_rad_pos_y_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin Y.
`fv_rad_pos_z_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin Z.
`fv_rad_pattern_scale` || number || 1 || factor || ui-only || Radiation viewer manual pattern scale factor.
`fv_rad_split_view` || boolean || true || bool || ui-only || Radiation viewer shows polar and 3D side by side.
`fv_rad_split_sizes` || array || [] || || ui-only || Radiation viewer splitter pixel widths.
`fv_rad_camera_position` || array || [] || || ui-only || Saved radiation viewer 3D camera (9 floats: position, focal point, up vector).
`fv_mesh_show_meshing` || boolean || true || bool || ui-only || Simulate mesh preview shows mesh cell outlines.
`fv_mesh_photorealistic` || boolean || false || bool || ui-only || Simulate mesh preview uses matte cream dielectric and rough copper with physical lighting.
`fv_sparams_visible_panels` || array || ['magnitude', 'smith'] || || ui-only || S-parameter tab visible panels, in canonical order (magnitude, smith, impedance, network). Never empty. The matching-network panel is selectable only when the run recorded network loss.
`fv_sparams_split_sizes` || array || [] || || ui-only || S-parameter tab splitter pixel widths, one per visible panel.

## Advanced-mode shadow values

Every Advanced field has an `adv_*` shadow that holds the Advanced-mode value while the project is in Auto mode. Each shadow is flagged **advanced, ui-only** and carries the same meaning as the setting it mirrors, documented above.

Setting || Type || Default || Units || Mirrors
`adv_dx_mm` || number || 0.2 || mm || `dx_mm`
`adv_dy_mm` || number || 0.2 || mm || `dy_mm`
`adv_dz_mm` || number || 0.2 || mm || `dz_mm`
`adv_air_around_mm` || number || 10 || mm || `air_around_mm`
`adv_air_above_mm` || number || 10 || mm || `air_above_mm`
`adv_air_below_mm` || number || 10 || mm || `air_below_mm`
`adv_auto_air_enabled` || boolean || false || bool || `auto_air_enabled`
`adv_auto_air_sides_factor` || number || 0.25 || fraction || `auto_air_sides_factor`
`adv_auto_air_above_factor` || number || 0.25 || fraction || `auto_air_above_factor`
`adv_auto_air_below_factor` || number || 0.25 || fraction || `auto_air_below_factor`
`adv_air_mode` || string || cells || || `air_mode`
`adv_air_cells` || integer || 6 || cells || `air_cells`
`adv_pml_thickness_mm` || number || 4 || mm || `pml_thickness_mm`
`adv_auto_pml_factor` || number || 0.1 || fraction || `auto_pml_factor`
`adv_pml_mode` || string || cells || || `pml_mode`
`adv_pml_cells_manual` || integer || 8 || cells || `pml_cells_manual`
`adv_ringdown_end_crit` || number || 1e-05 || ratio || `ringdown_end_crit`
`adv_ringdown_min_steps` || integer || 0 || steps || `ringdown_min_steps`
`adv_nf2ff_num_freqs` || integer || 51 || count || `nf2ff_num_freqs`
`adv_save_vtk` || boolean || false || bool || `save_vtk`
`adv_vtk_dump_interval` || integer || 50 || steps || `vtk_dump_interval`
`adv_vtk_dump_interval_auto` || boolean || false || bool || `vtk_dump_interval_auto`
`adv_field_format` || string || rff || || `field_format`
`adv_vdb_threshold_frac` || number || 0.001 || fraction || `vdb_threshold_frac`
`adv_field_export_range_enabled` || boolean || false || bool || `field_export_range_enabled`
`adv_field_export_range_start` || integer || 0 || steps || `field_export_range_start`
`adv_field_export_range_end` || integer || 10000 || steps || `field_export_range_end`
`adv_auto_mesh_enabled` || boolean || false || bool || `auto_mesh_enabled`
`adv_conv_source` || string || S11 || || `conv_source`
`adv_conv_num_resonances` || integer || 1 || count || `conv_num_resonances`
`adv_conv_freq_threshold_db` || number || -20 || dB || `conv_freq_threshold_db`
`adv_conv_depth_threshold_db` || number || -20 || dB || `conv_depth_threshold_db`
`adv_conv_mesh_refine_factor` || number || 1.26 || factor || `conv_mesh_refine_factor`
`adv_conv_max_passes` || integer || 8 || count || `conv_max_passes`
`adv_enable_surface_currents` || boolean || false || bool || `enable_surface_currents`
`adv_surface_current_layer` || string || || || `surface_current_layer`
`adv_surface_current_interval_auto` || boolean || true || bool || `surface_current_interval_auto`
`adv_surface_current_interval` || integer || 50 || steps || `surface_current_interval`
`adv_surface_current_num_freqs` || integer || 63 || count || `surface_current_num_freqs`
`adv_mesh_grading_x_mult` || number || 1 || factor || `mesh_grading_x_mult`
`adv_mesh_grading_y_mult` || number || 1 || factor || `mesh_grading_y_mult`
`adv_mesh_grading_z_mult` || number || 1 || factor || `mesh_grading_z_mult`
`adv_mesh_grading_margin_mm` || number || 1 || mm || `mesh_grading_margin_mm`
`adv_mesh_grading_x_max_ratio` || number || 1.3 || ratio || `mesh_grading_x_max_ratio`
`adv_mesh_grading_y_max_ratio` || number || 1.3 || ratio || `mesh_grading_y_max_ratio`
`adv_mesh_grading_z_max_ratio` || number || 1.3 || ratio || `mesh_grading_z_max_ratio`
`adv_cw_enabled` || boolean || false || bool || `cw_enabled`
`adv_cw_freq_hz` || number || 2.4e+09 || Hz || `cw_freq_hz`
`adv_sparam_early_stop` || boolean || false || bool || `sparam_early_stop`
`adv_sparam_delta_crit_db` || number || -40 || dB || `sparam_delta_crit_db`
`adv_sparam_min_steps` || integer || 10000 || steps || `sparam_min_steps`
`adv_enable_nf2ff` || boolean || true || bool || `enable_nf2ff`
`adv_skip_geometry_export` || boolean || false || bool || `skip_geometry_export`

---

# RF calculators

Closed-form transmission-line and antenna calculators for sizing a starting geometry before a full FDTD run. They open in a floating overlay in the Edit tab, and the same math is available headless through `rayrf calc`.


## The overlay

Open the overlay with Ctrl+R on the Edit tab, or with the Calculators button under the layer list in the Stackup section. Both toggle the same floating panel.

!fig RF calculators overlay open on the editor

The panel floats over the canvas. Drag it by its title bar, and resize it from any edge or corner. A dropdown at the top selects the calculator. Resize the panel larger and the dropdown becomes a grid of buttons, one per calculator. The body shows that calculator's inputs, its results, and a live cross-section preview that redraws as you type. Results are read-only text, so a value is shown exactly as the formula returned it at any scale. When a value leaves a formula's validity window, an amber notice under the form states the bound. When an input is outside the range the formula is defined on, the field turns red, the results and the preview clear, and the notice names the value and the limit. The calculators are read-only analytic tools, so they stay available when a project is open read-only.

Where a calculator has both an impedance and a width, a Solve for choice at the top of the page picks which one is the answer. The quantity being solved leaves the form and its value appears in the results.

## Pull from stackup

Each calculator that takes a substrate has a Pull from stackup checkbox. Check it, pick a Top and a Ref conductor layer, and the calculator fills the substrate height and epsilon_r from the active project between those layers, then locks those two fields. When more than one dielectric slab sits in the gap, the epsilon_r is a thickness-weighted average and the detail line lists the blend. Uncheck it to type the substrate values in directly. See [Stackup and layers](docs-stackup-and-layers.html) for how layers are defined.

## Calculators

Eleven calculators ship, in this order. Each solves the closed form named in its walkthrough. Wavelengths are reported as free-space lambda0 and guided lambda_g, and every guided-line calculator also reports the quarter wave lambda_g/4. The loss tangent is optional on every calculator that takes one: with it, the result adds the dielectric loss in dB/m and the dielectric Q, and without it those two rows are absent rather than zero.

Calculator || Inputs || Returns
Microstrip || h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
GCPW || gap S, h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
Stripline || plate spacing b, epsilon_r, copper t, frequency, tan delta, and Z0 or W || the other of Z0/W (IPC-2141A), epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q, and the copper thickness used
Patch || frequency, h, epsilon_r || patch W and L, epsilon_eff, the length extension delta L, lambda0, the radiating-edge resistance, and the fractional bandwidth
lambda Calc (guided wavelength) || frequency, and in PCB mode h, epsilon_r, trace W || lambda0, epsilon_eff, lambda/2, lambda/4
CPW || gap S, h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff (no bottom ground), lambda_g, lambda_g/4, dielectric loss and Q
Diff Pair (differential pair) || h, epsilon_r, spacing S, frequency, and Zdiff or W || the other, plus Z0 even/odd (Kirschning-Jansen coupled lines), Zcommon, and the odd-mode epsilon_eff and lambda_g
Embedded MS || W, h, epsilon_r, cover height h2, cover epsilon_r, frequency, tan delta || Z0, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
lambda0 free (free-space wavelength) || frequency || lambda0, half, quarter
lambda/4 xfmr (quarter-wave transformer) || Z1, Z2, frequency, h, epsilon_r || Zt = sqrt(Z1 Z2), length lambda_g/4, epsilon_eff, the residual Gamma and return loss, and microstrip widths W1, Wt, W2
Smith chart || R, X, system Z0 || Gamma magnitude, phase, and real and imaginary parts, VSWR, return loss, mismatch loss

The patch calculator sizes a rectangular patch from the transmission-line model and reports the resistance at the radiating edge so a feed can be designed against it. Feed matching is left to the geometry: the inset or the feed line width is drawn in the editor and checked by the run.

## The expression strip

A single expression line is at the bottom of the panel. Type arithmetic (`+ - * / **` and parentheses, numbers only) and press Enter to evaluate. The two buttons take the square root and cube root of the current result, so sqrt(Z1*Z2) is entered as 50*75 then the root button. The square-root button is offered only while the result has a real root. A negative result disables it and the button says why. A rejected expression shows err.

## Step-by-step breakdown

Each calculator footer links to Step-by-step breakdown on RayRF, which opens the matching walkthrough at [rayrf.com/calculators](rf-calculators.html) for that structure with the formula, its derivation, and the validity range of the model.

## CLI equivalent

`rayrf calc <kind>` runs the same formulas headless and prints a table, or `--json` for a machine-readable report. The kinds are `microstrip`, `stripline`, `cpw`, `gcpw`, `diff`, `embedded`, `patch`, `lambda`, `lambda-diel`, `qwt`, and `smith`. Inputs are given in GHz, mm, and ohms. Omit `--w` and pass `--z0` to solve a width from a target impedance. `--tand` is optional wherever a calculator reports dielectric loss, and the loss rows appear only when it is given. Stripline's `--t` is the copper thickness and defaults to 0.035 mm, and 0 is a strip of no thickness. `smith` uses 50 ohm when `--z0` is omitted. The CLI reports the same quantities as the overlay.

```
rayrf calc patch --er 4.4 --h 1.6 --freq 2.45
rayrf calc microstrip --freq 2.45 --er 4.4 --h 1.6 --tand 0.02 --z0 50
rayrf calc qwt --z1 50 --z2 75 --freq 2.4 --er 3.66 --h 0.5
```

The full flag list for `calc` is on the [CLI reference](docs-cli-reference.html).

---

# Keyboard shortcuts

Every keyboard shortcut in RayRF, grouped by area. Drawing, constraint, and canvas keys act on the Edit tab only, and every single-letter key is ignored while a text field or spin box has focus so typing a value never arms a tool.


## Application

These work from any tab.

Key || Action
`Ctrl+N` || New project
`Ctrl+O` || Open project
`Ctrl+S` || Save
`Ctrl+Shift+S` || Save As
`Ctrl+Z` || Undo
`Ctrl+Y` || Redo
`Ctrl+Shift+Z` || Redo

## Drawing tools

Each key selects the matching tool in the Edit tab toolbar. The Arc, Cutout, More Ports, Import PNG, and Script Shape tools have no key and are reached from the toolbar.

Key || Tool
`Esc` || Select (also cancels the active tool)
`R` || Rectangle
`C` || Circle
`P` || Polygon
`F` || Via
`O` || Hole
`Q` || Point port
`A` || Area port
`M` || Ruler

## Constraint and dimension tools

Each key arms a pick tool that then waits for you to click the anchors, edges, or rims it needs.

Key || Tool
`D` || Dimension
`I` || Coincident
`H` || Horizontal
`V` || Vertical
`T` || Tangent
`N` || Angle
`K` || Fix
`E` || Equal

## Editing the selection

These act on the current selection in the Edit tab canvas. Cut and paste are blocked in viewer mode, copy stays available.

Key || Action
`Ctrl+A` || Select all shapes
`Delete` || Delete the selection
`Backspace` || Delete the selection
`Space` || Rotate the selection 90 deg clockwise
`X` || Mirror the selection horizontally
`Y` || Mirror the selection vertically
`Ctrl+C` || Copy
`Ctrl+X` || Cut
`Ctrl+V` || Paste

While placing a port, Space or R rotates the port ghost 90 deg instead of rotating a selection.

## Canvas, grid, and layers

Key || Action
`S` || Toggle snap to grid
`Shift+S` || Cycle the view mode
`-` || Shrink the grid spacing one step
`=` || Grow the grid spacing one step
`+` || Grow the grid spacing one step
`1` || Select layer 1 (through 9 for layer 9)

## Stackup layers

These act on the Stackup layer list when it has focus, and on the layer bars in Mesh View.

Key || Action
`Delete` || Delete the selected layer
`Ctrl+Up` || Move the selected layer up one place
`Ctrl+Down` || Move the selected layer down one place

In Mesh View, Up and Down select the layer above or below.

## RF calculators

Key || Action
`Ctrl+R` || Toggle the RF calculators overlay

## S-parameter markers

These work on the S-parameters plot.

Key || Action
`M` || Toggle marker placement
`Esc` || Cancel marker placement
`Delete` || Delete the selected marker
`Backspace` || Delete the selected marker

## 3D views

The 3D mesh preview, the Radiation Pattern tab, and the Field Viewer share one set of mouse controls.

Input || Action
Left drag || Orbit
Right drag, middle drag, or Shift plus left drag || Pan
Wheel || Zoom toward the point under the pointer
`Esc` || End the drag

---

# File formats and outputs

File formats and filenames for RayRF projects, exported solver cases, and simulation results.


## Project files

A project is one JSON document with the `.rfsim` extension. It holds the whole project: the layer stackup, primitives, domain, FDTD settings, parametric variables and constraints and generators, native copper artwork, embedded conductor bitmaps, import provenance, the editor thumbnail, saved view state, and (unless externalized) the last run's simulation results. Imported reference traces remain embedded in the project. Run History also indexes a copy beside the project while leaving the embedded reference unchanged. The multi-run archive itself lives only in Run History.

The document carries a version, currently 32. An older release refuses to open a file with a newer version and reports both versions rather than silently dropping the fields it does not understand. Legacy `.emflux` project files still open.

Native artwork stores apertures, paths, regions, and polarity. Imported raster copper uses compressed one-bit masks. Existing embedded PNG data remains readable.

## Results sidecar

A project holds its latest run results inside the `.rfsim`, and can instead keep that heavy bundle out of it in a sidecar file next to it with the same base name and the `.rfsimout` extension, so `Patch.rfsim` pairs with `Patch.rfsimout`. The point is version control: commit the `.rfsim`, gitignore the `.rfsimout`. Only the latest simulation results move to the sidecar. Geometry, settings, plot settings, and imported references stay in the `.rfsim`.

Externalization is set two ways:

- **Per project**, from the "Separate geometry and settings from results into two files" option in Project Settings. The choice is saved in the `.rfsim` and is the default for every save.
- **Per run**, from the CLI: `rayrf run --externalize-results project|run|off`. `off` keeps results inline, the others write the sidecar.

If a sidecar carrying results cannot be written, the save fails with the sidecar error and leaves the existing project file alone. If the project says it has a sidecar and that sidecar is missing, corrupt, from a newer build, or from a different project, the open is refused rather than quietly showing you a project with no results. The two files carry the same project-state hash and the same result-payload digest, so a half-finished two-file save is refused too, even when two runs happened to use identical inputs. A no-results save records that absence in the project and ignores older same-stem sidecars, preventing a discarded result from returning. A project that has never externalized results opens normally before its first run.

## Run History folder

Run History is a separate bounded archive named `<stem>_history` beside the project. It holds content-addressed project snapshots and per-record results for completed runs, save points, parameter-sweep points, and copies of imported references. It is not the `.rfsimout` file and is not controlled by result externalization. Move or rename this folder with its `.rfsim` when the archive should follow the project. See [Run history](docs-run-history.html) for its record and pruning rules.

## Exported solver case

Before a run, the project is serialized into a case directory under `_sim_export/<case>/`. This is the solver input, and it is what a remote submit uploads. It contains `files.json` (the case manifest, schema version 3), one PNG bitmask raster per conductor and dielectric layer plus via and area-port masks, and `_license.token`, the signed machine-bound token the backend checks before it runs. The internal `files.json` fields are not part of the documented interface and have no `rayrf schema` target.

## Interchange exports

The File menu and headless project commands write these formats:

Export || File type || Content
DXF || `.dxf` || Resolved 2D layer geometry, via drills, holes and slots, cutouts, port footprints, and the board outline.
openEMS XML || `.xml` || Resolved geometry, materials, Gaussian pulse, mesh, boundaries, vias, holes and cutouts, and supported lumped ports.

Each export shows or prints exported, approximated, and omitted content. See [Exporting DXF drawings](docs-export-dxf.html) and [Exporting openEMS XML](docs-export-openems.html) for the exact mappings.

## Run outputs

A run writes to `<project_dir>/_sim_result/<case>/` by default, or to the directory given by `--out-dir`. The backend runs in that directory (in a `backend_run/` subdirectory when `--out-dir` is set) and writes its own raw files there. The post-processed artifacts, named after the case, are controlled by the CLI export flags. GUI runs and CLI runs place these files the same way.

Every CLI verb except `auth`, `schema`, `guide`, and `calc` requires a Pro license, including every run.

Post-processed artifacts, in the output directory:

Artifact || Filename || When
S-parameter table || `<case>.csv` || Always
S-parameter plot || `<case>.png` || Always
Touchstone || `<case>.sNp`, N the port count || `--export-touchstone`, see below
Smith chart || `<case>_smith.png` || `--export-smith`
Polar pattern || `<case>_polar.png` || `--export-polar`, with radiation data
Mesh preview || `<case>_mesh.png` || `--export-mesh`
Mesh preview grid || `<case>_mesh.vtk` || `--export-mesh-vtk`
3D radiation || `<case>_rad3d.png` || `--export-rad3d`, with radiation data
3D radiation surface || `<case>_rad3d.vtp` || `--export-rad3d-vtk`
Surface-current heatmap || `<case>_currents.png` || `--export-currents`
Radiation pattern table || `<case>_rad_pattern.csv` || `--export-rad-pattern`
Run manifest || `run_manifest.json` || Always

The S-parameter table is one long-form row per trace and frequency. Its columns are `param,f_MHz,real,imag,mag,mag_dB`, so the frequency column is in MHz.

The radiation pattern table concatenates the per-frequency backend patterns into one flat CSV. Its columns are `freq_hz` followed by the backend pattern columns: `freq_hz,theta_deg,phi_deg,D_total_dBi,D_theta_dBi,D_phi_dBi`.

Raw files the backend writes into the run directory:

Artifact || Filename || When
Geometry || `geometry.vtk`, plus `geometry.vdb` when `field_format=vdb` || NF2FF or field export on
S-parameters || `sparams.csv` || Runs with ports
Port time series || `port_signals_raw.csv` || `--export-port-csv`
Per-port voltage || `port_NN_voltage.csv` || `--export-port-csv`
Per-port reflection || `port_NN_s11.csv` || Always
Radiation summary || `nf2ff_summary.csv` || NF2FF on
Radiation per frequency || `nf2ff_pattern_fNN.csv` || NF2FF on
Surface-current index || `jsurf_meta.json` || Surface currents on
Surface-current geometry || `jsurf_geometry.bin` || Surface currents on
Surface-current time frame || `jsurf_td_NNNN.bin` || Surface currents on
Surface-current frequency frame || `jsurf_fd_NN.bin` || Surface currents on
3D field frame || `step_NNNN[_type].rff`, `.vtk`, or `.vdb` || `save_vtk` on
Field frame index || `frames.json` || 3D field export on

`geometry.vtk` includes the cell arrays `material` (air, conductor, dielectric, port, or PEC edge carrier) and `material_background` (0 for air, 2 for dielectric), so the medium under a PEC edge carrier stays readable.

The Field Viewer and Radiation Pattern tabs draw the board from the geometry recorded with the result, never from the current project's outlines or copper thickness.

Each selected field quantity writes its own frame file per capture step: E magnitude has no type suffix, and the others append a type suffix (`_h` for |H|, `_u` for energy density, `_ex` .. `_hz` for the signed components). `frames.json` lists the emitted quantities under `arrays` and records each frame's display-range stats, which the field viewer folds into its Global color range without re-reading the frames.

The `sparams.csv` file holds the S-parameters the backend computes online, one row per frequency. Its columns are `freq_hz`, then a real and imaginary pair per port for the driven column (`s_i_j_real,s_i_j_imag`, where i is the receiving port and j is the driven port, so a 4-port run driven at port 1 has `s_1_1` to `s_4_1`), then `zin_real,zin_imag` for the driven port's input impedance. This payload, not the time series, is what a saved project stores so the S-parameter tabs and headless exports work after reopening.

A cell is empty where the run measured nothing: an S-parameter or input impedance at a bin the excitation never reached, and a directivity in `nf2ff_summary.csv` at a frequency that radiated no power. The same rule holds in `port_NN_s11.csv`. Every reader in RayRF treats an empty cell as unmeasured, never as zero, and a script parsing these files needs to do the same.

The raw port time series is opt-in. `--export-port-csv` writes `port_signals_raw.csv` (columns `t_s,V1,I1`) and the per-port voltage CSVs for debugging and external post-processing. Without it, only the S-parameters and the per-port reflection tables are written.

## Touchstone

Touchstone export writes measured values as measured and discloses any filled value in the file. The port count decides the extension: `.s1p` for one port, `.s2p` for two, `.s3p` for three, and so on with the port count in the name. One writer serves the S-Parameters tab and the headless exporter, so both produce the same file.

A two-port file fills an unsimulated S12 from S21 and an unsimulated S22 from S11, and writes each fill into the file as a comment:

```
! S12 was not simulated: filled with S21 (reciprocity assumption)
! S22 was not simulated: filled with S11 (symmetry assumption)
```

In a three-or-more-port file a pair the run did not measure is written as 0 and named in a `!` comment. Per-port reference impedances that differ are renormalized to one reference, which is also disclosed. That operation needs every term, so a matrix missing a pair is refused rather than renormalized against invented zeros.

## RayRF compressed field export

With `field_format=rff`, each quantity is stored as an RFF archive. The archive contains its float64 coordinate axes, quantity, extrema, display range, signed logarithmic samples, and a pyramid of coarser levels. Each level stores independently compressed bricks. Fine samples retain their nonzero state and sign, while the archive records its relative quantization error bound. Coarse brick extrema include the fine samples beneath them.

`rayrf fields inspect FILE.rff` reports the dimensions, levels, compression ratio, and error bound as JSON without reading the sample payload. RFF files contain visualization samples. The solver advances its fields in fp32.

`rayrf fields convert FRAME --out FILE.rff` writes a new RFF archive from one VTK or OpenVDB frame and prints the same report. `--source` selects the quantity and defaults to `E_mag`. An existing output file is refused.

## OpenVDB field export

When `save_vtk` is on and `field_format=vdb`, 3D field export writes OpenVDB volumes (`step_NNNN[_type].vdb`) instead of VTK frames, one file per selected field quantity with the same type suffixes as the VTK path, and writes `geometry.vdb` beside `geometry.vtk`. `vdb_threshold_frac` sets a sparsification cutoff as a fraction of each frame's peak magnitude: voxels below it collapse into the pruned background, and 0 disables pruning. The field viewer reads the volumes back through the native engine (`backend_cpu --vdb-to-dense`), and DCC tools that read OpenVDB import them too. VDB volumes carry one uniform spacing per axis, so they can represent anisotropic uniform meshes but not graded cell edges. Use RFF or VTK for graded runs. The volume transform uses the domain corner as world origin and places voxel index (0, 0, 0) at half of the X, Y, and Z cell spacing from that corner.

Field export writes |E| by default. `export_h_mag` adds |H|, `export_energy` adds total energy density, and `export_ex` .. `export_hz` add the signed field components, one array file for each exported frame. See [Field viewer](docs-field-viewer.html) for viewing frames.

## Imports

The editor and the S-Parameters tab read these files.

Import || Where || File types
Conductor mask || Edit tab || PNG (`*.png`)
Gerber board || File, Import || Gerber, Excellon, and Gerber job files
DXF drawing || File, Import || DXF (`*.dxf`)
KiCad board || File, Import || KiCad board (`*.kicad_pcb`)
openEMS setup or CSXCAD geometry || File, Import || XML (`*.xml`)
Reference S-parameters || S-Parameters tab || Touchstone `.s1p` to `.s8p`, `.sNp`
Reference trace || S-Parameters tab || CSV (`*.csv`, `*.txt`)

A PNG import scales the raster to a conductor pattern on a chosen layer. A Touchstone or CSV import is embedded in the project, so the source file can be deleted afterward. Run History also receives an indexed copy. DXF creates a new project through the shared layer and stackup assignment flow. Gerber and KiCad use the same assignment flow and ask whether to add the board to the current project or create a new one. openEMS also asks for a destination and does not transfer source ports, boundaries, or domain spacing. See [Gerber import](docs-import-gerber.html), [DXF import](docs-import-dxf.html), [KiCad import](docs-import-kicad.html), and [openEMS import](docs-import-openems.html) for each transfer contract. The `dxf` primitive holds an imported outline rasterized as a conductor pattern.

---

# FDTD tips

Practical guidance on boundary spacing, frequency units, convergence, mesh resolution, and field export.


## Give every PEC and PMC face an explicit spacing

Set each hard wall and its gap on the run line:

```
rayrf run --project board.rfsim --per-face-bc z_lo=PEC:2.0
```

In auto mode the air margin between the geometry and each boundary is sized for a PML absorber. A PEC or PMC face is a hard wall with no absorber, and if you leave it on that inherited margin the geometry can end up touching or sitting inside the wall, which reflects energy back into the design and corrupts the S-parameters. Set a real clearance on each hard-wall face. In the GUI, set the face to Custom, check its spacing checkbox, and enter the gap in the spacing field.

The `:MM` is a fixed gap from the geometry to that face, realized exactly and overriding the air margin there, and any per-face setting forces `bc_all` to Custom. A gap of `0` is legal and puts the wall flush, which is what you want for an infinite ground plane under the board. A Custom-mode case that reaches export with a hard-wall face and no spacing is refused with an error naming the face, so a missed gap stops the run rather than silently corrupting it. The PEC and PMC box presets keep the derived margins instead. See [Boundaries](docs-boundaries.html).

## Watch the frequency units: Hz, GHz, and MHz

Frequency units differ between the project file, GUI, and CLI. The project and spec store `freq_min_hz` and `freq_max_hz` in Hz. The GUI frequency fields show and accept GHz. The `run` and `reprocess` flags (`--freq-min`, `--freq-max`, `--fmin`, `--fmax`) take MHz. The same band is `5.8e9` in a spec, `5.8` in the GUI, and `5800` on a run flag. Get the exponent wrong and the whole band moves, along with the mesh resolution and the PML sizing that both key off it.

## A run is only valid if it rang down

Ring-down is the convergence test. The excitation is a pulse, and the S-parameters and radiation pattern are only trustworthy once the field energy in the box has decayed to `ringdown_end_crit` (an energy ratio relative to peak, smaller means a longer and more converged run). The exported runtime `max_steps` is a hard cap on backend timesteps. Auto mode derives and stores it as `auto_max_steps`. Advanced mode uses `adv_max_steps` only when `adv_max_steps_enabled` is true. The `--max-steps` option applies a transient cap to either mode without changing either stored setting. If the effective cap stops the run before ring-down, the results are not physically converged. The title bar reads STEP CAP, the badge on the result tabs warns that the run stopped before its ring-down target, and the manifest (`ringdown_reached`) records that the run was cut short. If a run keeps hitting the cap, either raise the cap for the active mode or loosen `ringdown_end_crit` toward a larger ratio, and if you want a tighter result make the criterion smaller (auto mode derives it from quality, advanced mode uses the value as written). The criterion is evaluated every `ringdown_check_steps` timesteps (default 250). Each evaluation costs a full-domain energy pass and, on the CPU backend, bounds the solver's internal blocking. The 250-step default keeps that cost small on both backends while bounding stop overshoot to the same 250 steps. The optional S-parameter early stop (`sparam_early_stop`) ends the run once |dS11| between consecutive ring-down checks holds below its threshold. It saves steps on a clean resonator but can stop a slowly settling structure a little early, so leave it off for a full-length reference run.

## CW mode does not ring down

Continuous-wave excitation (`cw_enabled`) injects a steady sinusoid instead of a pulse. A steady drive does not ring down. The run ends at the step cap you set, or at the step counter's limit if no cap is set. S-parameter early stopping is disabled. Under a single tone the S-parameters are of limited meaning, the radiation pattern at the tone is meaningful, and both outputs stay available. Field animations for the [field viewer](docs-field-viewer.html) are the usual reason to use it. It is an advanced-mode feature, and an auto-mode project with CW on is refused rather than run as a pulse.

## Keep the mesh cells comparable across axes

Setting one per-axis minimum feature and leaving the others at zero, or hand-setting `dx` and `dy` far apart in advanced mode, resolves X and Y differently and can make the S-parameters depend on feed orientation, most visibly for planar ports. `rayrf validate` and `rayrf estimate` warn when `dx` and `dy` differ by more than 3x, and when `dz` is more than 3x the finer in-plane cell, because Z carries the stackup. A `dz` finer than the in-plane cells is the normal shape of a layered stackup and draws no warning. Keep `dx` and `dy` within a factor of three of each other unless the anisotropy is deliberate. A minimum feature set on only some axes draws its own warning for the same reason.

## Paint every dielectric layer or make it air

A dielectric layer with nothing painted on it is treated as air and its epsilon is ignored. `validate` and `run` warn on this, since running the layer as air is occasionally intended. If the layer is a real substrate, paint a board-sized rectangle across it so it carries its permittivity. If the gap really is meant to be air, set the layer's type to air so the intent is explicit rather than inferred from an empty dielectric.

## In advanced mode, let dz divide the substrate

Auto mode snaps `dz` so the stackup planes land on grid lines. Advanced mode uses `dz` exactly, and if it does not divide the substrate height the conductor stackup can land between grid lines and shift the resonance. `validate` warns about a non-dividing `dz`. Either pick a `dz` that divides every dielectric thickness, or use [auto mode](docs-auto-mode.html) and let it snap for you. When you set the cell sizes by hand, resolve the finest feature you care about and put several cells through the thinnest substrate, and keep `dy` near `dx` unless one direction really is finer, which the anisotropy tip above covers.

## Resolve the drills your result depends on

A via drill or plated hole under 2 cells across still conducts, as a coarse barrel of mesh nodes or a single conductor post, and a small unplated hole opens a few cells. The Opening Findings block in Derived Settings, `rayrf validate`, and `rayrf estimate` list each group of identical drills the mesh realizes differently from the drawing and quote what resolves them: the min feature X and Y with the cells and VRAM that costs, or in advanced mode the cell size. Spend those cells where current through the drill sets the result, such as a shorting via at the end of a resonator or a feed via. See [Auto mode](docs-auto-mode.html).

## Full-field export is heavy, so window it

`save_vtk` writes a 3D field frame per captured step across the whole volume. `field_format` defaults to `rff`, compressed for visualization without threshold pruning, and `field_format=vtk` stores fp32 samples. In advanced mode, narrow the capture to a step window with `field_export_range_enabled`, `field_export_range_start`, and `field_export_range_end`. With `field_format=vdb`, `vdb_threshold_frac` prunes the weak wave tail, so lowering it retains more of the field and enlarges the files. When you only need S-parameters, the run-line kill switches force capture off regardless of the saved settings: `--no-raw-fields`, `--no-nf2ff`, and `--no-surface-currents`.

## Validate before you wait for a full run

A full run can take a while, so catch the cheap mistakes first. In the GUI, the mesh preview on the Simulate tab draws the resolved grid over the geometry before any solve, so a feature that falls between cells or a boundary sitting on the structure shows up at a glance, and the Notices panel lists the same blocking errors, advisory warnings, and notes a run would raise. Headless, three commands do the same without a backend: `rayrf validate --project FILE` runs the blocking pre-flight gate plus the advisory checks above, `rayrf estimate --project FILE` derives the mesh and prints the cell count, VRAM, boundary sizing, and ring-down so you see the size before you commit to it, and `rayrf run --project FILE --dry-run` applies your overrides, validates, exports the case, and prints the estimate, then stops before launching the solver. See the [CLI reference](docs-cli-reference.html).

## Quality slider versus a convergence study

These solve two different problems. The quality slider (0.0 to 3.0) sets the resolution targets in one move, interpolating cells per wavelength, PML depth, and ring-down criterion between the presets, and raising it refines the whole mesh uniformly. A convergence study (`auto_convergence_enabled`) instead repeats the run over successive passes, refining the mesh each pass until the tracked resonances stop moving between passes, then reports whether it converged or hit the pass cap. Use a study when a result sits near a sharp resonance and you need evidence that it has stopped shifting with mesh density, which a single higher-quality run does not give you. It costs one backend pass per refinement, so spend the passes on a validation or a final number, not on routine iteration where a `medium` or `high` quality run is enough.
