# Install and first simulation
> Download RayRF Studio, sign in, and run the included 5.8 GHz patch antenna from the Welcome Panel. Follow the video or the written steps below.

## Download and install

RayRF Studio runs on Windows and Linux. The 3D field view needs a graphics driver with OpenGL 4.3 and compute shaders that reports free GPU memory. Sign in at [rayrf.com/dashboard/downloads](pricing.html) and download the build for your platform.

- **Windows.** Run `RayRF-Studio-<version>-Setup.exe` to install the app and add RayRF Studio to the Start menu.
- **Linux.** Download the `.AppImage`, make it executable (`chmod +x`), and run it. It needs no installer.

The desktop app has an automatic update checker that runs on startup to let you know when a new version is available.

## Sign in

On the first launch with no license saved, the sign-in dialog presents three options: sign in with browser (recommended, fastest), enter license key manually (get it from your dashboard), or continue as viewer-only. Viewer mode is always free, so you can view projects and results even after your trial or subscription expires.

Once signed in, the Welcome Panel is shown. You can open this menu at any time using `File` then `Welcome Panel`.

- **Blank Project** create an empty project with no layers, no geometry, and default settings.
- **Examples** open one of the included projects: the 5.8 GHz patch antenna, a 2 to 11 GHz Vivaldi antenna, a grounded coplanar line with a via fence, and a 2.4 GHz bandpass filter. Each opens with its S-parameter results, and none of them need more than a Hobby plan.
- **Recent projects** lists your recently opened `.rfsim` files.

The Welcome Panel can be closed with the Escape key or `x` in the top right. If you don't have a project open, a blank one will open, and if you already have a non-blank project open there is no effect.

## Activate a license

Activation assigns a machine to your account.

- **Sign in through RayRF** opens your browser to approve the sign-in. This requires logging into your RayRF account in this browser, which is convenient if it's your primary machine, but if it's a secondary/server machine you may want to use the license key method instead.
- **Enter license key** uses the key from your dashboard, copy it in and click Activate.

If sign-in or activation can't reach rayrf.com, the dialog says why (DNS, a timeout, a proxy, or the certificate, for example). If it keeps happening, click Copy details and send the details to [support@rayrf.com](mailto:support@rayrf.com). `rayrf auth --license-key` prints the same details.

The CLI uses the same license as the UI. `rayrf auth --license-key KEY` performs key-based activation, `rayrf auth --status` displays the current license state, and `rayrf auth --deactivate` releases the seat. Browser sign-in cannot be used with the CLI. For related information see the [CLI overview](docs-cli-overview.html).

## Patch antenna example

Select Patch Antenna from the Welcome Panel. The example is a 5.8 GHz inset-feed patch antenna on an FR-4 stackup with reasonable sim settings. Use the `Edit` tab to explore the stackup and the geometry.

Open the `Simulate` tab and click Run Simulation. RayRF will ask you to save the project before running.

When a simulation is running you can use Stop Early to conclude results early or discard the run. The Energy Ringdown plot indicates net energy remaining in the system. Auto mode targets an energy decay between -45 dB at Low quality and -60 dB at Very High, and a smaller target means a longer, more converged run. See [FDTD tips](docs-fdtd-tips.html). When the ringdown target is reached, timestepping stops and the title bar reads PREPARING RESULTS. The Simulate and result tabs show what is being processed until the results are ready. See [Running simulations](docs-running.html).

Open the `S-Parameters` tab to inspect return loss (S11 only in this example) with the resonant dip near 5.8 GHz. See [S-parameters](docs-s-parameters.html). Open the `Radiation Pattern` to inspect directionality in full 3D and 2D polar E/H slices. See [Radiation pattern](docs-radiation-pattern.html).

## Telemetry and updates

RayRF sends anonymous usage telemetry about which features you use. We never send any information about your designs, sim settings, results, or any text you input to the program. For full transparency you can view the exact raw telemetry data by going to `Settings` then `Anonymous Usage Telemetry` then `View raw events sent in this session` to see a json-format breakdown of the information sent. For example, the information includes time spent on each tab (40% on simulate, 30% on settings, 30% inspecting results, for example), simulation size (bucketed) such as <1M cells, 1M-10M cells, 10M-100M cells, >100M cells. It also includes hardware specifications, runtime, and, if the program crashed, the name of the module it crashed in, nothing more. Raw crash logs can carry a lot of identifying information about a user, and depending on the nature of the crash they can leak information about the project, so we never send crash logs, only the section of the program that crashed. For example "frontend_python", or "backend_cpu" or "backend_gpu". Anonymous usage telemetry is on by default everywhere except the EU and UK to comply with local regulations. You can turn it off at any time under `Settings`, and the setting takes effect immediately for the rest of the session.

The update checker runs at startup and offers Update now if a newer version is available. Update now does four things: Download the installer for the new version, verify its checksum, launch the new installer (you must click through the typical install steps), and close RayRF for you (if a project is open with unsaved changes or a running sim you will need to cancel or confirm closing). RayRF cannot be updated while it's running, so save your projects and finish simulation runs before updating. If you prefer to not install the update, use Remind me later or Skip this version. `Update <version> available` is shown in the title bar if the update was deferred, and you can click there to begin the update when convenient. `Settings` then Check for updates lets you check for a new version manually without restarting RayRF.
