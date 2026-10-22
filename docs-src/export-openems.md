# Exporting openEMS XML
> The openEMS exporter writes the current resolved project as a standalone XML setup containing an openEMS FDTD section and a CSXCAD ContinuousStructure. It is visible under File, Export and as a headless project command.

## Exporting from the editor

Select Export openEMS XML under File, Export, then choose an XML file. Preparation and writing run behind a loading overlay, and Cancel stops the export without leaving a partial file. The completed export shows the entity count and any warnings, and the Console prints the outcome. The XML starts with a comment naming the format, the entity count, and the warning count. The CLI's `--json` output carries the full exported, approximated, and omitted report. Unsupported electrical content refuses before the destination is replaced, with the source project, destination XML, and shape, via, port, or layer named in the error.

## Exporting from the CLI

```
machshev project export-openems --project FILE --out FILE [--replace] [--json]
```

The `.xml` suffix is added when absent. An existing output refuses unless `--replace` is set. Without `--json`, the outcome line goes to stdout and each warning to stderr. JSON output uses `machshev-openems-export-report/v1` and contains the output path, `entity_count`, all three report categories, and the warnings.

## Setup contract

The writer resolves project variables, bindings, constraints, and generator instances on a copy. Auto-mode simulation values are derived before export, and the report lists the resolution details that affect the setup. When project domain auto-fit is enabled, export recomputes the domain from the resolved geometry with the saved padding, matching the simulation export path. The XML uses millimeters and includes one resolved Gaussian pulse, boundary faces, mesh lines, stackup materials, supported geometry, vias, holes and cutouts, and lumped ports. When mesh grading is off, the grid uses the same whole-cell board, free-space, and absorber counts as the uniform Machshev Labs solver. Stackup planes use the same uniform-cell quantization. The report lists any padded board edge, expanded free-space gap, or shifted stackup plane. A dielectric that collapses to no solver cells refuses by layer name. When grading is on, each stackup edge uses the same nearest explicit Z line as the Machshev Labs solver, and a shifted edge is listed in the report.

A standalone export requires exactly one active port. Every port needs an explicit positive number, reference impedance, and Boolean active value. Point and rectangular ports need both terminal layers. A planar line or bar port needs its bottom layer. An omitted top layer means that same conductor, while an explicitly different top layer refuses. A present planar `excitation_mode` must be `planar`. Point ports excite Z. Rectangular, line, and bar ports must be unrotated and aligned with a Cartesian axis. Their boxes are aligned to the resolved mesh and the effective coordinates are listed as approximated when that changes the authored box. Line and bar polarity follows their authored director. Passive ports contain their resistance and probes without an excitation property.

A via is written in the `Machshev Labs vias` metal as a cylinder of its drill diameter over its conductor span. Its pad is written on each conductor layer it spans as a polygon with a vertex at no more than 5-degree spacing. A via drill Machshev Labs realizes as a post at one mesh node is written as drawn and listed as approximated.

An unplated hole or a cutout is written as vacuum in the `Machshev Labs openings` material: a cylinder for a round hole, a rectangle and a disc at each end for a slot, and the outline polygon for a cutout. Over the copper layers it cuts, the vacuum has a priority above copper, so it removes the copper there. Where the opening ends on copper that stays, the vacuum stops at the center of the first mesh cell short of that copper. The dielectric it cuts is vacuum at a priority between the dielectric and copper. A plated cutout also gets its wall in `Machshev Labs opening walls`: the cell faces Machshev Labs plates on the resolved mesh, written as conductor sheets. Where the drawing keeps a copper plane clear of that wall, the plane edges Machshev Labs clears around it are written as vacuum above copper priority on that plane. The report lists the wall and the cleared planes as approximated.

A plated hole cuts nothing. It is written in `Machshev Labs opening walls` as metal from the lowest to the highest copper layer it joins: a cylinder for a round hole, a rectangle and a disc at each end for a slot. A plated hole Machshev Labs realizes as a post at one mesh node is written as drawn and listed as approximated.

Zero-thickness conductor circles, annuli, and arc bands are written as polygon segments at no more than 5-degree spacing. The report names each segmented shape. Dielectric circles with positive thickness remain cylinders.

The output is prepared beside the destination and installed only after XML readback and structural checks pass. A failure leaves an existing destination unchanged.

Every emitted geometry bound must remain inside both the resolved project domain and the resolved mesh. Geometry that would be clipped by either boundary refuses instead of producing a different electrical model. Script clip ranges must increase on both axes. A missing required shape, via, or port coordinate refuses by shape id instead of placing content at the origin.

## Report contract

Every completed export has three sections:

Category || Meaning
Exported || Setup and geometry represented directly in the XML.
Approximated || Content represented with the stated flattening, segmentation, or substitution.
Omitted || Project content the XML does not carry or a supported export deliberately leaves out.

The target runtime realizes the geometry and materials itself, so its results are not bit-identical to a Machshev Labs run.

Parametric definitions, editor view state, stored results, Run History, and reference traces are not embedded. The current resolved geometry is embedded. Unsupported or degenerate electrical geometry refuses the whole export instead of producing a partial case. A failed export leaves an existing destination unchanged. Dielectric loss tangent is written as conductivity at the layer's own loss reference frequency and listed as approximated. Enabled NF2FF, surface-current, full-field, field-transform, and early-stop requests are named as omitted when the XML does not include them.

Machshev Labs re-import preserves the supported planar geometry, stackup, materials, units, and frequency band. A via returns as a via over the same span with its drill and pad at the drill diameter, and its pads return as polygons on the conductor layers. Re-import inventories ports rather than creating Machshev Labs ports, lists holes, cutouts and their plating as not imported, and rebuilds boundary faces, mesh lines, domain spacing, and automatic run controls through the import workflow.

## Limits

- Resolved maximum steps must be from 1 through 4294967295, the range carried by the target runtime.
- The exporter supports a positive Gaussian band. Continuous-wave run-to-cap stop semantics refuse because a zero ringdown criterion would not retain the same runtime behavior.
- A missing frequency band, invalid mesh, incomplete boundary definition, bad geometry value, unsupported primitive, fuzzy load-time layer remap, or failed project resolve refuses the export.
- Stored raster geometry is written at the image's own resolution, one rectangle per pixel run, and the report counts the rectangles.
- A hole or cutout a Machshev Labs run refuses also refuses the export, by shape: a parameter its kind does not have, a span the stack does not have, a plated opening with no board material around it to plate, or a via that reaches into it.
- Review every approximated and omitted entry before using the XML.

The writer uses the existing project, mesh, and geometry resolution code. It does not require an openEMS or CSXCAD installation. Its built-in checks parse the XML and re-read the supported structure, but they are not a target-runtime load test. Runtime acceptance requires testing the file with the target version.
