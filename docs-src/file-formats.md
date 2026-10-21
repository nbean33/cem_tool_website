# File formats and outputs
> File formats and filenames for RayRF projects, exported solver cases, and simulation results.

## Project files

A project is one JSON document with the `.rfsim` extension. It holds the whole project: the layer stackup, primitives, domain, FDTD settings, parametric variables and constraints and generators, native copper artwork, embedded conductor bitmaps, import provenance, the editor thumbnail, saved view state, and (unless externalized) the last run's simulation results. Imported reference traces remain embedded in the project. Run History also indexes a copy beside the project while leaving the embedded reference unchanged. The multi-run archive itself lives only in Run History.

The document carries a version, currently 32. An older release refuses to open a file with a newer version and reports both versions rather than silently dropping the fields it does not understand. Legacy `.emflux` project files still open.

Native artwork stores apertures, paths, regions, and polarity. Imported raster copper uses compressed one-bit masks. Existing embedded PNG data remains readable.

## Results sidecar

A project holds its latest run results inside the `.rfsim`, and can instead keep that heavy bundle out of it in a sidecar file next to it with the same base name and the `.rfsimout` extension, so `Patch.rfsim` pairs with `Patch.rfsimout`. The point is version control: commit the `.rfsim`, gitignore the `.rfsimout`. Only the latest simulation results move to the sidecar. Geometry, settings, plot settings, and imported references stay in the `.rfsim`.

Externalization is set two ways:

- **Per project**, from the "Separate geometry and settings from results into two files" option in Project Settings. The choice is saved in the `.rfsim` and is the default for every save.
- **Per run**, from the CLI: `rayrf run --externalize-results project|run|off`. `off` keeps results inline, the others write the sidecar.

If a sidecar carrying results cannot be written, the save fails with the sidecar error and leaves the existing project file alone. If the project says it has a sidecar and that sidecar is missing, corrupt, from a newer build, or from a different project, the open is refused rather than quietly showing you a project with no results. The two files carry the same project-state hash and the same result-payload digest, so a half-finished two-file save is refused too, even when two runs happened to use identical inputs. A no-results save records that absence in the project and ignores older same-stem sidecars, preventing a discarded result from returning. A project that has never externalized results opens normally before its first run.

## Run History folder

Run History is a separate bounded archive named `<stem>_history` beside the project. It holds content-addressed project snapshots and per-record results for completed runs, save points, parameter-sweep points, and copies of imported references. It is not the `.rfsimout` file and is not controlled by result externalization. Move or rename this folder with its `.rfsim` when the archive should follow the project. See [Run history](docs-run-history.html) for its record and pruning rules.

## Exported solver case

Before a run, the project is serialized into a case directory under `_sim_export/<case>/`. This is the solver input, and it is what a remote submit uploads. It contains `files.json` (the case manifest, schema version 3), one PNG bitmask raster per conductor and dielectric layer plus via and area-port masks, and `_license.token`, the signed machine-bound token the backend checks before it runs. The internal `files.json` fields are not part of the documented interface and have no `rayrf schema` target.

## Interchange exports

The File menu and headless project commands write these formats:

Export || File type || Content
DXF || `.dxf` || Resolved 2D layer geometry, via drills, holes and slots, cutouts, port footprints, and the board outline.
openEMS XML || `.xml` || Resolved geometry, materials, Gaussian pulse, mesh, boundaries, vias, holes and cutouts, and supported lumped ports.

Each export shows or prints exported, approximated, and omitted content. See [Exporting DXF drawings](docs-export-dxf.html) and [Exporting openEMS XML](docs-export-openems.html) for the exact mappings.

## Run outputs

A run writes to `<project_dir>/_sim_result/<case>/` by default, or to the directory given by `--out-dir`. The backend runs in that directory (in a `backend_run/` subdirectory when `--out-dir` is set) and writes its own raw files there. The post-processed artifacts, named after the case, are controlled by the CLI export flags. GUI runs and CLI runs place these files the same way.

Every CLI verb except `auth`, `schema`, `guide`, and `calc` requires a Pro license, including every run.

Post-processed artifacts, in the output directory:

