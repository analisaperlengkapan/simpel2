#!/usr/bin/env python3
"""`antarmuka/perlengkapan` renders on navy. Light-palette classes are bugs there.

The app's surface is `--surface-base: #0f172a`. A screen written against
Tailwind's light defaults — `bg-white`, `text-gray-700`, `border-gray-200` —
does not merely look inconsistent on it. Measured on staging, 2026-08-29:

  * `/admin/roles` drew four solid white cards on the navy page, and its `<h1>`
    was `text-gray-900` on that navy: a near-black heading no one could read.
  * `/admin/templates` did the same, and its template names had no colour class
    at all, so they inherited the app's light body colour onto a white panel
    and vanished.
  * `/pengelolaan/pemakaian/buat` and `/pengelolaan/penghapusan/buat` are white
    forms whose `<select>` elements carried NEITHER a background NOR a text
    colour — the browser default background with an inherited light text is an
    empty-looking control that in fact has a value selected.

None of that fails a build. `notifikasi/inbox.rs` records the same failure from
an earlier round, and its comment did not stop the next four.

Scope
-----
`antarmuka/perlengkapan` only. `antarmuka/portal` and `lib/ui` still carry
light-palette screens; converting them is separate work, and a guard that
fails on known-unconverted code teaches people to ignore it.

Deliberate exceptions carry their reason here, not a blanket suppression:

  * `pages/bank_aset/qrcode_page.rs` — a preview of a label sheet that will be
    printed on paper. It has to be the colour of the print, so darkening it
    would make the preview lie.

Exit 0 clean, 1 on findings. Stdlib only — the ARC runner image has no PyYAML.
"""

import re
import sys
from pathlib import Path

ROOT = "antarmuka/perlengkapan/src"

# Path suffix → why the light palette is correct there.
ALLOWED = {
    "pages/bank_aset/qrcode_page.rs": "pratinjau lembar label yang dicetak di atas kertas",
}

# `bg-white` only when solid — `bg-white/10` is an alpha tint on a dark
# surface and is exactly what the converted screens use.
LIGHT = re.compile(
    r"(?<![\w:/\[-])("
    r"bg-white(?![\w/\[-])"
    r"|(?:hover:|focus:|active:)?bg-gray-\d{2,3}"
    r"|text-gray-\d{2,3}"
    r"|border-gray-\d{2,3}"
    r"|divide-gray-\d{2,3}"
    r")(?![\w/\[-])"
)

STRING = re.compile(r'"(?:\\.|[^"\\])*"')


def strip_comments(src: str) -> str:
    """Drop // and /* */ comments but keep string literals.

    A guard that reads its subject matter out of a comment has been three
    separate bugs in this repo; the files this one scans document their own
    history in prose that names the very classes being banned.
    """
    out, i, n = [], 0, len(src)
    while i < n:
        if src[i] == '"':
            m = STRING.match(src, i)
            if m:
                out.append(m.group(0))
                i = m.end()
                continue
            out.append(src[i])
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif src.startswith("/*", i):
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append("".join(c if c == "\n" else " " for c in src[i:j]))
            i = j
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    base = repo / ROOT
    if not base.is_dir():
        print(f"::error::guard scope {ROOT} does not exist", file=sys.stderr)
        return 1

    scanned, total = 0, 0
    for path in sorted(base.rglob("*.rs")):
        rel = path.relative_to(base).as_posix()
        if rel in ALLOWED:
            continue
        scanned += 1
        src = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        for i, line in enumerate(src.splitlines(), 1):
            for m in LIGHT.finditer(line):
                print(f"{path.relative_to(repo)}:{i}  {m.group(1)}")
                total += 1

    if not scanned:
        print(
            "check-dark-theme-palette: scanned ZERO files — the scope is broken, "
            "not the repo.",
            file=sys.stderr,
        )
        return 1

    if total:
        print(
            f"\n{total} light-palette class(es) in a frontend that renders on navy.\n"
            f"Use the app's tokens: bg-surface-panel, border-white/[0.06],\n"
            f"text-slate-100/200/400/500. Alpha tints (`bg-white/10`) are fine —\n"
            f"only the solid light shades are flagged.\n"
            f"If a screen genuinely must be light (a print preview), add it to\n"
            f"ALLOWED in this script WITH its reason.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-dark-theme-palette: {scanned} files, no light-palette class "
        f"({len(ALLOWED)} documented exception)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
