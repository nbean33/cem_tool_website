"""Build the docs pages from docs-src/<slug>.md.

Source format (a small Markdown subset):
  line 1        # Page title
  line 2        > Lede paragraph (optional)
  ## / ###      headings (h2 get anchor ids)
  - item        bullet list (indent 2 spaces to continue an item)
  1. item       numbered list
  a || b || c   table row; the first row of a block is the header
  !caption ...  caption for the table directly above
  !note ...     callout box
  !fig ...      screenshot placeholder with this caption
  ```           fenced code block
  inline        `code`, **bold**, [text](href)

Outputs docs-<slug>.html, assets/docs-nav.js (sidebar + search index),
and rayrf-docs.md (every page in one file).
Run:  python build_docs.py
"""
import html
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).parent
SRC = ROOT / "docs-src"

NAV = [
    ("Getting started", [
        ("install", "Install and first simulation"),
        ("interface", "The interface"),
    ]),
    ("Editor", [
        ("stackup-and-layers", "Stackup and materials"),
        ("geometry", "Drawing geometry"),
        ("parametric", "Variables, constraints, and generators"),
        ("import-png", "Importing artwork"),
        ("import-gerber", "Importing Gerber boards"),
        ("import-dxf", "Importing DXF drawings"),
        ("import-kicad", "Importing KiCad boards"),
        ("import-openems", "Importing openEMS and CSXCAD XML"),
        ("export-dxf", "Exporting DXF drawings"),
        ("export-openems", "Exporting openEMS XML"),
        ("ports-and-excitation", "Ports and excitation"),
    ]),
    ("Simulation", [
        ("auto-mode", "Auto mode and quality"),
        ("advanced-mode", "Advanced mode"),
        ("mesh-regions", "Mesh regions"),
        ("boundaries", "Boundary conditions"),
        ("running", "Running simulations"),
        ("benchmarking", "Benchmarking"),
    ]),
    ("Results", [
        ("s-parameters", "S-parameters and Smith chart"),
        ("radiation-pattern", "Radiation pattern"),
        ("field-viewer", "Field viewer"),
        ("run-history", "Run history"),
        ("sweeps", "Parameter sweeps"),
    ]),
    ("Remote running", [
        ("remote-overview", "Remote running"),
        ("runner-setup", "Runner setup"),
    ]),
    ("CLI and automation", [
        ("cli-overview", "CLI overview"),
        ("cli-agent-driving", "Driving RayRF with an agent"),
        ("cli-reference", "CLI reference"),
        ("cli-scripting", "Scripting and specs"),
    ]),
    ("Reference", [
        ("settings", "Settings reference"),
        ("calculators", "RF calculators"),
        ("keyboard-shortcuts", "Keyboard shortcuts"),
        ("file-formats", "File formats and outputs"),
        ("fdtd-tips", "FDTD tips"),
    ]),
]

HEAD = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} | RayRF Docs</title>
<meta name="description" content="{desc}">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;600&display=swap">
<link rel="stylesheet" href="assets/style.css">
</head>
<body data-page="docs-{slug}">
<div id="site-header"></div>
<main id="main" class="container docs">
  <aside class="docs-side" id="docs-side" aria-label="Documentation"></aside>
  <article class="docs-main">
