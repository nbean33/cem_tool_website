# Radiation pattern
> The Radiation Pattern tab shows the antenna far field as a 2D polar plot and a 3D lobe over the geometry, one pattern per simulated frequency. It fills in when a run finishes with the radiation pattern enabled.

## Where the data comes from

The solver wraps the structure in a Huygens surface and records the near field on it during the run, then transforms that near field to the far field. Turn it on with `enable_nf2ff` (the Compute Radiation Pattern output toggle). The transform runs at a set of frequencies spread across the band, so every result below is available at each of those points. In auto mode the frequency count comes from the quality preset. In advanced mode `nf2ff_num_freqs` sets it directly, default 51, up to the backend ceiling of 512 points.

The far field is defined only when the box radiates into open space, so the run needs at least one PML face. Any mix of PML, PEC, and PMC on the remaining faces works. A PEC face acts as an infinite ground plane by image theory. Two cases are refused before the solve: a box with no PML face (a closed cavity, no radiated power), and a PEC or PMC pair on both faces of one axis (that axis is closed, so the pattern is undefined). See [Boundaries](docs-boundaries.html).

## Polar view

The polar view draws the two principal cuts, the E-plane and the H-plane. The right panel holds a frequency selector, a metrics block, and toggles for the two traces.

The frequency slider and the linked GHz spinbox pick which stored pattern to show. Patterns prepare in the background and stay cached while you scrub. If a pattern is still loading, the tab prioritizes the current selection and keeps navigation available. The dB scale is fixed across the slider so the traces stay comparable as you scrub. Clip below sets the floor (default -25 dBi), and a sample below it draws at the center. The ceiling is the peak directivity of the whole band plus 2 dB, rounded up. The legend names each cut and its phi angle.

The metrics block reports:

- **Directivity max.** Peak directivity at the selected frequency, in dBi.
- **HPBW theta and HPBW phi.** Half-power beamwidth of the E-plane and H-plane cuts, in degrees, measured across the connected -3 dB region around the peak.
- **Sidelobe.** Level of the largest lobe outside the main lobe, in dB relative to the peak, measured on the plane cut that carries the global peak.
- **F/B ratio.** Peak gain minus the gain in the opposite direction on that same cut, in dB.

## 3D view

The 3D view draws the pattern as a color-mapped lobe with the geometry overlaid at scale, so the beam direction reads against the antenna. Radius and color both follow directivity over a 40 dB window below the peak, and the scalar bar is labeled in dBi.

Two opacity sliders set the alpha blend between the lobe and the geometry, and either can be hidden with its visibility toggle. Show meshing controls the geometry cell outlines independently of Photorealistic, which lights cream dielectric and rough copper. New projects start with mesh outlines on and photorealistic shading off, keeping the dielectric translucent. The project saves this pair separately from Simulate and Field Viewer.

Position and scale each have an auto mode and a manual mode:

- **Auto position** centers the lobe on the geometry. Turn it off to set the X, Y, and Z origin directly in millimeters.
- **Auto scale** sizes the lobe so its tip-to-tip diameter matches the longest dimension of the geometry. Turn it off to set the scale directly, which holds a fixed lobe size while you step across frequencies.

Views picks which pattern panels are shown. Check both Polar Pattern and 3D Pattern to get them side by side with a draggable splitter, both driven by the one frequency slider. Check one to give it the whole workspace. At least one view stays checked.

The 3D view uses the same upright navigation as Simulate and Field Viewer. Left drag orbits. Right drag, middle drag or Shift plus left drag pans. The wheel zooms toward the point under the pointer, which stays under it. Each orbit starts around the closest board surface at the center of the view and keeps that pivot for the drag. Without a board, it uses the pattern surface. A miss keeps the last valid pivot, starting at the board or scene center. Orbit sensitivity is 0.25 degrees per logical pixel. The cursor is hidden during a drag and returns to where the drag started. A drag keeps going past the edge of the viewport. Release the button or press Esc to end it. Home fits the 3D scene and resets polar pan and zoom.

## Export to CSV

Export CSV asks what to write, then a save path. The scope dialog offers two choices:

- **E-plane and H-plane cuts** writes only the two displayed curves. Columns: `theta_deg`, `phi_deg`, `directivity_dBi`, `plane`.
- **Full 3D pattern** writes every theta/phi sample the run holds for that frequency, with no resampling. Columns: `theta_deg`, `phi_deg`, `directivity_dBi`.

Both files open with comment lines naming the frequency, the run, and the project. The export covers the frequency selected in the tab.

The CLI writes a flat table across the whole band instead, straight from the backend columns:

```
freq_hz,theta_deg,phi_deg,D_total_dBi,D_theta_dBi,D_phi_dBi
```

`D_total_dBi` is the total directivity, `D_theta_dBi` and `D_phi_dBi` the two polarization components, all in dBi.

## From the CLI

`machshev run` writes the same patterns headless: `--export-polar` for the polar PNG, `--export-rad3d` for the 3D PNG (add `--export-rad3d-vtk` for the surface as VTP), and `--export-rad-pattern` for the flat CSV above. `--polar-freq-mhz` and `--rad3d-freq-mhz` pick the frequency, defaulting to the band center. Full flag list on the [CLI reference](docs-cli-reference.html).
