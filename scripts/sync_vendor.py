"""Vendor pcb_toolkit and nec-rs into tools-src/, or check the copy for drift.

Usage:
  python scripts/sync_vendor.py                 # copy from ../pcb_toolkit and ../nec-rs
  python scripts/sync_vendor.py --check         # report drift, change nothing
  python scripts/sync_vendor.py --pcb PATH --nec PATH

What it does:
  1. Copies the library crates and the desktop app frontends byte for byte
     (build output, node_modules and the Tauri shells are left out).
  2. Generates tools-src/rust/pcb-wasm/src/dispatch.rs from the pcb_toolkit
     Tauri backend, removing only the Tauri-specific parts.
  3. Lays the web adapters in tools-src/overlays/<app>/ over the copied apps.
     Overlays are ours; everything else under tools-src/ that this script
     writes is upstream code and should not be edited by hand.
  4. Records the source commits and a hash of every upstream file in
     tools-src/vendor-manifest.json, and rewrites tools-src/VENDOR.md.

--check compares the source repos' current files against the manifest and
the vendored copies, and exits 1 if anything differs.
"""
import argparse
import datetime
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TOOLS = ROOT / "tools-src"
MANIFEST = TOOLS / "vendor-manifest.json"

# (source key, path in source repo, destination under the website repo)
COPIES = [
    ("pcb", "crates/pcb-toolkit", "tools-src/rust/vendor/pcb-toolkit"),
    ("nec", "crates/nec-core", "tools-src/rust/vendor/nec-core"),
    ("nec", "crates/nec-cards", "tools-src/rust/vendor/nec-cards"),
    ("nec", "crates/nec-geom", "tools-src/rust/vendor/nec-geom"),
    ("nec", "crates/nec-kernel", "tools-src/rust/vendor/nec-kernel"),
    ("nec", "crates/nec-solve", "tools-src/rust/vendor/nec-solve"),
    ("nec", "crates/nec-engine", "tools-src/rust/vendor/nec-engine"),
    ("pcb", "apps/desktop", "tools-src/pcb-app"),
    ("nec", "app", "tools-src/nec-app"),
]
EXCLUDE_DIRS = {"target", "node_modules", "dist", "src-tauri", "gen", ".vite", ".svelte-kit"}

# Overlay directory -> the copied tree it is laid over.
OVERLAYS = {
    "tools-src/overlays/pcb-app": "tools-src/pcb-app",
    "tools-src/overlays/nec-app": "tools-src/nec-app",
}

# Vendored from upstream (so dependency versions match the tested desktop apps) but rewritten by
# `npm install`, which prunes the Tauri packages the package.json patches remove. The drift
# check does not count those rewrites as local edits; the next sync restores the upstream file.
BUILD_MANAGED = {
    "tools-src/pcb-app/package-lock.json",
    "tools-src/nec-app/package-lock.json",
}

DISPATCH_SRC = ("pcb", "apps/desktop/src-tauri/src/lib.rs")
DISPATCH_DEST = "tools-src/rust/pcb-wasm/src/dispatch.rs"
ENGINE_SRC = ("nec", "app/src-tauri/src/engine.rs")
ENGINE_DEST = "tools-src/rust/nec-wasm/src/engine.rs"

# Calls that compile for wasm32-unknown-unknown but fail or panic when run in a browser.
WASM_HOSTILE = ("std::fs::", "std::thread::", "std::process::", "std::time::")

