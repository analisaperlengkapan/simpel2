#!/usr/bin/env python3
"""Every Playwright spec on disk must be reachable from some project.

`npx playwright test` only executes specs matched by a project's `testMatch`.
A spec file that no project matches is **never collected** — it is skipped
silently, and a suite that never runs is indistinguishable from a suite that
passes. Nothing else notices: the file compiles (TypeScript is only checked when
the file is loaded), the route-coverage gate greps spec *text* for route strings
(it never asks whether the spec is collected), and `cargo` does not know the file
exists.

Demonstrated, not hypothesised: `antarmuka/perlengkapan/tests/e2e/
pemakaian-monitoring.spec.ts` — 8 assertions over the six per-role monitoring
read endpoints (the exact scoping bug class the file was written to catch) — sat
on disk matched by no project. It is the same failure shape as the orphaned
`tests/` sub-directory suites that `lint-orphan-tests` guards for Cargo
(`tests/AGENTS.md`), one toolchain over.

Derives both sides rather than trusting either to be maintained:
  * specs on disk   = `*.spec.ts` / `*.setup.ts` under each e2e dir
  * covered specs   = the union of every `testMatch` in `playwright.config.ts`

A spec matched by no project is a failure. Adding one to a project, or adding
the spec file to the repo, is the fix.

Usage:  check-playwright-projects-cover-specs.py [--self-test]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# (label, e2e dir relative to repo root)
SUITES = [
    ("perlengkapan", "antarmuka/perlengkapan/tests/e2e"),
    ("portal", "antarmuka/portal/tests/e2e"),
]

SPEC_GLOBS = ("*.spec.ts", "*.setup.ts")


def glob_to_regex(pattern: str) -> re.Pattern[str]:
    """Translate a Playwright glob (`**`, `*`, `?`) into an anchored regex."""
    out = []
    i = 0
    while i < len(pattern):
        c = pattern[i]
        if c == "*":
            if pattern[i : i + 2] == "**":
                # `**/` may match zero path segments, so `**/*.ts` also matches
                # a file at the root of testDir.
                if pattern[i : i + 3] == "**/":
                    out.append("(?:.*/)?")
                    i += 3
                    continue
                out.append(".*")
                i += 2
                continue
            out.append("[^/]*")
            i += 1
            continue
        if c == "?":
            out.append("[^/]")
            i += 1
            continue
        out.append(re.escape(c))
        i += 1
    return re.compile("^" + "".join(out) + "$")


def specs_on_disk(e2e_dir: Path) -> list[str]:
    """Relative paths of every spec/setup file, recursively."""
    found: list[str] = []
    for glob in SPEC_GLOBS:
        for p in sorted(e2e_dir.rglob(glob)):
            rel = p.relative_to(e2e_dir).as_posix()
            if rel.startswith(("node_modules/", "results/", "test-results/", "playwright-report/")):
                continue
            found.append(rel)
    return sorted(set(found))


def _array_extent(src: str, open_idx: int) -> int:
    """Index just past the `]` matching the `[` at `open_idx`.

    Brackets inside string literals are ignored — `testMatch` patterns and the
    comments around them contain `[`/`]` (`'**/*.spec.ts'`, `bg-white/[0.04]`).
    """
    depth = 0
    i = open_idx
    quote: str | None = None
    while i < len(src):
        c = src[i]
        if quote is not None:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote = None
        elif c in "'\"`":
            quote = c
        elif c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(src)


def project_blocks(config_src: str) -> str:
    """The text of the `projects: [ ... ]` array, or '' if absent.

    Only project-level `testMatch` decides what runs. The top-level `testMatch`
    is the default every project inherits — treating it as coverage would let
    `**/*.spec.ts` mark every spec covered, which is precisely the bug this
    guard exists to find (a spec on disk that no *project* matches).
    """
    m = re.search(r"\bprojects\s*:\s*\[", config_src)
    if not m:
        return ""
    open_idx = config_src.index("[", m.start())
    return config_src[open_idx:_array_extent(config_src, open_idx)]


def test_match_patterns(config_src: str) -> list[str]:
    """Every string inside a project-level `testMatch:` value."""
    patterns: list[str] = []
    block = project_blocks(config_src)
    for m in re.finditer(r"testMatch\s*:\s*(\[[^\]]*\]|'[^']*'|\"[^\"]*\")", block):
        patterns.extend(re.findall(r"['\"]([^'\"]+)['\"]", m.group(1)))
    return patterns


def uncovered_specs(config_src: str, specs: list[str]) -> list[str]:
    regexes = [glob_to_regex(p) for p in test_match_patterns(config_src)]
    return [s for s in specs if not any(r.match(s) for r in regexes)]


def check() -> bool:
    ok = True
    for label, rel_dir in SUITES:
        e2e_dir = REPO / rel_dir
        config = e2e_dir / "playwright.config.ts"
        if not config.is_file():
            print(f"  ✗ {label}: no playwright.config.ts at {rel_dir}")
            ok = False
            continue

        specs = specs_on_disk(e2e_dir)
        missing = uncovered_specs(config.read_text(encoding="utf8"), specs)

        if missing:
            ok = False
            print(
                f"  ✗ {label}: {len(missing)} spec file(s) matched by NO project "
                f"— never collected, so never run:"
            )
            for s in missing:
                print(f"        {rel_dir}/{s}")
            print("      fix: add a project with testMatch for it (see playwright.config.ts)")
        else:
            print(f"  ok {label}: all {len(specs)} spec file(s) reachable from a project")
    return ok


def self_test() -> bool:
    """Prove the guard goes red on the exact regression it claims to catch."""
    ok = True
    label, rel_dir = SUITES[0]
    e2e_dir = REPO / rel_dir
    config_src = (e2e_dir / "playwright.config.ts").read_text(encoding="utf8")
    specs = specs_on_disk(e2e_dir)

    # 1. Baseline: the committed config must cover everything.
    baseline_missing = uncovered_specs(config_src, specs)
    if baseline_missing:
        print(f"  ✗ self-test: baseline already red: {baseline_missing}")
        return False
    print("  ok detects: nothing (healthy baseline is green)")

    # 2. Remove ONE project's testMatch and assert the guard now names its spec.
    victim = "pemakaian-monitoring.spec.ts"
    if victim not in specs:
        print(f"  ✗ self-test: expected {victim} on disk to use as the canary")
        return False
    mutated = re.sub(
        r"testMatch\s*:\s*\[\s*'\*\*/" + re.escape(victim) + r"'\s*\]",
        "testMatch: []",
        config_src,
    )
    if mutated == config_src:
        print(f"  ✗ self-test: could not find a testMatch for {victim} to remove")
        return False
    caught = uncovered_specs(mutated, specs)
    if victim not in caught:
        print(f"  ✗ self-test: guard did NOT notice {victim} losing its project")
        ok = False
    else:
        print(f"  ok detects: a spec whose project's testMatch was removed ({victim})")

    # 3. A brand-new spec file no project mentions must also be caught.
    caught_new = uncovered_specs(config_src, specs + ["brand-new-orphan.spec.ts"])
    if "brand-new-orphan.spec.ts" not in caught_new:
        print("  ✗ self-test: guard did NOT notice a newly added orphan spec")
        ok = False
    else:
        print("  ok detects: a newly added spec file with no project")

    return ok


def main() -> int:
    if "--self-test" in sys.argv:
        return 0 if self_test() else 1

    if check():
        print("check-playwright-projects-cover-specs: every spec is reachable from a project.")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
