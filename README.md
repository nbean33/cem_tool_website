# cem_tool_website

Static site for Machshev, computational electromagnetics software from Machshev Labs LLC,
served by GitHub Pages from the repository root. Plain HTML, CSS and JavaScript.

## Placeholders

Copy that is still to be written carries `class="ph"` (in the docs sources, a `!ph` line), which
draws it with a dashed amber outline and a "Placeholder" label. To list what is left:

```
grep -rn 'class="[^"]*ph' --include=*.html . ; grep -rn '^!ph' docs-src
```

The validation charts draw sample curves until `DUTS` in `assets/site.js` holds measured data
(set `SHOW_PLACEHOLDER_NOTE` to false then). The legal pages are outlines for counsel to complete.

## Generated pages

- `docs-*.html`, `assets/docs-nav.js` and `machshev-docs.md` come from `docs-src/*.md`:
  `python build_docs.py`.
- `blog.html` and `blog-*.html` come from `blog-src/*.html`: `python build_blog.py`.

## Online tools

Two tools run in the browser as WebAssembly, built from vendored copies of two Rust projects:

| Page | Source | What it is |
|---|---|---|
| `tools/pcb/` | [pcb_toolkit](https://github.com/nbean33/pcb_toolkit) | 50 PCB calculators (React UI from its desktop app) |
| `tools/antenna/` | [nec-rs](https://github.com/nbean33/nec-rs) | NEC Studio, a NEC-2 antenna modeler (Svelte UI from its desktop app) |

`tools/` is build output and is committed, so Pages serves it as plain files. The site's own
closed-form calculators (patch, inverted-F, quarter-wave, Smith chart, wavelength) are in
`assets/rf-calcs.js`.

### Layout

```
tools-src/
  rust/            cargo workspace: vendor/* (copied crates), pcb-wasm, nec-wasm
  pcb-app/         copied pcb_toolkit desktop frontend, with overlays applied
  nec-app/         copied NEC Studio frontend, with overlays applied
  overlays/        the web-specific files laid over the copied apps (ours)
  VENDOR.md        source commits, patches and overlays of the current copy
scripts/
  sync_vendor.py   copy from ../pcb_toolkit and ../nec-rs; --check reports drift
  build_tools.py   test, build both tools into tools/, regenerate assets/pcb-calcs.js
```

Everything under `tools-src/` except `overlays/`, `rust/Cargo.toml`, `rust/pcb-wasm/src/lib.rs`
and `rust/nec-wasm/src/lib.rs` is written by `sync_vendor.py`: change it in the source repo,
not here. The few unavoidable edits to vendored files are exact patches listed in
`scripts/sync_vendor.py` (`PATCHES`) and in `tools-src/VENDOR.md`.

### Updating the tools

With pcb_toolkit and nec-rs checked out next to this repository:

```
python scripts/sync_vendor.py --check   # what changed upstream
python scripts/sync_vendor.py           # re-copy
python scripts/build_tools.py           # test and rebuild tools/
```

Then commit `tools/`, `tools-src/` and `assets/pcb-calcs.js`.

Needs Rust with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`),
`wasm-pack` (`cargo install wasm-pack`) and Node.js.

## Previewing locally

The tools load WebAssembly, so open the site through a local server rather than from disk:

```
python -m http.server 8765
```

Then visit http://localhost:8765/.
