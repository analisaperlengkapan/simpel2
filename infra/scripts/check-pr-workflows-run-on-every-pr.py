#!/usr/bin/env python3
"""A workflow that gates pull requests must run on EVERY pull request.

`ci.yml`, `security.yml` and `autofix.yml` each carried:

    on:
      pull_request:
        branches: [main]

which reads as "gate pull requests" and behaves as "gate pull requests that
happen to target main". A stacked PR — base = another PR's branch, which is how
a dependent change is reviewed — matched no trigger, so **not one job ran**.

That alone would be a coverage gap. What made it a hole is the second half: the
branch ruleset targets `main`, so its `required_status_checks` did not apply to
a PR targeting anything else either. With no checks required and no checks run,
GitHub reported `mergeStateStatus: CLEAN` — the same value a fully green PR
gets. Three PRs in one stack sat at "CLEAN" with zero verification, and the
merge button was live on all of them.

The auto-retarget that happens when the parent merges does not repair it: base
changes fire `pull_request.edited`, which is not in the default trigger types,
so the child is still never built before it can be merged into main.

The rule is therefore total rather than a list: any workflow with a
`pull_request` trigger runs on every pull request, whatever its base. A
workflow that genuinely must be scoped states so in `PERMITTED` below with a
reason — the exemption is a decision on the record, not a filter nobody reads.

Run: python3 infra/scripts/check-pr-workflows-run-on-every-pr.py
     python3 infra/scripts/check-pr-workflows-run-on-every-pr.py --self-test
"""

from __future__ import annotations

import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github/workflows"

# workflow filename -> written reason it may restrict which PRs it runs on.
PERMITTED: dict[str, str] = {}


def triggers(doc: dict) -> dict:
    # PyYAML parses a bare `on:` key as the boolean True.
    on = doc.get(True, doc.get("on"))
    return on if isinstance(on, dict) else {}


def problems_in(name: str, text: str) -> list[str]:
    doc = yaml.safe_load(text)
    if not isinstance(doc, dict):
        return []
    pr = triggers(doc).get("pull_request")
    if pr is None or not isinstance(pr, dict):
        return []
    found = [k for k in ("branches", "branches-ignore") if k in pr]
    if not found:
        return []
    if name in PERMITTED:
        return []
    return [
        f"{name}: `pull_request` is filtered by {', '.join(found)} "
        f"({pr[found[0]]!r}). A PR based on another branch then runs none of "
        f"this workflow, and — because the ruleset targets main — requires none "
        f"of it either, so it reads as CLEAN unverified. Drop the filter, or "
        f"add {name!r} to PERMITTED with a reason."
    ]


def scan() -> list[str]:
    out: list[str] = []
    for path in sorted(WORKFLOWS.glob("*.y*ml")):
        out += problems_in(path.name, path.read_text())
    return out


def main() -> int:
    found = scan()
    for f in found:
        print(f"::error::{f}")
    if found:
        return 1
    # A bare `pull_request:` (the correct, unfiltered form) parses to None, so
    # membership is the test — `.get(...) is not None` would count zero of them
    # and report a coverage number smaller than the truth.
    total = sum(
        1
        for p in WORKFLOWS.glob("*.y*ml")
        if "pull_request" in triggers(yaml.safe_load(p.read_text()) or {})
    )
    print(
        f"check-pr-workflows-run-on-every-pr: {total} workflow(s) trigger on "
        f"pull_request, none filtered by base branch "
        f"({len(PERMITTED)} exempted with a reason)"
    )
    return 0


def _canaries() -> int:
    failures = 0

    def expect(label: str, name: str, text: str, should_fail: bool) -> None:
        nonlocal failures
        ok = bool(problems_in(name, text)) == should_fail
        print(f"  {'ok  ' if ok else 'FAIL'} {label}")
        if not ok:
            failures += 1

    expect("the workflows as committed pass", "ci.yml", (WORKFLOWS / "ci.yml").read_text(), False)
    expect(
        "detects: the exact regression — branches: [main] restored",
        "ci.yml",
        "on:\n  pull_request:\n    branches: [main]\njobs: {}\n",
        True,
    )
    expect(
        "detects: branches-ignore, the same hole spelled the other way",
        "security.yml",
        "on:\n  pull_request:\n    branches-ignore: [docs/**]\njobs: {}\n",
        True,
    )
    expect(
        "tolerates: a pull_request trigger with types but no branch filter",
        "ci.yml",
        "on:\n  pull_request:\n    types: [opened, synchronize]\njobs: {}\n",
        False,
    )
    expect(
        "tolerates: a workflow with no pull_request trigger at all",
        "release.yml",
        "on:\n  push:\n    branches: [main]\njobs: {}\n",
        False,
    )
    return failures


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(_canaries())
    sys.exit(main())
