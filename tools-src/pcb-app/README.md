# PCB Toolkit — Desktop

A desktop GUI for the [`pcb-toolkit`](../../crates/pcb-toolkit) calculator library, in the
spirit of Saturn PCB Toolkit but with a modern interface. Built with
[Tauri 2](https://tauri.app), React and TypeScript.

Every calculator in the library is exposed, grouped into Impedance, Differential Pairs,
Current & Thermal, Via & Padstack, Signal Integrity, Ohm's Law, Crystal & PPM, and Reference.
The sidebar shows the live count, which is why this sentence does not — the hard-coded "37"
it used to carry was wrong within a day.

The calculator list is data, in [`src/schema/calculators.ts`](src/schema/calculators.ts):
fields, outputs, units and grouping all come from there, and the Rust side dispatches on the
`id`. `schema_ids_match_the_calculator_id_list` in `src-tauri/src/lib.rs` reads that file at
compile time and fails the build if the two drift apart.

## Running

```bash
cd apps/desktop
npm install
npm run tauri dev
```

> **Don't launch `target/debug/pcb-toolkit-desktop.exe` directly.** In a debug build Tauri
> loads the `devUrl` dev server rather than the embedded frontend, so the window opens on a
> "can't reach this page" error unless Vite is already running. Use `npm run tauri dev`,
> which starts both — or build in release mode, which embeds the bundle.

## Building installers

```bash
npm run tauri build
```

Artifacts land in `../../target/release/bundle/` — MSI and NSIS on Windows, `.deb`/AppImage
on Linux, `.dmg` on macOS.

### Prerequisites

Rust plus the platform WebView. Windows needs the WebView2 runtime, present by default on
Windows 11. macOS needs nothing extra. On Debian/Ubuntu:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  patchelf
```

Note that `tauri-build` needs the frontend bundle to exist at the configured `frontendDist`
before the Rust crate will compile, so run `npm ci && npm run build` before
`cargo build -p pcb-toolkit-desktop`.

## How it is wired

The GUI is **schema-driven**. Adding or changing a calculator is a data change in two
places rather than new components:

| File | Role |
| ---- | ---- |
| [`src/schema/calculators.ts`](src/schema/calculators.ts) | One entry per calculator: fields, units, defaults, outputs |
| [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs) | One `match` arm per calculator id, calling the library |

The frontend sends `{ id, params }` to a single `calculate` command and renders whatever
comes back, so there is no per-calculator UI code. A calculator's `id` in the schema must
match its dispatch arm, and each field `key` must match the parameter name that arm reads.
A Rust test (`every_calculator_dispatches_and_returns_its_primary_output`) exercises every
arm and asserts each returns the field the UI reads, and
`schema_ids_match_the_calculator_id_list` reads `calculators.ts` with `include_str!` and
compares its ids against the backend's. Drift between the two fails the test run.

Those tests cannot be *run* on Windows under the GNU toolchain — the crate fails to link
("export ordinal too large") — so run them on Linux or macOS, or with the MSVC toolchain.
`cargo clippy -p pcb-toolkit-desktop` does work everywhere, since clippy does not link.

Two further commands supply reference data: `list_materials` (the 45-entry substrate
database, used by the Er picker) and `list_awg` (wire gauge dropdown).

## Units

Numeric fields accept a unit suffix, exactly like the CLI — `0.254mm`, `1GHz`, `10nF`,
`1.5in`. A bare number is read as the field's canonical unit, shown greyed inside the
input. Conversion happens in [`src/units.ts`](src/units.ts) before the value is sent, so
the backend always receives canonical units (mils, Hz, Farads, Henries).

## Accuracy caveats

The GUI presents every calculator identically. Some are less well validated than others —
notably broadside-coupled differential, crosstalk, the embedded-microstrip Er_eff blend,
and the IPC-2152 modifier approximations. See [`VALIDATION.md`](../../VALIDATION.md) at the
repository root for the per-module confidence assessment.
