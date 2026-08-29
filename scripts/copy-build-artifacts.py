#!/usr/bin/env python3
"""Copy build-time artifacts (rss.xml, sitemap.xml, robots.txt) into dist/.

Usage: copy-build-artifacts <src> <dst> [<src> <dst> ...]
"""
import shutil
import sys

if len(sys.argv) < 5 or len(sys.argv) % 2 != 1:
    print(
        "usage: copy-build-artifacts <src> <dst> [<src> <dst> ...]",
        file=sys.stderr,
    )
    sys.exit(1)

pairs = list(zip(sys.argv[1::2], sys.argv[2::2]))
for src, dst in pairs:
    shutil.copyfile(src, dst)
    print(f"copied {src} -> {dst}")