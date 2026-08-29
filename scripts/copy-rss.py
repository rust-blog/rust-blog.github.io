#!/usr/bin/env python3
import shutil
import sys

if len(sys.argv) != 3:
    print("usage: copy-rss <src> <dst>", file=sys.stderr)
    sys.exit(1)

src, dst = sys.argv[1], sys.argv[2]
shutil.copyfile(src, dst)
print(f"copied {src} -> {dst}")
