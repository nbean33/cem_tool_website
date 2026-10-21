# S-parameters and Smith chart
> The S-Parameters tab plots the network response of a finished run: magnitude in dB against frequency, the same data on a Smith chart, and the port input impedance against frequency. It also carries the markers, reference overlays, imports, and exports for that data.

After a run, each active port contributes a column of traces. The tab shows the reflection Sii for every active port, and both Sij and Sji for every active pair. The reflection trace of the port that was driven this run is the one that carries impedance, so it also appears on the Smith chart.

Trace labels follow the port identities recorded by the run, including sparse result sets. A result for port 2 stays S22 even when no port 1 result is present. It uses port 2's recorded reference impedance rather than its position in the result file. Labels through port 9 use compact forms such as S21. If either port number has two digits, a comma keeps the identities unambiguous, as in S10,1, S1,10, and S10,10.

## Traces and parameter chips

The toolbar carries a chip for S11 and S21 and a More Parameters popover. A chip toggles that trace on the plot and colors itself to match it. More Parameters lists every remaining Sij for the port count, plus S12 and S22 even for a one-port run, and holds one checkbox per imported overlay. Each trace keeps a fixed color so a port reads as the same hue across the plot, the chart, and the editor canvas.

The active ports determine which traces are initially visible. Changing the active ports resets the trace selection. Chips remain enabled when data is missing. A selected trace with no data means the run did not produce that result.

Once you edit the project after a run, a badge in the corner says so ("Geometry changed since this run", "Config changed since this run", or both). A run that stopped before its ring-down target also gets a badge that says how it stopped. A stale badge means re-run before trusting the numbers.

## The panels

Panels is a dropdown of checkable entries, one per panel: Magnitude (dB), Smith chart, and Impedance. Any subset can be checked, and the visible panels sit side by side in that fixed order, divided by draggable splitters. At least one panel is always visible: unchecking the last one is refused. A new project starts with the magnitude plot and the Smith chart. The selection and the splitter widths are stored in the project.

When a run recorded matching-network loss, the Panels list adds Matching-network loss. It plots each network part's dissipated power and the total, as a percent of the power sent toward the port, against frequency. With more than one matched port, each curve is named by its port.

The dB plot has frequency in GHz on the x-axis and magnitude in dB on the y-axis. Drag to pan, scroll to zoom both axes, Ctrl+scroll to zoom frequency only, Shift+scroll to zoom magnitude only.

The Smith chart is normalized to 50 Ω. Constant-resistance circles and constant-reactance arcs are labeled in ohms, the center is the matched load, the left real-axis endpoint is a short and the right is an open, and inductive (+jX) reactance is the upper half. It stays square and aspect-locked, so it never stretches. Only reflection traces carry the complex impedance the chart needs, so a magnitude-only trace does not draw here. An exact short stays at the left endpoint and reads 0 ohm. An exact open stays at the right endpoint and reads infinite ohm rather than a clipped finite value.

The impedance panel plots the port input impedance Zin = Z0 (1 + S) / (1 - S) of every visible reflection trace, using that port's reference impedance recorded with the run for Z0. Editing the live port after a run does not change the plotted impedance. Recorded reference-impedance metadata must contain finite positive values through the highest recorded backend port. A partial record is refused rather than padded with 50 ohms. Each trace contributes a solid Re Z curve and a dash-dot Im Z curve in the trace color, against frequency, with the axis in ohms. Reference and imported overlays draw here the same way they do on the magnitude plot, dashed. Transmission and magnitude-only traces carry no input impedance and do not appear. The hover readout shows the same R + jX value the magnitude plot's marker readout shows at that frequency. At S = +1 the readout is infinite ohm, and at S = -1 it is 0 ohm.

## Ports with matching networks

A port that carries a matching network is plotted at the system plane: its reflection is referenced at the ideal source behind the network, against the port's reference impedance. That is what an instrument connected through the same network would read, and every export (CSV, Touchstone, sweep output) uses the same plane. The run keeps recording the port's other recorded plane too, so the input impedance is available at both planes from one run: the port terminal when there is no measurement shift, the de-embedded plane when there is. The per-port network-loss CSV carries both, under headers that name which plane each one is.

A measurement shift on a port that carries a matching network is the one case where the reference plane is a choice. The shift moves the spot where the run records voltage and current, and the network sits between the ideal source and the port footprint, so the shifted spot is past the network as well as past the feed, and the two planes give genuinely different answers. The editor's Reported plane control, `network_plane` in a spec file, decides which one is reported. `system`, the default, keeps the plotted and exported response at the system plane through the network, and the de-embedded input impedance is recorded beside it in the per-port loss CSV. `deembed` reports the de-embedded plane instead, where the network is absent from the reflection by construction, and the system-plane impedance is recorded beside it. Either way the dissipated-power columns are referred to the power sent toward the port and do not change with the choice, the Smith chart's other-plane overlay is labeled with the plane it draws, and the exported Touchstone names its reference plane in a comment line.

The loss CSV reports one baseline and only one: each column is that circuit part's dissipated power as a share of the power sent toward the port, meaning the incident wave at the port reference resistance. The reflection follows from `zin_sys` and that same reference resistance, so the share of accepted power is derivable from the file rather than duplicated in it. The file opens with comment lines starting with `#` that state this and name the reference locations its impedance columns describe, and a frequency where a quantity is undefined is written as an empty cell, never as a number. The run manifest names both planes per port: `loss_reference_plane` for the loss columns and `reported_plane` for the S-parameters.

