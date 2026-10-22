"""Wrap blog post fragments in the site shell and regenerate blog.html.

Each fragment in blog-src/<slug>.html starts with
  <!--META|date|read time|title|tag,tag,...-->
and may end with
  <!--CTA|heading|body-->
Output: blog-<slug>.html for each post, plus blog.html (the index).
Run:  python build_blog.py
"""
import html
import pathlib
import re
from datetime import datetime

ROOT = pathlib.Path(__file__).parent
SRC = ROOT / "blog-src"

# Index excerpts, newest first order is derived from the dates.
EXCERPTS = {
    "pmc-ground-plane-validation": "Machshev Labs' new PMC boundary and the mushroom EBG array from a classic IEEE EBG paper, cross-checked against openEMS on identical meshes. The EBG match dip agrees between the two solvers to 0.4 percent. Replication also uncovered and fixed a real via discretization bug. Full setup, mesh ladder, runtimes, and all project files.",
    "validated-against-vna-measurements": "I fabricated 4 RF PCBs on PTFE, measured 43 structures on a calibrated VNA, and simulated the same structures in Machshev Labs. Resonances and poles track the measurements within 1-2% at the higher mesh refinements. Method, results, and what is next.",
    "cst-alternative": "CST Studio Suite is a broad 3D EM suite leased at roughly 14,500 to 22,000 USD per year. For planar PCB RF, Machshev Labs is a GPU FDTD tool at 1,499 USD per year with live results. What you keep and what you give up.",
    "hfss-alternative-affordable": "For planar PCB RF, Machshev Labs is a GPU FDTD tool at 1,499 USD per year with a GUI and fast iteration, against published HFSS figures of 5,000 to 50,000 USD per year. What you gain and what you give up.",
    "machshev-vs-openems-benchmark": "A like-for-like FDTD benchmark: an 82.8M-cell 5.8 GHz patch, 1,000,000 timesteps, one workstation. Machshev Labs GPU ran 173.7x faster than openEMS (39,600 vs 228 MCell/s, 34m51s vs 4d5h). Numbers and method.",
    "patch-antenna-simulation-guide": "A practical walkthrough: estimate patch dimensions, build the stackup and feed, run a full-wave FDTD solve, read S11, and tune the match. Uses free calculators and Machshev Labs.",
    "openems-alternative": "openEMS is free and capable but needs scripting and runs on CPU. Machshev Labs is a GUI FDTD simulator with a GPU solver that ran a head-to-head patch case 173.7x faster, validated against 43 VNA-measured structures.",
    "what-is-fdtd": "FDTD, the finite-difference time-domain method, steps the electric and magnetic fields forward in time on a grid. A plain explanation of how it works, its trade-offs, and why GPUs suit it.",
}

HEAD = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} | Machshev Labs</title>
<meta name="description" content="{desc}">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&display=swap">
<link rel="stylesheet" href="assets/style.css">
</head>
<body data-page="blog">
<div id="site-header"></div>
<main id="main">
"""
FOOT = """</main>
<div id="site-footer"></div>
<script src="assets/site.js"></script>
</body>
</html>
"""


def tags_html(tags):
    return '<div class="tags">' + "".join(f"<span>{html.escape(t)}</span>" for t in tags) + "</div>"


def build():
    posts = []
    for path in SRC.glob("*.html"):
        slug = path.stem
        text = path.read_text(encoding="utf-8")
        meta = re.match(r"<!--META\|(.*?)\|(.*?)\|(.*?)\|(.*?)-->\s*", text)
        if not meta:
            raise SystemExit(f"{path.name}: missing META line")
        date, read, title, tags = meta.groups()
        tags = [t.strip() for t in tags.split(",")]
        body = text[meta.end():]
        cta = re.search(r"<!--CTA\|(.*?)\|(.*?)-->\s*$", body)
        if cta:
            body = body[: cta.start()]
            body += (f'<div class="post-cta"><h3>{cta.group(1)}</h3><p>{cta.group(2)}</p>'
                     f'<a class="btn lg" href="pricing.html">Start the free trial</a></div>\n')
        page = HEAD.format(title=html.escape(title), desc=html.escape(EXCERPTS.get(slug, title))) + f"""  <article class="article">
    <div class="container narrow">
      <a class="back" href="blog.html">&larr; All posts</a>
      <div class="post-meta" style="margin-top:22px">{date} / {read}</div>
      <h1>{html.escape(title)}</h1>
      <div class="prose">
{body}      </div>
      {tags_html(tags)}
    </div>
  </article>
""" + FOOT
        (ROOT / f"blog-{slug}.html").write_text(page, encoding="utf-8")
        posts.append((datetime.strptime(date, "%B %d, %Y"), slug, date, read, title, tags))

    posts.sort(reverse=True)
    items = "\n".join(
        f"""      <a class="post-item reveal" href="blog-{slug}.html">
        <div class="post-meta">{date} / {read}</div>
        <h2>{html.escape(title)}</h2>
        <p>{html.escape(EXCERPTS.get(slug, ""))}</p>
        {tags_html(tags)}
      </a>"""
        for _, slug, date, read, title, tags in posts
    )
    index = HEAD.format(title="Blog", desc="Notes on RF simulation: tool comparisons, benchmarks, antenna how-tos, and validation.") + f"""  <section class="page-hero">
    <div class="container narrow">
      <div class="eyebrow">Blog</div>
      <h1 class="h1" style="margin-top:14px">Notes on RF simulation</h1>
      <p class="lead">Tool comparisons, a matched-mesh FDTD benchmark, antenna how-tos, and how Machshev Labs is checked against measured hardware. Written for working RF engineers.</p>
    </div>
  </section>
  <section style="padding-top:10px">
    <div class="container narrow post-list">
{items}
    </div>
  </section>
""" + FOOT
    (ROOT / "blog.html").write_text(index, encoding="utf-8")
    print(f"built {len(posts)} posts + blog.html")


if __name__ == "__main__":
    build()
