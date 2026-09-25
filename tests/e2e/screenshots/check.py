#!/usr/bin/env python3
"""Reject screenshots that are blank, uniform, or otherwise not worth publishing.

The capture script can only see the DOM. Two failures are invisible from there
and are exactly the ones a reader notices first:

  * a page that mounted but painted nothing (the WASM bundle served, the route
    resolved, the DOM has text — and the screenshot is a single flat colour);
  * a page drowned in one colour (a full-bleed overlay, or a table whose rows
    collapsed to zero height).

So the pixels are checked too, adversarially: for each image this reports

  * `unique_colours`  — Python's quantised palette size. 1 means literally one
    colour; a handful means a flat panel.
  * `non_bg_fraction` — share of pixels that differ from the image's own most
    common colour. A real page is mostly NOT its background: chrome, type,
    rules and tables all contribute.
  * `ink_rows`        — number of rows carrying at least 0.5% non-background
    pixels. This is what separates "a hero banner with a paragraph" from "a
    legend strip above an empty table": both can have the same global
    non-background fraction, but only one has content spread down the page.

A threshold that fires on nothing is worthless, so each is calibrated against
the failure modes actually seen: pure-white exports (unique_colours == 1),
near-white exports (< 8 colours and < 2% ink), and "blank except the header"
(a single ink band). Every image is written to the JSON report with its
metrics, so a threshold can be re-tuned from evidence rather than guessed.

Usage:  python3 tests/e2e/screenshots/check.py [--dir DIR] [--json]
Exit 0 if nothing is flagged, 1 otherwise.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

from PIL import Image

REPO = Path(__file__).resolve().parents[3]
DEFAULT_DIR = REPO / "docs" / "assets" / "screenshots"

# How far from the background colour a pixel must sit to count as ink. Small
# enough to catch 20%-opacity body text on a dark theme, large enough to ignore
# PNG anti-aliasing noise.
INK_DISTANCE = 24

# A row counts as carrying structure when its pixels span at least this much
# luma. This is the dark-theme-safe companion to INK_DISTANCE: on a near-black
# page a slate-800 card sits only ~20 luma from the background, so a
# background-distance test cannot see it at all and reports the page as empty.
# Luma spread within the row sees the card because the card's text is bright.
ROW_CONTRAST = 25

# A screenshot this short has lost the page: either the SPA rendered a single
# line, or fullPage capture saw a zero-height root.
MIN_HEIGHT = 200
MIN_WIDTH = 600


def metrics(path: Path) -> dict:
    with Image.open(path) as im:
        rgb = im.convert("RGB")
        w, h = rgb.size
        # Quantise before counting: an 8-bit-per-channel full-page screenshot
        # can carry thousands of distinct anti-aliased colours even when it
        # looks flat, which would make a raw `len(set(pixels))` useless.
        quantised = rgb.quantize(colors=64).convert("RGB")
        pixels = list(quantised.getdata())
        counts = Counter(pixels)
        bg, bg_count = counts.most_common(1)[0]
        # Luma is read off the UNQUANTISED image: quantisation collapses the
        # subtle card/background gradient a dark theme relies on, which is
        # precisely the signal the row-contrast test needs to keep.
        luma = list(rgb.convert("L").getdata())

    def is_ink(p: tuple[int, int, int]) -> bool:
        return (
            abs(p[0] - bg[0]) + abs(p[1] - bg[1]) + abs(p[2] - bg[2])
        ) > INK_DISTANCE

    ink = sum(1 for p in pixels if is_ink(p))
    rows_with_ink = 0
    rows_with_structure = 0
    content_rows: list[int] = []
    for y in range(h):
        row = pixels[y * w : (y + 1) * w]
        ink_in_row = sum(1 for p in row if is_ink(p))
        luma_row = luma[y * w : (y + 1) * w]
        has_structure = max(luma_row) - min(luma_row) >= ROW_CONTRAST
        if ink_in_row / w >= 0.005:
            rows_with_ink += 1
        if has_structure:
            rows_with_structure += 1
        if ink_in_row / w >= 0.005 or has_structure:
            content_rows.append(y)

    # Where the content sits, vertically. A header-only page puts everything in
    # the top sliver; a centred error card straddles the middle. Telling those
    # apart is the difference between "the page never loaded its data" and "the
    # page deliberately shows a short message" — both have one ink band.
    reaches_middle = bool(content_rows) and content_rows[-1] >= h * 0.5

    return {
        "file": path.name,
        "width": w,
        "height": h,
        "unique_colours": len(counts),
        "background": "#%02x%02x%02x" % bg,
        "bg_fraction": round(bg_count / max(len(pixels), 1), 4),
        "non_bg_fraction": round(ink / max(len(pixels), 1), 4),
        "ink_rows": rows_with_ink,
        "ink_row_fraction": round(rows_with_ink / max(h, 1), 4),
        "structure_rows": rows_with_structure,
        "structure_row_fraction": round(rows_with_structure / max(h, 1), 4),
        "content_reaches_middle": reaches_middle,
    }


def classify(m: dict) -> list[str]:
    """The reasons this image must not be published. Empty means usable.

    Two families of evidence are accepted, and either one is enough to pass:

      * ink — pixels far from the background colour. This is what a light
        theme produces, and what the original thresholds were calibrated on.
      * structure — rows whose own pixels span enough luma. This is the only
        evidence a dark theme offers: a slate-on-near-black card is invisible
        to a background-distance test, but its text still lights up its row.

    Requiring BOTH would reintroduce the false positive that flagged the
    dark-theme `/admin/roles` guard page as "content only at the top" while it
    plainly had a card in the middle of the page.
    """
    has_plenty = (
        m["non_bg_fraction"] >= 0.02
        or m["structure_row_fraction"] >= 0.25
    )

    flags = []
    if m["unique_colours"] <= 1:
        flags.append("SATU-WARNA")  # literally one colour: a blank canvas
    if m["width"] < MIN_WIDTH or m["height"] < MIN_HEIGHT:
        flags.append("UKURAN-MENYUSUT")
    # < 8 colours AND almost nothing anywhere: a flat panel with a stray glyph.
    if m["unique_colours"] < 8 and not has_plenty:
        flags.append("NYARIS-KOSONG")
    # A little content, but confined to the top of the page — the header over an
    # empty body, which is what a route that mounted but never loaded its data
    # looks like. A centred confirmation/error card also has one band, but it
    # reaches the middle; only the top-only band is the failure shape.
    elif not has_plenty and not m["content_reaches_middle"]:
        flags.append("KONTEN-HANYA-DI-ATAS")
    return flags


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", default=str(DEFAULT_DIR))
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    directory = Path(args.dir)
    if not directory.is_dir():
        print(f"no such directory: {directory}", file=sys.stderr)
        return 2

    images = sorted(p for p in directory.glob("*.png"))
    if not images:
        print(f"no PNGs in {directory}", file=sys.stderr)
        return 2

    report = []
    flagged = []
    for p in images:
        m = metrics(p)
        m["flags"] = classify(m)
        report.append(m)
        if m["flags"]:
            flagged.append(m)

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"checked {len(images)} screenshots in {directory}")
        if flagged:
            print(f"\nNOT FIT TO PUBLISH ({len(flagged)}):")
            for m in flagged:
                print(
                    f"  {m['file']}  {','.join(m['flags'])}  "
                    f"colours={m['unique_colours']} bg={m['background']} "
                    f"ink={m['non_bg_fraction']} ink_rows={m['ink_rows']}/{m['height']}"
                )
        else:
            print("every screenshot carries real content")

    (directory / "image-report.json").write_text(json.dumps(report, indent=2))
    return 1 if flagged else 0


def selftest() -> int:
    """Prove the thresholds still separate a real page from a wasted one.

    The thresholds are the entire value of this script, and they are the part
    most likely to be "relaxed" to make a red run go away. So they are pinned
    against synthetic images whose verdict is not a matter of taste: a flat
    canvas must be rejected, and a page with content down its length must not.
    Measured through `metrics`/`classify` — the real code path — rather than by
    re-implementing the arithmetic here, which is how a test drifts from the
    thing it claims to cover.
    """
    import tempfile

    from PIL import Image, ImageDraw

    cases: list[tuple[str, bool, tuple]] = [
        # (name, should_be_flagged, (width, height, bg, content, dark))
        ("white canvas", True, (1600, 1000, (255, 255, 255), False, False)),
        ("header only, light", True, (1600, 1000, (255, 255, 255), True, False)),
        ("dark canvas", True, (1600, 1000, (15, 23, 43), False, True)),
        ("collapsed height", True, (1600, 120, (200, 200, 200), False, False)),
        ("empty body, dark", True, (1600, 1000, (15, 23, 43), True, True)),
        # A centred card is the shape of every confirmation/error page; it must
        # survive, or the gallery loses exactly the states worth documenting.
        ("centred card, light", False, (1600, 1000, (249, 250, 251), "centre", False)),
        ("centred card, dark", False, (1600, 1000, (15, 23, 43), "centre", True)),
    ]

    failures = []
    with tempfile.TemporaryDirectory() as tmp:
        for name, should_flag, spec in cases:
            w, h, bg, content, dark = spec
            im = Image.new("RGB", (w, h), bg)
            d = ImageDraw.Draw(im)
            if content is True:
                # App chrome only, confined to the top — a logo mark and a thin
                # rule, NOT a full-width slab. A tall filled band would put 6%
                # of the page in ink and genuinely pass the density test, which
                # is why the real header-only failures measure ~1%. Modelling
                # the failure faithfully is the difference between a test of
                # the classifier and a test of the drawing.
                bar = (30, 41, 59) if dark else (226, 232, 240)
                d.rectangle([0, 0, int(w * 0.22), 34], fill=bar)
                d.rectangle([0, 34, w, 36], fill=bar)
            elif content == "centre":
                card = (30, 41, 59) if dark else (255, 255, 255)
                top, bottom = int(h * 0.35), int(h * 0.65)
                d.rectangle([int(w * 0.25), top, int(w * 0.75), bottom], fill=card)
                ink = (241, 245, 249) if dark else (17, 24, 39)
                for i in range(6):
                    y = top + 20 + i * 24
                    d.rectangle([int(w * 0.28), y, int(w * 0.72), y + 8], fill=ink)
            path = Path(tmp) / f"{name.replace(' ', '_')}.png"
            im.save(path)
            flags = classify(metrics(path))
            got = bool(flags)
            if got != should_flag:
                failures.append(f"  {name}: expected flagged={should_flag}, got {flags}")

    if failures:
        print("SELF-TEST GAGAL:", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"check.py self-test lulus ({len(cases)} kasus)")
    return 0


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        sys.exit(selftest())
    sys.exit(main())