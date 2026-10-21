# Importing DXF drawings
> The DXF importer turns a CAD drawing into a new RayRF project the same way the [Gerber importer](docs-import-gerber.html) turns a fabrication set into one: assign roles, get stackup layers, artwork, holes, and the domain rectangle. It lives in the editor's Import dropdown and in File, Import. Parsing uses the ezdxf library (MIT licensed).

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
