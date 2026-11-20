#!/usr/bin/env python3
"""Keep the Rust-only jobs of security.yml and autofix.yml honestly gated.

cargo-audit, cargo-deny, Verus (security.yml) and the Rust autofixer (autofix.yml)
cost 7, 7, 7 and ~27 runner-minutes and can only react to Rust sources, Cargo
manifests, their own config and the workflow that runs them. They are skipped on a
pull request that touches none of those. Two ways that goes wrong silently, both
caught here:

  * a filter that does not cover something the job reads (`.cargo/audit.toml`
    drops out, a new crate with a `verus!` proof is not matched): the PR that
    breaks the job's answer is the PR that never runs it;
  * a gate that stops failing closed: a `changes` job whose failure leaves every
    gated job SKIPPED and the Security Summary green, or a scope step that no
    longer runs everything for push/schedule/dispatch or for a diff long enough to
    have been truncated.

What each job reads is DERIVED, not listed: the workspace manifests, the files that
contain `verus!`, the crates' sources, and every `infra/scripts/*` file and local
action (`./.github/actions/*`) the job's own steps invoke. A hand-written list would
go stale the day a crate is added or a step starts calling one more script, which is
the failure this guard exists to catch (review of #1120: cargo-audit ran
install-cargo-audit.sh and check-audit-ignores-live.py, and neither was in the filter,
while this guard passed because it listed only retry-dependency-check.py).

STDLIB ONLY (the ARC runner image has no PyYAML); workflows are scanned as text.
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"

_spec = importlib.util.spec_from_file_location("check_ci_scope", ROOT / "infra" / "scripts" / "check-ci-scope.py")
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)  # type: ignore[union-attr]
ci_scope = cs.ci_scope

# workflow file -> {gated job id: the filter it is gated on}
GATES = {
    "security.yml": {"cargo-audit": "rust-deps", "cargo-deny": "rust-deps", "verus-verify": "rust-source"},
    "autofix.yml": {"rust": "rust"},
}
# Files the gated jobs read that must exist (a rename makes this list stale: say so).
MUST_EXIST = {
    "rust-deps": ["Cargo.lock", "deny.toml", ".cargo/audit.toml", "infra/scripts/retry-dependency-check.py"],
    "rust-source": ["Cargo.lock"],
    "rust": ["Cargo.lock", "infra/scripts/check-cargo-versioning.py"],
}


def filters_of(text: str) -> dict[str, list[str]]:
    """{filter name: [globs]} from the `filters: |` block of a paths-filter step."""
    out: dict[str, list[str]] = {}
    lines = text.splitlines()
    for i, line in enumerate(lines):
        m = re.match(r"^(\s*)filters:\s*\|\s*$", line)
        if not m:
            continue
        base = len(m.group(1))
        current = None
        for follow in lines[i + 1:]:
            if follow.strip() and len(follow) - len(follow.lstrip()) <= base:
                break
            fm = re.match(r"^\s*([A-Za-z0-9_-]+):\s*$", follow)
            if fm:
                current = fm.group(1)
                out.setdefault(current, [])
                continue
            pm = re.match(r"^\s*-\s*'([^']+)'\s*$", follow)
            if pm and current:
                out[current].append(pm.group(1))
    return out


def reads(filter_name: str, graph: dict) -> set[str]:
    """What the jobs gated on `filter_name` read, derived from the repository."""
    paths = {p for p in MUST_EXIST.get(filter_name, [])}
    paths |= {f"{info['dir']}/Cargo.toml" for info in graph.values()}
    paths.add("Cargo.toml")
    if filter_name == "rust-source":
        for f in ROOT.rglob("*.rs"):
            rel = f.relative_to(ROOT).as_posix()
            if "/target/" in f"/{rel}" or "/node_modules/" in f"/{rel}":
                continue
            try:
                if "verus!" in f.read_text(encoding="utf-8", errors="ignore"):
                    paths.add(rel)
            except OSError:
                pass
    if filter_name in ("rust", "rust-source"):
        for info in graph.values():
            for entry in ("src/lib.rs", "src/main.rs"):
                if (ROOT / info["dir"] / entry).is_file():
                    paths.add(f"{info['dir']}/{entry}")
                    break
    return paths


SCRIPT_REF = re.compile(r"infra/scripts/[A-Za-z0-9_.-]+\.(?:py|sh)")
ACTION_REF = re.compile(r"\./\.github/actions/([A-Za-z0-9_-]+)")


def invoked_by(job_body: str) -> set[str]:
    """Repo files a job's own steps call: scripts under infra/scripts and local actions."""
    code = "\n".join(ln for ln in job_body.splitlines() if not ln.lstrip().startswith("#"))
    out = set(SCRIPT_REF.findall(code))
    out |= {f".github/actions/{name}/action.yml" for name in ACTION_REF.findall(code)}
    return out


