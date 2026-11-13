#!/usr/bin/env python3
"""Fail when SIMPEL v1 names a file under public/assets that git does not track.

The simpelv1 image copies `monolith/simpelv1` from a checkout. A file that exists
only in a developer's working tree therefore does not exist in any image CI builds,
in staging or in production.

The incident (found 2026-11-13, on staging rc49): the root `.gitignore` carries
`**/*.js` for wasm-bindgen output. It also matched all 1458 scripts of v1's template:
jQuery, Bootstrap, ApexCharts, app.js, custom.js and the rest. Because the pattern has
no trailing slash it also matched directories named like `choices.js/`, which took
their stylesheets with them.

The v1 dashboard still rendered its server-side HTML, so every check that looked at
it passed, including the cross-app SSO e2e. Meanwhile each of the 224 views that call
jQuery was dead in the browser. Local images built from a working tree had the files,
which is why nobody saw it.

WHAT THIS CHECKS: every asset path v1 names is a tracked file under
`monolith/simpelv1/public/`. The names come from:
  * Blade views: `url('/assets/...')`, `asset('assets/...')` and literal
    `src="/assets/..."` / `href="/assets/..."`;
  * v1's own scripts in `public/assets/js/`, which write further `<script>` tags at
    run time (`${baseUrl}/assets/...` in plugins.js).
Blade comments (`{{-- --}}`) and HTML comments (`<!-- -->`) are removed first,
because a commented-out tag loads nothing.

WHAT IT DOES NOT CHECK: CDN URLs; paths a script assembles from variables; and the
relative image paths in the template's app.js (`"assets/images/flags/us.svg"`,
`"assets/images/svg/bell.svg"`). Those belong to a language switcher and a
notification placeholder that v1's layouts never render, and the files were not in
v1's own copy either. The browser-level check is
`antarmuka/portal/tests/e2e/v1-sso-flow.spec.ts`, which fails when the v1 dashboard
gets a 4xx for anything under `/assets/`.

Usage: check-simpelv1-assets-tracked.py [--self-test]
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
V1 = "monolith/simpelv1"
PUBLIC = f"{V1}/public/"

# Named by layout/auth.blade.php and layout/error.blade.php. v1 never shipped this
# path: the file is assets/js/lord-icon-2.1.0.js, which layout/main.blade.php names.
# No view uses <lord-icon>, so the 404 costs nothing. Correcting the path means
# editing two templates that the Blade Prettier gate cannot parse yet ("Unknown
# node type"). An entry here must still be named and still be missing, or the
# guard fails and asks for it to be removed.
KNOWN_MISSING = {
    "assets/js/pages/plugins/lord-icon-2.1.0.js": "never shipped; no view uses <lord-icon>",
}

BLADE_COMMENT = re.compile(r"\{\{--.*?--\}\}", re.S)
HTML_COMMENT = re.compile(r"<!--.*?-->", re.S)
# url('/assets/x') | asset("assets/x") | src="/assets/x" | href=/assets/x
VIEW_REF = re.compile(
    r"""(?:\b(?:url|asset)\(\s*['"]\s*/?|\b(?:src|href)\s*=\s*['"]?/?)(assets/[^'"\s>?#)]+)"""
)
# `${baseUrl}/assets/x`, the form plugins.js uses for the tags it writes. Anchoring on
# baseUrl also keeps `.../choices.js/public/assets/scripts/x.js` from being read as
# its own tail.
SCRIPT_REF = re.compile(r"""\$\{baseUrl\}/(assets/[^'"`\s)]+)""")


def strip_comments(text: str) -> str:
    return HTML_COMMENT.sub("", BLADE_COMMENT.sub("", text))


def names_in_view(text: str) -> set[str]:
    return {m.group(1) for m in VIEW_REF.finditer(strip_comments(text))}


def names_in_script(text: str) -> set[str]:
    return {m.group(1) for m in SCRIPT_REF.finditer(text)}


def findings(
    named: dict[str, set[str]], tracked: set[str], known: dict[str, str] = KNOWN_MISSING
) -> list[str]:
    """`named` maps an asset path to the files that name it."""
    out = []
    for path in sorted(named):
        if path in tracked or path in known:
            continue
        where = ", ".join(sorted(named[path]))
        out.append(f"{path} is named by {where} but is not tracked by git")
    for path, why in sorted(known.items()):
        if path in tracked:
            out.append(f"KNOWN_MISSING lists {path} ({why}), but it is tracked now: remove the entry")
        elif path not in named:
            out.append(f"KNOWN_MISSING lists {path} ({why}), but nothing names it now: remove the entry")
    return out


def git_files(*paths: str) -> list[str]:
    res = subprocess.run(
        ["git", "-C", str(REPO), "ls-files", "-z", "--", *paths],
        check=True,
        capture_output=True,
    )
    return [p for p in res.stdout.decode().split("\0") if p]


def collect() -> tuple[dict[str, set[str]], set[str]]:
    named: dict[str, set[str]] = {}

    def add(paths: set[str], source: str) -> None:
        for p in paths:
            named.setdefault(p, set()).add(source)

    for f in git_files(f"{V1}/resources/views"):
        if f.endswith(".blade.php"):
            add(names_in_view((REPO / f).read_text(encoding="utf-8", errors="replace")), f)
    for f in git_files(f"{PUBLIC}assets/js"):
        # v1's own scripts sit directly in assets/js; vendor libraries live in assets/libs.
        if f.endswith(".js") and "/" not in f[len(f"{PUBLIC}assets/js/"):]:
            add(names_in_script((REPO / f).read_text(encoding="utf-8", errors="replace")), f)
    tracked = {f[len(PUBLIC):] for f in git_files(PUBLIC)}
    return named, tracked


def self_test() -> int:
    view = """
      <script src="{{ url('/assets/js/app.js') }}"></script>
      <script src={{ url('/assets/libs/jquery/jquery-3.7.0.min.js') }}></script>
      <link href="{{ asset('assets/css/app.min.css') }}" rel="stylesheet">
      <img src="/assets/images/logo.png">
      {{-- <script src="{{ url('/assets/js/blade-commented.js') }}"></script> --}}
      <!-- <script src="{{ url('/assets/js/html-commented.js') }}"></script> -->
    """
    got = names_in_view(view)
    want = {
        "assets/js/app.js",
        "assets/libs/jquery/jquery-3.7.0.min.js",
        "assets/css/app.min.css",
        "assets/images/logo.png",
    }
    assert got == want, f"view names: {sorted(got)}"

    script = (
        "document.writeln(`<script src='${baseUrl}/assets/libs/choices.js/public/assets/scripts/choices.min.js'>`);"
        "document.writeln(`<script src='${baseUrl}/assets/libs/flatpickr/l10n/id.js'></script>`);"
        'var flag = "assets/images/flags/us.svg";'
    )
    got = names_in_script(script)
    want = {
        "assets/libs/choices.js/public/assets/scripts/choices.min.js",
        "assets/libs/flatpickr/l10n/id.js",
    }
    assert got == want, f"script names: {sorted(got)}"

    named = {"assets/js/app.js": {"layout/main.blade.php"}, "assets/css/app.min.css": {"x"}}
    both = {"assets/js/app.js", "assets/css/app.min.css"}
    assert findings(named, both, known={}) == [], "all tracked"
    bad = findings(named, {"assets/css/app.min.css"}, known={})
    assert len(bad) == 1 and bad[0].startswith("assets/js/app.js is named by layout/main"), bad

    known = {"assets/js/gone.js": "never shipped"}
    assert findings({"assets/js/gone.js": {"a"}}, set(), known) == [], "known missing tolerated"
    stale = findings({"assets/js/gone.js": {"a"}}, {"assets/js/gone.js"}, known)
    assert any("tracked now" in f for f in stale), stale
    assert any("nothing names it" in f for f in findings({}, set(), known)), "stale: unnamed"
    print("self-test ok")
    return 0


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    named, tracked = collect()
    if not named:
        print(f"::error::no asset names found under {V1}: the scan itself is broken")
        return 1
    problems = findings(named, tracked)
    for p in problems:
        print(f"::error::{p}")
    if problems:
        print(
            f"{len(problems)} problem(s). A file v1 names must be committed: the image is built "
            "from a checkout, so an untracked file does not exist in staging or production."
        )
        return 1
    print(f"ok: all {len(named)} asset path(s) SIMPEL v1 names are tracked "
          f"({len(KNOWN_MISSING)} known-missing, see KNOWN_MISSING)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