Artifact || Filename || When
S-parameter table || `<case>.csv` || Always
S-parameter plot || `<case>.png` || Always
Touchstone || `<case>.sNp`, N the port count || `--export-touchstone`, see below
Smith chart || `<case>_smith.png` || `--export-smith`
Polar pattern || `<case>_polar.png` || `--export-polar`, with radiation data
Mesh preview || `<case>_mesh.png` || `--export-mesh`
Mesh preview grid || `<case>_mesh.vtk` || `--export-mesh-vtk`
3D radiation || `<case>_rad3d.png` || `--export-rad3d`, with radiation data
3D radiation surface || `<case>_rad3d.vtp` || `--export-rad3d-vtk`
Surface-current heatmap || `<case>_currents.png` || `--export-currents`
Radiation pattern table || `<case>_rad_pattern.csv` || `--export-rad-pattern`
Run manifest || `run_manifest.json` || Always

The S-parameter table is one long-form row per trace and frequency. Its columns are `param,f_MHz,real,imag,mag,mag_dB`, so the frequency column is in MHz.

The radiation pattern table concatenates the per-frequency backend patterns into one flat CSV. Its columns are `freq_hz` followed by the backend pattern columns: `freq_hz,theta_deg,phi_deg,D_total_dBi,D_theta_dBi,D_phi_dBi`.

Raw files the backend writes into the run directory:

Artifact || Filename || When
Geometry || `geometry.vtk`, plus `geometry.vdb` when `field_format=vdb` || NF2FF or field export on
S-parameters || `sparams.csv` || Runs with ports
Port time series || `port_signals_raw.csv` || `--export-port-csv`
Per-port voltage || `port_NN_voltage.csv` || `--export-port-csv`
Per-port reflection || `port_NN_s11.csv` || Always
Radiation summary || `nf2ff_summary.csv` || NF2FF on
Radiation per frequency || `nf2ff_pattern_fNN.csv` || NF2FF on
Surface-current index || `jsurf_meta.json` || Surface currents on
Surface-current geometry || `jsurf_geometry.bin` || Surface currents on
Surface-current time frame || `jsurf_td_NNNN.bin` || Surface currents on
Surface-current frequency frame || `jsurf_fd_NN.bin` || Surface currents on
3D field frame || `step_NNNN[_type].rff`, `.vtk`, or `.vdb` || `save_vtk` on
Field frame index || `frames.json` || 3D field export on

`geometry.vtk` includes the cell arrays `material` (air, conductor, dielectric, port, or PEC edge carrier) and `material_background` (0 for air, 2 for dielectric), so the medium under a PEC edge carrier stays readable.

The Field Viewer and Radiation Pattern tabs draw the board from the geometry recorded with the result, never from the current project's outlines or copper thickness.

Each selected field quantity writes its own frame file per capture step: E magnitude has no type suffix, and the others append a type suffix (`_h` for |H|, `_u` for energy density, `_ex` .. `_hz` for the signed components). `frames.json` lists the emitted quantities under `arrays` and records each frame's display-range stats, which the field viewer folds into its Global color range without re-reading the frames.

The `sparams.csv` file holds the S-parameters the backend computes online, one row per frequency. Its columns are `freq_hz`, then a real and imaginary pair per port for the driven column (`s_i_j_real,s_i_j_imag`, where i is the receiving port and j is the driven port, so a 4-port run driven at port 1 has `s_1_1` to `s_4_1`), then `zin_real,zin_imag` for the driven port's input impedance. This payload, not the time series, is what a saved project stores so the S-parameter tabs and headless exports work after reopening.

A cell is empty where the run measured nothing: an S-parameter or input impedance at a bin the excitation never reached, and a directivity in `nf2ff_summary.csv` at a frequency that radiated no power. The same rule holds in `port_NN_s11.csv`. Every reader in RayRF treats an empty cell as unmeasured, never as zero, and a script parsing these files needs to do the same.

The raw port time series is opt-in. `--export-port-csv` writes `port_signals_raw.csv` (columns `t_s,V1,I1`) and the per-port voltage CSVs for debugging and external post-processing. Without it, only the S-parameters and the per-port reflection tables are written.

