# Importing KiCad boards
> The KiCad importer reads one `.kicad_pcb` layout and sends it through the same layer assignment and project builder as the Gerber and DXF importers. It is available in the editor's Import dropdown and under File, Import.

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
