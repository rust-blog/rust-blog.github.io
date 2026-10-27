#!/usr/bin/env sh
# Fail when the gzipped WASM bundle exceeds the enforced budget.
#
# The app ships pre-rendered HTML and no markdown renderer, so the budget
# guards against silently re-introducing heavy runtime dependencies (syntect,
# pulldown-cmark) or unstripped debug builds.
set -eu

budget_kib="${WASM_GZIP_BUDGET_KIB:-320}"
limit=$((budget_kib * 1024))

wasm=$(find dist -maxdepth 1 -name '*_bg.wasm' -print -quit)
if [ -z "$wasm" ]; then
  echo "error: no *_bg.wasm in dist/; run 'trunk build --release' first" >&2
  exit 1
fi

raw=$(wc -c < "$wasm" | tr -d ' ')
gzipped=$(gzip -9 -c "$wasm" | wc -c | tr -d ' ')

gzip_whole=$((gzipped / 1024))
gzip_tenth=$(((gzipped % 1024) * 10 / 1024))
printf 'wasm: %s bytes raw\n' "$raw"
printf 'wasm: %s bytes gzip (%s.%s KiB); budget %s KiB\n' \
  "$gzipped" "$gzip_whole" "$gzip_tenth" "$budget_kib"

if [ "$gzipped" -gt "$limit" ]; then
  echo "error: gzipped WASM exceeds the ${budget_kib} KiB budget" >&2
  exit 1
fi
