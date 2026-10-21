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
    "free-online-rf-tools": "The PCB Toolkit and Antenna Studio now run in the browser: 50 PCB calculators and a NEC-2 antenna solver, compiled from their original Rust code to WebAssembly. No install, no account, and nothing leaves your machine.",
    "introducing-machshev": "Placeholder: an introduction to Machshev, what it does, and who it is for.",
}

HEAD = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} | Machshev</title>
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
                     f'<a class="btn lg" href="rf-calculators.html">Try the online tools</a></div>\n')
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
    index = HEAD.format(title="Blog", desc="News and notes on computational electromagnetics from Machshev Labs.") + f"""  <section class="page-hero">
    <div class="container narrow">
      <div class="eyebrow">Blog</div>
      <h1 class="h1" style="margin-top:14px">News and notes</h1>
      <p class="lead">Product news, the online tools, and notes on computational electromagnetics from Machshev Labs.</p>
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
