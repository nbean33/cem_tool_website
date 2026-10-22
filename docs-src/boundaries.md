# Boundary conditions
> The simulation sits in a box of six faces, and each face carries one of three boundary conditions. This page covers the types, the presets, the per-face custom matrix, the spacing between the geometry and each face, and the PML depth controls.

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
machshev run --project p.rfsim --per-face-bc z_lo=PEC:0 --per-face-bc x_lo=PMC:4
```

## Spacing between geometry and each face

On the CLI, set a hard wall and its fixed gap together on the run line:

```
machshev run --project p.rfsim --per-face-bc z_lo=PEC:2.0
```

The PEC and PMC box presets keep the derived margins on every face. Every face has an air gap between the bounding box of the geometry and the wall. Two regimes exist per face:

- **Derived:** with the spacing checkbox unchecked, the face inherits the air margins (auto mode sizes them by cell count with a substrate-height floor). The field shows the derived gap grayed out as a placeholder, never as an editable value. This sizing exists to give a PML face room to absorb.
- **Fixed:** with the checkbox set, the gap realizes exactly as entered, in mm, overriding the margins on that face. A fixed gap of 0 puts the wall flush against the geometry, which turns a PEC bottom face into an infinite ground plane.

A PEC or PMC face in the Custom matrix requires an explicit spacing. A hard wall that inherits the PML-sized margin can end up against, or inside, the geometry, reflecting energy back into the design and corrupting the S-parameters. The GUI does not let that happen: for a PEC or PMC face the spacing checkbox locks on and the field stays marked required until you enter a value. A Custom-mode case that reaches export with a hard-wall face and no spacing is refused with an error naming the face, so the run never starts on an unspecified gap. The PEC box and PMC box presets are the exception: they keep the derived margins on every face.

## PML depth

`pml_thickness_mm` is the authoritative absorber depth. The panel offers three ways to set it: a manual cell count (default 8 cells), a manual thickness in mm, or an automatic fraction of the wavelength at f_min (default 0.10 λ). A resolved cell count of 0 derives the count from the thickness at export.

The PML is added outside the air margin. It extends the domain rather than consuming the gap, so it never overlaps the geometry. Deeper PML only increases total cell count. Air margins themselves are covered under [Advanced mode](docs-advanced-mode.html), and every boundary and spacing field is listed in the [settings reference](docs-settings.html).