# Small edits to vendored files that the browser build cannot do without. Each is an exact
# (old, new) replacement; if `old` is no longer in the upstream file the sync stops, so an
# upstream change is never patched blindly. Keep this list short and say why for each.
FAER_LINE = "faer = { workspace = true }\n"
PATCHES = {
    "tools-src/rust/vendor/nec-solve/src/linalg.rs": (
        "faer's thread pool (spindle -> atomic-wait) does not build for wasm32, so the browser "
        "build factors on one thread. The native function is unchanged.",
        [(
            "fn parallelism(n: usize) -> Par {\n",
            "#[cfg(target_arch = \"wasm32\")]\n"
            "fn parallelism(_n: usize) -> Par {\n"
            "    Par::Seq // cem_tool_website: single-threaded in the browser\n"
            "}\n\n"
            "#[cfg(not(target_arch = \"wasm32\"))]\n"
            "fn parallelism(n: usize) -> Par {\n",
        )],
    ),
    "tools-src/rust/vendor/nec-solve/Cargo.toml": (
        "Turns on faer's `rayon` feature only off wasm32 (the workspace declares faer without it).",
        [
            (FAER_LINE, ""),
            ("[lints]\n",
             "# cem_tool_website: faer's thread pool only where threads exist.\n"
             "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]\n"
             "faer = { workspace = true, features = [\"rayon\"] }\n\n"
             "[target.'cfg(target_arch = \"wasm32\")'.dependencies]\n"
             "faer = { workspace = true }\n\n"
             "[lints]\n"),
        ],
    ),
    # ---- PCB Toolkit frontend --------------------------------------------------------
    "tools-src/pcb-app/package.json": (
        "Drops the Tauri packages; the website build has no Tauri runtime.",
        [
            ('    "preview": "vite preview",\n    "tauri": "tauri"\n', '    "preview": "vite preview"\n'),
            ('    "@tauri-apps/api": "^2.9.0",\n', ""),
            ('    "@tauri-apps/cli": "^2.9.0",\n', ""),
        ],
    ),
    "tools-src/pcb-app/tsconfig.json": (
        "Points the `@tauri-apps/api/core` import at src/backend.ts for type checking, matching "
        "the Vite alias.",
        [('    "noEmit": true\n',
          '    "noEmit": true,\n    "paths": { "@tauri-apps/api/core": ["./src/backend.ts"] }\n')],
    ),
    "tools-src/pcb-app/src/main.tsx": (
        "Loads the site palette after the app's own stylesheet.",
        [('import "./styles/app.css";\n', 'import "./styles/app.css";\nimport "./styles/site-theme.css";\n')],
    ),
    "tools-src/pcb-app/src/App.tsx": (
        "Deep links: the URL hash names the open calculator (tools/pcb/#impedance.microstrip).",
        [
            ('import { Results } from "./components/Results";\n',
             'import { Results } from "./components/Results";\nimport { idFromHash } from "./deeplink";\n'),
            ("useState(CALCULATORS[0].id);", "useState(() => idFromHash() ?? CALCULATORS[0].id);"),
            ("  const selectCalculator = (id: string) => {\n    setActiveId(id);\n",
             "  const selectCalculator = (id: string) => {\n    setActiveId(id);\n"
             '    if (idFromHash() !== id) history.replaceState(null, "", "#" + id);\n'),
            ("  // One-time reference data for the dropdowns.\n",
             "  // Follow a #calculator link that changes while the page is open.\n"
             "  useEffect(() => {\n"
             "    const onHash = () => {\n"
             "      const id = idFromHash();\n"
             "      if (id) selectCalculator(id);\n"
             "    };\n"
             '    window.addEventListener("hashchange", onHash);\n'
             '    return () => window.removeEventListener("hashchange", onHash);\n'
             "  }, []);\n\n"
             "  // One-time reference data for the dropdowns.\n"),
        ],
    ),
    # ---- NEC Studio frontend -----------------------------------------------------------
    "tools-src/nec-app/package.json": (
        "Drops the Tauri packages; api.ts and the dialog alias replace them.",
        [
            ('    "@tauri-apps/api": "^2.9.0",\n', ""),
            ('    "@tauri-apps/plugin-dialog": "^2.4.0",\n', ""),
        ],
    ),
    "tools-src/nec-app/src/main.ts": (
        "Loads the site palette after the app's own stylesheet.",
        [('import "./app.css";\n', 'import "./app.css";\nimport "./site-theme.css";\n')],
    ),
}


