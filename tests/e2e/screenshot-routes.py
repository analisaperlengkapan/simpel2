#!/usr/bin/env python3
"""Derive the full navigable route inventory for both frontends.

Why this exists separately from `route-coverage.mjs` and `_routes.py`:

`route-coverage.mjs` reads the `pub const NAME: &str = "/<base>/..."` consts out
of each FE's `routes.rs`. That is the right source for a *coverage gate* — it is
stable, typed, and a route that has no const is a route nobody declared. It is
the WRONG source for a *screenshot sweep*, and the difference is not academic:
those consts are curated. `routes.rs` omits the parameterised detail routes (it
exposes them as `url::kebutuhan_detail(id)` builders instead), so a sweep driven
by it silently photographs the list page and never the detail page.

`_routes.py` (perlengkapan) walks the actual router nesting for exactly this
reason. This script does the same, for BOTH frontends, and emits one JSON
document the capture script consumes:

    { "portal":       { "base": "/portal",              "routes": [...] },
      "perlengkapan": { "base": "/perlengkapan/simpel/v2","routes": [...] } }

Routes whose path contains a `:param` segment are emitted with the placeholder
intact and `"parameterised": true`; the capture script resolves those against
real ids fetched from the API, because a screenshot of `/daftar/:id` with a
literal `:id` is a screenshot of the 404 page.

Derivation, not a hand-written list — a page added tomorrow is swept tomorrow.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

PERLENGKAPAN_LIB = REPO / "antarmuka" / "perlengkapan" / "src" / "lib.rs"
PORTAL_APP = REPO / "antarmuka" / "portal" / "src" / "app.rs"
PORTAL_ROUTES = REPO / "antarmuka" / "portal" / "src" / "routes.rs"


def join(prefix: str, seg: str) -> str:
    """Join two path fragments, collapsing the seam."""
    if not seg:
        return prefix or "/"
    out = (prefix.rstrip("/") + "/" + seg.lstrip("/")).replace("//", "/")
    return out if out.startswith("/") else "/" + out


# ─────────────────────────────────────────────────────────────────────────────
# perlengkapan — `lib.rs` nests `<ParentRoute path=path!("/admin")>` around its
# children, so the child's real URL is parent + child. Concatenating the raw
# `path!()` literals turns seven admin pages into URLs the app does not serve;
# that mistake produced 24 screenshots of the 404 page (see `_routes.py`).
# ─────────────────────────────────────────────────────────────────────────────

OPEN_PARENT = re.compile(r"<ParentRoute\s+path=path!\(\"([^\"]*)\"\)")
ROUTE_OPEN = re.compile(r"<Route\b")
PATH_ATTR = re.compile(r'path=path!\(\s*"([^"]*)"\s*\)')
CLOSE_PARENT = re.compile(r"</ParentRoute>")

# A `path!(...)` macro may be written across lines:
#     <Route path=path!(
#         "/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi"
#     ) … />
# A line-by-line scan sees `path=path!(` on one line and the literal on the
# next, so the route is dropped — and a dropped route is a page that never gets
# photographed while the run reports full coverage. Collapse such macros to one
# line before parsing, across the WHOLE source, so the walk below sees the same
# shape regardless of formatting.
MULTILINE_MACRO = re.compile(r'path!\(\s*"([^"]*)"\s*\)', re.S)


def collapse_path_macros(src: str) -> str:
    return MULTILINE_MACRO.sub(lambda m: f'path!("{m.group(1)}")', src)


def perlengkapan_routes() -> list[str]:
    src = collapse_path_macros(PERLENGKAPAN_LIB.read_text())
    stack: list[str] = []
    routes: list[str] = []
    pending = False

    for line in src.splitlines():
        if CLOSE_PARENT.search(line):
            if stack:
                stack.pop()
            continue
        m = OPEN_PARENT.search(line)
        if m:
            stack.append(join(stack[-1] if stack else "", m.group(1)))
            continue
        if ROUTE_OPEN.search(line):
            pending = True
            inline = PATH_ATTR.search(line)
            if inline:
                routes.append(join(stack[-1] if stack else "/", inline.group(1)))
                pending = False
            continue
        if pending:
            m = PATH_ATTR.search(line)
            if m:
                routes.append(join(stack[-1] if stack else "/", m.group(1)))
                pending = False

    if not routes:
        raise SystemExit(
            "derived ZERO perlengkapan routes — the router shape changed, not the app"
        )
    return routes


# ─────────────────────────────────────────────────────────────────────────────
# portal — `app.rs` is the router. Its paths are written two ways:
#   `path=StaticSegment(routes::segment::DASHBOARD)`  (a const in routes.rs)
#   `path=path!("/mfa/setup")`                        (a literal)
# and children may be tuples: `path=(StaticSegment("users"), ParamSegment("id"))`.
# ─────────────────────────────────────────────────────────────────────────────

SEGMENT_CONST = re.compile(
    r'pub const (\w+)\s*:\s*&str\s*=\s*"([^"]+)"'
)


def portal_segment_consts() -> dict[str, str]:
    src = PORTAL_ROUTES.read_text()
    out = {}
    for m in SEGMENT_CONST.finditer(src):
        out[m.group(1)] = m.group(2)
    return out


PARENT_ROUTE = re.compile(r"<ParentRoute\b")
STATIC_SEG = re.compile(r'StaticSegment\(\s*(?:routes::segment::(\w+)|"([^"]*)")\s*\)')
PARAM_SEG = re.compile(r'ParamSegment\(\s*"([^"]*)"\s*\)')
PATH_MACRO = re.compile(r'path!\(\s*"([^"]*)"\s*\)')


def portal_routes() -> list[str]:
    src = collapse_path_macros(PORTAL_APP.read_text())
    consts = portal_segment_consts()
    # ParentRoute and Route may span lines; normalise by joining the whole file
    # into element-sized chunks split on `>` boundaries is fragile. Instead walk
    # line-by-line but accumulate while a tag is unterminated.
    lines = src.splitlines()
    stack: list[str] = []
    routes: list[str] = []
    buf = ""
    depth = 0

    for line in lines:
        buf += " " + line.strip()
        # A tag is complete once its angle bracket closes.
        if buf.count("<") > buf.count(">"):
            continue
        chunk, buf = buf, ""

        if CLOSE_PARENT.search(chunk):
            if stack:
                stack.pop()
            continue

        if PARENT_ROUTE.search(chunk):
            # Parent path may be a macro literal or a StaticSegment const.
            segs: list[str] = []
            mp = PATH_MACRO.search(chunk)
            if mp:
                segs.append(mp.group(1))
            for ms in STATIC_SEG.finditer(chunk):
                segs.append(consts.get(ms.group(1), ms.group(2) or "") if ms.group(1) else (ms.group(2) or ""))
            stack.append(join(stack[-1] if stack else "", "".join(segs)))
            continue

        if ROUTE_OPEN.search(chunk):
            # Prefer the macro form; else compose Static/Param segments in order.
            mp = PATH_MACRO.search(chunk)
            if mp:
                seg = mp.group(1)
            else:
                parts: list[tuple[int, str]] = []
                for ms in STATIC_SEG.finditer(chunk):
                    val = consts.get(ms.group(1), "") if ms.group(1) else (ms.group(2) or "")
                    parts.append((ms.start(), val))
                for mpar in PARAM_SEG.finditer(chunk):
                    parts.append((mpar.start(), ":" + mpar.group(1)))
                seg = "/".join(v for _, v in sorted(parts) if v)
            routes.append(join(stack[-1] if stack else "/", seg))
            continue

    if not routes:
        raise SystemExit("derived ZERO portal routes — the router shape changed, not the app")
    return routes


def dedupe(routes: list[str]) -> list[str]:
    seen, out = set(), []
    for r in routes:
        r = r or "/"
        if r not in seen:
            seen.add(r)
            out.append(r)
    return out


def main() -> int:
    doc = {
        "portal": {"base": "/portal", "routes": portal_routes()},
        "perlengkapan": {
            "base": "/perlengkapan/simpel/v2",
            "routes": perlengkapan_routes(),
        },
    }
    # Router paths are written RELATIVE to `<Router base=…>`; the browser URL is
    # base + path. A path already carrying the base is left alone (routes.rs
    # consts are absolute, lib.rs/app.rs literals are not) — joining twice would
    # yield `/perlengkapan/simpel/v2/perlengkapan/simpel/v2/dashboard`, a URL
    # whose 404 page screenshots fine and proves nothing.
    for fe, meta in doc.items():
        base = meta["base"]
        normalized = []
        for r in dedupe(meta["routes"]):
            full = r if r.startswith(base) else join(base, r)
            normalized.append(full)
        meta["routes"] = dedupe(normalized)
        for r in meta["routes"]:
            if not r.startswith(base):
                raise SystemExit(
                    f"{fe}: route {r!r} does not sit under base {base!r} — "
                    "the base or the derivation is wrong"
                )
    json.dump(doc, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())