# Settings reference
> All simulation settings, grouped by category. Settings are saved in the project's FDTD settings block.

## Reading and changing settings

The same values reach the solver from four surfaces:

- The Simulate tab panels edit the run settings in the GUI. Advanced-mode settings appear when Advanced mode is on. Viewer settings such as Photorealistic, Show meshing, and Max FPS are set in the viewer that uses them and do not change the run.
- `machshev project dump-settings --project FILE` prints every setting with its value, default, units, and description, grouped by category. `--changed-only` shows only settings that differ from their default, `--category NAME` limits to one category, `--advanced` or `--no-advanced` limits to advanced-only or non-advanced settings, and `--json` emits a machine-readable list.
- `machshev run --setting name=value` overrides a setting for one run (repeatable). It is value-gated by the same gate as the GUI. A hard bound out of range refuses with exit 2, a soft bound out of range prints one warning and runs the value exactly as set. An unknown setting refuses with exit 2, and a renamed one names its replacement.
- `machshev schema --what settings` prints the machine contract: every field with its type, default, units, enum, and category.

## Reading the table

Type and Default are the live values from the registry. A **ui-only** setting persists viewer or editor state and does not change the physics. An **advanced** setting surfaces in Advanced mode. A **Caution** note is the registry's warning about a common mistake for that setting.

## Frequency band

Setting || Type || Default || Units || Notes || Description
`freq_min_hz` || number || 0 || Hz || || Lower edge of the excitation band, in Hz. Also sets the reference wavelength for PML and air sizing. **Caution:** This field is in Hz. The CLI run flags and the GUI use MHz/GHz, so a value of 3e9 is 3 GHz.
`freq_max_hz` || number || 0 || Hz || || Upper edge of the excitation band, in Hz. Sets the required mesh resolution (cells per wavelength at f_max). **Caution:** Must be greater than `freq_min_hz`.
`cw_enabled` || boolean || false || bool || || Inject a continuous sinusoid at `cw_freq_hz` instead of a Gaussian pulse. Advanced-mode only: export refuses an auto-mode project with CW enabled. **Caution:** Continuous wave never rings down, so the run stops only at whichever step cap is live: `auto_max_steps` in Auto mode, `adv_max_steps` in Advanced mode when `adv_max_steps_enabled` is true, otherwise at the step counter's limit. Under a single tone the S-parameters are of limited meaning, the radiation pattern at the tone is meaningful, and both outputs stay available.
`cw_freq_hz` || number || 2.4e+09 || Hz || || Continuous-wave excitation frequency, in Hz, used when `cw_enabled` is on.
`auto_freq_mode_centre` || boolean || false || bool || ui-only || UI input style for the auto-mode frequency band: f_min / f_max (False, the default) or centre plus bandwidth (True).

## Mesh / cell size

Setting || Type || Default || Units || Notes || Description
`dx_mm` || number || 0.2 || mm || || FDTD cell size in X. Smaller cells raise accuracy, memory, and runtime. **Caution:** In advanced mode this is used exactly. In auto mode it is derived from quality and overwritten.
`dy_mm` || number || 0.2 || mm || || FDTD cell size in Y.
`dz_mm` || number || 0.2 || mm || || FDTD cell size in Z. Must resolve the substrate thickness (aim for several cells per dielectric layer). **Caution:** In advanced mode dz is not snapped. If it does not divide the stackup height the substrate can land between grid lines.

## Non-uniform mesh grading

Setting || Type || Default || Units || Notes || Description
`mesh_strategy` || string || ratio || enum || enum: ratio / custom || Select the sole mesh-grading owner: board-region Ratio grading or the Custom profile edited in Mesh View.
`mesh_grading_x_mult` || number || 1 || factor || || Board-region refinement factor in X (1.0 means no refinement).
`mesh_grading_y_mult` || number || 1 || factor || || Board-region refinement factor in Y.
`mesh_grading_z_mult` || number || 1 || factor || || Board-region refinement factor in Z. **Caution:** A value of 2.0 halves the Z cell near the board, roughly doubling Z cells there and raising memory.
`mesh_grading_margin_mm` || number || 1 || mm || || How far the fine-cell region extends beyond the board in every direction.
`mesh_grading_x_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the X grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.
`mesh_grading_y_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the Y grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.
`mesh_grading_z_max_ratio` || number || 1.3 || ratio || || Maximum cell-to-cell size ratio in the Z grading transition. At or below 1 no transition can satisfy the ratio and the size jump would violate it.

