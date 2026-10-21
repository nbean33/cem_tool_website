# Importing Gerber boards
> The Gerber importer turns a fabrication file set into RayRF geometry: stackup layers from the copper files and the dielectric gaps you assign, copper artwork on those layers, holes and slots from the drill files, and the substrate and domain from the board outline. It lives in the editor's Import dropdown and in File, Import.

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
