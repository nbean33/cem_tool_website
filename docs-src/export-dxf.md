# Exporting DXF drawings
> The DXF exporter writes the current project's resolved 2D geometry as a millimeter DXF drawing with the ezdxf library (MIT licensed). It is available under File, Export and as a headless project command.

## Exporting from the editor

Select File, Export, Export DXF drawing. The export dialog lists every project layer, all checked, and three utility layers under Include: Board outline, Port footprints, and Drill holes. A utility the project has nothing for is greyed out. File layout writes a Single file or a Separate file per layer, named `name-[layer].dxf`. Click Export and choose the output file. The export runs behind a loading overlay, and Cancel stops it without leaving a partial file.

The completed export shows the entity count and any warnings, and the Console prints the outcome. The full exported, approximated, and omitted report is embedded in the drawing as leading 999 comment lines, so the file carries its own record. A failed export names the shape, input, or file operation that prevented a complete drawing.

## Exporting from the CLI

The CLI reads one saved project and writes one drawing, or one per layer:

```
machshev project export-dxf --project FILE --out FILE [--replace] [--json]
    [--layer NAME]... [--separate] [--no-outline] [--no-ports] [--no-drills]
```

`--layer` names one project layer to export and can repeat. Without it every layer is exported. `--separate` writes `FILE-[layer].dxf` for each selected layer and utility layer. `--no-outline` leaves out Board Outline and the board cutouts, `--no-ports` leaves out Ports, and `--no-drills` leaves out via drills and holes.

The `.dxf` suffix is added when `--out` does not have it. An existing output refuses unless `--replace` is set. Parent directories are created as needed. The output is prepared in the destination directory and installed only after it passes a serialized DXF readback and structural audit.

Without `--json`, the outcome line goes to stdout and each warning to stderr. `--json` writes this schema to stdout:

```
{
  "schema": "machshev-dxf-export-report/v1",
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

Machshev Labs geometry || DXF representation
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

Filled polygons, discs, rings, and arc bands include a solid HATCH and an ordinary visible boundary tagged as its companion. Machshev Labs suppresses that duplicate boundary on re-import only while its geometry still matches the HATCH boundary. If another editor removes the HATCH, the ordinary boundary remains and imports as geometry. If the boundary is edited independently, Machshev Labs retains both and reports the difference. Board Outline is boundary-only.

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
- A hole or cutout a Machshev Labs run refuses also refuses the export, by shape, where the export writes it: no width or area, a parameter its kind does not have, a span that does not end on a conductor or dielectric layer of the stack, or a via that reaches into it.
- Cutouts that cover the board, cut it into separate pieces (a ring of cutouts around an island of board does), or meet the board edge or each other at a single point refuse by shape, because Board Outline carries one board with separate cutouts inside it.
- A failed parametric resolve, invalid shape value, or structural DXF error refuses the output with a named reason.