def check(texts: dict[str, str], graph: dict, exists=lambda p: (ROOT / p).exists()) -> list[str]:
    problems: list[str] = []
    for wf, gated in GATES.items():
        text = texts[wf]
        file = f".github/workflows/{wf}"
        err = lambda msg, file=file: problems.append(f"::error file={file}::{msg}")  # noqa: E731
        bodies = cs.job_bodies(text)
        if "changes" not in bodies:
            err("no `changes` job: nothing decides when the Rust-only jobs run")
            continue
        filters = filters_of(bodies["changes"])
        scope = bodies["changes"]
        if not re.search(r"-ge\s+2900\b", scope):
            err("the `changes` scope step no longer runs everything for a file list that may be truncated (2900)")
        if wf == "security.yml" and not re.search(r'!=\s*"pull_request"', scope):
            err("the `changes` scope step no longer makes push, schedule and dispatch runs full")
        if "all" not in filters:
            err("the `changes` job lacks the catch-all `all` filter its truncation check counts")

        for job, flt in gated.items():
            body = bodies.get(job)
            if body is None:
                err(f"gated job `{job}` no longer exists; update GATES in check-rust-job-gating.py")
                continue
            needs = re.search(r"^ {4}needs:\s*(.*)$", body, re.M)
            if not needs or "changes" not in needs.group(0) + body.split("steps:")[0]:
                err(f"job `{job}` does not `needs: changes`: its gate would read an output that does not exist")
            if f"needs.changes.outputs.{flt}" not in cs.job_if(body):
                err(f"job `{job}` is not gated on needs.changes.outputs.{flt}: it would run on every pull request")
            patterns = filters.get(flt)
            if not patterns:
                err(f"filter `{flt}` (gate of `{job}`) is missing from the `changes` job")
                continue
            compiled = [ci_scope.glob_to_regex(p) for p in patterns]
            for must in MUST_EXIST.get(flt, []):
                if not exists(must):
                    err(f"`{must}` no longer exists; update MUST_EXIST in check-rust-job-gating.py")
            for path in sorted(reads(flt, graph) | invoked_by(body) | {f".github/workflows/{wf}"}):
                if exists(path) and not any(rx.match(path) for rx in compiled):
                    err(f"`{job}` reads `{path}`, but no pattern of the `{flt}` filter matches it: a pull request that "
                        "changes only that file would skip the job that depends on it")

    sec = texts["security.yml"]
    summary = cs.job_bodies(sec).get("security-summary", "")
    if not re.search(r"^\s*-\s*changes\s*$", summary, re.M):
        problems.append("::error file=.github/workflows/security.yml::security-summary does not `needs: changes`")
    blocking = re.search(r'BLOCKING="([^"]*)"', summary)
    if not blocking or "changes" not in blocking.group(1).split():
        problems.append(
            "::error file=.github/workflows/security.yml::`changes` is not in BLOCKING: if the scope job fails, "
            "cargo-audit/cargo-deny/verus are SKIPPED and the Security Summary turns green with nothing checked"
        )
    return problems


def self_test(texts: dict[str, str], graph: dict) -> int:
    failures = 0

    def expect(description: str, needle: str, wf: str, old: str, new: str) -> None:
        nonlocal failures
        if texts[wf].count(old) < 1:
            print(f"::error::self-test is stale: `{old[:50]}` not found in {wf}")
            failures += 1
            return
        mutated = dict(texts)
        mutated[wf] = texts[wf].replace(old, new, 1)
        if any(needle in p for p in check(mutated, graph)):
            print(f"  ok   detects: {description}")
        else:
            print(f"::error::self-test FAILED — guard did not detect: {description}")
            failures += 1

    expect("`.cargo/**` drops out of rust-deps (cargo-audit reads .cargo/audit.toml)", "audit.toml",
           "security.yml", "              - '.cargo/**'\n              - 'rust-toolchain*'\n", "              - 'rust-toolchain*'\n")
    expect("cargo-audit's installer drops out of rust-deps (the miss found in review)", "install-cargo-audit.sh",
           "security.yml", "              - 'infra/scripts/install-cargo-audit.sh'\n", "")
    expect("cargo-audit's ignore checker drops out of rust-deps", "check-audit-ignores-live.py",
           "security.yml", "              - 'infra/scripts/check-audit-ignores-live.py'\n", "")
    expect("a Verus source is no longer matched by rust-source", ".rs",
           "security.yml", "              - '**/*.rs'\n", "")
    expect("cargo-deny loses its gate", "cargo-deny",
           "security.yml", "  cargo-deny:\n    name: cargo-deny\n    needs: changes\n    if: needs.changes.outputs.rust-deps == 'true'\n",
           "  cargo-deny:\n    name: cargo-deny\n    needs: changes\n    if: true\n")
    expect("a failing scope job would leave the summary green", "BLOCKING",
           "security.yml", 'BLOCKING="gitleaks helm-lint changes"', 'BLOCKING="gitleaks helm-lint"')
    expect("the scope step stops failing open on a truncated list", "truncated",
           "security.yml", 'if [ "$n" -ge 2900 ]', 'if [ "$n" -ge 29000 ]')
    expect("push, schedule and dispatch stop being full", "push, schedule",
           "security.yml", 'if [ "$EVENT" != "pull_request" ]', 'if [ "$EVENT" = "never" ]')
    expect("the Rust autofixer loses its gate", "`rust`",
           "autofix.yml", " && needs.changes.outputs.rust == 'true'\n", "\n")
    return failures


def main() -> int:
    texts = {wf: (WORKFLOWS / wf).read_text(encoding="utf-8") for wf in GATES}
    graph = ci_scope.load_workspace(ROOT)
    if self_test(texts, graph):
        return 1
    problems = check(texts, graph)
    for p in problems:
        print(p)
    if problems:
        print(f"\n{len(problems)} Rust job gating problem(s) found.", file=sys.stderr)
        return 1
    verus = sorted(p for p in reads("rust-source", graph) if p.endswith(".rs") and "verus!" in (ROOT / p).read_text(errors="ignore"))
    print(f"Rust job gating is consistent: {sum(len(g) for g in GATES.values())} gated jobs, "
          f"{len(graph)} crates, {len(verus)} Verus source(s) covered by the filters.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
