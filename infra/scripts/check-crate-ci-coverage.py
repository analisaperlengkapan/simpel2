#!/usr/bin/env python3
"""Fail when a workspace crate is not covered by the CI cargo gates.

WHY THIS EXISTS
---------------
`.github/workflows/ci.yml` names the crates it lints and tests in hand-written
matrices (`packages: "-p a -p b ..."`). A crate added to `Cargo.toml` but not to
those matrices is silently ungated: it never gets clippy'd and its tests never
run, while CI stays green. That is exactly what happened to
`secreton-auto-unseal` (6.5k lines incl. a 700-line integration suite) and
`secreton-k8s-operator` (933 lines) — absent from BOTH matrices, so their only
compile in CI came from a `--workspace` release build that runs after merge.

A comment saying "remember to add new crates to the matrix" would rot the same
way the matrix did. So derive both sides mechanically: the crate list from the
workspace manifest, the covered set from the workflow file, and diff them.

WHAT IT CHECKS
--------------
For every workspace member:
  * clippy coverage  — some CI job that runs `cargo clippy` names it;
  * test coverage    — some CI job that runs `cargo test` names it,
                       unless it is in COMPILE_ONLY (see below), in which case
                       some job must at least `cargo build`/`cargo check` it;
  * doctest coverage — required only for crates that HAVE a runnable doc
                       example, and only if they have a lib target. Whether a
                       crate has one is read from its sources, not declared:
                       add the first ```rust example to a crate with no `--doc`
                       step and this turns red.
It also flags a `-p <name>` in the workflow that is not a workspace member,
which is how a crate rename quietly drops a crate from a matrix.

Deriving the doctest requirement (rather than adding a `--doc` step everywhere)
is not just tidiness. `perlengkapan-microfrontend` is otherwise built for wasm32
only, so a `--doc` pass there compiles an entire second, host-target dependency
tree — and it did so for ZERO doctests, because the crate's only doc fence is
```rust,ignore. That step starved its 6Gi ARC runner to death ("the self-hosted
runner lost communication with the server"). A gate must not cost more than it
gates.

DEPENDENCIES
------------
Python standard library only — `tomllib` (3.11+) and `re`. Deliberately NO YAML
parser: the ARC runner image ships no PyYAML, and a CI guard that cannot run on
the runner is not a guard (see the actionlint incident in maintenance.yml). The
workflow is scanned as TEXT; `actionlint` already proves it is valid YAML.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    import tomli as tomllib  # type: ignore

ROOT_DIR = Path(__file__).resolve().parents[2]
ROOT_MANIFEST = ROOT_DIR / "Cargo.toml"
WORKFLOW = ROOT_DIR / ".github" / "workflows" / "ci.yml"

# Crates that cannot run host `cargo test`, with the reason. They must still be
# compiled by SOME job — `cargo check`/`cargo build` counts for these and only
# these. Keep this list short and justified; it is the one hand-written part.
COMPILE_ONLY = {
    "lib-ui": "Leptos lib for the WASM frontends; CI compiles it with --features ssr",
    "portal-microfrontend": "wasm32 CSR binary; host test targets pull native tokio/mio",
    "perlengkapan-microfrontend": "wasm32 CSR binary; host test targets pull native tokio/mio",
}

# `  job-id:` at exactly two spaces of indent — the job keys in ci.yml.
JOB_RE = re.compile(r"^ {2}([A-Za-z0-9_-]+):\s*$")
CARGO_RE = re.compile(r"\bcargo\s+(clippy|test|build|check)\b")
# `cargo test --doc` is still the `test` subcommand, so it needs its own probe;
# cargo rejects `--doc` next to any other target selector, which is why it is
# always a separate step and can be detected on the same line as the `cargo`.
DOC_RE = re.compile(r"\bcargo\s+test\b[^\n]*--doc\b")
# `- name: ...` / `name: ...` — a display string, never an executed command.
NAME_KEY_RE = re.compile(r"^\s*(?:-\s+)?name:")
# `-p <crate>`. At least one letter is required so `docker run -p 5433:5432`
# does not read as a package named "5433" — the port-mapping flag is spelled
# identically. Lines invoking docker are skipped as well; belt and braces,
# because a future `-p my-svc:8080` would otherwise look like a crate.
PKG_RE = re.compile(r"-p\s+([A-Za-z0-9_-]*[A-Za-z][A-Za-z0-9_-]*)(?![\w:.-])")


# A fence in a doc comment: `/// ```info` or `//! ```info`.
FENCE_RE = re.compile(r"^\s*(?:///|//!)\s*```(.*)$")
# Info-string tokens rustdoc understands as doctest attributes. Anything else
# (`text`, `bash`, `json`, `mermaid`, ...) marks the block as another language,
# which rustdoc does not compile.
DOCTEST_ATTRS = {
    "rust",
    "should_panic",
    "no_run",
    "compile_fail",
    "test_harness",
    # `ignore` is handled separately: it is the one attribute that means "do not
    # compile this", which is exactly what makes such a block not a gate.
}
EDITION_RE = re.compile(r"^edition\d{4}$")


def is_runnable_doctest(info: str) -> bool:
    """Would rustdoc compile a fence carrying this info string?

    `no_run` and `compile_fail` count: both are compiled, and compiling is the
    part a CI gate is protecting. Only `ignore` opts out of compilation.
    """
    tokens = [tok for tok in re.split(r"[,\s]+", info.strip()) if tok]
    if not tokens:
        return True
    if "ignore" in tokens:
        return False
    return all(tok in DOCTEST_ATTRS or EDITION_RE.match(tok) for tok in tokens)


def has_runnable_doctest(manifest: Path) -> bool:
    """True if the crate has a lib target carrying at least one live example.

    `cargo test --doc` only ever looks at the lib target, so a bin-only crate
    cannot have doctests no matter what its doc comments say.
    """
    src = manifest.parent / "src"
    if not (src / "lib.rs").is_file():
        return False

    for path in sorted(src.rglob("*.rs")):
        inside = False
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            fence = FENCE_RE.match(line)
            if not fence:
                continue
            if inside:  # closing fence; its info string is meaningless
                inside = False
                continue
            inside = True
            if is_runnable_doctest(fence.group(1)):
                return True
    return False


def workspace_members() -> dict[str, Path]:
    """Map crate name -> manifest path for every workspace member."""
    with ROOT_MANIFEST.open("rb") as handle:
        root = tomllib.load(handle)

    members: dict[str, Path] = {}
    for rel in root["workspace"]["members"]:
        manifest = ROOT_DIR / rel / "Cargo.toml"
        if not manifest.is_file():
            print(f"::error::workspace member {rel} has no Cargo.toml", file=sys.stderr)
            continue
        with manifest.open("rb") as handle:
            members[tomllib.load(handle)["package"]["name"]] = manifest
    return members


def coverage_from_workflow(workflow_text: str) -> dict[str, set[str]]:
    """Map cargo subcommand -> crate names named by a job that runs it.

    Attribution is per JOB, not per line: the matrices hold the crate names in
    `packages:` values while the `cargo ...` invocation a few lines below refers
    to them as `${{ matrix.packages }}`. Every job in this workflow runs exactly
    one kind of cargo subcommand, so job-level attribution is exact here; a job
    mixing `cargo build` and `cargo test` would over-attribute, which the naming
    convention (`build-*` vs `test-*`) makes obvious in review.
    """
    coverage: dict[str, set[str]] = {}
    subcommands: set[str] = set()
    packages: set[str] = set()

    def flush() -> None:
        for sub in subcommands:
            coverage.setdefault(sub, set()).update(packages)

    for line in workflow_text.splitlines():
        job = JOB_RE.match(line)
        if job:
            flush()
            subcommands, packages = set(), set()
            continue
        # A comment can mention `cargo test -p foo` while describing why a crate
        # is NOT covered; counting it would let prose satisfy the gate.
        if line.lstrip().startswith("#"):
            continue
        # Step display names echo the command they wrap (`- name: cargo test
        # --doc (host)`), so a job whose `run:` was deleted but whose `name:`
        # survived would still look covered. Caught by testing the guard against
        # the very regression it exists to catch, which is the only way to know
        # a guard works.
        if NAME_KEY_RE.match(line):
            continue
        if "docker " in line:
            continue
        subcommands.update(CARGO_RE.findall(line))
        if DOC_RE.search(line):
            subcommands.add("doc")
        packages.update(PKG_RE.findall(line))
    flush()
    return coverage


def check(
    workflow_text: str, members: dict[str, Path], doctest_crates: set[str]
) -> list[str]:
    """Return one message per coverage problem; empty means fully gated."""
    coverage = coverage_from_workflow(workflow_text)

    linted = coverage.get("clippy", set())
    tested = coverage.get("test", set())
    doc_tested = coverage.get("doc", set())
    compiled = coverage.get("build", set()) | coverage.get("check", set()) | linted | tested

    problems: list[str] = []
    for name in sorted(members):
        if name not in linted:
            problems.append(
                f"::error file=.github/workflows/ci.yml::crate '{name}' is in no "
                f"`cargo clippy` job — add it to the clippy-rust matrix"
            )
        if name in COMPILE_ONLY:
            if name not in compiled:
                problems.append(
                    f"::error file=.github/workflows/ci.yml::crate '{name}' is "
                    f"compile-only ({COMPILE_ONLY[name]}) but no job compiles it"
                )
        elif name not in tested:
            problems.append(
                f"::error file=.github/workflows/ci.yml::crate '{name}' is in no "
                f"`cargo test` job — add it to a test matrix, or to COMPILE_ONLY "
                f"in {Path(__file__).name} with the reason it cannot be tested"
            )

        # Derived, not declared: a crate needs a `--doc` step exactly when it
        # has an example rustdoc would compile. Adding the first one to a crate
        # with no such step turns this red instead of silently going ungated.
        if name in doctest_crates and name not in doc_tested:
            problems.append(
                f"::error file=.github/workflows/ci.yml::crate '{name}' has a "
                f"runnable doc example but no `cargo test --doc` step — add one "
                f"(`--all-targets` does NOT cover doctests), or mark the example "
                f"```ignore if it is illustrative only"
            )

    for name in sorted(set().union(*coverage.values()) if coverage else set()):
        if name not in members:
            problems.append(
                f"::error file=.github/workflows/ci.yml::`-p {name}` names a crate "
                f"that is not a workspace member (renamed or removed?)"
            )

    for name in sorted(COMPILE_ONLY):
        if name not in members:
            problems.append(
                f"::error::COMPILE_ONLY lists '{name}', which is not a workspace "
                f"member — drop the stale entry"
            )

    return problems


def self_test(workflow_text: str, members: dict[str, Path], doctest_crates: set[str]) -> int:
    """Mutate the workflow in memory and assert each mutation is detected.

    A guard is only worth its runtime if it fails when it should. The `name:`
    exclusion in coverage_from_workflow exists solely because this test caught
    the guard passing a workflow whose `run: cargo test --doc` line had been
    deleted — the step's DISPLAY NAME still contained the command, so it looked
    covered. Nothing but a mutation would have surfaced that.
    """
    mutations: list[tuple[str, str, str]] = [
        (
            "drop a crate from the clippy matrix",
            "-p layanan-gateway",
            "layanan-gateway",
        ),
    ]
    # Find a real `cargo test --doc` command that names its crate literally
    # (the matrix jobs pass `${{ matrix.packages }}`, so they cannot be used to
    # target one crate). Derived from the file so the mutation cannot go stale.
    for line in workflow_text.splitlines():
        if NAME_KEY_RE.match(line) or not DOC_RE.search(line):
            continue
        named = [pkg for pkg in PKG_RE.findall(line) if pkg in doctest_crates]
        if named:
            mutations.append(
                (
                    "delete a `cargo test --doc` command, keeping the step name",
                    line,
                    named[0],
                )
            )
            break

    failures = 0
    for description, needle, expect_crate in mutations:
        if needle not in workflow_text:
            print(f"::error::self-test is stale: {needle!r} no longer in ci.yml")
            failures += 1
            continue
        mutated = workflow_text.replace(needle, "", 1)
        found = [p for p in check(mutated, members, doctest_crates) if expect_crate in p]
        if found:
            print(f"  ok   detects: {description}")
        else:
            print(f"::error::self-test FAILED — guard did not detect: {description}")
            failures += 1
    return failures


def main() -> int:
    members = workspace_members()
    workflow_text = WORKFLOW.read_text(encoding="utf-8")
    doctest_crates = {
        name for name, manifest in members.items() if has_runnable_doctest(manifest)
    }

    if self_test(workflow_text, members, doctest_crates):
        return 1

    problems = check(workflow_text, members, doctest_crates)
    for problem in problems:
        print(problem)
    if problems:
        print(f"\n{len(problems)} crate coverage problem(s) found.", file=sys.stderr)
        return 1

    print(
        f"all {len(members)} workspace crates are clippy-gated; "
        f"{len(members) - len(COMPILE_ONLY)} are test-gated, "
        f"{len(COMPILE_ONLY)} are compile-only by declaration; "
        f"{len(doctest_crates)} have runnable doc examples and all are `--doc`-gated"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
