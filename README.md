# cem_tool_website

Static site for Machshev Labs LLC: upload a PCB design and find out which high-speed connections
will work before ordering the board. Served by GitHub Pages from the repository root. Plain
HTML, CSS and JavaScript.

## Pages

| Page | File |
|---|---|
| Home | `index.html` |
| How it works | `how-it-works.html` |
| Sample report | `sample-report.html` (the example board lives in `assets/report.js`) |
| Pricing, with the consulting section | `pricing.html` |
| Accuracy | `accuracy.html` (`validation.html` redirects here), plus `benchmarks.html` |
| Security and privacy | `security.html` |
| Free tools | `rf-calculators.html`, `rf-calc-*.html`, `tools/pcb/` |
| Brand kit | `brand.html` |
| App screen mockups | `app-upload.html`, `app-results.html`, `app-connection.html`, `app-account.html` |
| Docs and blog | generated, see below |
| Legal | `legal-terms.html`, `legal-privacy.html`, `legal-cookies.html` |

The app screens are static previews: nothing is uploaded, and buttons that would act on an
account say so. The sample report's board, connections and numbers are an illustrative example.

## Placeholders

Copy that is still to be written carries `class="ph"`, which draws it with a dashed amber
outline and a "Placeholder" label: prices, file retention, hosting and similar facts that only
the company can supply. To list them:

```
grep -rn 'class="[^"]*\bph\b' --include=*.html .
```

The Accuracy charts draw sample curves until `DUTS` in `assets/site.js` holds measured data
(set `SHOW_PLACEHOLDER_NOTE` to false then). The legal pages are outlines for counsel to complete.

## Generated pages

- `docs-*.html`, `assets/docs-nav.js` and `machshev-docs.md` come from `docs-src/*.md`:
  `python build_docs.py`.
- `blog.html` and `blog-*.html` come from `blog-src/*.html`: `python build_blog.py`.

## PCB Toolkit

`tools/pcb/` runs [pcb_toolkit](https://github.com/nbean33/pcb_toolkit)'s 50 calculators in the
browser as WebAssembly, with the React UI from its desktop app. `tools/` is build output and is
committed, so Pages serves it as plain files. The site's own closed-form calculators (patch,
inverted-F, quarter-wave, Smith chart, wavelength) are in `assets/rf-calcs.js`.

```
tools-src/
  rust/            cargo workspace: vendor/pcb-toolkit (copied crate), pcb-wasm
  pcb-app/         copied pcb_toolkit desktop frontend, with overlays applied
  overlays/        the web-specific files laid over the copied app (ours)
  VENDOR.md        source commit, patches and overlays of the current copy
scripts/
  sync_vendor.py   copy from ../pcb_toolkit; --check reports drift
  build_tools.py   test, build the toolkit into tools/pcb, regenerate assets/pcb-calcs.js
```

Everything under `tools-src/` except `overlays/`, `rust/Cargo.toml` and
`rust/pcb-wasm/src/lib.rs` is written by `sync_vendor.py`: change it in the source repo, not
here. The few unavoidable edits to vendored files are exact patches listed in
`scripts/sync_vendor.py` (`PATCHES`) and in `tools-src/VENDOR.md`.

With pcb_toolkit checked out next to this repository:

```
python scripts/sync_vendor.py --check   # what changed upstream
python scripts/sync_vendor.py           # re-copy
python scripts/build_tools.py           # test and rebuild tools/pcb
```

Then commit `tools/`, `tools-src/` and `assets/pcb-calcs.js`. Needs Rust with the
`wasm32-unknown-unknown` target, `wasm-pack` and Node.js.

## Previewing locally

The toolkit loads WebAssembly, so open the site through a local server rather than from disk:

```
python -m http.server 8765
```

Then visit http://localhost:8765/.
