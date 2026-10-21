# PCB Toolkit
> Fifty PCB design calculators that run in the browser, from trace impedance to crosstalk and current capacity.

The [PCB Toolkit](tools/pcb/) is the pcb_toolkit calculator library compiled from Rust to WebAssembly. The page downloads once and then runs entirely on your machine: nothing you type is sent anywhere.

## Using a calculator

Pick a calculator from the list on the left. Its inputs appear in the middle with their units, and the results update as you type. When an input is outside the range a formula is defined for, the result area names the field and the limit instead of showing a number.

Dielectric inputs can be filled from the material library, which lists 45 common laminates with their dielectric constant and loss tangent. Wire-gauge inputs use the AWG table.

## Linking to a calculator

Each calculator has its own address. The part after `#` is the calculator's id, so a link to `tools/pcb/#impedance.microstrip` opens the microstrip calculator directly. The address updates as you move between calculators, so the back button and bookmarks work as expected.

## Calculators

Group || Calculators
Impedance || Microstrip, stripline, offset stripline, embedded microstrip, coplanar waveguide, round wire over a plane, coaxial cable, twisted pair
Differential pairs || Edge-coupled external, edge-coupled internal (symmetric and asymmetric), edge-coupled embedded, broadside-coupled
Current and thermal || Conductor current (IPC-2221A and IPC-2152), fusing current (by trace and by area), thermal management, plane spreading resistance
Via and padstack || Via properties, thru-hole padstack, corner-to-corner distance, conductor spacing
Signal integrity || Crosstalk (NEXT), edge bandwidth, critical length, reflection and termination, Er from delay, wavelength, reactance, reactance to an edge, planar spiral inductor, circular and rectangular loop inductance, PDN impedance
Ohm's law || Ohm's law, LED bias resistor, pi-pad and T-pad attenuators, resistors, capacitors and inductors in series and in parallel
Crystal and PPM || Hz to PPM, PPM to Hz, crystal load capacitance
Reference || Wire gauge (AWG), gauge from diameter

!note The calculators use published closed-form models. Each is valid over a stated range of inputs, and real boards differ from the idealised cross-sections, so treat the results as starting points to confirm by simulation or measurement.
