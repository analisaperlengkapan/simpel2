#!/usr/bin/env python3
"""Keep CI scoping honest: .github/ci-scope.json, ci.yml and the scripts must agree.

ci-scope.py decides which Rust units and e2e suites a change needs; ci.yml acts on
that decision job by job. Two ways this silently stops working, both caught here:

  * a job that was never wired to the decision runs on EVERY pull request again
    (a new `test-lib-foo` job copied from a neighbour without its `if:`), so the
    queue fills up and nobody notices which job did it;
  * the data drifts from the code it describes — a crate that belongs to no unit,
    an e2e image the table does not know, a cache name the build script spells
    differently — and the scope is computed from a picture that is no longer true.

The direction of every failure of ci-scope.py itself is "run more" (see its header).
This guard exists so that wiring mistakes cannot quietly turn that into "run
everything, always", or worse, into "skip something".

DEPENDENCIES: Python standard library only (the ARC runner image has no PyYAML);
ci.yml is scanned as text, as in check-crate-ci-coverage.py.
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
BUILD_SCRIPT = ROOT / "infra" / "scripts" / "ci-build-e2e-images.sh"
COMPOSE = ROOT / "docker-compose.yml"

spec = importlib.util.spec_from_file_location("ci_scope", ROOT / "infra" / "scripts" / "ci-scope.py")
ci_scope = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci_scope)  # type: ignore[union-attr]

JOB_RE = re.compile(r"^ {2}([A-Za-z0-9_-]+):\s*$", re.M)

# Jobs that run per unit / per suite and therefore MUST be gated on the decision.
# Matched by id prefix so a new job in the family is caught the day it is added.
# `build-docker` is deliberately absent: it is gated on the ref (main/schedule), and
# `e2e-build-images` on the union of the suites that run.
SCOPED_PREFIXES = ("test-", "build-antarmuka-", "build-backend", "clippy-", "e2e-")
SCOPED_EXEMPT = {"build-docker", "e2e-build-images"}

# Matrix jobs: job id -> detect output holding its matrix, and the output that says
# whether the matrix has any entry (an empty `include` is a workflow error).
MATRIX_JOBS = {
    "clippy-rust": ("clippy-matrix", "clippy-any"),
    "test-services": ("test-matrix", "test-any"),
    "build-backend": ("build-backend-matrix", "build-backend-any"),
}


def job_bodies(text: str) -> dict[str, str]:
    spans = [(m.group(1), m.start()) for m in JOB_RE.finditer(text)]
    out: dict[str, str] = {}
    for i, (job, start) in enumerate(spans):
        end = spans[i + 1][1] if i + 1 < len(spans) else len(text)
        out[job] = text[start:end]
    return out


def job_if(body: str) -> str:
    """The text of the job-level `if:` (single-line or block), comments dropped."""
    lines = body.splitlines()
    for i, line in enumerate(lines):
        if re.match(r"^ {4}if:", line):
            parts = [line]
            for follow in lines[i + 1:]:
                if follow.startswith("     ") or not follow.strip():
                    parts.append(follow)
                else:
                    break
            return "\n".join(p for p in parts if not p.lstrip().startswith("#"))
    return ""


def check(workflow_text: str, cfg: dict, graph: dict, build_script: str, compose: str) -> list[str]:
    problems: list[str] = []
    err = lambda msg: problems.append(f"::error file=.github/workflows/ci.yml::{msg}")  # noqa: E731

    for problem in ci_scope.validate(cfg, graph):
        problems.append(f"::error file=.github/ci-scope.json::{problem}")

    bodies = job_bodies(workflow_text)
    units = list(cfg["units"])

    # 1. Every scoped job is wired to the decision.
    for job, body in bodies.items():
        if job in SCOPED_EXEMPT or not job.startswith(SCOPED_PREFIXES):
            continue
        cond = job_if(body)
        if job in MATRIX_JOBS:
            matrix_out, any_out = MATRIX_JOBS[job]
            if f"needs.detect.outputs.{any_out}" not in cond:
                err(f"job `{job}` must be gated on needs.detect.outputs.{any_out} (an empty matrix is a workflow error)")
            if f"fromJson(needs.detect.outputs.{matrix_out})" not in body:
                err(f"job `{job}` must take its matrix from needs.detect.outputs.{matrix_out}")
            continue
        if job.startswith("e2e-"):
            if f"contains(fromJson(needs.detect.outputs.e2e-jobs), '{job}')" not in cond:
                err(f"e2e suite `{job}` is not gated on its own id in needs.detect.outputs.e2e-jobs: it would run on every PR")
            continue
        # per-unit jobs: test-<unit> / build-<unit>
        unit = job.split("-", 1)[1] if job.startswith("test-") else job[len("build-"):]
        if unit not in units:
            err(f"job `{job}` looks per-unit but `{unit}` is not a unit in ci-scope.json — wire it or exempt it deliberately")
            continue
        if f"contains(fromJson(needs.detect.outputs.units), '{unit}')" not in cond:
            err(f"job `{job}` is not gated on unit `{unit}` (needs.detect.outputs.units): it would run on every PR")

    for job in MATRIX_JOBS:
        if job not in bodies:
            err(f"matrix job `{job}` no longer exists; update MATRIX_JOBS in check-ci-scope.py and check-crate-ci-coverage.py")

    # 2. Every e2e suite pulls images ci-scope.json knows, and at least one.
    images = {k: v for k, v in cfg["images"].items() if not k.startswith("_")}
    suites = ci_scope.e2e_images_by_job(workflow_text)
    for job, pulled in suites.items():
        if not pulled:
            err(f"e2e suite `{job}` has no readable `for img in simpel2-...; do docker pull` loop: it would always run")
        for image in pulled:
            if image not in images:
                err(f"e2e suite `{job}` pulls `{image}`, which is not in ci-scope.json images")

    # 3. The build script's cache names are the unit names of the images.
    declared = dict(re.findall(r"\[([A-Za-z0-9_-]+)\]=([A-Za-z0-9_-]+)", build_script))
    expected = {spec_["service"]: spec_["unit"] for spec_ in images.values()}
    if declared != expected:
        problems.append(
            "::error file=infra/scripts/ci-build-e2e-images.sh::CACHE_NAME "
            f"{declared} must equal the service->unit map of ci-scope.json images {expected}"
        )

    # 4. Every image compose builds for the e2e stack is known (and vice versa).
    in_compose = set(re.findall(r"image:\s*(simpel2-[A-Za-z0-9_-]+):local", compose))
    for image in sorted(in_compose - set(images)):
        problems.append(f"::error file=docker-compose.yml::`{image}` is built by compose but missing from ci-scope.json images")
    for image in sorted(set(images) - in_compose):
        problems.append(f"::error file=.github/ci-scope.json::images lists `{image}`, which docker-compose.yml does not build")

    return problems


def self_test(workflow_text: str, cfg: dict, graph: dict, build_script: str, compose: str) -> int:
    """Each mutation below is a mistake this guard exists to catch."""
    failures = 0

    def expect(description: str, mutated_text: str | None = None, mutated_cfg: dict | None = None,
               mutated_script: str | None = None, needle: str = "") -> None:
        nonlocal failures
        found = check(mutated_text if mutated_text is not None else workflow_text,
                      mutated_cfg if mutated_cfg is not None else cfg, graph,
                      mutated_script if mutated_script is not None else build_script, compose)
        if any(needle in p for p in found):
            print(f"  ok   detects: {description}")
        else:
            print(f"::error::self-test FAILED — guard did not detect: {description}")
            failures += 1

    line = "if: contains(fromJson(needs.detect.outputs.units), 'lib-core')"
    if line in workflow_text:
        expect("a per-unit test job loses its scope condition",
               mutated_text=workflow_text.replace(line, "if: true", 1), needle="test-lib-core")
    else:
        print("::error::self-test is stale: test-lib-core's scope condition is not where expected")
        failures += 1

    suite = "contains(fromJson(needs.detect.outputs.e2e-jobs), 'e2e-portal')"
    if suite in workflow_text:
        expect("an e2e suite loses its scope condition",
               mutated_text=workflow_text.replace(suite, "true", 1), needle="e2e-portal")
    else:
        print("::error::self-test is stale: e2e-portal's scope condition is not where expected")
        failures += 1

    expect("a new per-unit job is copied without the condition",
           mutated_text=workflow_text + "\n  test-lib-newcomer:\n    needs: detect\n    runs-on: x\n",
           needle="test-lib-newcomer")

    no_unit = json.loads(json.dumps(cfg))
    no_unit["units"]["lib-core"]["packages"] = []
    expect("a workspace crate belongs to no unit", mutated_cfg=no_unit, needle="lib-core")

    unknown_image = json.loads(json.dumps(cfg))
    unknown_image["images"].pop("simpel2-portal")
    expect("an e2e suite pulls an image the table does not know", mutated_cfg=unknown_image, needle="simpel2-portal")

    expect("the build script spells a cache name differently",
           mutated_script=build_script.replace("[portal]=antarmuka-portal", "[portal]=portal", 1),
           needle="CACHE_NAME")
    return failures


def main() -> int:
    workflow_text = WORKFLOW.read_text(encoding="utf-8")
    cfg = ci_scope.load_config(ROOT)
    graph = ci_scope.load_workspace(ROOT)
    build_script = BUILD_SCRIPT.read_text(encoding="utf-8")
    compose = COMPOSE.read_text(encoding="utf-8")

    if self_test(workflow_text, cfg, graph, build_script, compose):
        return 1
    problems = check(workflow_text, cfg, graph, build_script, compose)
    for problem in problems:
        print(problem)
    if problems:
        print(f"\n{len(problems)} CI scoping problem(s) found.", file=sys.stderr)
        return 1
    scoped = sum(
        1 for j in job_bodies(workflow_text)
        if j.startswith(SCOPED_PREFIXES) and j not in SCOPED_EXEMPT
    )
    print(f"CI scoping is consistent: {len(cfg['units'])} units, {scoped} scoped jobs, "
          f"{len(ci_scope.e2e_images_by_job(workflow_text))} e2e suites.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
