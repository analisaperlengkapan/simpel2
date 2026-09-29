#!/usr/bin/env python3
"""Print the CSP `script-src` hash sources for a page's inline scripts.

Why: `script-src 'unsafe-inline'` lets ANY injected `<script>` run, which defeats
most of what a Content-Security-Policy is for. The frontends need it only because
Trunk boots the WASM from an INLINE `<script type="module">` (plus two small
hand-written inline scripts in `index.html`). CSP3 lets a policy allow an exact
script by its hash instead — `'sha256-<base64 of the SHA-256 of the script text>'`
— so an injected script, having a different hash, is refused.

The hash must be computed over the **built** `dist/index.html`: the Trunk script
carries content-hashed filenames, so it changes on every build. Docker therefore
runs this after `trunk build` and ships the result next to the page; the
entrypoint turns it into an nginx include when `CSP_HASH_MODE=true`.

    csp-script-hashes.py dist/index.html        # -> 'sha256-…' 'sha256-…'
    csp-script-hashes.py --self-test

Exit 2 when the page has no inline script at all: a real Trunk build always has
one, and an empty list written into `script-src` would refuse the app's own boot
script — an empty result must fail the build, not ship a page that cannot start.

Standard library only (the build image has python3; the nginx image does not, which
is why this runs at build time and the entrypoint only reads its output).
"""

from __future__ import annotations

import base64
import hashlib
import re
import sys
from pathlib import Path

# `<script …>body</script>` — inline when it has no `src`. Non-greedy body, so two
# scripts are two matches. Tags nested in comments are not a concern for the pages
# this runs on (Trunk output + our own index.html); the self-test pins the cases.
SCRIPT_RE = re.compile(
    r"<script(?P<attrs>(?:\s[^>]*)?)>(?P<body>.*?)</script\s*>", re.DOTALL | re.IGNORECASE
)
SRC_RE = re.compile(r"(?:^|\s)src\s*=", re.IGNORECASE)
# Only JavaScript executes under `script-src`. A JSON data block does not.
DATA_TYPE_RE = re.compile(
    r"""\btype\s*=\s*["']?(?:application/(?:ld\+)?json|importmap|speculationrules)""",
    re.IGNORECASE,
)


def hash_source(text: str) -> str:
    digest = hashlib.sha256(text.encode("utf-8")).digest()
    return "'sha256-" + base64.b64encode(digest).decode("ascii") + "'"


def inline_script_hashes(html: str) -> list[str]:
    """Hash sources of every inline script, in document order, de-duplicated."""
    out: list[str] = []
    for m in SCRIPT_RE.finditer(html):
        attrs = m.group("attrs")
        if SRC_RE.search(attrs) or DATA_TYPE_RE.search(attrs):
            continue
        if not m.group("body").strip():
            continue
        source = hash_source(m.group("body"))
        if source not in out:
            out.append(source)
    return out


def self_test() -> int:
    failures: list[str] = []

    def expect(name: str, got: object, want: object) -> None:
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    # Reference vector: the CSP spec's own example style — hash of `alert('Hello, world.');`
    expect(
        "reference hash",
        hash_source("alert('Hello, world.');"),
        "'sha256-qznLcsROx4GACP2dm0UCKCzCG+HiZ1guq6ZZDob/Tng='",
    )

    page = """<head>
<script>var a = 1;</script>
<script type="module">import init from '/x-abc123.js'; await init();</script>
<script src="/app.js"></script>
<script type="application/json">{"k": 1}</script>
<SCRIPT>var a = 1;</SCRIPT>
<script>   </script>
</head>"""
    hashes = inline_script_hashes(page)
    expect("inline only, deduplicated, empty/JSON/src skipped", len(hashes), 2)
    expect("first is the plain script", hashes[0], hash_source("var a = 1;"))
    expect(
        "the hash covers the exact text, whitespace included",
        hash_source("import init from '/x-abc123.js'; await init();"),
        hashes[1],
    )
    expect("no scripts → empty", inline_script_hashes("<p>hi</p>"), [])

    if failures:
        print("SELF-TEST FAILED:\n  " + "\n  ".join(failures), file=sys.stderr)
        return 1
    print("SELF-TEST OK")
    return 0


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        return self_test()
    if len(argv) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    html = Path(argv[0]).read_text(encoding="utf-8")
    hashes = inline_script_hashes(html)
    if not hashes:
        print(
            f"FAIL: {argv[0]} has no inline <script>; a Trunk build always has one "
            "(the WASM boot). Refusing to emit an empty script-src.",
            file=sys.stderr,
        )
        return 2
    print(" ".join(hashes))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
