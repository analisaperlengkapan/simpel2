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

Parsed by line rather than with PyYAML, like every other guard here: the ARC
runner image ships a bare Python, so `import yaml` is `exit 1` on the runner and
green on a developer machine. This script learned that the hard way — its first
CI run failed on exactly that import, which is why
`check-guard-imports-are-stdlib.py` now exists.

Run: python3 infra/scripts/check-pr-workflows-run-on-every-pr.py
     python3 infra/scripts/check-pr-workflows-run-on-every-pr.py --self-test
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github/workflows"

# workflow filename -> written reason it may restrict which PRs it runs on.
PERMITTED: dict[str, str] = {}


ON_RE = re.compile(r"^(?:on|\"on\"|'on'|true|True):\s*(.*)$")
PR_KEY_RE = re.compile(r"^(\s+)pull_request:\s*(.*)$")
FILTER_RE = re.compile(r"^\s*(branches|branches-ignore)\s*:")


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def pull_request_filters(text: str) -> list[str] | None:
    """Filter keys under the `pull_request` trigger, or None if there is none.

    An empty list means the trigger exists and is unfiltered — the correct
    state. That is deliberately distinct from None, so "no trigger" and "an
    unfiltered trigger" cannot be confused by a caller counting coverage.
    """
    lines = text.splitlines()
    in_on = False
    on_indent = 0
    for i, raw in enumerate(lines):
        line = raw.split("#", 1)[0].rstrip() if not raw.lstrip().startswith("#") else ""
        if not line.strip():
            continue
        if _indent(line) == 0:
            m = ON_RE.match(line)
            in_on = bool(m)
            on_indent = 0
            continue
        if not in_on:
            continue
        m = PR_KEY_RE.match(line)
        if not m or _indent(line) <= on_indent:
            continue
        # Inline mapping: `pull_request: {branches: [main]}`.
        inline = m.group(2).strip()
        if inline.startswith("{"):
            return [k for k in ("branches", "branches-ignore") if f"{k}:" in inline]
        # Block body: every following line indented deeper than the key.
        pr_indent = _indent(line)
        found: list[str] = []
        for nxt in lines[i + 1 :]:
            if not nxt.strip() or nxt.lstrip().startswith("#"):
                continue
            if _indent(nxt) <= pr_indent:
                break
            fm = FILTER_RE.match(nxt)
            if fm:
                found.append(fm.group(1))
        return found
    return None


def problems_in(name: str, text: str) -> list[str]:
    found = pull_request_filters(text)
    if not found:  # None (no trigger) or [] (unfiltered) are both fine
        return []
    if name in PERMITTED:
        return []
    return [
        f"{name}: `pull_request` is filtered by {', '.join(found)}. A PR based "
        f"on another branch then runs none of this workflow, and — because the "
        f"ruleset targets main — requires none of it either, so it reads as "
        f"CLEAN unverified. Drop the filter, or add {name!r} to PERMITTED with "
        f"a reason."
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
    total = sum(
        1
        for p in WORKFLOWS.glob("*.y*ml")
        if pull_request_filters(p.read_text()) is not None
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
