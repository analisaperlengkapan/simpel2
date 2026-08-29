#!/usr/bin/env python3
"""Derive the sweep's route list from the router, honouring nesting.

The first version grepped `path!("…")` and concatenated nothing, so the seven
children of `<ParentRoute path=path!("/admin")>` came out as `/roles`,
`/audit`, `/master`, `/templates`, `/workflow`, `/workflow-monitoring` and
`/workflow-delegation`. None of those is a URL this app serves: the sweep spent
24 of its 99 screenshots photographing the 404 page and reported them as
"halaman-nyaris-kosong" — a content observation about a page that does not
exist.

Reading the nesting is the whole point of deriving the list. A hand-written
list would at least have been written from the URLs people actually visit.
"""

import re
import sys
from pathlib import Path

LIB = Path(__file__).resolve().parents[2] / "src" / "lib.rs"
OPEN_PARENT = re.compile(r'<ParentRoute\s+path=path!\("([^"]*)"\)')
LEAF = re.compile(r'<Route\s+(?:[^>]*?\s)?path=path!\("([^"]*)"\)', re.S)
CLOSE_PARENT = re.compile(r"</ParentRoute>")
# `path=path!(…)` may sit on its own line inside a multi-line <Route …>.
ROUTE_OPEN = re.compile(r"<Route\b")
PATH_ATTR = re.compile(r'path=path!\("([^"]*)"\)')


def join(prefix: str, seg: str) -> str:
    out = (prefix.rstrip("/") + "/" + seg.lstrip("/")).replace("//", "/")
    return out if out.startswith("/") else "/" + out


def main() -> int:
    src = LIB.read_text()
    stack: list[str] = []
    routes: list[str] = []
    pending_route = False

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
            pending_route = True
            inline = PATH_ATTR.search(line)
            if inline:
                routes.append(join(stack[-1] if stack else "/", inline.group(1)))
                pending_route = False
            continue
        if pending_route:
            m = PATH_ATTR.search(line)
            if m:
                routes.append(join(stack[-1] if stack else "/", m.group(1)))
                pending_route = False

    if not routes:
        print("derived ZERO routes — the router shape changed, not the app",
              file=sys.stderr)
        return 1

    seen, out = set(), []
    for r in routes:
        if r not in seen:
            seen.add(r)
            out.append(r)
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
