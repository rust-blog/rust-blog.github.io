#!/usr/bin/env python3
"""Regenerate the 1200x630 `og-image.png` social-share banner from source.

Single source of truth: the ASCII wordmark lives in `src/pages/home.rs` (the
hero banner) and the tagline in `src/site.rs`. This script reads both, then
renders the same cool-gray wordmark + tagline plus an orange brand footer on
the paper background, reproducing the look of the checked-in og-image without
the stale render's irregular letter spacing.

Dependencies: Pillow (`pip install pillow`). On macOS the script prefers
SF Mono (the site's `--font-mono`); it falls back to any monospace font found
in common system locations.

Usage:
    python3 scripts/render-og-image.py            # writes og-image.png
    python3 scripts/render-og-image.py --output /tmp/og.png
"""

from __future__ import annotations

import argparse
import glob
import os
import re
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent

# Palette from the original og-image (light theme).
BG = (249, 248, 246)      # --paper-ish off-white
INK = (96, 100, 106)      # cool gray wordmark / tagline
ORANGE = (249, 115, 22)   # --rust-400 footer

WIDTH, HEIGHT = 1200, 630

# Left-aligned layout matching the committed og-image. The Y values are the
# text-box top-left passed to draw.text(); each font's internal ascent shifts
# the first ink row ~5-8px down, so these are tuned so the ink starts at the
# same pixel rows as the original render (banner 146, tagline 348, footer 450).
BANNER_X, BANNER_Y = 90, 141
TAGLINE_X, TAGLINE_Y = 93, 341
FOOTER_X, FOOTER_Y = 91, 442


def banner_rows() -> list[str]:
    """Return the 6 rows of the wordmark from the README code block.

    The README banner is the canonical wordmark. It is the fenced code block
    directly under the `# rust-blog` title (first block in the file); we take
    its 6 lines verbatim.
    """
    src = (ROOT / "README.md").read_text(encoding="utf-8")
    fences = src.split("```")
    if len(fences) < 3:
        sys.exit("render-og-image: no fenced code block in README.md")
    rows = [line.rstrip("\r") for line in fences[1].split("\n") if line.strip()]
    if len(rows) < 6:
        sys.exit("render-og-image: README banner block has fewer than 6 rows")
    return rows[:6]


def site_constant(name: str) -> str:
    src = (ROOT / "src" / "site.rs").read_text(encoding="utf-8")
    m = re.search(rf'pub const {name}: &str = "([^"]*)";', src)
    if not m:
        sys.exit(f"render-og-image: {name} not found in site.rs")
    return m.group(1)


def find_font(candidates: list[str]) -> str:
    for c in candidates:
        if c and os.path.exists(c):
            return c
    return candidates[-1]


MONO_CANDIDATES = [
    "/System/Library/Fonts/SFNSMono.ttf",
    "/System/Library/Fonts/Menlo.ttc",
    "/System/Library/Fonts/Monaco.ttf",
    "/System/Library/Fonts/Courier.ttc",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
]

SANS_CANDIDATES = [
    "/System/Library/Fonts/Avenir.ttc",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/System/Library/Fonts/SFNS.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
]


def mono_font(size: float) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(find_font(MONO_CANDIDATES), size)


def sans_font(size: float) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(find_font(SANS_CANDIDATES), size)


def text_width(draw: ImageDraw.ImageDraw, text: str, font) -> int:
    bb = draw.textbbox((0, 0), text, font=font)
    return bb[2] - bb[0]


def draw_tracked(draw, xy, text, font, fill, tracking):
    """Draw text with per-character letter-spacing, emulating CSS tracking."""
    x, y = xy
    for ch in text:
        bb = draw.textbbox((x, y), ch, font=font)
        draw.text((x, y), ch, font=font, fill=fill)
        x = bb[2] + tracking


def render() -> Image.Image:
    rows = banner_rows()
    tagline = site_constant("TAGLINE")
    footer = "WRITTEN IN RUST · LEPTOS · WEBASSEMBLY"

    img = Image.new("RGB", (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(img)

    # --- wordmark ---
    banner_font = mono_font(22)  # cell ≈ 14 x 27 px; 68 cells ≈ 952 px wide
    row_h = banner_font.getmetrics()[0] + banner_font.getmetrics()[1]
    for i, row in enumerate(rows):
        y = BANNER_Y + i * row_h
        draw.text((BANNER_X, y), row, font=banner_font, fill=INK)

    # --- tagline (mono, matches hero-tagline font stack) ---
    tagline_font = mono_font(34)
    draw.text((TAGLINE_X, TAGLINE_Y), tagline, font=tagline_font, fill=INK)

    # --- orange footer ---
    footer_font = sans_font(26)
    draw_tracked(draw, (FOOTER_X, FOOTER_Y), footer, footer_font, ORANGE, 4.75)

    return img


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default=str(ROOT / "og-image.png"))
    args = parser.parse_args()

    img = render()
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)
    print(f"wrote {out} ({img.size[0]}x{img.size[1]})")


if __name__ == "__main__":
    main()