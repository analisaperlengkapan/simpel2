#!/usr/bin/env python3
"""Fail when production's Istio route table differs from staging's.

WHY THIS EXISTS
---------------
`values-staging.yaml` and `values-production.yaml` each carry their own
`istio.virtualService.routes`. Staging is where every route is exercised
(WAJIB staging -> promote -> production), so a route that exists only in
production has never been tested, and a route missing from production is a
feature that works on staging and silently 404s for real users.

That is exactly what happened: production's table was a pre-F5 copy with NOT ONE
authenc route (`/api/v1/auth`, `/api/v1/iam`, `/api/v1/oauth2`, `/api/v1/captcha`,
`/api/auth`, `/captcha/`...), `/perlengkapan/simpel/v2` pointed at the backend
instead of the frontend, and `/portal/` did not exist. Login would have been
impossible on day one. Nothing noticed, because production had never been
deployed and nothing compared the two tables.

What is compared, in order (Istio takes the FIRST matching route, so order is
part of the contract): match kind+value, rewrite, redirect, destination
host:port, timeout, retries, and whether a corsPolicy is attached. The CORS
*origin* is deliberately not compared — it is the one thing that must differ.

STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

HELM = Path(__file__).resolve().parents[2] / "infra" / "helm" / "simpel"
STAGING = HELM / "values-staging.yaml"
PRODUCTION = HELM / "values-production.yaml"

# Routes allowed on exactly one side, each WITH the reason. Empty on purpose.
STAGING_ONLY: dict[str, str] = {}
PRODUCTION_ONLY: dict[str, str] = {}


def routes_block(text: str, name: str) -> list[str]:
    """Lines of `istio.virtualService.routes` (everything indented deeper than it)."""
    lines = text.splitlines()
    try:
        start = next(i for i, l in enumerate(lines) if re.fullmatch(r"    routes:\s*", l))
    except StopIteration:
        raise SystemExit(f"FAIL: no `    routes:` block in {name} — values layout changed, "
                         "this guard would be measuring nothing.")
    out: list[str] = []
    for l in lines[start + 1:]:
        if l.strip() == "" or l.lstrip().startswith("#"):
            continue
        if len(l) - len(l.lstrip()) <= 4:  # back to a sibling of `routes:`
            break
        out.append(l)
    return out


def parse(lines: list[str], name: str) -> list[tuple]:
    entries: list[list[str]] = []
    for l in lines:
        if re.match(r"^      - ", l):
            entries.append([l])
        elif entries:
            entries[-1].append(l)
    if not entries:
        raise SystemExit(f"FAIL: parsed no routes from {name}.")
    parsed = []
    for e in entries:
        blob = "\n".join(e)
        m = re.search(r"match:\s*\{\s*(exact|prefix|regex):\s*([^\s,}]+)", blob)
        rw = re.search(r"^\s+rewrite:\s*(\S+)", blob, re.M)
        rd = re.search(r"redirect:\s*\{\s*uri:\s*(\S+),\s*code:\s*(\d+)", blob)
        d = re.search(r"destination:\s*\{\s*host:\s*([^\s,}]+),\s*port:\s*(\d+)", blob)
        to = re.search(r"^\s+timeout:\s*(\S+)", blob, re.M)
        rt = re.search(r"retries:\s*(\{[^}]*\})", blob)
        parsed.append((
            f"{m.group(1)} {m.group(2)}" if m else "?",
            rw.group(1) if rw else None,
            f"{rd.group(1)} {rd.group(2)}" if rd else None,
            f"{d.group(1)}:{d.group(2)}" if d else None,
            to.group(1) if to else None,
            re.sub(r"\s+", " ", rt.group(1)) if rt else None,
            "corsPolicy" in blob,
        ))
    return parsed


def main() -> int:
    s = parse(routes_block(STAGING.read_text(encoding="utf-8"), STAGING.name), STAGING.name)
    p = parse(routes_block(PRODUCTION.read_text(encoding="utf-8"), PRODUCTION.name), PRODUCTION.name)

    s = [r for r in s if r[0] not in STAGING_ONLY]
    p = [r for r in p if r[0] not in PRODUCTION_ONLY]

    if s == p:
        print(f"OK: production's {len(p)} routes are identical to staging's "
              f"(same order, destinations, rewrites, timeouts, retries).")
        return 0

    print("FAIL: production and staging route tables differ — what staging proved "
          "is not what production would serve:")
    ss, ps = {r[0]: r for r in s}, {r[0]: r for r in p}
    for k in ss:
        if k not in ps:
            print(f"  - missing in production: {k}  -> {ss[k][3] or ss[k][2]}")
    for k in ps:
        if k not in ss:
            print(f"  - only in production:    {k}  -> {ps[k][3] or ps[k][2]}")
    for k in ss:
        if k in ps and ss[k] != ps[k]:
            print(f"  - differs: {k}\n      staging   {ss[k][1:]}\n      production {ps[k][1:]}")
    if not (set(ss) ^ set(ps)) and [r[0] for r in s] != [r[0] for r in p]:
        print("  - same routes, different ORDER (Istio matches the first hit)")
    print("\nMake production match staging, or list the exception in "
          "STAGING_ONLY / PRODUCTION_ONLY in this script WITH the reason.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
