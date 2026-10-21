# Antenna Studio
> A NEC-2 wire-antenna modeler in the browser: build a model, solve it, and see currents, impedance and gain.

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