## Touchstone

Touchstone export writes measured values as measured and discloses any filled value in the file. The port count decides the extension: `.s1p` for one port, `.s2p` for two, `.s3p` for three, and so on with the port count in the name. One writer serves the S-Parameters tab and the headless exporter, so both produce the same file.

A two-port file fills an unsimulated S12 from S21 and an unsimulated S22 from S11, and writes each fill into the file as a comment:

```
! S12 was not simulated: filled with S21 (reciprocity assumption)
! S22 was not simulated: filled with S11 (symmetry assumption)
```

In a three-or-more-port file a pair the run did not measure is written as 0 and named in a `!` comment. Per-port reference impedances that differ are renormalized to one reference, which is also disclosed. That operation needs every term, so a matrix missing a pair is refused rather than renormalized against invented zeros.

## RayRF compressed field export

With `field_format=rff`, each quantity is stored as an RFF archive. The archive contains its float64 coordinate axes, quantity, extrema, display range, signed logarithmic samples, and a pyramid of coarser levels. Each level stores independently compressed bricks. Fine samples retain their nonzero state and sign, while the archive records its relative quantization error bound. Coarse brick extrema include the fine samples beneath them.

`rayrf fields inspect FILE.rff` reports the dimensions, levels, compression ratio, and error bound as JSON without reading the sample payload. RFF files contain visualization samples. The solver advances its fields in fp32.

`rayrf fields convert FRAME --out FILE.rff` writes a new RFF archive from one VTK or OpenVDB frame and prints the same report. `--source` selects the quantity and defaults to `E_mag`. An existing output file is refused.

## OpenVDB field export

When `save_vtk` is on and `field_format=vdb`, 3D field export writes OpenVDB volumes (`step_NNNN[_type].vdb`) instead of VTK frames, one file per selected field quantity with the same type suffixes as the VTK path, and writes `geometry.vdb` beside `geometry.vtk`. `vdb_threshold_frac` sets a sparsification cutoff as a fraction of each frame's peak magnitude: voxels below it collapse into the pruned background, and 0 disables pruning. The field viewer reads the volumes back through the native engine (`backend_cpu --vdb-to-dense`), and DCC tools that read OpenVDB import them too. VDB volumes carry one uniform spacing per axis, so they can represent anisotropic uniform meshes but not graded cell edges. Use RFF or VTK for graded runs. The volume transform uses the domain corner as world origin and places voxel index (0, 0, 0) at half of the X, Y, and Z cell spacing from that corner.

Field export writes |E| by default. `export_h_mag` adds |H|, `export_energy` adds total energy density, and `export_ex` .. `export_hz` add the signed field components, one array file for each exported frame. See [Field viewer](docs-field-viewer.html) for viewing frames.

## Imports

The editor and the S-Parameters tab read these files.

Import || Where || File types
Conductor mask || Edit tab || PNG (`*.png`)
Gerber board || File, Import || Gerber, Excellon, and Gerber job files
DXF drawing || File, Import || DXF (`*.dxf`)
KiCad board || File, Import || KiCad board (`*.kicad_pcb`)
openEMS setup or CSXCAD geometry || File, Import || XML (`*.xml`)
Reference S-parameters || S-Parameters tab || Touchstone `.s1p` to `.s8p`, `.sNp`
Reference trace || S-Parameters tab || CSV (`*.csv`, `*.txt`)

A PNG import scales the raster to a conductor pattern on a chosen layer. A Touchstone or CSV import is embedded in the project, so the source file can be deleted afterward. Run History also receives an indexed copy. DXF creates a new project through the shared layer and stackup assignment flow. Gerber and KiCad use the same assignment flow and ask whether to add the board to the current project or create a new one. openEMS also asks for a destination and does not transfer source ports, boundaries, or domain spacing. See [Gerber import](docs-import-gerber.html), [DXF import](docs-import-dxf.html), [KiCad import](docs-import-kicad.html), and [openEMS import](docs-import-openems.html) for each transfer contract. The `dxf` primitive holds an imported outline rasterized as a conductor pattern.
