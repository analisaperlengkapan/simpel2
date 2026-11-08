#!/usr/bin/env python3
"""Fail when a real environment opts out of keeping its Namespace on `helm uninstall`.

WHY THIS EXISTS
---------------
The chart can render the Namespace object. Helm deletes everything it rendered on
`helm uninstall`, and deleting a Namespace cascade-deletes every PVC, Secret and
database in it (incident 2026-06-17: the whole staging namespace and its data).
`templates/namespace.yaml` therefore annotates it `helm.sh/resource-policy: keep`.

`namespace.retain: false` removes that annotation. It exists for exactly one caller:
the throwaway production rehearsal (RUNBOOK 1.9), whose point is that uninstalling it
leaves nothing behind. A real environment must never set it, so this fails the build if
`values-staging.yaml` or `values-production.yaml` does.

STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq. The values files are
parsed by indentation, which is enough for "a `retain:` key directly under `namespace:`".
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

HELM = Path(__file__).resolve().parents[2] / "infra" / "helm" / "simpel"
FILES = ("values.yaml", "values-staging.yaml", "values-production.yaml")
FALSY = {"false", "no", "off", "0", '"false"', "'false'"}


def namespace_retain(text: str) -> tuple[str | None, int | None]:
    """Value (raw, comment-stripped) and 1-based line of `namespace.retain`, if set."""
    in_block = False
    for n, line in enumerate(text.splitlines(), 1):
        if re.match(r"^namespace:\s*(#.*)?$", line):
            in_block = True
            continue
        if in_block:
            if line.strip() and not line.startswith((" ", "\t", "#")):
                in_block = False  # next top-level key
                continue
            m = re.match(r"^\s+retain:\s*(.*?)\s*(#.*)?$", line)
            if m:
                return m.group(1).lower(), n
    return None, None


def main() -> int:
    bad = []
    for name in FILES:
        value, line = namespace_retain((HELM / name).read_text())
        if value is None:
            if name == "values.yaml":
                bad.append(f"{name}: `namespace.retain` must be declared (default true)")
            continue
        if name == "values.yaml":
            if value in FALSY:
                bad.append(f"{name}:{line}: default must stay true, found `{value}`")
        elif value in FALSY:
            bad.append(
                f"{name}:{line}: `namespace.retain: false` would let `helm uninstall` "
                "cascade-delete the namespace and its data; only the rehearsal may set it, via --set"
            )
    for b in bad:
        print(f"ERROR {b}", file=sys.stderr)
    if not bad:
        print("namespace.retain: default true, not disabled in staging or production")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
