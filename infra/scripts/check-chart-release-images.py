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

    if rc:
        print("\nMake the two agree, or — for an image intentionally built "
              "without a chart consumer — add it to RELEASE_ONLY in this "
              "script, WITH the reason.")
    else:
        print(f"OK: {len(chart)} chart image name(s) match release.yml's build "
              f"matrix exactly, {len(RELEASE_ONLY)} excluded with a reason.")
    return rc


if __name__ == "__main__":
    sys.exit(main())
