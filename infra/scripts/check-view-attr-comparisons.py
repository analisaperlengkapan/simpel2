#!/usr/bin/env python3
"""A comparison inside a Leptos `view!` attribute must be wrapped in braces.

`view!` tokenises `>` as the end of the opening tag. It does this inside an
attribute value too, so

    prop:disabled=move || page.get() >= total_pages

does not compile to what it reads as. The macro closes the tag at the `>`, and
`= total_pages` — plus every attribute after it — becomes a **text node**:

    prop:disabled=move || page.get()      <- takes an i32, truthy for page != 0
    >
        = total_pages                     <- rendered on the page, as text
        on:click=next_handler             <- never attached; the button is dead
        >
        "Selanjutnya"

so the button is simultaneously disabled, inert, and printing its own source
code at the user. Twenty-six attribute values across both frontends and
`lib/ui` were written this way, including `DarkPagination`; no paginated list
in either app could reach page two.

The fix is a brace group, which the tokeniser reads as one token tree:

    prop:disabled=move || { page.get() >= total_pages }

Why this survived everything, and why the FIRST version of this guard was wrong
--------------------------------------------------------------------------
`cargo check` sees valid markup and a valid text node. No test clicks page two.
A Playwright route sweep loads the page happily. The one true signal is rustc
reporting the callback as an unused variable — and that had been silenced two
different ways: a hand-written `#[allow(unused_variables)]` on two components
whose comment asserted the lint was mistaken, and an `_`-prefix rename on a
component prop (`_total_pages`) and two handlers (`_next_handler`).

This guard was first written to match the *formatting* left behind — a line
beginning with a bare `=`. That was a mistake worth recording. It matched only
sites leptosfmt had already reflowed, so `pemakaian_bmn_monitoring.rs`, still
on one line, was invisible to it; and after a "fix" that rejoined the `>=` onto
the attribute line without bracing it, the guard went green while the handler
was still detached. It tested the scar, not the wound.

The rule below tests the wound: an attribute value whose closure body carries a
bare `>` outside any bracket group.

`<` is deliberately NOT flagged. Only `>` closes a tag; a `<` in attribute-value
position is ordinary Rust. Measured, not assumed: `mfa_backup_input` has
`disabled=move || ... .len() < 8` immediately above an `on:click=move |_|
on_submit()`, and `on_submit` was never reported unused — the handler is
attached, so the `<` did not end the tag. The same holds for the thirteen
`page.get() <= 1` predicates on every working "Sebelumnya" button.

Exit 0 clean, 1 on findings. Stdlib only — the ARC runner image has no PyYAML.
"""

import re
import sys
from pathlib import Path

ROOTS = ["antarmuka", "lib/ui"]
SKIP_DIRS = {"target", "node_modules", "dist", ".git"}

# `something=move |…| BODY` as an attribute in a view! tag.
ATTR_CLOSURE = re.compile(
    r"^\s*(?P<attr>(?:prop:|on:|attr:)?[A-Za-z_][\w:-]*)=move \|_?\|\s*(?P<body>.+)$"
)

STRING = re.compile(r'"(?:\\.|[^"\\])*"')


def strip_groups(text: str) -> str:
    """Remove string literals and every balanced bracket group."""
    text = STRING.sub("", text)
    out, depth = [], 0
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(ch)
    return "".join(out)


def scan(path: Path) -> list[tuple[int, str]]:
    findings = []
    for i, raw in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines()):
        stripped = raw.strip()
        if stripped.startswith("//"):
            continue
        m = ATTR_CLOSURE.match(raw)
        if not m:
            continue
        bare = strip_groups(m.group("body"))
        # `=>` and `->` are not comparisons; neither is a lone `-`.
        bare = bare.replace("=>", "").replace("->", "")
        if ">" in bare:
            findings.append((i + 1, stripped))
    return findings


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    total = 0
    scanned = 0
    for root in ROOTS:
        base = repo / root
        if not base.is_dir():
            print(f"::error::guard scope {root} does not exist", file=sys.stderr)
            return 1
        for path in sorted(base.rglob("*.rs")):
            if any(p in SKIP_DIRS for p in path.parts):
                continue
            scanned += 1
            for line_no, text in scan(path):
                print(f"{path.relative_to(repo)}:{line_no}\n    {text}")
                total += 1

    if not scanned:
        # "Found nothing" must never read as "all clean".
        print("check-view-attr-comparisons: scanned ZERO files — the scope is "
              "broken, not the repo.", file=sys.stderr)
        return 1

    if total:
        print(
            f"\n{total} unbraced comparison(s) in a view! attribute. Each one ends "
            f"its tag early: the attributes after it become page text, and any "
            f"`on:` handler among them is never attached.\n"
            f"Wrap the closure body in braces:  =move || {{ a >= b }}",
            file=sys.stderr,
        )
        return 1

    print(f"check-view-attr-comparisons: {scanned} files, no unbraced comparison in a view! attribute")
    return 0


if __name__ == "__main__":
    sys.exit(main())