"""
FOOT = """  </article>
</main>
<div id="site-footer"></div>
<script src="assets/docs-nav.js"></script>
<script src="assets/site.js"></script>
</body>
</html>
"""

ROW_SEP = " || "


def href(slug):
    return f"docs-{slug}.html"


def anchor(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


def inline(text):
    out = []
    for part in re.split(r"(`[^`]+`)", text):
        if len(part) > 1 and part.startswith("`") and part.endswith("`"):
            out.append("<code>" + html.escape(part[1:-1], quote=False) + "</code>")
            continue
        part = html.escape(part, quote=False)
        part = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", part)
        part = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", r'<a href="\2">\1</a>', part)
        out.append(part)
    return "".join(out)


def plain(text):
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)
    return text.replace("`", "").replace("**", "")


def render(body_lines):
    """Return (html, plain_text) for the body of one page."""
    out, text = [], []
    i, n = 0, len(body_lines)
    para = []

    def flush_para():
        if para:
            joined = " ".join(para)
            out.append(f"<p>{inline(joined)}</p>")
            text.append(plain(joined))
            para.clear()

    while i < n:
        line = body_lines[i].rstrip()
        if not line.strip():
            flush_para()
            i += 1
            continue
        if line.startswith("```"):
            flush_para()
            code = []
            i += 1
            while i < n and not body_lines[i].startswith("```"):
                code.append(body_lines[i].rstrip())
                i += 1
            i += 1
            out.append("<pre><code>" + html.escape("\n".join(code), quote=False) + "</code></pre>")
            text.append(" ".join(code))
            continue
        heading = re.match(r"^(#{2,4}) ", line)
        if heading:
            flush_para()
            level = len(heading.group(1))
            title = line[level + 1:]
            out.append(f'<h{level} id="{anchor(plain(title))}">{inline(title)}</h{level}>')
            text.append(plain(title))
            i += 1
            continue
        if line.startswith("!note "):
            flush_para()
            out.append(f'<p class="note">{inline(line[6:])}</p>')
            text.append(plain(line[6:]))
            i += 1
            continue
        if line.startswith("!fig "):
            flush_para()
            cap = inline(line[5:])
            out.append(f'<figure><div class="media ph"><span class="tag">screenshot</span></div><figcaption>{cap}</figcaption></figure>')
            i += 1
            continue
        if ROW_SEP in line:
            flush_para()
            rows = []
            while i < n and ROW_SEP in body_lines[i]:
                rows.append([c.strip() for c in body_lines[i].rstrip().split(ROW_SEP)])
                i += 1
            caption = ""
            if i < n and body_lines[i].startswith("!caption "):
                caption = f"<caption>{inline(body_lines[i][9:].strip())}</caption>"
                i += 1
            width = max(len(r) for r in rows)
            rows = [r + [""] * (width - len(r)) for r in rows]
            head = "".join(f"<th>{inline(c)}</th>" for c in rows[0])
            body = "".join("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>" for r in rows[1:])
            out.append(f'<div class="table-wrap"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody>{caption}</table></div>')
            text.extend(plain(" ".join(r)) for r in rows)
            continue
        bullet = re.match(r"^- ", line)
        number = re.match(r"^\d+\. ", line)
        if bullet or number:
            flush_para()
            tag = "ul" if bullet else "ol"
            pattern = r"^- " if bullet else r"^\d+\. "
            # A numbered list split by a code block keeps counting from its own first number.
            start = "" if bullet or number.group(0) == "1. " else f' start="{number.group(0)[:-2]}"'
            items = []
            while i < n:
                cur = body_lines[i].rstrip()
                if re.match(pattern, cur):
                    items.append(re.sub(pattern, "", cur, count=1))
                elif cur.startswith("  ") and items and cur.strip():
                    items[-1] += " " + cur.strip()
                else:
                    break
                i += 1
            out.append(f"<{tag}{start}>" + "".join(f"<li>{inline(it)}</li>" for it in items) + f"</{tag}>")
            text.extend(plain(it) for it in items)
            continue
        para.append(line.strip())
        i += 1
    flush_para()
    return "\n".join(out), " ".join(text)


def build():
    flat = [(sec, slug, title) for sec, items in NAV for slug, title in items]
    index, bundle = [], []
    for pos, (section, slug, nav_title) in enumerate(flat):
        path = SRC / f"{slug}.md"
        lines = path.read_text(encoding="utf-8").splitlines()
        if not lines or not lines[0].startswith("# "):
            raise SystemExit(f"{path.name}: first line must be '# Title'")
        title = lines[0][2:].strip()
        rest = lines[1:]
        lede = ""
        if rest and rest[0].startswith("> "):
            lede, rest = rest[0][2:].strip(), rest[1:]
        body, body_text = render(rest)

        prev_link = next_link = ""
        if pos > 0:
            p = flat[pos - 1]
            prev_link = f'<a class="prev" href="{href(p[1])}"><span>Previous</span>{html.escape(p[2])}</a>'
        if pos < len(flat) - 1:
            q = flat[pos + 1]
            next_link = f'<a class="next" href="{href(q[1])}"><span>Next</span>{html.escape(q[2])}</a>'

        page = HEAD.format(title=html.escape(title), desc=html.escape(plain(lede) or title), slug=slug)
        page += f'    <div class="post-meta">{html.escape(section)}</div>\n'
        page += f'    <h1 style="margin-top:8px">{inline(title)}</h1>\n'
        if lede:
            page += f'    <p class="lede">{inline(lede)}</p>\n'
        page += f'    <div class="prose">\n{body}\n    </div>\n'
        page += f'    <nav class="docs-pager" aria-label="Pager">{prev_link}{next_link}</nav>\n'
        page += FOOT
        (ROOT / href(slug)).write_text(page, encoding="utf-8")

        index.append({"t": title, "h": href(slug), "s": section,
                      "x": re.sub(r"\s+", " ", plain(lede) + " " + body_text).strip()})
        bundle.append("\n".join([lines[0], "", lede, ""] + rest).strip())

    nav_js = [{"section": sec, "items": [{"t": t, "h": href(s)} for s, t in items]} for sec, items in NAV]
    (ROOT / "assets" / "docs-nav.js").write_text(
        "// Generated by build_docs.py - do not edit by hand.\n"
        f"window.DOCS_NAV = {json.dumps(nav_js, ensure_ascii=False)};\n"
        f"window.DOCS_INDEX = {json.dumps(index, ensure_ascii=False)};\n",
        encoding="utf-8")
    (ROOT / "rayrf-docs.md").write_text(
        "# RayRF documentation\n\n" + "\n\n---\n\n".join(bundle) + "\n", encoding="utf-8")
    print(f"built {len(flat)} docs pages, assets/docs-nav.js, rayrf-docs.md")


if __name__ == "__main__":
    build()