def apply_patch(dest: str, text: str) -> str:
    _, edits = PATCHES[dest]
    for old, new in edits:
        if old not in text:
            raise SystemExit(f"patch for {dest} no longer applies: upstream changed {old.strip()[:60]!r}. "
                             "Update PATCHES in scripts/sync_vendor.py.")
        text = text.replace(old, new, 1)
    return text


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(repo: pathlib.Path, *args: str) -> str:
    try:
        return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def walk(src: pathlib.Path):
    """Every file under src, skipping excluded directories, as paths relative to src."""
    for p in sorted(src.rglob("*")):
        rel = p.relative_to(src)
        if any(part in EXCLUDE_DIRS for part in rel.parts[:-1]) or (p.is_dir() and p.name in EXCLUDE_DIRS):
            continue
        if p.is_file():
            yield rel


def overlay_files() -> dict:
    """{destination path: overlay file} for every overlay file."""
    out = {}
    for ov, dest in OVERLAYS.items():
        base = ROOT / ov
        if base.is_dir():
            for p in sorted(base.rglob("*")):
                if p.is_file():
                    out[(pathlib.PurePosixPath(dest) / p.relative_to(base).as_posix()).as_posix()] = p
    return out


def transform_dispatch(text: str) -> str:
    """The Tauri backend with its Tauri-specific parts removed.

    Removes the `#[tauri::command]` attributes and the `run()` entry point (with the doc
    comment and attributes directly above it), makes the three commands public, and points the
    schema `include_str!` at the vendored frontend. Everything else is kept verbatim, tests
    included, so the dispatch the website runs is the one the desktop app tests.
    """
    lines = text.splitlines(keepends=True)
    start = next(i for i, l in enumerate(lines) if l.startswith("pub fn run()"))
    top = start
    while top > 0 and lines[top - 1].lstrip().startswith(("///", "#[")):
        top -= 1
    end = next(i for i in range(start, len(lines)) if lines[i].rstrip() == "}")
    lines = lines[:top] + lines[end + 1:]
    lines = [l for l in lines if l.strip() != "#[tauri::command]"]
    out = "".join(lines)
    for name in ("fn calculate(", "fn list_materials(", "fn list_awg(", "struct MaterialEntry"):
        if f"\npub {name}" not in out:
            out = out.replace(f"\n{name}", f"\npub {name}", 1)
    out = out.replace('include_str!("../../src/schema/calculators.ts")',
                      'include_str!("../../../pcb-app/src/schema/calculators.ts")')
    if "tauri" in out.replace("Tauri", ""):
        leftover = [l.strip() for l in out.splitlines() if "tauri" in l and not l.lstrip().startswith("//")]
        if leftover:
            raise SystemExit(f"dispatch transform left Tauri code behind: {leftover[:3]}")
    header = ("// GENERATED by scripts/sync_vendor.py from pcb_toolkit/apps/desktop/src-tauri/src/lib.rs.\n"
              "// Do not edit: change the source and re-run the sync.\n")
    return header + out


def transform_engine(text: str) -> str:
    """NEC Studio's engine bridge, which is already free of Tauri types.

    The one change: `std::time::Instant` panics on wasm32-unknown-unknown, so it becomes
    `web_time::Instant`, which is `performance.now()` in the browser and `std::time` natively.
    """
    out = text.replace("std::time::Instant", "web_time::Instant")
    hostile = [l.strip() for l in out.splitlines()
               if any(h in l for h in WASM_HOSTILE) and not l.lstrip().startswith("//")]
    if hostile:
        raise SystemExit(f"engine.rs gained a call that fails in the browser: {hostile[:3]}")
    header = ("// GENERATED by scripts/sync_vendor.py from nec-rs/app/src-tauri/src/engine.rs.\n"
              "// Do not edit: change the source and re-run the sync.\n")
    return header + out


