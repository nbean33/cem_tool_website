# Stackup and materials
> The stackup is the ordered set of layers that gives your board its vertical structure and its material properties. This page covers the layer model, the dielectric and conductor properties, and where to edit them. Drawing copper and placing ports is covered in [Drawing geometry](docs-geometry.html).

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
