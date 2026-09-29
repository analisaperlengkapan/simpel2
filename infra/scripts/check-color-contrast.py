#!/usr/bin/env python3
"""Colour tokens the frontends DECLARE must meet WCAG 2.2 contrast.

The design tokens are the one place contrast can be checked mechanically: a
utility class such as `text-gray-400` has no known background until it is
rendered, but `--text-dim` on `--surface-raised` is a fixed pair.

Checked pairs (WCAG 2.2 SC 1.4.3 text >= 4.5:1, SC 1.4.11 UI components >= 3:1):

  * perlengkapan (dark surfaces): every `--text-*`, `--state-*` and `--brand-gold`
    variable against every `--surface-*` variable, >= 4.5:1;
  * both apps: white on `primary-600` and on `navy-800` (button text), >= 4.5:1;
    `gold-400` on `navy-800` / `navy-900` (accent on the sidebar), >= 4.5:1;
  * both apps: the focus indicator against the surface it is drawn on, >= 3:1.
    Portal is light-first, so the default indicator must contrast with WHITE and
    a gold outline (2.2:1 on white) does not;
  * both apps: the `primary` scale is an exact alias of `navy` — one brand
    palette, and `primary` cannot drift into a colour nobody reviewed.

A pair that cannot be resolved (a token was renamed, a file moved) exits 2: a
derivation that finds nothing must not read as a pass.

    python3 infra/scripts/check-color-contrast.py             # check the tree
    python3 infra/scripts/check-color-contrast.py --self-test # prove it can fail

Standard library only: the ARC runner image has no PyYAML.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APPS = ("portal", "perlengkapan")
WHITE = "#ffffff"

HEX = r"#[0-9a-fA-F]{6}\b"


def luminance(hex_colour: str) -> float:
    h = hex_colour.lstrip("#")
    channels = []
    for i in (0, 2, 4):
        c = int(h[i : i + 2], 16) / 255
        channels.append(c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4)
    r, g, b = channels
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: str, b: str) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def scale(config_src: str, token: str) -> dict[str, str] | None:
    """`token: { 50: '#..', .. }` from a tailwind config, keyed by shade."""
    m = re.search(rf"\b{re.escape(token)}:\s*\{{", config_src)
    if not m:
        return None
    depth, i = 0, m.end() - 1
    while i < len(config_src):
        if config_src[i] == "{":
            depth += 1
        elif config_src[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = config_src[m.end() : i]
    return {
        k: v
        for k, v in re.findall(rf"['\"]?(\w+)['\"]?\s*:\s*['\"]({HEX})['\"]", body)
    }


def css_vars(css_src: str) -> dict[str, str]:
    return {
        name: value
        for name, value in re.findall(rf"(--[\w-]+)\s*:\s*({HEX})\s*;", css_src)
    }


def focus_outline(css_src: str) -> str | None:
    """The colour of `*:focus-visible { outline: 2px solid #… }`."""
    m = re.search(r"\*:focus-visible\s*\{[^}]*?outline:[^;#]*?(" + HEX + ")", css_src)
    return m.group(1) if m else None


def check_app(app: str, root: Path) -> tuple[list[str], int]:
    """Return (problems, pairs_checked). Raises LookupError on an unresolved pair."""
    base = root / "antarmuka" / app
    cfg = (base / "tailwind.config.js").read_text(encoding="utf-8")
    problems: list[str] = []
    pairs = 0

    def need(min_ratio: float, fg: str, bg: str, what: str) -> None:
        nonlocal pairs
        pairs += 1
        ratio = contrast(fg, bg)
        if ratio < min_ratio:
            problems.append(
                f"  {app}: {what} is {ratio:.2f}:1 ({fg} on {bg}), needs {min_ratio}:1"
            )

    navy, primary, gold = scale(cfg, "navy"), scale(cfg, "primary"), scale(cfg, "gold")
    for name, sc in (("navy", navy), ("primary", primary), ("gold", gold)):
        if not sc:
            raise LookupError(f"{app}: tailwind.config.js defines no `{name}` scale")

    # One brand palette: `primary` is navy, shade for shade.
    for shade, value in navy.items():
        pairs += 1
        if primary.get(shade, "").lower() != value.lower():
            problems.append(
                f"  {app}: primary-{shade} is {primary.get(shade)!r}, navy-{shade} "
                f"is {value!r} — `primary` is an alias of navy and must match"
            )

    need(4.5, WHITE, primary["600"], "white text on primary-600")
    need(4.5, WHITE, navy["800"], "white text on navy-800")
    need(4.5, gold["400"], navy["800"], "gold-400 on navy-800")
    need(4.5, gold["400"], navy["900"], "gold-400 on navy-900")
    need(3.0, primary["500"], WHITE, "primary-500 (focus ring / control edge) on white")

    if app == "perlengkapan":
        css = (base / "styles" / "tailwind.css").read_text(encoding="utf-8")
        v = css_vars(css)
        surfaces = {k: c for k, c in v.items() if k.startswith("--surface-")}
        if not surfaces:
            raise LookupError("perlengkapan: no --surface-* hex variables found")
        fgs = {
            k: c
            for k, c in v.items()
            if k.startswith(("--text-", "--state-")) or k == "--brand-gold"
        }
        if not fgs:
            raise LookupError("perlengkapan: no --text-*/--state-* variables found")
        for fk, fc in sorted(fgs.items()):
            for sk, sc_ in sorted(surfaces.items()):
                need(4.5, fc, sc_, f"{fk} on {sk}")
    else:
        css = (base / "styles" / "main.css").read_text(encoding="utf-8")
        outline = focus_outline(css)
        if outline is None:
            raise LookupError("portal: no `*:focus-visible { outline: … #hex }` rule")
        # The default indicator is drawn on the light page background.
        need(3.0, outline, WHITE, "default focus outline on the white page")

    return problems, pairs


def run(root: Path) -> int:
    problems: list[str] = []
    total = 0
    for app in APPS:
        try:
            p, n = check_app(app, root)
        except (LookupError, FileNotFoundError) as e:
            print(f"FAIL: cannot resolve the contrast pairs — {e}", file=sys.stderr)
            return 2
        problems += p
        total += n
    if problems:
        print(
            "Declared colour tokens below WCAG 2.2 contrast (text 4.5:1, "
            "UI 3:1):\n",
            file=sys.stderr,
        )
        print("\n".join(problems), file=sys.stderr)
        return 1
    print(f"OK: {total} declared colour pair(s), all meet contrast.")
    return 0


def self_test() -> int:
    import shutil
    import tempfile

    failures: list[str] = []

    def expect(name: str, got: int, want: int) -> None:
        if got != want:
            failures.append(f"{name}: exit {got}, want {want}")

    # The arithmetic first: known reference values.
    assert abs(contrast("#000000", "#ffffff") - 21.0) < 0.01
    assert abs(contrast("#777777", "#ffffff") - 4.48) < 0.02

    expect("the real tree passes", run(ROOT), 0)

    with tempfile.TemporaryDirectory() as tmp:
        tmp_root = Path(tmp)
        for app in APPS:
            dst = tmp_root / "antarmuka" / app
            (dst / "styles").mkdir(parents=True)
            shutil.copy(ROOT / "antarmuka" / app / "tailwind.config.js", dst)
            for css in (ROOT / "antarmuka" / app / "styles").glob("*.css"):
                shutil.copy(css, dst / "styles")

        def mutate(rel: str, old: str, new: str) -> None:
            path = tmp_root / rel
            src = path.read_text(encoding="utf-8")
            assert old in src, f"self-test fixture drifted: {old!r} not in {rel}"
            path.write_text(src.replace(old, new, 1), encoding="utf-8")

        def restore(rel: str) -> None:
            shutil.copy(ROOT / rel, tmp_root / rel)

        expect("copy of the tree passes", run(tmp_root), 0)

        rel = "antarmuka/perlengkapan/styles/tailwind.css"
        # --text-dim identical to the base surface: unreadable.
        path = tmp_root / rel
        path.write_text(
            re.sub(
                rf"(--text-dim:\s*){HEX}",
                r"\g<1>#0f172a",
                path.read_text(encoding="utf-8"),
                count=1,
            ),
            encoding="utf-8",
        )
        expect("a text token equal to its surface goes red", run(tmp_root), 1)
        restore(rel)

        rel = "antarmuka/portal/styles/main.css"
        mutate(rel, "outline: 2px solid", "outline: 2px solid #ffffff; --was:")
        expect("a white focus ring on white goes red", run(tmp_root), 1)
        restore(rel)

        rel = "antarmuka/portal/tailwind.config.js"
        src = (tmp_root / rel).read_text(encoding="utf-8")
        idx = src.index("primary: {")
        drifted = src[idx:].replace("600: '#486581'", "600: '#a0b4c8'", 1)
        (tmp_root / rel).write_text(src[:idx] + drifted, encoding="utf-8")
        expect("primary drifting away from navy goes red", run(tmp_root), 1)
        restore(rel)

        (tmp_root / "antarmuka/portal/styles/main.css").write_text("/* empty */")
        expect("an unresolvable pair goes to 2, not a pass", run(tmp_root), 2)

    if failures:
        print("SELF-TEST FAILED:\n  " + "\n  ".join(failures), file=sys.stderr)
        return 1
    print("SELF-TEST OK: the guard passes the tree and fails on each seeded defect.")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    sys.exit(run(ROOT))
