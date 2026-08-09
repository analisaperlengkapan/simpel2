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
  * clippy coverage — some CI job that runs `cargo clippy` names it;
  * test coverage   — some CI job that runs `cargo test` names it,
                      unless it is in COMPILE_ONLY (see below), in which case
                      some job must at least `cargo build`/`cargo check` it.
It also flags a `-p <name>` in the workflow that is not a workspace member,
which is how a crate rename quietly drops a crate from a matrix.

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
# `-p <crate>`. At least one letter is required so `docker run -p 5433:5432`
# does not read as a package named "5433" — the port-mapping flag is spelled
# identically. Lines invoking docker are skipped as well; belt and braces,
# because a future `-p my-svc:8080` would otherwise look like a crate.
PKG_RE = re.compile(r"-p\s+([A-Za-z0-9_-]*[A-Za-z][A-Za-z0-9_-]*)(?![\w:.-])")


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


def coverage_from_workflow() -> dict[str, set[str]]:
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

    for line in WORKFLOW.read_text(encoding="utf-8").splitlines():
        job = JOB_RE.match(line)
        if job:
            flush()
            subcommands, packages = set(), set()
            continue
        # A comment can mention `cargo test -p foo` while describing why a crate
        # is NOT covered; counting it would let prose satisfy the gate.
        if line.lstrip().startswith("#"):
            continue
        if "docker " in line:
            continue
        subcommands.update(CARGO_RE.findall(line))
        packages.update(PKG_RE.findall(line))
    flush()
    return coverage


def main() -> int:
    members = workspace_members()
    coverage = coverage_from_workflow()

    linted = coverage.get("clippy", set())
    tested = coverage.get("test", set())
    compiled = coverage.get("build", set()) | coverage.get("check", set()) | linted | tested

    errors = 0
    for name in sorted(members):
        if name not in linted:
            print(
                f"::error file=.github/workflows/ci.yml::crate '{name}' is in no "
                f"`cargo clippy` job — add it to the clippy-rust matrix",
            )
            errors += 1
        if name in COMPILE_ONLY:
            if name not in compiled:
                print(
                    f"::error file=.github/workflows/ci.yml::crate '{name}' is "
                    f"compile-only ({COMPILE_ONLY[name]}) but no job compiles it",
                )
                errors += 1
        elif name not in tested:
            print(
                f"::error file=.github/workflows/ci.yml::crate '{name}' is in no "
                f"`cargo test` job — add it to a test matrix, or to COMPILE_ONLY "
                f"in {Path(__file__).name} with the reason it cannot be tested",
            )
            errors += 1

    for name in sorted(set().union(*coverage.values()) if coverage else set()):
        if name not in members:
            print(
                f"::error file=.github/workflows/ci.yml::`-p {name}` names a crate "
                f"that is not a workspace member (renamed or removed?)",
            )
            errors += 1

    for name in sorted(COMPILE_ONLY):
        if name not in members:
            print(
                f"::error::COMPILE_ONLY lists '{name}', which is not a workspace "
                f"member — drop the stale entry",
            )
            errors += 1

    if errors:
        print(f"\n{errors} crate coverage problem(s) found.", file=sys.stderr)
        return 1

    print(
        f"all {len(members)} workspace crates are clippy-gated; "
        f"{len(members) - len(COMPILE_ONLY)} are test-gated, "
        f"{len(COMPILE_ONLY)} are compile-only by declaration"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