GENERATED = {DISPATCH_DEST: (DISPATCH_SRC, transform_dispatch), ENGINE_DEST: (ENGINE_SRC, transform_engine)}


def plan(sources: dict):
    """Everything the sync would write: {dest rel path: (kind, source path or bytes)}."""
    files = {}
    for key, src_rel, dest in COPIES:
        src = sources[key] / src_rel
        if not src.is_dir():
            raise SystemExit(f"missing source directory: {src}")
        for rel in walk(src):
            files[f"{dest}/{rel.as_posix()}"] = ("copy", src / rel)
    for dest, ((key, rel), _) in GENERATED.items():
        files[dest] = ("generated", sources[key] / rel)
    return files


def sync(sources: dict):
    files = plan(sources)
    overlays = overlay_files()
    # Remove only what an earlier sync wrote and this one will not: never whole folders, so
    # node_modules and other untracked build state under the app copies survive a re-sync.
    if MANIFEST.exists():
        old = json.loads(MANIFEST.read_text(encoding="utf-8"))
        for dest in set(old.get("upstream", {})) | set(old.get("overlays", [])):
            if dest not in files and dest not in overlays:
                (ROOT / dest).unlink(missing_ok=True)
    hashes = {}
    for dest, (kind, src) in files.items():
        out = ROOT / dest
        out.parent.mkdir(parents=True, exist_ok=True)
        if kind == "copy" and dest in PATCHES:
            text = src.read_bytes().decode("utf-8").replace("\r\n", "\n")
            with open(out, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(apply_patch(dest, text))
        elif kind == "copy":
            shutil.copyfile(src, out)
        else:
            transform = GENERATED[dest][1]
            with open(out, "w", encoding="utf-8", newline="\n") as fh:  # Python 3.9 has no write_text(newline=)
                fh.write(transform(src.read_text(encoding="utf-8")))
        hashes[dest] = sha(src)
    for dest, src in overlays.items():
        out = ROOT / dest
        out.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, out)

    meta = {}
    for key, repo in sources.items():
        meta[key] = {
            "repo": git(repo, "remote", "get-url", "origin") or str(repo),
            "commit": git(repo, "rev-parse", "HEAD"),
            "dirty": bool(git(repo, "status", "--porcelain", "--untracked-files=no")),
        }
    manifest = {
        "synced": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "sources": meta,
        "upstream": hashes,
        "overlays": sorted(overlays),
    }
    MANIFEST.write_text(json.dumps(manifest, indent=1) + "\n", encoding="utf-8")
    write_vendor_md(manifest)
    print(f"vendored {len(files)} files, applied {len(overlays)} overlay files")
    for key, m in meta.items():
        print(f"  {key}: {m['commit'][:10]}{' (uncommitted changes in source!)' if m['dirty'] else ''}")