Every inductor and capacitor in a matching network requires a positive Q and the frequency that Q is exact at. The loss resistance it realizes is computed once at that frequency and is constant over frequency, so nothing about the simulation band or the excitation changes a component value. A resistor takes no Q: it dissipates the resistance you entered. The continuous preview, the stepped simulation, and the loss report all use the same realized values.

When a run recorded matching-network results, a checkbox for the port's other recorded plane appears in the controls row, labeled Device plane or De-embedded plane for the plane the run actually recorded. It overlays each matched port's other recorded input impedance on the Smith chart, dimmer, beside its system-plane trace, so the transformation the network applies is visible as the movement between the two loci.

A matching network attaches across the whole port footprint: it drives the footprint with a uniform current density and loads the area-averaged voltage over the same footprint. That pair is the network's terminal, and it is the plane the device-plane numbers of a networked port describe, on every footprint. A bare port's device plane is the center-line voltage over the same Ampere loop, which is a different functional. On a single-column footprint the two coincide, and on a wide footprint they do not. Removing the network from a wide port therefore moves the reported device plane, by design.

Because both planes of a networked port are read off one terminal, the system-plane impedance minus the device-plane impedance is the authored ladder, on any footprint, to the run's own discretization error. What is left over is that discretization and nothing else. The plotted response is still the co-simulated one, not a hand cascade: the port loads the network with whatever the structure presents, and the two only agree to the extent that the cascade is a good model of the geometry.

## Range and Home

Range opens a popover to set the frequency and magnitude axis limits. Leave a bound on Auto to fit the data. The popover reflects the plot's true current extent: a manual bound shows its value, and an Auto bound shows the value it resolves to in the gray caption beneath. Home clears every bound back to Auto and refits every visible panel. Range applies to the dB plot only.

## Markers

Marker (shortcut M) arms placement. Click the plot to drop a marker on the nearest trace point, and placement disarms after one marker. A marker reads out its frequency and magnitude, and the impedance where the trace carries it. On the Smith chart the readout is frequency, |Gamma|, angle of Gamma, and impedance.

Drag a marker to slide it along the trace, or to another trace. Esc cancels placement, Delete removes the selected marker, and Ctrl+Z undoes the last marker edit.

Right-click a marker for Rename, Hide, Delete, and, on the dB plot, Properties. The properties dialog:

- Sets the source trace and renames the marker.
- Follows a chosen dip. Pick the rank N and the ordering (deepest, shallowest, lowest frequency, highest frequency). A tracked marker re-snaps to that dip on every replot, including each Live-mode rerun, so a linked [surface-current or radiation](docs-field-viewer.html) viewer stays on the resonance as it moves. If the dip is gone the marker shows (no match) and holds its last frequency.
- Draws a vertical line at the center frequency, shades the bandwidth band below a dB threshold (default -10 dB, reflection traces only), and shows a center-frequency and bandwidth readout on the plot.

## Set as Reference

Set as Reference freezes the traces currently on screen as gray dashed overlays, so you can change the design and watch each trace shift against the frozen one. The baseline is stored in the project, and one is kept at a time: setting a new reference replaces the old one.

The reference is view-neutral. Reflection traces keep their complex impedance and phase, so the baseline shows on both the dB plot and the Smith chart, while transmission traces keep magnitude and show on the dB plot only. Setting a reference does not move the axes. In live mode the button moves into the More Parameters popover.

## Importing reference traces

Import a measured or external sweep to overlay it on the run. Both live in the Import / Export dropdown, render as dashed traces in distinct colors, and are stored in the project so the source file can be deleted afterward. Importing requires a signed-in license.

- **Touchstone** reads `.s1p` through `.s8p` and `.sNp`. Each S-parameter in the file becomes one overlay trace.
- **CSV** opens a column-mapping dialog, because a plain CSV is not self-describing. Choose the frequency column and one of three data types: an S-parameter as dB magnitude plus phase in degrees, an S-parameter as real plus imaginary, or an impedance Z as real plus imaginary in ohms. The second value column is optional. The frequency unit is taken from an explicit choice, then from the column header, then guessed from the values, with the guess confirmed before import. The reference Z0 you set converts an imported impedance to S and is stored with the trace so the Smith chart renders it correctly.

## Exporting

Every export is in the Import / Export dropdown. Exact file layouts are on [File formats and outputs](docs-file-formats.html).

- **CSV** writes one row per frequency, with each trace interpolated onto the first parameter's frequency grid and points outside a trace's own band left blank. When both plots are visible and reflection data is present, a dialog offers the dB magnitude, the reflection coefficient, or both.
- **Touchstone** writes all S-parameters as one network file, labeled by port count (`.s2p` for two ports). Filled values are disclosed in the export dialog and in file comments. The fill rules are on [File formats and outputs](docs-file-formats.html). Touchstone export is a Pro feature and shows a lock badge otherwise.
- **PNG** saves the plot as an image. PNG export covers the dB plot and the Smith chart. When both are visible a dialog offers the dB plot alone, both as two files, or both stitched into one wide image. Marker and curve legends are drawn into the saved image.
