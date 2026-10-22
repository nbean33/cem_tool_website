# FDTD tips
> Practical guidance on boundary spacing, frequency units, convergence, mesh resolution, and field export.

## Give every PEC and PMC face an explicit spacing

Set each hard wall and its gap on the run line:

```
machshev run --project board.rfsim --per-face-bc z_lo=PEC:2.0
```

In auto mode the air margin between the geometry and each boundary is sized for a PML absorber. A PEC or PMC face is a hard wall with no absorber, and if you leave it on that inherited margin the geometry can end up touching or sitting inside the wall, which reflects energy back into the design and corrupts the S-parameters. Set a real clearance on each hard-wall face. In the GUI, set the face to Custom, check its spacing checkbox, and enter the gap in the spacing field.

The `:MM` is a fixed gap from the geometry to that face, realized exactly and overriding the air margin there, and any per-face setting forces `bc_all` to Custom. A gap of `0` is legal and puts the wall flush, which is what you want for an infinite ground plane under the board. A Custom-mode case that reaches export with a hard-wall face and no spacing is refused with an error naming the face, so a missed gap stops the run rather than silently corrupting it. The PEC and PMC box presets keep the derived margins instead. See [Boundaries](docs-boundaries.html).

## Watch the frequency units: Hz, GHz, and MHz

Frequency units differ between the project file, GUI, and CLI. The project and spec store `freq_min_hz` and `freq_max_hz` in Hz. The GUI frequency fields show and accept GHz. The `run` and `reprocess` flags (`--freq-min`, `--freq-max`, `--fmin`, `--fmax`) take MHz. The same band is `5.8e9` in a spec, `5.8` in the GUI, and `5800` on a run flag. Get the exponent wrong and the whole band moves, along with the mesh resolution and the PML sizing that both key off it.

## A run is only valid if it rang down

Ring-down is the convergence test. The excitation is a pulse, and the S-parameters and radiation pattern are only trustworthy once the field energy in the box has decayed to `ringdown_end_crit` (an energy ratio relative to peak, smaller means a longer and more converged run). The exported runtime `max_steps` is a hard cap on backend timesteps. Auto mode derives and stores it as `auto_max_steps`. Advanced mode uses `adv_max_steps` only when `adv_max_steps_enabled` is true. The `--max-steps` option applies a transient cap to either mode without changing either stored setting. If the effective cap stops the run before ring-down, the results are not physically converged. The title bar reads STEP CAP, the badge on the result tabs warns that the run stopped before its ring-down target, and the manifest (`ringdown_reached`) records that the run was cut short. If a run keeps hitting the cap, either raise the cap for the active mode or loosen `ringdown_end_crit` toward a larger ratio, and if you want a tighter result make the criterion smaller (auto mode derives it from quality, advanced mode uses the value as written). The criterion is evaluated every `ringdown_check_steps` timesteps (default 250). Each evaluation costs a full-domain energy pass and, on the CPU backend, bounds the solver's internal blocking. The 250-step default keeps that cost small on both backends while bounding stop overshoot to the same 250 steps. The optional S-parameter early stop (`sparam_early_stop`) ends the run once |dS11| between consecutive ring-down checks holds below its threshold. It saves steps on a clean resonator but can stop a slowly settling structure a little early, so leave it off for a full-length reference run.

## CW mode does not ring down

Continuous-wave excitation (`cw_enabled`) injects a steady sinusoid instead of a pulse. A steady drive does not ring down. The run ends at the step cap you set, or at the step counter's limit if no cap is set. S-parameter early stopping is disabled. Under a single tone the S-parameters are of limited meaning, the radiation pattern at the tone is meaningful, and both outputs stay available. Field animations for the [field viewer](docs-field-viewer.html) are the usual reason to use it. It is an advanced-mode feature, and an auto-mode project with CW on is refused rather than run as a pulse.

## Keep the mesh cells comparable across axes

