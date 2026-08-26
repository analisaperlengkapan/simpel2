#!/usr/bin/env python3
"""Fail when the workflows disagree about which actionlint they run.

WHY THIS EXISTS
---------------
`ci.yml` (blocking, runs on PRs) and `maintenance.yml` (nightly, NOT a required
check) both lint workflows with actionlint. They used to do it differently:

    ci.yml          actionlint 1.7.7, standalone binary  -> NO shellcheck
    maintenance.yml rhysd/actionlint:1.7.12 image        -> bundles shellcheck

So the gate that could block a PR ran a weaker linter than the gate that could
not. #824 merged green while introducing two SC2094 findings, and Maintenance
then failed five runs straight (2026-08-25 → 2026-08-26) with nothing blocked —
the same shape as the 40-day invisible-red incident that P-A cleaned up.

Both now run the same pinned image. This keeps them that way: a version bumped
in one file and not the other quietly reopens the gap, because two pinned
versions of the same tool can carry different checks.

STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"
PIN_RE = re.compile(r"rhysd/actionlint:([0-9]+\.[0-9]+\.[0-9]+)")
# Any other way of getting actionlint reintroduces the capability gap.
BINARY_RE = re.compile(r"actionlint_[0-9.]+_linux|raven-actions/actionlint")


def main() -> int:
    pins: dict[str, set[str]] = {}
    offenders: list[str] = []

    for wf in sorted(WORKFLOWS.glob("*.yml")):
        text = wf.read_text(encoding="utf-8")
        # Comments explain history (including the old binary); only real steps
        # count, so drop comment lines before looking for the binary install.
        body = "\n".join(l for l in text.split("\n") if not l.lstrip().startswith("#"))
        found = set(PIN_RE.findall(body))
        if found:
            pins[wf.name] = found
        if BINARY_RE.search(body):
            offenders.append(wf.name)

    if not pins:
        print("GAGAL: tak satu pun workflow memakai `rhysd/actionlint:<versi>`.")
        print("Itu bukan 'tak ada yang me-lint' — itu bentuknya berubah dan")
        print("pemeriksaan ini berhenti mengukur apa pun.")
        return 1

    rc = 0
    if offenders:
        print("GAGAL: actionlint dipasang sebagai biner/action, bukan image ter-pin:")
        for name in offenders:
            print(f"  - {name}")
        print("Biner standalone TIDAK membundel shellcheck, jadi ia diam-diam")
        print("me-lint lebih sedikit daripada image-nya.")
        rc = 1

    versions = {v for vs in pins.values() for v in vs}
    if len(versions) > 1:
        print(f"GAGAL: {len(versions)} versi actionlint berbeda ter-pin:")
        for name, vs in sorted(pins.items()):
            print(f"  - {name}: {', '.join(sorted(vs))}")
        print("Dua versi ter-pin dari alat yang sama bisa membawa cek berbeda;")
        print("gerbang yang memblokir PR tak boleh lebih lemah dari yang tidak.")
        rc = 1

    if rc == 0:
        v = versions.pop()
        print(f"OK: {len(pins)} workflow memakai rhysd/actionlint:{v}, seragam.")
    return rc


if __name__ == "__main__":
    sys.exit(main())
