#!/usr/bin/env python3
"""Fail when the Helm chart and release.yml disagree about image names.

WHY THIS EXISTS
---------------
The chart pulls `<prefix><image.name>`; release.yml publishes
`<prefix><matrix name>`. Two hand-maintained lists that must agree exactly, with
nothing checking that they do — so they drifted:

  chart   simpelv1.gateway.image.name = gateway   -> simpelv2-gateway
  release matrix name                = layanan-gateway
                                                  -> simpelv2-layanan-gateway

Nothing in CI notices, because CI never deploys the chart and the release never
reads it. It surfaces at `helm upgrade`, as ImagePullBackOff — and for the
gateway specifically that is a SIDECAR in the simpelv1 pod, so the whole pod
never becomes Ready. Every other backend already used its `layanan/<dir>` name;
gateway was the lone outlier.

Both directions are checked, because each is a real defect:
  * chart names an image the release never builds -> deploy pulls nothing;
  * release builds an image the chart never uses  -> usually a half-finished
    rename, i.e. the same bug seen from the other end, plus wasted build time.

STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq, so importing yaml
here would exit 127 in CI.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[2]
VALUES = ROOT_DIR / "infra" / "helm" / "simpel" / "values.yaml"
RELEASE = ROOT_DIR / ".github" / "workflows" / "release.yml"
PROMOTE = ROOT_DIR / ".github" / "workflows" / "promote.yml"

# `image:` followed by `name: <x>` — first-party images only. Third-party pins
# (postgres:16-alpine, redis:7-alpine, alpine:3.20) are written as literal
# `image:` strings in the templates and never go through this key.
# Comment lines may sit between the two keys, so skip them rather than requiring
# `name:` to be adjacent — the first draft of this regex missed the very entry
# it was written for, because that entry now carries an explanatory comment.
CHART_IMAGE_RE = re.compile(
    r"image:[ \t]*\n(?:[ \t]*#[^\n]*\n)*[ \t]*name:\s*([A-Za-z0-9._-]+)"
)
# `- { name: layanan-authenc, dockerfile: ... }` in the build-docker matrix.
MATRIX_NAME_RE = re.compile(r"^\s*-\s*\{\s*name:\s*([A-Za-z0-9._-]+)\s*,", re.M)

# `matrix:\n  image:\n    - antarmuka-portal\n    - ...` in promote.yml: plain names, one per
# line (release.yml writes flow mappings instead, hence a second pattern).
PROMOTE_MATRIX_RE = re.compile(
    r"matrix:[ \t]*\n[ \t]*image:[ \t]*\n((?:[ \t]*-[ \t]*[A-Za-z0-9._-]+[ \t]*\n)+)"
)

# Images built but deliberately not referenced by the chart, each with a reason.
# Empty today: chart and release are an exact bijection, and keeping it that way
# is the point.
RELEASE_ONLY: dict[str, str] = {}


def main() -> int:
    chart = set(CHART_IMAGE_RE.findall(VALUES.read_text(encoding="utf-8")))
    release = set(MATRIX_NAME_RE.findall(RELEASE.read_text(encoding="utf-8")))

    # Finding nothing would make this a rubber stamp, so treat it as failure.
    if not chart:
        print(f"FAIL: parsed no `image:/name:` pairs from {VALUES.name} — the "
              "chart changed shape and this guard was measuring nothing.")
        return 1
    if not release:
        print(f"FAIL: parsed no build matrix names from {RELEASE.name} — the "
              "matrix changed shape and this guard was measuring nothing.")
        return 1

    m = PROMOTE_MATRIX_RE.search(PROMOTE.read_text(encoding="utf-8"))
    if not m:
        print(f"FAIL: parsed no image matrix from {PROMOTE.name} — the matrix "
              "changed shape and this guard was measuring nothing.")
        return 1
    promote = set(re.findall(r"-[ \t]*([A-Za-z0-9._-]+)", m.group(1)))

    missing = sorted(chart - release)
    unused = sorted(release - chart - set(RELEASE_ONLY))

    rc = 0
    if missing:
        print(f"FAIL: the chart pulls {len(missing)} image(s) that release.yml "
              "never builds — `helm upgrade` would ImagePullBackOff:")
        for n in missing:
            print(f"  - simpelv2-{n}  (values.yaml image.name: {n})")
        rc = 1

    if unused:
        print(f"FAIL: release.yml builds {len(unused)} image(s) the chart never "
              "references — usually half of a rename:")
        for n in unused:
            print(f"  - simpelv2-{n}  (release.yml matrix name: {n})")
        rc = 1

    # promote.yml re-tags rc -> final WITHOUT rebuilding, so an image missing from
    # its matrix has no final tag at all: production's `helm install` pulls
    # `<image>:vX.Y.Z` and gets ImagePullBackOff (layanan-gateway is a SIDECAR in
    # the simpelv1 pod, so that pod would never become Ready). Found 2026-11-08:
    # promote.yml listed 7 of the 9 images release.yml builds, and the first
    # production deploy would have been the first time anyone noticed.
    not_promoted = sorted(release - promote)
    stale_promote = sorted(promote - release)
    if not_promoted:
        print(f"FAIL: release.yml builds {len(not_promoted)} image(s) that "
              "promote.yml never re-tags — production would ImagePullBackOff "
              "on the final tag:")
        for n in not_promoted:
            print(f"  - simpelv2-{n}  (missing from promote.yml matrix)")
        rc = 1
    if stale_promote:
        print(f"FAIL: promote.yml re-tags {len(stale_promote)} image(s) "
              "release.yml no longer builds — `docker manifest inspect` fails:")
        for n in stale_promote:
            print(f"  - simpelv2-{n}  (in promote.yml matrix only)")
        rc = 1

    if rc:
        print("\nMake the two agree, or — for an image intentionally built "
              "without a chart consumer — add it to RELEASE_ONLY in this "
              "script, WITH the reason.")
    else:
        print(f"OK: {len(chart)} chart image name(s) match release.yml's build "
              f"matrix exactly, {len(RELEASE_ONLY)} excluded with a reason; "
              f"promote.yml re-tags all {len(promote)} of them.")
    return rc


if __name__ == "__main__":
    sys.exit(main())
