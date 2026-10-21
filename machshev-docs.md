# Machshev documentation

# Installing Machshev

How to get Machshev onto your machine and confirm it runs.


## System requirements

!ph Supported operating systems, CPU and GPU requirements, memory and disk space.

## Download and install

!ph Where to download Machshev, and the installation steps for each platform.

## Licence activation

!ph How a licence is activated, and how to move it to another machine.

## Checking the installation

!ph A quick test that confirms the solver runs.

---

# Your first simulation

A short walkthrough from an empty project to a first result.


## Create a project

!ph Starting a new project and choosing units.

## Build the model

!ph Drawing a simple structure, such as a microstrip patch.

## Add a port and run

!ph Placing an excitation and starting the solver.

## Read the results

!ph Where to find S-parameters, impedance and patterns.

---

# The interface

A tour of the main window and where each tool lives.


## Main window

!ph The panels of the main window and what each one holds.

## Projects and files

!ph How projects are organised on disk.

## Preferences

!ph The settings a user can change, and their defaults.

---

# Building a model

Geometry, materials and the layer stackup.


## Materials

!ph The material library, and how to define a custom material.

## Drawing geometry

!ph The drawing tools, and how shapes are edited.

## Variables and parameters

!ph Parametric dimensions, and how they drive the geometry.

## Importing

!ph File formats Machshev imports, and the options for each.

---

# Ports and excitation

How a model is excited, and how ports are defined and referenced.


## Port types

!ph The port types available, and when to use each.

## Reference impedance

!ph Setting the reference impedance for network parameters.

## Sources

!ph Other excitations, such as plane waves.

---

# Running a simulation

Solver settings, meshing, and what happens during a run.


## Solver settings

!ph Frequency range, accuracy settings and their defaults.

## Meshing

!ph How the mesh is generated, and how to refine it.

## Boundaries

!ph The boundary conditions available.

## Monitoring a run

!ph Progress, convergence, and stopping a run early.

---

# Viewing results

S-parameters, impedance, fields and radiation patterns.


## Network parameters

!ph S-parameter, impedance and Smith chart views.

## Fields and currents

!ph Near-field and surface-current views.

## Radiation patterns

!ph Far-field patterns, gain and efficiency.

## Exporting

!ph Export formats, such as Touchstone, CSV and images.

---

# PCB Toolkit

Fifty PCB design calculators that run in the browser, from trace impedance to crosstalk and current capacity.


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

---

# Antenna Studio

A NEC-2 wire-antenna modeler in the browser: build a model, solve it, and see currents, impedance and gain.


[Antenna Studio](tools/antenna/index.html) is NEC Studio, a method-of-moments solver for wire antennas written in Rust and compiled to WebAssembly. Models are solved on your machine, in the browser.

## Starting a model

The model browser opens on a set of built-in templates: a half-wave dipole, an inverted V, a ground-plane vertical, a 3-element Yagi, a quad loop, a delta loop and a J-pole. Pick one to load it, then change its dimensions with the variables or in the geometry editor. The ground can be free space, a perfect ground, or one of several real soils from poor dry ground to sea water.

You can also open a NEC deck (`.nec`) or a project file (`.toml`) from your computer. Saving downloads the model as a file.

## Results

Each change is solved straight away. The 3D view shows the wires coloured by current and the radiation pattern as a surface. The side panels show the feed-point impedance and SWR, polar pattern cuts and a Smith chart.

## Sweeps, matching and the optimizer

- A frequency sweep plots impedance, SWR and gain across a band, adding each point as it is solved.
- A parameter sweep steps one variable through a range and plots the results.
- The matching panel designs a matching network for the feed point and can add it to the model.
- The optimizer adjusts chosen variables toward goals such as SWR, gain or front-to-back ratio, and reports its progress as it runs.

Long jobs run in the background, so the model stays responsive while a sweep runs. Stop cancels a job at once.

## Limits in the browser

The browser solves on a single thread. Models above about 2,000 unknowns take noticeably longer, and Antenna Studio shows a note when a model is that large. The layout needs a window at least 1,100 pixels wide; on a narrower screen, scroll sideways to reach every panel.

---

# RF calculators

Closed-form calculators for patch and inverted-F antennas, quarter-wave matching, the Smith chart and wavelength.


These calculators sit alongside the [PCB Toolkit](docs-pcb-toolkit.html) on the [RF calculators](rf-calculators.html) page and cover structures it does not. Each shows its result as you type, with the references for its equations.

Calculator || Inputs || Returns
[Patch antenna](rf-calc-patch.html) || frequency, substrate dielectric constant and height, feed impedance || patch width and length, effective permittivity, edge resistance, inset feed depth, bandwidth
[Inverted-F antenna](rf-calc-ifa.html) || frequency, substrate dielectric constant, arm height above the ground edge, trace width || starting dimensions from rules of thumb
[Quarter-wave transformer](rf-calc-qwt.html) || source and load impedance, frequency, substrate, copper thickness, acceptable reflection || transformer impedance, length, microstrip widths and bandwidth
[Smith chart](rf-calc-smith.html) || load resistance and reactance, system impedance || reflection coefficient, VSWR, return loss and mismatch loss, plotted on the chart
[Free-space wavelength](rf-calc-wavelength.html) || frequency || wavelength, half and quarter wavelength

The microstrip, stripline, coplanar, differential-pair and guided-wavelength calculators are in the PCB Toolkit, and the cards on the RF calculators page link straight to them.

---

# File formats

The files Machshev reads and writes.


## Project files

!ph The project file format and what it contains.

## Import formats

!ph Geometry and data formats Machshev can import.

## Export formats

!ph Result formats Machshev can export.

---

# Keyboard shortcuts

Shortcuts for the editor and the result views.


## Editor

!ph Shortcut keys for the model editor.

## Views

!ph Shortcut keys for navigating the 2D and 3D views.