Setting one per-axis minimum feature and leaving the others at zero, or hand-setting `dx` and `dy` far apart in advanced mode, resolves X and Y differently and can make the S-parameters depend on feed orientation, most visibly for planar ports. `machshev validate` and `machshev estimate` warn when `dx` and `dy` differ by more than 3x, and when `dz` is more than 3x the finer in-plane cell, because Z carries the stackup. A `dz` finer than the in-plane cells is the normal shape of a layered stackup and draws no warning. Keep `dx` and `dy` within a factor of three of each other unless the anisotropy is deliberate. A minimum feature set on only some axes draws its own warning for the same reason.

## Paint every dielectric layer or make it air

A dielectric layer with nothing painted on it is treated as air and its epsilon is ignored. `validate` and `run` warn on this, since running the layer as air is occasionally intended. If the layer is a real substrate, paint a board-sized rectangle across it so it carries its permittivity. If the gap really is meant to be air, set the layer's type to air so the intent is explicit rather than inferred from an empty dielectric.

## In advanced mode, let dz divide the substrate

Auto mode snaps `dz` so the stackup planes land on grid lines. Advanced mode uses `dz` exactly, and if it does not divide the substrate height the conductor stackup can land between grid lines and shift the resonance. `validate` warns about a non-dividing `dz`. Either pick a `dz` that divides every dielectric thickness, or use [auto mode](docs-auto-mode.html) and let it snap for you. When you set the cell sizes by hand, resolve the finest feature you care about and put several cells through the thinnest substrate, and keep `dy` near `dx` unless one direction really is finer, which the anisotropy tip above covers.

## Resolve the drills your result depends on

A via drill or plated hole under 2 cells across still conducts, as a coarse barrel of mesh nodes or a single conductor post, and a small unplated hole opens a few cells. The Opening Findings block in Derived Settings, `machshev validate`, and `machshev estimate` list each group of identical drills the mesh realizes differently from the drawing and quote what resolves them: the min feature X and Y with the cells and VRAM that costs, or in advanced mode the cell size. Spend those cells where current through the drill sets the result, such as a shorting via at the end of a resonator or a feed via. See [Auto mode](docs-auto-mode.html).

## Full-field export is heavy, so window it

`save_vtk` writes a 3D field frame per captured step across the whole volume. `field_format` defaults to `rff`, compressed for visualization without threshold pruning, and `field_format=vtk` stores fp32 samples. In advanced mode, narrow the capture to a step window with `field_export_range_enabled`, `field_export_range_start`, and `field_export_range_end`. With `field_format=vdb`, `vdb_threshold_frac` prunes the weak wave tail, so lowering it retains more of the field and enlarges the files. When you only need S-parameters, the run-line kill switches force capture off regardless of the saved settings: `--no-raw-fields`, `--no-nf2ff`, and `--no-surface-currents`.

## Validate before you wait for a full run

A full run can take a while, so catch the cheap mistakes first. In the GUI, the mesh preview on the Simulate tab draws the resolved grid over the geometry before any solve, so a feature that falls between cells or a boundary sitting on the structure shows up at a glance, and the Notices panel lists the same blocking errors, advisory warnings, and notes a run would raise. Headless, three commands do the same without a backend: `machshev validate --project FILE` runs the blocking pre-flight gate plus the advisory checks above, `machshev estimate --project FILE` derives the mesh and prints the cell count, VRAM, boundary sizing, and ring-down so you see the size before you commit to it, and `machshev run --project FILE --dry-run` applies your overrides, validates, exports the case, and prints the estimate, then stops before launching the solver. See the [CLI reference](docs-cli-reference.html).

## Quality slider versus a convergence study

These solve two different problems. The quality slider (0.0 to 3.0) sets the resolution targets in one move, interpolating cells per wavelength, PML depth, and ring-down criterion between the presets, and raising it refines the whole mesh uniformly. A convergence study (`auto_convergence_enabled`) instead repeats the run over successive passes, refining the mesh each pass until the tracked resonances stop moving between passes, then reports whether it converged or hit the pass cap. Use a study when a result sits near a sharp resonance and you need evidence that it has stopped shifting with mesh density, which a single higher-quality run does not give you. It costs one backend pass per refinement, so spend the passes on a validation or a final number, not on routine iteration where a `medium` or `high` quality run is enough.
