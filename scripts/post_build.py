"""Trunk post_build hook: emit one static index.html per published post.

GitHub Pages (and Facebook's crawler) only serve files that physically exist,
so a history-mode SPA route like /post/<slug> would otherwise 404. We copy the
already-built dist/index.html (with its hashed JS/CSS) into
dist/post/<slug>/index.html and swap the static Open Graph / Twitter / <title>
tags for per-post values. Real browsers still boot the SPA; crawlers now get a
200 page with the right share card.
"""
import html
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
TPL = DIST / "index.html"
MANIFEST = ROOT / "posts-manifest.json"

tpl = TPL.read_text(encoding="utf-8")
posts = json.loads(MANIFEST.read_text(encoding="utf-8"))

_prop_re = {}


def set_meta(text: str, prop: str, content: str) -> str:
    rx = _prop_re.get(prop)
    if rx is None:
        rx = re.compile(
            r'(<meta\b[^>]*\b(?:property|name)="%s"[^>]*?\bcontent=")[^"]*(")'
            % re.escape(prop),
            re.IGNORECASE | re.DOTALL,
        )
        _prop_re[prop] = rx
    return rx.sub(
        lambda m: m.group(1) + html.escape(content, quote=True) + m.group(2), text
    )


for p in posts:
    slug = p["slug"]
    title = p["title"]
    desc = p["description"]
    url = f"https://rust-blog.github.io/post/{slug}"
    out = tpl
    out = re.sub(
        r"<title>.*?</title>",
        f"<title>{html.escape(title)} - rust-blog</title>",
        out,
        count=1,
        flags=re.IGNORECASE | re.DOTALL,
    )
    out = set_meta(out, "og:title", title)
    out = set_meta(out, "og:description", desc)
    out = set_meta(out, "og:type", "article")
    out = set_meta(out, "og:url", url)
    out = set_meta(out, "twitter:title", title)
    out = set_meta(out, "twitter:description", desc)
    # og:image / twitter:image keep the site banner from the template.
    d = DIST / "post" / slug
    d.mkdir(parents=True, exist_ok=True)
    (d / "index.html").write_text(out, encoding="utf-8")
    print("post_build: wrote", d / "index.html")
