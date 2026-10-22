# Advanced mode
> Advanced mode hands you the mesh, air margins, PML depth, run length, and convergence criteria that [Auto mode](docs-auto-mode.html) derives for you. Boundary face types live on [Boundaries](docs-boundaries.html). This page covers everything else the Advanced toggle exposes.

## Switching to Advanced

Select Advanced on the Simulate tab to set `dx_mm`, `dy_mm`, `dz_mm`, air margins, PML depth, and run length manually. The run uses these values without snapping or re-deriving them.

Advanced values persist in the project independently of the auto values. Every Advanced field has an `adv_*` shadow, so a project saved in Auto mode keeps its Advanced entries and reopening in Advanced mode restores them. Transfer to Advanced Mode, on the Derived Settings card in Auto mode, copies the current auto-derived mesh, margins, PML, and boundary preset into the Advanced fields and turns on Max steps at the auto step cap. You edit from there.

## Manual mesh

The Mesh / Cell Size card sets the three base cell sizes.

`dx_mm`, `dy_mm`, and `dz_mm` are the Yee cell dimensions in millimeters, default `0.2` each. The run uses them exactly. [FDTD tips](docs-fdtd-tips.html) covers picking cell sizes.

If `dz` does not evenly divide the conductor stackup height, the substrate lands between grid lines. `validate` warns (`dz does not divide the substrate height`) and reports the closest fitting dz without changing your value. See [FDTD tips](docs-fdtd-tips.html) for choosing dz.

## Mesh grading

Grading keeps fine cells in the board region and lets them grow toward the air and PML, cutting total cells while holding resolution where the fields are. It is what the Ratio grading strategy does, and it is the default. The controls:

- **Refine margin** (`mesh_grading_margin_mm`, default `1.0`): how far the fine-cell region extends past the board in every direction.
- **Per-axis multiplier** (`mesh_grading_{x,y,z}_mult`, default `1.0`): the board-region refinement factor, `1.0` meaning no refinement.
- **Per-axis max ratio** (`mesh_grading_{x,y,z}_max_ratio`, default `1.3`): the largest cell-to-cell size step allowed in that axis' transition. Must exceed `1.0`, or no transition can satisfy it.

## Air margins

The air gap between the geometry and the boundary has three modes, selected by the Air Margins mode combo.

- **Manual (cell count)** (`air_cells`, default `6`): the same number of air cells on every face. Keyed to the mesh, so the physical gap follows the cell size.
- **Manual (mm)** (`air_around_mm`, `air_above_mm`, `air_below_mm`, default `10.0`): each face set in mm directly.
- **Manual (frequency)** (`auto_air_sides_factor`, `auto_air_above_factor`, `auto_air_below_factor`, default `0.25`): each face sized as a fraction of the wavelength at `f_min`.

PML depth has its own three modes (`pml_cells_manual` default `8`, `pml_thickness_mm` default `4.0`, `auto_pml_factor` default `0.10`), set on the Boundary Conditions card and covered under [Boundaries](docs-boundaries.html).

## Run length

The Energy Ringdown card sets when the run stops.

The ring-down criterion is the remaining field energy relative to peak at which the solver stops. It is edited in dB and stored as a ratio (`ringdown_end_crit`), default `-50 dB` (ratio 1e-5). More negative is a longer, more converged run. The floor is `-120 dB`, since the control is logarithmic and zero has no dB value. The criterion is checked every 250 steps by default. Every run keeps a built-in floor of two excitation lengths before the criterion may stop it: one length is the source injection itself, and the one that follows lets the source transient leave the port before the energy is judged. The Minimum steps field shows that floor greyed, worked out from the band and the timestep, while its checkbox is off. Tick Minimum steps to raise the floor when the model needs a longer observation window. Then `ringdown_min_steps` carries your value, and a value below the built-in floor has no effect.

Max steps adds a hard timestep cap (`adv_max_steps`). Leave it off and the run has no cap and stops on ring-down alone. If the cap is hit before ring-down, the time record is truncated before the fields settle, so the S-parameter and radiation results are not converged. See [FDTD tips](docs-fdtd-tips.html).

S-param early stop (`sparam_early_stop`, off by default) stops the run once the S-parameters stop changing by more than `sparam_delta_crit_db` (default `-40 dB`) between checks. The threshold must be negative and does not enable or disable the criterion. `sparam_early_stop` is its sole enable. `sparam_min_steps` (default `10000`) is a floor so it cannot fire during the source excitation transient.

## Mesh convergence

The Mesh Convergence Study checkbox on the Mesh / Cell Size card, which writes `auto_mesh_enabled`, runs a multi-pass study that refines the cell size until the tracked resonances stabilize. It starts from the card's cell sizes and refines only the mesh each pass. The study settings are on the same card, and the Mesh Convergence plot beside the runtime stats tracks each pass.

- **Source** (`conv_source`, default `S11`) and **resonance count** (`conv_num_resonances`, default `1`): which S-parameter and how many of its deepest dips to track, matched between passes by rank.
- **Freq shift** (`conv_freq_threshold_db`, default `-20 dB`) and **depth diff** (`conv_depth_threshold_db`, default `-20 dB`): the per-resonance thresholds a pass must meet to count as converged. `-20 dB` is about 1 percent.
- **Mesh step** (`conv_mesh_refine_factor`, default `1.26`): the division factor each pass. 1.26 is about cbrt(2), roughly doubling total cells.
- **Max passes** (`conv_max_passes`, default `8`): the hard iteration limit.

## CLI equivalent

Every field above is a project setting reachable headless: override one with `--setting <name>=<value>` or carry the whole set in a JobSpec. See [Scripting and specs](docs-cli-scripting.html). The full field list with units and bounds is on [Settings reference](docs-settings.html).
