# Drawing geometry
> The Edit tab is a 2D drafting canvas: draw copper and dielectric shapes on the active layer, then place vias and ports through the stack. This page covers the coordinate model, the draw tools, and every shape primitive.

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
