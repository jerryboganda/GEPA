#!/usr/bin/env python3
"""Assert the initial client payload stays under budget (M12 gate).

Spec (docs/11_MILESTONES_TASKS.md M12): "bundle < 400 kB gz initial".
"Initial" = what the browser must download before first meaningful
render of the start page: the HTML, the CSS it references, and every JS
asset referenced (directly or via astro-island component/renderer URLs)
by dist/index.html. Lazy/later-route chunks (admin, review, about) are
excluded — they are only fetched when the user navigates there.

Uses the same gzip level as a typical CDN (level 6) and enforces:
  - total initial gzipped weight < 400,000 bytes (spec budget)
  - the single largest initial JS chunk < 250,000 bytes gzipped
    (keeps one monolith island from silently eating the whole budget)

Usage: python scripts/check_bundle_size.py [dist_dir] [budget]
Exits 0 = within budget, 1 = over.
"""

from __future__ import annotations

import gzip
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIST = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "client" / "dist"
BUDGET = int(sys.argv[2]) if len(sys.argv) > 2 else 400_000
MAX_CHUNK = 250_000

ASSET_RE = re.compile(r'(?:src|href|component-url|renderer-url)="/(_astro/[^"]+)"')


def gz(b: bytes) -> int:
    return len(gzip.compress(b, 6))


def main() -> int:
    index = DIST / "index.html"
    if not index.exists():
        print(f"error: {index} not found", file=sys.stderr)
        return 1

    html_bytes = index.read_bytes()
    html_gz = gz(html_bytes)

    refs = sorted(set(ASSET_RE.findall(html_bytes.decode("utf-8", "replace"))))
    missing = [r for r in refs if not (DIST / r).exists()]
    if missing:
        print(f"error: index.html references missing assets: {missing}", file=sys.stderr)
        return 1

    rows = [(f"/{r}", gz((DIST / r).read_bytes())) for r in refs]
    total = html_gz + sum(size for _, size in rows)

    print(f"initial payload (gzip level 6):")
    print(f"  {'/index.html':>44} {html_gz:>8,} B")
    for name, size in rows:
        print(f"  {name:>44} {size:>8,} B")
    print(f"  {'TOTAL':>44} {total:>8,} B  (budget {BUDGET:,} B)")

    ok = True
    if total > BUDGET:
        print(f"FAIL: initial gzipped payload {total:,} B exceeds {BUDGET:,} B budget", file=sys.stderr)
        ok = False
    for name, size in rows:
        if size > MAX_CHUNK:
            print(f"FAIL: single chunk {name} is {size:,} B gz (> {MAX_CHUNK:,} B)", file=sys.stderr)
            ok = False
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
