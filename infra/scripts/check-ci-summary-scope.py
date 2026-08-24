#!/usr/bin/env python3
"""Fail when a job is missing from its required-check summary.

WHY THIS EXISTS
---------------
`CI Summary` and `Security Summary` are the ONLY TWO required status checks on
`main`. Both aggregate by way of a hand-written `needs:` list, so a job missing
from one may go red without blocking a single merge.

That is not hypothetical. In run 32673074609 (main @202aea6f) the job
`E2E simpelv1 ↔ gateway` FAILED while `CI Summary` reported SUCCESS. Four
cross-service e2e jobs had never been added to the list, and `e2e-build-images`
was absent too — which is worse than it sounds, because when it fails the e2e
jobs below it are SKIPPED, and the aggregator counts `skipped` as a pass.

The defect is the SHAPE of the list, not the five names: a hand-written `needs:`
does not grow when a job is added. So derive it mechanically — every job in the
file must be gated, except a short allowlist that has to state its reason.

Same failure mode the neighbouring guards exist for: check-crate-ci-coverage.py
(hand-written `-p` matrices) and the Dependabot-prefix check in maintenance.yml
(whose comment notes the title job "is not one of the required status checks, so
the red never blocked a merge" — this is that bug one level up).

STDLIB ONLY, deliberately: the ARC runner image ships neither PyYAML nor yq, so
importing yaml here would exit 127 in CI. Job keys are matched the same way
check-crate-ci-coverage.py matches them.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT_DIR / ".github" / "workflows"

# `  job-id:` at exactly two spaces of indent — the job keys in ci.yml. Only
# inside the top-level `jobs:` block: `on:` holds `  push:` / `  pull_request:`
# at the very same indent, and counting those as jobs makes this guard demand
# that trigger names be gated.
TOP_KEY_RE = re.compile(r"^([A-Za-z0-9_-]+):")
JOB_RE = re.compile(r"^ {2}([A-Za-z0-9_-]+):\s*$")
NEEDS_KEY_RE = re.compile(r"^ {4}needs:\s*(.*)$")
NEEDS_ITEM_RE = re.compile(r"^ {6}-\s+([A-Za-z0-9_-]+)\s*$")

# Both required status checks, each with the jobs that intentionally do NOT
# gate merges. Every exception states why. Keep these short: if you are tempted
# to add one, first ask whether that job going red really ought to let a PR
# through.
#
# NOTE this is about PRESENCE in `needs:`, not about blocking-vs-advisory.
# `security-summary` deliberately treats cargo-audit/cargo-deny/semgrep/trivy-fs
# and friends as advisory (an advisory DB moving on its own must not redden
# every unrelated PR) — that is a recorded decision, and it still requires the
# job to be listed so its result is reported at all.
GATES = {
    "ci.yml": (
        "ci-summary",
        {
            "cleanup-runner": (
                "housekeeping (`if: always()`; wipes target/, dist/, dangling "
                "images). Its failure says nothing about the code, and it runs "
                "AFTER everything else by design — gating on it would make the "
                "gate wait for cleanup."
            ),
        },
    ),
    "security.yml": ("security-summary", {}),
}


def parse(text: str, summary_job: str) -> tuple[set[str], set[str]]:
    """Return (all job ids, the summary job's needs)."""
    jobs: set[str] = set()
    needs: set[str] = set()
    current: str | None = None
    in_needs = False
    in_jobs = False

    for line in text.splitlines():
        top = TOP_KEY_RE.match(line)
        if top:
            in_jobs = top.group(1) == "jobs"
            current, in_needs = None, False
            continue

        job = JOB_RE.match(line) if in_jobs else None
        if job:
            current = job.group(1)
            jobs.add(current)
            in_needs = False
            continue

        if current != summary_job:
            continue

        if in_needs:
            item = NEEDS_ITEM_RE.match(line)
            if item:
                needs.add(item.group(1))
                continue
            # Comments interleaved in the list are fine; anything else ends it.
            if line.lstrip().startswith("#"):
                continue
            in_needs = False

        key = NEEDS_KEY_RE.match(line)
        if key:
            inline = key.group(1).strip()
            if inline.startswith("["):
                needs.update(
                    n.strip()
                    for n in inline.strip("[]").split(",")
                    if n.strip()
                )
            else:
                in_needs = True

    return jobs, needs


def check_one(filename: str, summary_job: str, allowed: dict[str, str]) -> int:
    path = WORKFLOWS / filename
    jobs, needs = parse(path.read_text(encoding="utf-8"), summary_job)

    # A parser that silently finds nothing would turn this guard into a
    # rubber stamp, so treat "found nothing" as the failure it is.
    if summary_job not in jobs:
        print(f"FAIL {filename}: job `{summary_job}` not found — renamed? "
              "this guard must be updated with it.")
        return 1
    if not needs:
        print(f"FAIL {filename}: parsed no `needs:` for `{summary_job}` — the "
              "list moved or changed shape, and this guard was silently "
              "measuring nothing.")
        return 1

    everything = jobs - {summary_job}
    ungated = everything - needs - set(allowed)
    phantom = needs - everything

    rc = 0
    if phantom:
        print(f"FAIL {filename}: `{summary_job}.needs` names jobs that do not "
              "exist (a rename drops a job from the gate silently):")
        for j in sorted(phantom):
            print(f"  - {j}")
        rc = 1

    if ungated:
        print(f"FAIL {filename}: {len(ungated)} job(s) are not reported by "
              f"`{summary_job}`, so they may go red without blocking a merge:")
        for j in sorted(ungated):
            print(f"  - {j}")
        print(f"\nAdd them to `{summary_job}.needs` in .github/workflows/"
              f"{filename}, or — if they genuinely must not be reported — to "
              "GATES in this script, WITH the reason.")
        rc = 1

    if rc == 0:
        print(f"OK {filename}: {len(needs)} job reported by `{summary_job}`, "
              f"{len(allowed)} excluded with a written reason.")
    return rc


def main() -> int:
    rc = 0
    for filename, (summary_job, allowed) in GATES.items():
        rc |= check_one(filename, summary_job, allowed)
    return rc


if __name__ == "__main__":
    sys.exit(main())