## Boundary conditions and PML

Setting || Type || Default || Units || Notes || Description
`bc_all` || string || PML || || enum: PML / PEC / PMC / Custom || Boundary preset applied to all six faces, or Custom to set each face individually. **Caution:** PEC and PMC are hard walls with no absorber. Give every PEC/PMC face an explicit per-face spacing, otherwise it inherits the air margin sized for PML and the geometry can sit on the wall.
`pml_thickness_mm` || number || 4 || mm || || Physical thickness of the PML absorber, the authoritative depth when `pml_mode` is manual. In the cells and auto modes the export derives the depth (`pml_cells_manual` x finest cell, or `auto_pml_factor` x wavelength) on every path and this field is display state. The PML is added outside the air margin (it extends the domain), so it never overlaps the geometry. Thickness only increases total cell count.
`pml_cells` || integer || 0 || cells || || Resolved PML cell count. 0 means derive it from `pml_thickness_mm` at export. A positive value is a per-face floor, and the axis count is raised to ceil(`pml_thickness_mm` / cell) when the thickness needs more. In advanced mode with a manual thickness the thickness alone is authoritative and this count is not read.
`auto_pml_factor` || number || 0.1 || fraction || || PML thickness as a fraction of the wavelength at f_min when auto PML is on (0.10 means lambda/10). The depth is realized as written. A thin absorber is warned about, never enlarged.
`pml_mode` || string || cells || || enum: cells / auto / manual || How PML depth is specified: a cell count (`pml_cells_manual`), auto-sized from the wavelength at f_min, or the explicit `pml_thickness_mm`.
`pml_cells_manual` || integer || 8 || cells || || PML absorber depth in cells per face when `pml_mode` is cells. The export pins every face to exactly this count on every path. A face declared PML needs at least one absorber cell to be one.
`bc_x_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -X face when `bc_all` is Custom.
`bc_x_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +X face when `bc_all` is Custom.
`bc_y_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -Y face when `bc_all` is Custom.
`bc_y_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +Y face when `bc_all` is Custom.
`bc_z_lo` || string || PML || || enum: PML / PEC / PMC || Boundary type on the -Z (bottom) face when `bc_all` is Custom.
`bc_z_hi` || string || PML || || enum: PML / PEC / PMC || Boundary type on the +Z (top) face when `bc_all` is Custom.
`bc_x_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -X gap (`bc_x_lo_spacing_mm`) instead of the auto/global air margin.
`bc_x_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +X gap (`bc_x_hi_spacing_mm`) instead of the auto/global air margin.
`bc_y_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -Y gap (`bc_y_lo_spacing_mm`) instead of the auto/global air margin.
`bc_y_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +Y gap (`bc_y_hi_spacing_mm`) instead of the auto/global air margin.
`bc_z_lo_spacing_enabled` || boolean || false || bool || || Use a fixed -Z gap (`bc_z_lo_spacing_mm`) instead of the auto/global air margin.
`bc_z_hi_spacing_enabled` || boolean || false || bool || || Use a fixed +Z gap (`bc_z_hi_spacing_mm`) instead of the auto/global air margin.
`bc_x_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -X boundary when its spacing override is enabled.
`bc_x_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +X boundary when its spacing override is enabled.
`bc_y_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -Y boundary when its spacing override is enabled.
`bc_y_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +Y boundary when its spacing override is enabled.
`bc_z_lo_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the -Z boundary when its spacing override is enabled. A Custom PEC/PMC face requires it (0 puts the wall flush, e.g. an infinite ground plane). PML faces fall back to the derived margin.
`bc_z_hi_spacing_mm` || number || 0 || mm || || Fixed gap from the geometry to the +Z boundary when its spacing override is enabled. A Custom PEC/PMC face requires it. PML faces fall back to the derived margin.

## Air margins / domain

Setting || Type || Default || Units || Notes || Description
`air_around_mm` || number || 10 || mm || || Air gap in the XY plane from the meshed board edge to the side boundaries. A per-face spacing overrides it on that face.
`air_above_mm` || number || 10 || mm || || Air gap above the topmost stackup plane to the upper boundary. A per-face top spacing overrides it.
`air_below_mm` || number || 10 || mm || || Air gap below the bottom stackup plane to the lower boundary. A per-face bottom spacing overrides it.
`air_mode` || string || cells || || enum: cells / mm / auto || How the advanced air margin is specified: a per-face cell count, the explicit `air_*_mm` values, or auto-sized from the wavelength at f_min. **Caution:** Every mode is resolved at export on every path: cells from `air_cells` x the face's own cell size, auto from `auto_air_*_factor` x the wavelength at f_min. In those two modes the `air_*_mm` fields are display state. Only `air_mode=mm` reads them.
`air_cells` || integer || 6 || cells || || Air cells on every face when `air_mode` is cells. The export resolves the count against the face's own cell size (the coarser in-plane cell laterally, dz above and below).
`auto_air_enabled` || boolean || false || bool || || Size the air margins automatically from the wavelength at f_min instead of using the fixed `air_*_mm` values.
`auto_air_sides_factor` || number || 0.25 || fraction || || Side air margin as a fraction of the wavelength at f_min when auto air is on.
`auto_air_above_factor` || number || 0.25 || fraction || || Above air margin as a fraction of the wavelength at f_min when auto air is on.
`auto_air_below_factor` || number || 0.25 || fraction || || Below air margin as a fraction of the wavelength at f_min when auto air is on.

## Run length and telemetry

Setting || Type || Default || Units || Notes || Description
`typical_speed_gcells_per_s` || number || 10 || GCell/s || ui-only || Saved throughput estimate used only for the UI runtime prediction.
`auto_max_steps` || integer || 1000000 || steps || || Auto-mode safety cap on backend timesteps. In Advanced mode, `adv_max_steps` applies only when `adv_max_steps_enabled` is true. The max-steps CLI option forces a one-run cap in either mode. **Caution:** If the cap is hit before ringdown, the S-parameter and radiation results are not physically converged.
`ringdown_end_crit` || number || 1e-05 || ratio || || Energy ratio (relative to peak) at which the run stops. Smaller means a longer, more converged run. Auto mode derives it from quality. Advanced mode honors the value as written. The floor is -120 dB: the control is logarithmic and zero has no dB representation.
`ringdown_check_steps` || integer || 250 || steps || || How often, in steps, the ringdown energy criterion is evaluated. Also the step-telemetry window and the engines' temporal-blocking round length, so it is consumed on every run including CW.
`ringdown_min_steps` || integer || 0 || steps || || Optional user floor before the ringdown criterion may stop the run, above the built-in floor of two excitation lengths that every run keeps. 0 (the default) adds nothing above that floor.
`sparam_delta_crit_db` || number || -40 || dB || || Threshold on the change in S-parameters between ringdown checks. It must be negative when `sparam_early_stop` is on. The threshold itself never enables or disables the stop.
`sparam_delta_check_steps` || integer || 0 || steps || advanced || How often, in steps, the S-parameter early-stop delta is evaluated. 0 reuses `ringdown_check_steps`.
`sparam_delta_consecutive` || integer || 2 || count || advanced || Consecutive stable S-parameter checks required before the early stop fires.
`sparam_min_steps` || integer || 10000 || steps || || Minimum steps before the S-parameter early stop is allowed to fire (a floor so it cannot stop during source excitation).
`sparam_early_stop` || boolean || false || bool || advanced || Enable the early stop once the S-parameters stop changing by `sparam_delta_crit_db` across consecutive checks. This boolean is the sole enable for the criterion.
`telemetry_interval` || integer || 25 || steps || ui-only || Emit step telemetry every N steps. Lower values update the live charts more often at some compute cost.
`adv_max_steps` || integer || 1000000 || steps || advanced || Hard step cap applied when 'Max steps' is enabled. Honored on the GUI and CLI run paths. **Caution:** If the cap is hit before ringdown, the S-parameter and radiation results are not physically converged.
`adv_max_steps_enabled` || boolean || false || || advanced || When set, the run stops at `adv_max_steps`. Honored on the GUI and CLI run paths.

## Mesh convergence study

Setting || Type || Default || Units || Notes || Description
`auto_mesh_enabled` || boolean || false || bool || advanced || Advanced-mode toggle for the mesh convergence study.
`auto_convergence_enabled` || boolean || false || bool || || Run a multi-pass mesh convergence study, refining until the tracked resonances stabilise.
`conv_source` || string || S11 || || enum: S11 / S21, advanced || Which S-parameter the convergence study tracks.
`conv_num_resonances` || integer || 1 || count || advanced || Number of deepest resonance dips matched between convergence passes.
`conv_freq_threshold_db` || number || -20 || dB || advanced || Allowed per-resonance frequency shift between passes, as 10*log10(df/f) (-20 dB is about 1 percent).
`conv_depth_threshold_db` || number || -20 || dB || advanced || Allowed per-resonance depth change between passes, in dB of the linear-magnitude difference.
`conv_mesh_refine_factor` || number || 1.26 || factor || advanced || Mesh division factor applied each convergence pass. **Caution:** 1.26 is about cbrt(2), roughly doubling cell count per pass. Larger values explode the cell count. Below 1 the step would coarsen instead of refine.
`conv_max_passes` || integer || 8 || count || advanced || Maximum number of convergence passes before stopping.

## Radiation (NF2FF)

Setting || Type || Default || Units || Notes || Description
`enable_nf2ff` || boolean || true || bool || || Compute the 3D radiation pattern, gain, and directivity via a near-field to far-field transform.
`skip_geometry_export` || boolean || false || bool || || Skip writing geometry.vtk / geometry.vdb when a radiation or 3D-field run would normally request it. A suppression: with no geometry output requested it changes nothing and is still honored.
`nf2ff_num_freqs` || integer || 51 || count || || Number of frequency points for the radiation-pattern output. The solver's NF2FF buffers hold 512 points, a fixed backend contract.

## Surface currents

Setting || Type || Default || Units || Notes || Description
`enable_surface_currents` || boolean || false || bool || || Record surface current density on a conductor layer (time-domain frames plus a frequency-domain DFT).
`surface_current_layer` || string || || || || Conductor layer name to capture surface currents on.
`surface_current_interval_auto` || boolean || true || bool || || Pick the time-domain capture stride automatically for 10 samples per period at f_max. **Caution:** When on, this overrides the manual `surface_current_interval`.
`surface_current_interval` || integer || 50 || steps || || Save a time-domain surface-current frame every N steps when the auto stride is off.
`surface_current_num_freqs` || integer || 63 || count || || Number of frequency points for the surface-current DFT. The solver's DFT buffers hold 512 points, a fixed backend contract.
`surface_current_view_domain` || string || frequency || || enum: frequency / time, ui-only || Saved viewer preference for surface currents: frequency-domain or time-domain.

## 3D field export

Setting || Type || Default || Units || Notes || Description
`save_vtk` || boolean || false || bool || || Write full 3D field frames during the run. **Caution:** Produces very large files and is disk-bound. Enable only when you need full-volume fields.
`export_e_mag` || boolean || true || bool || || Write the E-field magnitude |E| (V/m) for each exported frame. The default field-export content. **Caution:** Turn it off only to export H or energy alone. With all three field types off there is nothing to write.
`export_h_mag` || boolean || false || bool || || Also write the H-field magnitude |H| (A/m) for each exported frame. H is time-centered to the E level before the magnitude.
`export_energy` || boolean || false || bool || || Also write the total field energy density U (J/m^3) for each exported frame, combining the aligned E and H at each cell.
`export_ex` || boolean || false || bool || || Also write the signed Ex component (V/m) for each exported frame.
`export_ey` || boolean || false || bool || || Also write the signed Ey component (V/m) for each exported frame.
`export_ez` || boolean || false || bool || || Also write the signed Ez component (V/m) for each exported frame.
`export_hx` || boolean || false || bool || || Also write the signed Hx component (A/m) for each exported frame, time-centered to the E level.
`export_hy` || boolean || false || bool || || Also write the signed Hy component (A/m) for each exported frame, time-centered to the E level.
`export_hz` || boolean || false || bool || || Also write the signed Hz component (A/m) for each exported frame, time-centered to the E level.
`vtk_dump_interval` || integer || 50 || steps || || Write a field frame every N steps when field export is on and the auto stride is off.
`vtk_dump_interval_auto` || boolean || false || bool || || Floor the field-export stride to the Nyquist-safe value derived from f_max. **Caution:** When on, this overrides the manual `vtk_dump_interval`.
`field_export_range_enabled` || boolean || false || bool || || Limit field frames to a step window. The solver still runs every step but skips writes outside it. **Caution:** Off by default, so frames are written at the export stride for the entire run when `save_vtk` is on.
`field_export_range_start` || integer || 0 || steps || || First step (inclusive) written when the field-export range is enabled.
`field_export_range_end` || integer || 10000 || steps || || Last step (inclusive) written when the field-export range is enabled.
`field_format` || string || rff || || enum: rff / vtk / vdb || 3D field storage format: RFF compresses signed logarithmic samples without threshold pruning, VTK stores fp32 samples, and VDB stores sparse volumes.
`vdb_threshold_frac` || number || 0.001 || fraction || || VDB sparsification cutoff as a fraction of each selected array's peak magnitude. Voxels below it collapse to the pruned background (0 disables). **Caution:** Very small values keep more of the wave tail and enlarge files.

## Backend selection

Setting || Type || Default || Units || Notes || Description
`use_cpu_backend` || boolean || none || bool || || Run on the CPU (OpenMP) backend. None defers to the saved app preference. The GUI auto-enables this when no CUDA GPU is found.

## Mode and quality

Setting || Type || Default || Units || Notes || Description
`sim_mode` || string || auto || || enum: auto / advanced || Simulation mode: auto derives the mesh from quality, advanced uses the explicit mesh and boundary fields.
`auto_quality` || number || 1 || level || || Continuous auto-mode quality from 0.0 to 3.0, interpolating cells per wavelength, PML, and convergence criteria.
`auto_min_feature_x_mm` || number || 0 || mm || || Smallest feature the mesh resolves in X. Set it and the mesh refines X to resolve it. Measured copper below it is reported as a warning, not silently meshed. 0 means no feature-size constraint on X, so measured copper only warns and the cell stays at the cells-per-wavelength and dielectric baseline. **Caution:** Per-axis and independent. Setting one axis does not tighten the others, which can leave an anisotropic mesh.
`auto_min_feature_y_mm` || number || 0 || mm || || Smallest feature the mesh resolves in Y. Set it and the mesh refines Y to resolve it. Measured copper below it is reported as a warning, not silently meshed. 0 means no feature-size constraint on Y, so measured copper only warns and the cell stays at the cells-per-wavelength and dielectric baseline.
`auto_min_feature_z_mm` || number || 0 || mm || || Smallest feature the mesh resolves in Z. A pure resolution floor: the mesh refines Z to resolve it. 0 means no feature-size constraint on Z, so the cell stays at the dielectric and cells-per-wavelength baseline. Stackup layers are always resolved and never excluded.

## Editor preferences

Setting || Type || Default || Units || Notes || Description
`editor_grid_mm` || number || 1 || mm || ui-only || Editor snap-grid spacing.
`snap_enabled` || boolean || true || bool || ui-only || Whether the editor snaps to the grid.

## Result-viewer UI state

Setting || Type || Default || Units || Notes || Description
`fv_viewer_mode` || string || currents || || enum: fields / currents, ui-only || Field viewer active mode.
`fv_cmap` || string || coolwarm || || ui-only || Field viewer colormap name.
`fv_log_scale` || boolean || false || bool || ui-only || Field viewer linear vs log scale (fields only).
`fv_field_scale_mode_idx` || integer || 1 || || ui-only || Field viewer scale mode index (0 frame-based, 1 local, 2 global).
`fv_field_ref_frame` || integer || 0 || || ui-only || Field viewer reference frame index for scaling.
`fv_show_geometry` || boolean || true || bool || ui-only || Field viewer geometry overlay visibility.
`fv_geom_opacity` || integer || 100 || percent || ui-only || Field viewer geometry overlay opacity in percent.
`fv_photorealistic_geometry` || boolean || true || bool || ui-only || E/H field viewer geometry uses studio lighting, matte cream dielectric, rough copper, local field illumination, and contact shading around copper edges.
`fv_show_meshing` || boolean || false || bool || ui-only || Field viewer geometry shows mesh cell outlines, independently of material lighting.
`fv_max_fps` || number || 30 || FPS || ui-only || Maximum playback rate for E/H fields and time-domain surface currents, with slower playback when loading or rendering takes longer.
`fv_field_frame_idx` || integer || 0 || || ui-only || Field viewer current frame index.
`fv_sc_freq_idx` || integer || -1 || || ui-only || Surface-current viewer frequency index (-1 defaults to centre).
`fv_sc_time_idx` || integer || 0 || || ui-only || Surface-current viewer time-frame index.
`fv_sc_scale_mode_idx` || integer || 2 || || ui-only || Surface-current viewer scale mode index (0 ref-based, 1 local, 2 global).
`fv_sc_ref_idx` || integer || 0 || || ui-only || Surface-current viewer reference index for scaling.
`fv_sc_cmap` || string || inferno || || ui-only || Surface-current viewer colormap name.
`fv_rad_freq_idx` || integer || -1 || || ui-only || Radiation tab frequency index (-1 defaults to centre).
`fv_rad_viewer_mode` || string || polar || || enum: polar / 3d, ui-only || Radiation viewer mode.
`fv_rad_show_geometry` || boolean || true || bool || ui-only || Radiation viewer geometry overlay visibility.
`fv_rad_show_pattern` || boolean || true || bool || ui-only || Radiation viewer 3D pattern visibility.
`fv_rad_geom_opacity` || integer || 100 || percent || ui-only || Radiation viewer geometry opacity in percent.
`fv_rad_show_meshing` || boolean || true || bool || ui-only || Radiation viewer geometry shows mesh cell outlines.
`fv_rad_photorealistic` || boolean || false || bool || ui-only || Radiation viewer geometry uses matte cream dielectric and rough copper with physical lighting.
`fv_rad_pattern_opacity` || integer || 85 || percent || ui-only || Radiation viewer pattern-surface opacity in percent.
`fv_rad_polar_floor_dbi` || number || -25 || dBi || ui-only || Radiation viewer polar plot floor in dBi. A sample below it is drawn at the chart center, and the floor ring is labeled with this value.
`fv_rad_auto_position` || boolean || true || bool || ui-only || Radiation viewer auto-computes the pattern origin.
`fv_rad_auto_scale` || boolean || true || bool || ui-only || Radiation viewer auto-computes the pattern scale.
`fv_rad_pos_x_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin X.
`fv_rad_pos_y_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin Y.
`fv_rad_pos_z_mm` || number || 0 || mm || ui-only || Radiation viewer manual pattern origin Z.
`fv_rad_pattern_scale` || number || 1 || factor || ui-only || Radiation viewer manual pattern scale factor.
`fv_rad_split_view` || boolean || true || bool || ui-only || Radiation viewer shows polar and 3D side by side.
`fv_rad_split_sizes` || array || [] || || ui-only || Radiation viewer splitter pixel widths.
`fv_rad_camera_position` || array || [] || || ui-only || Saved radiation viewer 3D camera (9 floats: position, focal point, up vector).
`fv_mesh_show_meshing` || boolean || true || bool || ui-only || Simulate mesh preview shows mesh cell outlines.
`fv_mesh_photorealistic` || boolean || false || bool || ui-only || Simulate mesh preview uses matte cream dielectric and rough copper with physical lighting.
`fv_sparams_visible_panels` || array || ['magnitude', 'smith'] || || ui-only || S-parameter tab visible panels, in canonical order (magnitude, smith, impedance, network). Never empty. The matching-network panel is selectable only when the run recorded network loss.
`fv_sparams_split_sizes` || array || [] || || ui-only || S-parameter tab splitter pixel widths, one per visible panel.

## Advanced-mode shadow values

Every Advanced field has an `adv_*` shadow that holds the Advanced-mode value while the project is in Auto mode. Each shadow is flagged **advanced, ui-only** and carries the same meaning as the setting it mirrors, documented above.

Setting || Type || Default || Units || Mirrors
`adv_dx_mm` || number || 0.2 || mm || `dx_mm`
`adv_dy_mm` || number || 0.2 || mm || `dy_mm`
`adv_dz_mm` || number || 0.2 || mm || `dz_mm`
`adv_air_around_mm` || number || 10 || mm || `air_around_mm`
`adv_air_above_mm` || number || 10 || mm || `air_above_mm`
`adv_air_below_mm` || number || 10 || mm || `air_below_mm`
`adv_auto_air_enabled` || boolean || false || bool || `auto_air_enabled`
`adv_auto_air_sides_factor` || number || 0.25 || fraction || `auto_air_sides_factor`
`adv_auto_air_above_factor` || number || 0.25 || fraction || `auto_air_above_factor`
`adv_auto_air_below_factor` || number || 0.25 || fraction || `auto_air_below_factor`
`adv_air_mode` || string || cells || || `air_mode`
`adv_air_cells` || integer || 6 || cells || `air_cells`
`adv_pml_thickness_mm` || number || 4 || mm || `pml_thickness_mm`
`adv_auto_pml_factor` || number || 0.1 || fraction || `auto_pml_factor`
`adv_pml_mode` || string || cells || || `pml_mode`
`adv_pml_cells_manual` || integer || 8 || cells || `pml_cells_manual`
`adv_ringdown_end_crit` || number || 1e-05 || ratio || `ringdown_end_crit`
`adv_ringdown_min_steps` || integer || 0 || steps || `ringdown_min_steps`
`adv_nf2ff_num_freqs` || integer || 51 || count || `nf2ff_num_freqs`
`adv_save_vtk` || boolean || false || bool || `save_vtk`
`adv_vtk_dump_interval` || integer || 50 || steps || `vtk_dump_interval`
`adv_vtk_dump_interval_auto` || boolean || false || bool || `vtk_dump_interval_auto`
`adv_field_format` || string || rff || || `field_format`
`adv_vdb_threshold_frac` || number || 0.001 || fraction || `vdb_threshold_frac`
`adv_field_export_range_enabled` || boolean || false || bool || `field_export_range_enabled`
`adv_field_export_range_start` || integer || 0 || steps || `field_export_range_start`
`adv_field_export_range_end` || integer || 10000 || steps || `field_export_range_end`
`adv_auto_mesh_enabled` || boolean || false || bool || `auto_mesh_enabled`
`adv_conv_source` || string || S11 || || `conv_source`
`adv_conv_num_resonances` || integer || 1 || count || `conv_num_resonances`
`adv_conv_freq_threshold_db` || number || -20 || dB || `conv_freq_threshold_db`
`adv_conv_depth_threshold_db` || number || -20 || dB || `conv_depth_threshold_db`
`adv_conv_mesh_refine_factor` || number || 1.26 || factor || `conv_mesh_refine_factor`
`adv_conv_max_passes` || integer || 8 || count || `conv_max_passes`
`adv_enable_surface_currents` || boolean || false || bool || `enable_surface_currents`
`adv_surface_current_layer` || string || || || `surface_current_layer`
`adv_surface_current_interval_auto` || boolean || true || bool || `surface_current_interval_auto`
`adv_surface_current_interval` || integer || 50 || steps || `surface_current_interval`
`adv_surface_current_num_freqs` || integer || 63 || count || `surface_current_num_freqs`
`adv_mesh_grading_x_mult` || number || 1 || factor || `mesh_grading_x_mult`
`adv_mesh_grading_y_mult` || number || 1 || factor || `mesh_grading_y_mult`
`adv_mesh_grading_z_mult` || number || 1 || factor || `mesh_grading_z_mult`
`adv_mesh_grading_margin_mm` || number || 1 || mm || `mesh_grading_margin_mm`
`adv_mesh_grading_x_max_ratio` || number || 1.3 || ratio || `mesh_grading_x_max_ratio`
`adv_mesh_grading_y_max_ratio` || number || 1.3 || ratio || `mesh_grading_y_max_ratio`
`adv_mesh_grading_z_max_ratio` || number || 1.3 || ratio || `mesh_grading_z_max_ratio`
`adv_cw_enabled` || boolean || false || bool || `cw_enabled`
`adv_cw_freq_hz` || number || 2.4e+09 || Hz || `cw_freq_hz`
`adv_sparam_early_stop` || boolean || false || bool || `sparam_early_stop`
`adv_sparam_delta_crit_db` || number || -40 || dB || `sparam_delta_crit_db`
`adv_sparam_min_steps` || integer || 10000 || steps || `sparam_min_steps`
`adv_enable_nf2ff` || boolean || true || bool || `enable_nf2ff`
`adv_skip_geometry_export` || boolean || false || bool || `skip_geometry_export`
