#!/usr/bin/env python3
"""Every colour shade a frontend USES must be defined in its tailwind config.

Tailwind generates a utility only for a shade it knows. Ask for one it does
not — `text-warning-300` when the scale jumps 100 → 400 — and NOTHING is
emitted: no warning, no build error, no missing-class error at runtime. The
element simply inherits whatever colour was around it.

That is what happened here. `success/warning/danger/info` were defined as
`50 100 400 500 600 700 900`, and the tree referenced `-200` and `-300` in 115
places. Backgrounds and borders were right, because those used `-500`; every
status MESSAGE was rendering in plain slate. It was found by reading a
screenshot, which is the only thing that could have found it — the class is in
the HTML, so the DOM looks correct, and only the computed colour is wrong.

It is the same failure as `bg-primary` in #903: a token used but never defined,
silent in both directions.

Derivation, not a list:

  * the frontends are every `antarmuka/*/tailwind.config.js`;
  * the files each one covers come from ITS OWN `content` globs, so a shared
    crate like `lib/ui` is checked against every config that pulls it in;
  * the shades it defines come from parsing that same config.

A config whose globs match no file, or that defines no scale the tree uses,
exits 2 — a derivation that comes back empty must not read as a pass.

Standard library only: the ARC runner image has no PyYAML and no yq.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

CONFIG_GLOB = "antarmuka/*/tailwind.config.js"
SKIP_DIRS = {"target", "node_modules", ".git", "dist"}

# Tailwind's own palette. A token outside this set that the config does not
# define is not "someone else's scale" — it is a colour that does not exist.
TAILWIND_PALETTE = {
    "slate", "gray", "zinc", "neutral", "stone", "red", "orange", "amber",
    "yellow", "lime", "green", "emerald", "teal", "cyan", "sky", "blue",
    "indigo", "violet", "purple", "fuchsia", "pink", "rose",
}
# `bg-opacity-50`, `ring-offset-…` and friends share the shape but are not colours.
NOT_A_COLOUR = {"opacity", "offset", "inset", "width", "current", "spacing"}

# `text-warning-300`, `bg-danger-500/10`, `ring-info-400` …
USE_RE = re.compile(
    r"\b(?:text|bg|border|ring|from|to|via|divide|outline|shadow|accent|caret"
    r"|fill|stroke|placeholder|decoration)-([a-z]+)-(\d{2,3})\b"
)
CONTENT_RE = re.compile(r"content:\s*\[(.*?)\]", re.S)
GLOB_RE = re.compile(r"['\"]([^'\"]+)['\"]")


def scale_shades(src: str, token: str) -> set[str] | None:
    """The shades `token` defines, or None when the config has no such scale."""
    # Find `token: {` and walk to its matching brace, so a nested object cannot
    # end the scan early.
    m = re.search(rf"\b{re.escape(token)}:\s*\{{", src)
    if not m:
        return None
    depth, i = 0, m.end() - 1
    while i < len(src):
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = src[m.end() : i]
    return set(re.findall(r"(\d{2,3})\s*:", body))


def files_for(config: Path) -> list[Path]:
    src = config.read_text(encoding="utf-8")
    m = CONTENT_RE.search(src)
    if not m:
        return []
    out: list[Path] = []
    for glob in GLOB_RE.findall(m.group(1)):
        if not glob.endswith(".rs"):
            continue
        for path in sorted((config.parent).glob(glob)):
            if any(part in SKIP_DIRS for part in path.parts):
                continue
            out.append(path)
    return out


def main() -> int:
    configs = sorted(ROOT.glob(CONFIG_GLOB))
    if not configs:
        print(
            f"FAIL: no tailwind config matched {CONFIG_GLOB} — the derivation "
            "is broken, not the code.",
            file=sys.stderr,
        )
        return 2

    problems: list[str] = []
    checked_pairs = 0
    for config in configs:
        src = config.read_text(encoding="utf-8")
        files = files_for(config)
        if not files:
            print(
                f"FAIL: {config.relative_to(ROOT)} covers no .rs file — its "
                "content globs no longer match anything.",
                file=sys.stderr,
            )
            return 2

        # token -> shade -> one example file, so the report can point somewhere.
        used: dict[str, dict[str, Path]] = {}
        for path in files:
            for token, shade in USE_RE.findall(
                path.read_text(encoding="utf-8", errors="replace")
            ):
                used.setdefault(token, {}).setdefault(shade, path)

        for token, shades in sorted(used.items()):
            defined = scale_shades(src, token)
            if defined is None:
                if token in TAILWIND_PALETTE or token in NOT_A_COLOUR:
                    continue
                # Neither a scale this config defines nor one Tailwind ships:
                # `bg-primary-600` with no `primary` scale. Tailwind emits
                # nothing for it, exactly as for a gap inside a scale, and the
                # element renders with no colour at all.
                for shade, example in sorted(shades.items()):
                    checked_pairs += 1
                    problems.append(
                        f"  {config.relative_to(ROOT)}: `{token}-{shade}` is used "
                        f"but no `{token}` scale is defined (and it is not a "
                        f"Tailwind palette name)\n"
                        f"      e.g. {example.relative_to(ROOT)}"
                    )
                continue
            for shade, example in sorted(shades.items()):
                checked_pairs += 1
                if shade not in defined:
                    problems.append(
                        f"  {config.relative_to(ROOT)}: `{token}-{shade}` is used "
                        f"but the scale defines only "
                        f"{' '.join(sorted(defined, key=int))}\n"
                        f"      e.g. {example.relative_to(ROOT)}"
                    )

    if not checked_pairs:
        print(
            "FAIL: no custom-scale colour use found at all — either the scales "
            "were renamed or the scan is broken.",
            file=sys.stderr,
        )
        return 2

    if problems:
        print(
            "Tailwind generates NOTHING for a shade it does not know: no "
            "warning,\nno build error. The class stays in the HTML and the "
            "element inherits\nwhatever colour was around it.\n",
            file=sys.stderr,
        )
        print("\n".join(problems), file=sys.stderr)
        return 1

    print(f"OK: {checked_pairs} colour shade use(s), every one defined.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
