# Ports and excitation
> A port is where the simulation feeds energy into the structure and measures the response. This page covers the four port types, the options every port carries, the excitation waveform, and what validate checks before a run.

## What a port is

A port is a shape of kind `port`. Each port is both an excitation source and a measurement site. Its `active` flag decides which: an active port is driven, a passive port is a matched load that the run still measures.

A run drives one port and produces that port's S-matrix column: driving port n yields S1n, S2n, and so on across every measured port. S-parameter extraction requires exactly one excited port per run. Both surfaces read the same rule, so a project with several active ports, with none active, or with no ports at all is refused before the backend launches, naming the ports involved. A full matrix is built one driven port at a time, in the desktop app and in `rayrf run` alike.

!fig The four port types placed on traces in the Edit tab

## Port types

Four port types cover vertical and in-plane feeds, straight from the geometry registry. Every port type also carries the matching-network and measurement-shift keys listed under [Shared port keys](#shared-port-keys).

### `point`

Single-cell vertical probe between two conductor layers.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the feed point.
`y_mm` || number || mm || 0 || yes || Y of the feed point.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`top_layer` || string || || || yes || Positive-terminal conductor layer.
`bottom_layer` || string || || || yes || Negative-terminal conductor layer.

### `rect`

Distributed rectangular (area) feed between two conductor layers.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || X of the footprint center.
`y_mm` || number || mm || 0 || yes || Y of the footprint center.
`w_mm` || number || mm || none || yes || Footprint width.
`h_mm` || number || mm || none || yes || Footprint height.
`port_type` || string || || rect || yes || Must be `rect`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`direction` || string || || z || || Feed axis: z (transverse), x, or y.
`top_layer` || string || || || yes || Positive-terminal conductor layer.
`bottom_layer` || string || || || yes || Negative-terminal conductor layer.

### `line`

In-plane line feed across a gap on a single conductor layer. The feed drives along its director, p0 to p1, at any angle: on an axis it is an X or Y feed, off the axes it drives both in-plane components and measures along the drawn director.

Param || Type || Units || Default || Required || Meaning
`p0_x_mm` || number || mm || 0 || yes || Endpoint 0 X (negative terminal).
`p0_y_mm` || number || mm || 0 || yes || Endpoint 0 Y (negative terminal).
`p1_x_mm` || number || mm || none || yes || Endpoint 1 X (positive terminal).
`p1_y_mm` || number || mm || none || yes || Endpoint 1 Y (positive terminal).
`width_mm` || number || mm || 0.1 || || Strip width perpendicular to the line.
`port_type` || string || || line || yes || Must be `line`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`bottom_layer` || string || || || yes || Conductor layer the feed sits on.
`top_layer` || string || || || || Same layer as `bottom_layer` for a planar feed.
`excitation_mode` || string || || planar || || Must be `planar`.
`deembed_ref_layer` || string || || || || Conductor layer the shifted recording point measures voltage against, required when this in-plane feed carries a measurement shift. It must not be the layer the feed sits on. Refused on a z-fed port, whose `bottom_layer` already names it.

### `bar`

In-plane bar feed on a single conductor layer. The feed drives along its director at any angle: on an axis it is an X or Y feed, off the axes it drives both in-plane components and measures along the drawn director.

Param || Type || Units || Default || Required || Meaning
`x_mm` || number || mm || 0 || yes || Bar center X.
`y_mm` || number || mm || 0 || yes || Bar center Y.
`length_mm` || number || mm || 1 || yes || Gap length along the feed director.
`width_mm` || number || mm || 4 || yes || Transverse bar width.
`angle_deg` || number || deg || 0 || || Director angle from +X, any value. The feed drives along it, and off the axes the gap is realized as a staircase on the mesh.
`port_type` || string || || bar || yes || Must be `bar`.
`port_number` || int || || 1 || yes || Port index.
`impedance_ohm` || number || Ohm || 50 || || Reference impedance.
`active` || bool || || false || || Drive (True) or load (False).
`bottom_layer` || string || || || yes || Conductor layer the feed sits on.
`top_layer` || string || || || || Same layer as `bottom_layer` for a planar feed.
`excitation_mode` || string || || planar || || Must be `planar`.
`deembed_ref_layer` || string || || || || Conductor layer the shifted recording point measures voltage against, required when this in-plane feed carries a measurement shift. It must not be the layer the feed sits on.

### Shared port keys

These keys are the same on all four port types:

Param || Type || Units || Default || Meaning
`network` || object || || none || Matching ladder between the source and the port terminal, written system to device: `{branches: [{orient: series or shunt, parts: [{kind, value, q, q_ref_hz}]}]}`. `kind` is L, C, or R. `value` is H, F, or ohm as a number or an expression over variables. `q` is a required positive quality factor on L and C, refused on R. `q_ref_hz` is the required frequency in Hz that q is exact at. A resistor dissipates the resistance you author. Absent means none.
`deembed_mm` || number || mm || 0 || Distance the point where voltage and current are recorded moves from the feed toward the device, along `deembed_dir`. The recorder moves during the run: nothing is corrected afterwards, and the port still injects where it sits. 0 records at the feed. This is the technique RF practice calls de-embedding.
`deembed_dir` || string || || || Direction the recording point moves: one of x+, x-, y+, y-. It moves in a straight line, so the trace has to run straight through the whole shift. A landing point that crosses no metal is refused at the start of the run. A trace that curves away is not refused, and leaves the recorder reading whatever other copper is at the landing point. An in-plane feed drives the trace it is cut into, so only its own axis is legal.
`network_plane` || string || || system || Reference plane for the reported S-parameters when this port has both a matching network and a measurement shift: `system` keeps the network in the reported reflection, `deembed` reports the device alone at the shifted recording point. Ignored on a port that lacks either.

Placement notes the table does not carry:

- A **point** port is a single-cell vertical probe. It drives the gap between a `top_layer` and a `bottom_layer`, so the two must be different conductor layers. Use it as the default lumped feed on a narrow trace. The point and area placement dialogs label `top_layer` Positive layer and `bottom_layer` Negative layer.
- A **rect** port spreads the same vertical feed across a footprint you drag out. Use it when a single cell is too coarse for the conductor, such as a wide trace or a patch edge. Its `direction` sets the feed axis: z feeds vertically between `bottom_layer` and `top_layer`, x or y feeds in-plane along that axis.
- A **line** port is an in-plane feed across a gap on one layer, set by two endpoints: p0 is the negative terminal, p1 the positive, and current runs p0 to p1. Use it for a coplanar or edge feed where both terminals sit on the same layer. The endpoints come from the two-click placement, not a dialog, and snap to the nearest axis.
- A **bar** port is a fixed in-plane bar on one layer, placed with a single click. Use it when you want a set-orientation in-plane gap rather than two picked endpoints. The bar's director angle is free: on an axis the bar is an X or Y feed, off the axes the solver drives and measures along the drawn director and realizes the gap as a staircase on the mesh.

The point and line dialogs set the electrical options and the layers:

!fig The point port placement dialog

!fig The in-plane line port placement dialog

## Common options

Every port carries the same electrical options.

- `port_number` names the S-matrix column, so keep it unique across the project. Two ports sharing a number leave the S-matrix mapping undefined, which validate refuses. A new port takes the lowest unused number.
- `impedance_ohm` is the reference impedance the S-parameters normalize to, 50 by default. It must be positive: the wave math divides by its square root.
- **Measurement shift**, a checkbox in the port dialog, moves the spot where the run records this port's voltage and current, on any port type. This is what RF practice calls de-embedding, and the stored keys keep that name. Turn it on, then set `deembed_mm` to the distance and `deembed_dir` to the direction (x+, x-, y+, or y-) pointing from the feed toward the device. Off, the port records at the feed. The matching network sits behind its own checkbox at the bottom of the same dialog. Use it to leave a length of feed line out of the reported impedance and S-parameters. The distance cannot be negative, as the direction control sets the sign. On a port that also carries a matching network, the shift moves the recording spot past the network as well, so `network_plane` chooses which plane is reported.

The run samples voltage and current at the shifted position from the first timestep instead of correcting feed measurements afterward. The excitation remains at the port.

The recording spot travels down the line the feed drives, and that is what decides which directions a port accepts. A z-fed feed sits between a strip and the conductor under it, so the line it drives runs in-plane and all four directions are legal. An in-plane feed sits in a gap cut across a trace and drives that trace, so only its own axis is legal. A spot shifted across such a feed would leave the trace, and the editor does not offer it.

A measurement shift requires a straight conductor of constant width. At run start, RayRF checks the realized copper cell by cell from the feed to the recording position. The run is refused by port name if the position is on bare substrate or disconnected metal, or if the path includes a bend, taper, or junction.

An in-plane feed also has to name the conductor its shifted voltage is measured against, in `deembed_ref_layer`. It sits on one layer, so unlike a z-fed feed it carries no second terminal of its own. At the shifted spot the reported voltage is the one between the trace and the return conductor, usually the ground plane under it. It cannot be the layer the feed sits on. A z-fed port refuses the key, since its `bottom_layer` already names that conductor. With more than one conductor on offer the editor asks rather than choosing: the control starts on "Choose layer", and a port left that way is refused by name at the run decision.

The shift lands on a mesh line, so what the run records is a whole number of cells from the feed and not the distance you typed. The property panel says where it lands in its Records at row, estimate prints the same number per port under records at, and the run log and the per-port network-loss file carry it as well.

The shift rows for a line or bar port live in the Edit tab's property panel, not in its placement dialog. Those two ports get their director from the placement gesture, so while the dialog is open the port has no axis yet and a direction control there could not know what it was offering.

Ports serialize into the solver input as lumped feeds: a point port becomes a `lumped_point`, the others a `lumped_rect`, with the impedance carried as `R_ohm`. An in-plane line or bar on an axis emits as an X or Y feed. Off the axes it emits as an xy feed carrying its unit director and one footprint mask per in-plane component, and the backend drives both components along the director.

## Excitation

The source is a Gaussian pulse spanning the frequency band, from `freq_min_hz` to `freq_max_hz`. One broadband pulse excites the whole band in a single run, and the S-parameters come from the ratio of the transformed port voltages and currents.

Continuous-wave mode replaces the pulse with a steady sinusoid at one frequency. It never rings down, so the run ends at its step count. Under a single tone the S-parameters are of limited meaning, while the radiation pattern at the tone is meaningful. Both outputs stay available, and field animations are the usual reason to use it. CW is an advanced-mode feature, and an auto-mode project with CW on is refused rather than run as a pulse. See [FDTD tips](docs-fdtd-tips.html).

## What validate catches

Run `validate` before a run. For ports it reports:

- A zero-height span, as an error. A point or rect port whose top and bottom conductors land on the same realized z mesh line collapses to no vertical extent and injects nothing. Setting `top_layer` equal to `bottom_layer` lands on one line by definition, so it is refused the same way.
- No active port, as an error. With nothing driven there is no excitation and no S-parameters, and a project with no ports at all, or with more than one active port, is refused the same way.
- A port that maps to zero cells or lies outside the domain, as an error, and a non-positive impedance or a negative shift distance, as an error.
- A duplicate `port_number` or an unrecognized `deembed_dir`, as an error, and so is a shift running across an in-plane feed rather than down the trace it is cut into, or one on a feed with no extent at all, which names no axis.
- A shifted in-plane feed that names no return conductor, or names one this project has no conductor layer for, as an error.
- A port with no copper under it on one of its reference layers, as an error. Copper that a hole or cutout removes does not count.

Whether the trace actually runs under the shift is not something validate can see: it depends on the mesh, so the run checks it at the start and refuses there instead.