def write_vendor_md(manifest: dict):
    src = manifest["sources"]
    rows = "\n".join(f"| {k} | {v['repo']} | `{v['commit'][:12]}`{' + uncommitted changes' if v['dirty'] else ''} |"
                     for k, v in src.items())
    copies = "\n".join(f"- `{d}` from `{k}:{s}`" for k, s, d in COPIES)
    overlays = "\n".join(f"- `{o}`" for o in manifest["overlays"]) or "- (none)"
    patches = "\n".join(f"- `{d}`: {why}" for d, (why, _) in PATCHES.items())
    (TOOLS / "VENDOR.md").write_text(f"""# Vendored code

Generated by `scripts/sync_vendor.py` on {manifest['synced']}. Do not edit by hand.

| Source | Repository | Commit |
|---|---|---|
{rows}

## Copied verbatim

{copies}

Excluded from every copy: {', '.join(sorted(EXCLUDE_DIRS))}.

Build-managed: {', '.join(f'`{b}`' for b in sorted(BUILD_MANAGED))} are copied from upstream so
the dependency versions match the desktop apps, then rewritten by `npm install` (which prunes
the Tauri packages). The drift check ignores those rewrites; each sync restores upstream.

## Generated

- `{DISPATCH_DEST}` from `pcb:{DISPATCH_SRC[1]}`: the Tauri backend with the
  `#[tauri::command]` attributes and `run()` removed, the three commands made `pub`, and the
  schema `include_str!` pointed at `tools-src/pcb-app`. Nothing else changes.
- `{ENGINE_DEST}` from `nec:{ENGINE_SRC[1]}`: verbatim except `std::time::Instant` becomes
  `web_time::Instant` (the std one panics in the browser).

## Patched vendored files

Exact replacements, defined in `PATCHES` in `scripts/sync_vendor.py`. A sync stops if the
upstream text a patch replaces has changed.

{patches}

## Local changes (overlays)

These files replace or add to the vendored apps. They live in `tools-src/overlays/` and are
re-applied on every sync. If `--check` reports that upstream changed one of them, review the
overlay against the new upstream file.

{overlays}

## Updating

```
python scripts/sync_vendor.py --check   # what changed upstream
python scripts/sync_vendor.py           # re-vendor
python scripts/build_tools.py           # rebuild tools/
```
""", encoding="utf-8")


def check(sources: dict) -> int:
    if not MANIFEST.exists():
        print("no vendor-manifest.json: run the sync first")
        return 1
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    recorded = manifest["upstream"]
    overlays = set(manifest.get("overlays", [])) | set(overlay_files())
    files = plan(sources)
    upstream_changed, local_edits, added, removed, overlay_review = [], [], [], [], []
    for dest, (kind, src) in files.items():
        if dest not in recorded:
            added.append(dest)
            continue
        if sha(src) != recorded[dest]:
            (overlay_review if dest in overlays else upstream_changed).append(dest)
    for dest in recorded:
        if dest not in files:
            removed.append(dest)
    for dest, (kind, src) in files.items():
        out = ROOT / dest
        if dest in overlays or dest in BUILD_MANAGED or kind != "copy" or dest not in recorded:
            continue
        if dest in PATCHES:
            expected = apply_patch(dest, src.read_bytes().decode("utf-8").replace("\r\n", "\n"))
            if not out.exists() or out.read_bytes().decode("utf-8").replace("\r\n", "\n") != expected:
                local_edits.append(dest)
        elif not out.exists() or sha(out) != recorded[dest]:
            local_edits.append(dest)
    for key, m in manifest["sources"].items():
        now = git(sources[key], "rev-parse", "HEAD")
        print(f"{key}: vendored {m['commit'][:10]}, source now {now[:10] or '?'}")
    sections = [
        ("changed upstream since the last sync", upstream_changed),
        ("new upstream files", added),
        ("removed upstream", removed),
        ("overlaid files whose upstream original changed (review the overlay)", overlay_review),
        ("vendored files edited locally (edit an overlay or the source instead)", local_edits),
    ]
    drift = False
    for title, items in sections:
        if items:
            drift = True
            print(f"\n{len(items)} {title}:")
            for i in items[:40]:
                print(f"  {i}")
            if len(items) > 40:
                print(f"  ... and {len(items) - 40} more")
    print("\nin sync" if not drift else "\ndrift found: re-run without --check to re-vendor")
    return 1 if drift else 0


def main():
    gh = ROOT.parent
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--pcb", type=pathlib.Path, default=gh / "pcb_toolkit")
    ap.add_argument("--nec", type=pathlib.Path, default=gh / "nec-rs")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    sources = {"pcb": args.pcb.resolve(), "nec": args.nec.resolve()}
    sys.exit(check(sources) if args.check else sync(sources) or 0)


if __name__ == "__main__":
    main()
