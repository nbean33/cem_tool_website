# RF calculators
> Closed-form transmission-line and antenna calculators for sizing a starting geometry before a full FDTD run. They open in a floating overlay in the Edit tab, and the same math is available headless through `rayrf calc`.

## The overlay

Open the overlay with Ctrl+R on the Edit tab, or with the Calculators button under the layer list in the Stackup section. Both toggle the same floating panel.

!fig RF calculators overlay open on the editor

The panel floats over the canvas. Drag it by its title bar, and resize it from any edge or corner. A dropdown at the top selects the calculator. Resize the panel larger and the dropdown becomes a grid of buttons, one per calculator. The body shows that calculator's inputs, its results, and a live cross-section preview that redraws as you type. Results are read-only text, so a value is shown exactly as the formula returned it at any scale. When a value leaves a formula's validity window, an amber notice under the form states the bound. When an input is outside the range the formula is defined on, the field turns red, the results and the preview clear, and the notice names the value and the limit. The calculators are read-only analytic tools, so they stay available when a project is open read-only.

Where a calculator has both an impedance and a width, a Solve for choice at the top of the page picks which one is the answer. The quantity being solved leaves the form and its value appears in the results.

## Pull from stackup

Each calculator that takes a substrate has a Pull from stackup checkbox. Check it, pick a Top and a Ref conductor layer, and the calculator fills the substrate height and epsilon_r from the active project between those layers, then locks those two fields. When more than one dielectric slab sits in the gap, the epsilon_r is a thickness-weighted average and the detail line lists the blend. Uncheck it to type the substrate values in directly. See [Stackup and layers](docs-stackup-and-layers.html) for how layers are defined.

## Calculators

Eleven calculators ship, in this order. Each solves the closed form named in its walkthrough. Wavelengths are reported as free-space lambda0 and guided lambda_g, and every guided-line calculator also reports the quarter wave lambda_g/4. The loss tangent is optional on every calculator that takes one: with it, the result adds the dielectric loss in dB/m and the dielectric Q, and without it those two rows are absent rather than zero.

Calculator || Inputs || Returns
Microstrip || h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
GCPW || gap S, h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
Stripline || plate spacing b, epsilon_r, copper t, frequency, tan delta, and Z0 or W || the other of Z0/W (IPC-2141A), epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q, and the copper thickness used
Patch || frequency, h, epsilon_r || patch W and L, epsilon_eff, the length extension delta L, lambda0, the radiating-edge resistance, and the fractional bandwidth
lambda Calc (guided wavelength) || frequency, and in PCB mode h, epsilon_r, trace W || lambda0, epsilon_eff, lambda/2, lambda/4
CPW || gap S, h, epsilon_r, frequency, tan delta, and Z0 or W || the other of Z0/W, epsilon_eff (no bottom ground), lambda_g, lambda_g/4, dielectric loss and Q
Diff Pair (differential pair) || h, epsilon_r, spacing S, frequency, and Zdiff or W || the other, plus Z0 even/odd (Kirschning-Jansen coupled lines), Zcommon, and the odd-mode epsilon_eff and lambda_g
Embedded MS || W, h, epsilon_r, cover height h2, cover epsilon_r, frequency, tan delta || Z0, epsilon_eff, lambda_g, lambda_g/4, dielectric loss and Q
lambda0 free (free-space wavelength) || frequency || lambda0, half, quarter
lambda/4 xfmr (quarter-wave transformer) || Z1, Z2, frequency, h, epsilon_r || Zt = sqrt(Z1 Z2), length lambda_g/4, epsilon_eff, the residual Gamma and return loss, and microstrip widths W1, Wt, W2
Smith chart || R, X, system Z0 || Gamma magnitude, phase, and real and imaginary parts, VSWR, return loss, mismatch loss

The patch calculator sizes a rectangular patch from the transmission-line model and reports the resistance at the radiating edge so a feed can be designed against it. Feed matching is left to the geometry: the inset or the feed line width is drawn in the editor and checked by the run.

## The expression strip

A single expression line is at the bottom of the panel. Type arithmetic (`+ - * / **` and parentheses, numbers only) and press Enter to evaluate. The two buttons take the square root and cube root of the current result, so sqrt(Z1*Z2) is entered as 50*75 then the root button. The square-root button is offered only while the result has a real root. A negative result disables it and the button says why. A rejected expression shows err.

## Step-by-step breakdown

Each calculator footer links to Step-by-step breakdown on RayRF, which opens the matching walkthrough at [rayrf.com/calculators](rf-calculators.html) for that structure with the formula, its derivation, and the validity range of the model.

## CLI equivalent

`rayrf calc <kind>` runs the same formulas headless and prints a table, or `--json` for a machine-readable report. The kinds are `microstrip`, `stripline`, `cpw`, `gcpw`, `diff`, `embedded`, `patch`, `lambda`, `lambda-diel`, `qwt`, and `smith`. Inputs are given in GHz, mm, and ohms. Omit `--w` and pass `--z0` to solve a width from a target impedance. `--tand` is optional wherever a calculator reports dielectric loss, and the loss rows appear only when it is given. Stripline's `--t` is the copper thickness and defaults to 0.035 mm, and 0 is a strip of no thickness. `smith` uses 50 ohm when `--z0` is omitted. The CLI reports the same quantities as the overlay.

```
rayrf calc patch --er 4.4 --h 1.6 --freq 2.45
rayrf calc microstrip --freq 2.45 --er 4.4 --h 1.6 --tand 0.02 --z0 50
rayrf calc qwt --z1 50 --z2 75 --freq 2.4 --er 3.66 --h 0.5
```

The full flag list for `calc` is on the [CLI reference](docs-cli-reference.html).
