# Importing openEMS and CSXCAD XML
> The openEMS importer reads a complete openEMS XML setup or a geometry-only CSXCAD `ContinuousStructure`. It is available in the editor's Import dropdown and under File, Import.

After parsing, a dialog asks for the destination. Create new project follows the normal save prompt and replaces the open project. Merge into current project keeps the open project and offers Import frequency band when the file declares a supported band. Neither destination is preselected. Every approximation and every item that does not transfer is listed in the import report and the Console.

Ports, boundary conditions, and domain spacing are not imported. The source mesh lines are kept as an explicit grid that follows the reviewed stack, and Machshev Labs sets boundaries, air, and PML to Auto mode. Add and check ports before running.

## Reviewing layers and materials

The shared assignment dialog groups conductor properties on each CSXCAD Z plane. Each plane can be included, excluded, assigned a role, and placed in the Machshev Labs copper order. Vias, drawn as vertical metal cylinders or round extruded polygons, appear as one plated-drill row. The row offers only plated, because the vias are metal in the source, and a CLI spec that sets it non-plated is refused. Dielectric gaps show the source spacing and scalar relative permittivity.

A source dielectric region that only partly covers a conductor gap is refused. Two lateral material regions in the same gap must have the same permittivity and loss tangent. This prevents the single reviewed gap value from hiding a Z subdivision or unlike electrical materials. Free space is the exception, see Holes and free space below. Free space over part of a gap reads as empty space there, and an empty gap takes its reviewed material over the imported geometry bounds.

The reviewed assignment is authoritative. Excluded conductor properties and vias are listed as skipped. A reassigned outline row sets the initial XY extent. Every material substitution or combined region is listed in approximated.

## Holes and free space

A `Material` with relative permittivity 1 and no conductivity is free space. Where it outranks the substrate it removes it, so it imports as an unplated opening. A vertical `Cylinder` becomes a round hole, and a `Box` or `LinPoly` with height becomes a cutout with the same outline. The import report counts the free-space regions per material and names the layers their openings run between. Openings are not rows in the review. They import with the board and can be edited or deleted afterward like any hole or cutout.

Priority decides what an opening cuts, by the CSXCAD rule: the highest priority wins, and a primitive's face counts as inside it. On each conductor plane the free space reaches, the copper is cut where the free space outranks it and kept where the copper outranks it, so a mounting hole drawn as air above the copper priority clears the copper inside its outline. Copper that lies under a higher shape inside the outline, such as a trace under a pad, is covered there and not cut. Copper drawn with thickness is cut only where the free space spans its whole thickness, and free space that ends on its face, or inside copper hanging from a plane, leaves it in place. CSXCAD does not define which of two equal priorities wins, so at a tie the substrate or copper is kept and the report says so. Free space drawn as stacked pieces of one outline at one priority counts as one region, even when the pieces come from different materials.

What free space removes is decided in source coordinates with the reviewed copper rows, so excluding a copper row changes it the way the same edit to the source would, and every message quotes the Z values in the file. Excluding the vias leaves their bores and fills out with them. A reordered stack keeps an opening as long as the copper rows it runs through stay next to each other, in either direction. Material past the outer kept planes is listed as skipped, and free space that outranks only that material removes nothing.

Free space that fills every gap it touches from plane to plane, as the only material there, and removes nothing imports as an ordinary dielectric layer with permittivity 1, such as an air spacer under a patch. Free space that the board material outranks everywhere, such as a whole-domain air box at a priority below the substrate, removes nothing and is listed as skipped, with the reason. So is free space beside the substrate that overlaps no board material, and free space that only touches the face of thick copper.

Free space that outranks a whole via over its height removes the via. Free space inside a via over the via's height is its bore and is not imported, and a dielectric fill inside a via is not imported either. A part drawn past the via into the open space above or below the board does not change that. A fill that runs past its via into the board is a dielectric insert beyond it, which the one-material gap rule refuses.

A Machshev Labs opening runs through whole layers and cuts its whole outline. Free space that cannot be expressed that way refuses the file, with the primitive named:

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
  "schema": "machshev-board-import/v1",
  "format": "openems",
  "sources": ["setup.xml"],
  "destination": {"mode": "new"},
  "role_source": "detected",
  "stackup": {"source": "detected"}
}
```

```
machshev project import-openems --spec setup.import.json --out setup.rfsim
```

For a merge, `destination.project` names the saved input and `destination.bring_frequency_settings` must be `true` or `false`. `--out` always names the result. The generic `machshev project import` command accepts the same spec. A `true` frequency choice refuses the import when the XML has no supported positive Gaussian band.

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
- Holes, cutouts, and plating that a Machshev Labs export wrote as `Machshev Labs openings` and `Machshev Labs opening walls` are counted and listed as not imported.
- XML with a DTD or entity declaration is refused in every supported text encoding.
- One file is limited to 64 MiB, 1,000,000 XML elements, and 250,000 CSXCAD primitives.
- A grid is limited to 500,000 lines per axis and 1,000,000 lines in total.

The importer uses the Python standard library. It does not invoke or link openEMS or CSXCAD code.
