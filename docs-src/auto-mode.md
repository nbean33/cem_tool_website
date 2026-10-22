# Auto mode and quality
> Auto mode derives the mesh, CFS-CPML absorber, air margins, ring-down criterion, and output-capture bins from the frequency band, quality level, stackup, and copper geometry. The derived-settings panel shows the rule used for each value.

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
machshev estimate --project patch.rfsim --quality medium
```

`estimate` derives the same mesh and prints the cell count, memory, boundary sizing, ring-down, runtime cost, the measured per-axis suggestion, and the same advisory warnings and notes, without launching the solver. `--quality` accepts a name or a number in 0 to 3.

## Convergence study

The Perform Convergence Study checkbox, which writes `auto_convergence_enabled`, runs the solver several times with an increasingly fine mesh until the tracked resonance dips stabilize, instead of a single pass. The frequency-shift and depth thresholds follow the quality level, and each pass refines the mesh by a fixed factor up to a pass cap. This costs several full runs. The threshold, refine-factor, and pass-cap fields are in the [settings reference](docs-settings.html).
