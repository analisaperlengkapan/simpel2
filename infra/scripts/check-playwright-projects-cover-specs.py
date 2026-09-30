#!/usr/bin/env python3
"""Every Playwright spec on disk must actually be collected by some project.

`npx playwright test` only executes specs matched by a project's `testMatch`
*and not excluded by that project's effective `testIgnore`*. A spec file that no
project collects is skipped silently, and a suite that never runs is
indistinguishable from a suite that passes. Nothing else notices: the file
compiles (TypeScript is only checked when the file is loaded), the
route-coverage gate greps spec *text* for route strings (it never asks whether
the spec is collected), and `cargo` does not know the file exists.

Demonstrated, not hypothesised: `antemuka/perlengkapan/tests/e2e/
pemakaian-monitoring.spec.ts` — 8 assertions over the six per-role monitoring
read endpoints (the exact scoping bug class the file was written to catch) — sat
on disk matched by no project. Same failure shape as the orphaned `tests/`
sub-directory suites that `lint-orphan-tests` guards for Cargo, one toolchain
over.

Derives both sides rather than trusting either to be maintained:
  * specs on disk = `*.spec.ts` / `*.setup.ts` under each e2e dir
  * collected     = union over projects of (its `testMatch` minus its effective
                    `testIgnore`), where a project's effective ignore is its own
                    `testIgnore` if declared, else the top-level one

The model is per-project and the source is masked before it is parsed. Both are
load-bearing, and each was a bug first:

  1. **Per project, not two flat sets.** Playwright evaluates `testMatch` and
     `testIgnore` once per project and collects the union. Pooling all match
     patterns and all ignore patterns loses the association in both directions:
     an unrelated project's `testIgnore` cancels a valid match (false failure),
     and a project-level `testIgnore: []` cannot switch off the inherited
     top-level ignore (false pass). A spec is covered when *some* project
     matches it and that same project does not ignore it.
  2. **Top-level `testMatch` is not coverage.** Only the `projects:` array
     collects; treating the top-level `testMatch` as coverage would let
     `**/*.spec.ts` mark everything covered, which is the original bug.
  3. **Comments are not configuration.** The source is masked — comments blanked,
     strings blanked *separately* — before anything is parsed, so a
     commented-out project is not read as live config and a quoted pattern
     *inside a comment* is not read as a pattern. Two masks are needed because
     the glob patterns themselves are strings: reading literals from the
     comment-masked-but-string-intact text is the only way to get the patterns,
     and reading structure from the fully-masked text is the only way to avoid
     seeing `projects:` inside a comment.
  4. **A bare-string `testMatch` is still a `testMatch`.** Strings mask to NUL
     (not whitespace) so a value lookup cannot walk *through* a blanked literal
     and miss it.

Usage:  check-playwright-projects-cover-specs.py [--self-test] [--cross-check]
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

try:
    REPO: Path | None = Path(__file__).resolve().parents[2]
except IndexError:
    # Piped in as `python3 -` from stdin: `__file__` is `<stdin>`, which has no
    # parents. Only relative suite paths need the repo root, and callers that
    # pipe the script in pass an absolute `--e2e-dir`.
    REPO = None

# (label, e2e dir relative to repo root)
SUITES = [
    ("perlengkapan", "antarmuka/perlengkapan/tests/e2e"),
    ("portal", "antarmuka/portal/tests/e2e"),
]

# Overridable via `--e2e-dir`, so the same script can run inside the Playwright
# image (where /e2e already holds the config, the specs and node_modules) and
# compare itself against `--list`. `__file__` is `<stdin>` when piped in, so REPO
# is only a default — an absolute `--e2e-dir` does not need it.
ACTIVE_SUITES = list(SUITES)

SPEC_GLOBS = ("*.spec.ts", "*.setup.ts")
SKIP_PREFIXES = ("node_modules/", "results/", "test-results/", "playwright-report/")


# ── masking ───────────────────────────────────────────────────────────────────
# Two masks, same length as the source, so every offset is valid in both:
#   * structural mask: comments AND strings blanked. Used to find structure, so
#     `projects:` inside a comment or a string cannot be mistaken for config.
#   * literal mask:    comments only blanked, strings kept. Used to read the glob
#     patterns, which are strings by nature.
# Comments always blank to spaces. Strings blank to NUL in the structural mask so
# that whitespace-skipping cannot pass through a blanked literal and miss a
# bare-string value.

def _mask(src: str, *, keep_strings: bool) -> str:
    out = list(src)
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                out[i] = " "
                i += 1
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            out[i] = out[i + 1] = " "
            i += 2
            while i < n and not (src[i] == "*" and i + 1 < n and src[i + 1] == "/"):
                out[i] = "\n" if src[i] == "\n" else " "
                i += 1
            if i < n:
                out[i] = out[i + 1] = " "
                i += 2
            continue
        if c in "'\"`":
            quote = c
            if not keep_strings:
                out[i] = "\x00"
            i += 1
            while i < n and src[i] != quote:
                if src[i] == "\\" and i + 1 < n:
                    if not keep_strings:
                        out[i] = out[i + 1] = "\x00"
                    i += 2
                    continue
                if not keep_strings:
                    out[i] = "\n" if src[i] == "\n" else "\x00"
                i += 1
            if i < n:
                if not keep_strings:
                    out[i] = "\x00"
                i += 1
            continue
        i += 1
    return "".join(out)


def structural_mask(src: str) -> str:
    """Comments and strings blanked (strings to NUL)."""
    return _mask(src, keep_strings=False)


def literal_mask(src: str) -> str:
    """Comments blanked, strings left intact."""
    return _mask(src, keep_strings=True)


def _match_bracket(src: str, open_idx: int) -> int:
    """Index just past the bracket matching `src[open_idx]` (strings pre-masked)."""
    pairs = {"[": "]", "{": "}", "(": ")"}
    close = pairs[src[open_idx]]
    depth, i = 0, open_idx
    while i < len(src):
        if src[i] == src[open_idx]:
            depth += 1
        elif src[i] == close:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(src)


def _projects_array_span(structural: str) -> tuple[int, int] | None:
    """Span of the `[...]` array assigned to `projects:`."""
    m = re.search(r"\bprojects\s*:\s*\[", structural)
    if not m:
        return None
    arr_start = structural.index("[", m.start())
    return arr_start, _match_bracket(structural, arr_start)


def _object_spans(structural: str) -> list[tuple[int, int]]:
    """Spans of the brace objects that sit *directly* in the `projects: [...]` array.

    Deliberately shallow: only the array's own elements are projects. Scanning
    for `name:` anywhere would also pick up a nested `use: { ... }`, and
    scanning for every `{` would pick up objects inside a project.
    """
    span = _projects_array_span(structural)
    if span is None:
        return []
    arr_start, arr_end = span
    spans, i = [], arr_start + 1
    while i < arr_end:
        if structural[i] == "{":
            end = _match_bracket(structural, i)
            spans.append((i, end))
            i = end
            continue
        i += 1
    return spans


def _literals_in(literal: str, structural: str, start: int, end: int) -> list[str]:
    """Glob literals in `[start, end)`, ignoring anything inside a comment.

    Strings are read from the literal mask (where they survive), but a span that
    the structural mask shows as blanked-to-spaces — i.e. a comment — is dropped
    first. That is what stops `testMatch: [/* '**/x.spec.ts' */]` from counting:
    the quoted pattern is real text, but it sits in a comment.
    """
    out: list[str] = []
    for m in re.finditer(r"'([^'\\]*)'|\"([^\"\\]*)\"", literal[start:end]):
        if structural[start + m.start()] == " ":
            continue  # inside a comment
        out.append(m.group(1) or m.group(2))
    return out


def _key_extent(structural: str, key: str, start: int, end: int) -> tuple[int, int] | None:
    """Span of the value assigned to `key` within [start, end)."""
    m = re.search(r"\b" + re.escape(key) + r"\s*:\s*", structural[start:end])
    if not m:
        return None
    vstart = start + m.end()
    while vstart < end and structural[vstart] in " \t\r\n":
        vstart += 1
    if vstart >= end:
        return None
    if structural[vstart] in "[{(":
        return vstart, _match_bracket(structural, vstart)
    return vstart, vstart


def _value_literals(literal: str, structural: str, vstart: int, vend: int) -> list[str]:
    """Literals of a value span; a zero-length span means a bare string literal."""
    if vend <= vstart:
        if vstart < len(literal) and literal[vstart] in "'\"":
            quote = literal[vstart]
            j = vstart + 1
            while j < len(literal) and literal[j] != quote:
                j += 2 if literal[j] == "\\" else 1
            vend = j + 1
        else:
            return []
    return _literals_in(literal, structural, vstart, vend)


# ── globs ─────────────────────────────────────────────────────────────────────


def glob_to_regex(pattern: str) -> re.Pattern[str]:
    """Translate a Playwright glob (`**`, `*`, `?`) into an anchored regex."""
    out, i = [], 0
    while i < len(pattern):
        c = pattern[i]
        if c == "*":
            if pattern[i : i + 2] == "**":
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


def config_model(raw_src: str) -> list[tuple[list[re.Pattern[str]], list[re.Pattern[str]]]]:
    """One `(matches, ignores)` pair per project, ignores already effective.

    Keeping the pairs together is what makes overlapping projects and
    `testIgnore` overrides come out right; see the module docstring.
    """
    structural = structural_mask(raw_src)
    literal = literal_mask(raw_src)

    # The top-level `testIgnore` is the one OUTSIDE the projects array. Searching
    # the whole file would also match the first project that declares its own,
    # which then reads as "the inherited ignore" and silently becomes every other
    # project's ignore too.
    top_ignore: list[re.Pattern[str]] = []
    span = _projects_array_span(structural)
    if span is None:
        ti = _key_extent(structural, "testIgnore", 0, len(structural))
    else:
        head = _key_extent(structural, "testIgnore", 0, span[0])
        tail = _key_extent(structural, "testIgnore", span[1], len(structural))
        ti = head or tail
    if ti is not None:
        top_ignore = [glob_to_regex(p) for p in _value_literals(literal, structural, *ti)]

    projects: list[tuple[list[re.Pattern[str]], list[re.Pattern[str]]]] = []
    for start, end in _object_spans(structural):
        tm = _key_extent(structural, "testMatch", start, end)
        if tm is None:
            continue
        matches = [glob_to_regex(p) for p in _value_literals(literal, structural, *tm)]
        own = _key_extent(structural, "testIgnore", start, end)
        ignores = top_ignore if own is None else [
            glob_to_regex(p) for p in _value_literals(literal, structural, *own)
        ]
        projects.append((matches, ignores))
    return projects


def specs_on_disk(e2e_dir: Path) -> list[str]:
    """Relative paths of every spec/setup file, recursively."""
    found: set[str] = set()
    for glob in SPEC_GLOBS:
        for p in sorted(e2e_dir.rglob(glob)):
            rel = p.relative_to(e2e_dir).as_posix()
            if rel.startswith(SKIP_PREFIXES):
                continue
            found.add(rel)
    return sorted(found)


def uncovered_specs(raw_src: str, specs: list[str]) -> list[str]:
    """Specs no single project both matches and refrains from ignoring."""
    projects = config_model(raw_src)
    out = []
    for s in specs:
        if not any(
            any(r.match(s) for r in matches) and not any(r.match(s) for r in ignores)
            for matches, ignores in projects
        ):
            out.append(s)
    return out


# ── checks ────────────────────────────────────────────────────────────────────


def suite_dir(rel_dir: str) -> Path:
    """Absolute e2e dir for a suite entry (honours the `--e2e-dir` override)."""
    p = Path(rel_dir)
    if p.is_absolute():
        return p
    if REPO is None:
        raise SystemExit(
            "relative suite paths need the repo root; pass an absolute --e2e-dir"
        )
    return REPO / p


def check() -> bool:
    ok = True
    for label, rel_dir in ACTIVE_SUITES:
        e2e_dir = suite_dir(rel_dir)
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
                f"  ✗ {label}: {len(missing)} spec file(s) NO project collects "
                f"— never collected, so never run:"
            )
            for s in missing:
                print(f"        {rel_dir}/{s}")
            print("      fix: add a project whose testMatch selects it, and make")
            print("           sure no testIgnore excludes it (see playwright.config.ts)")
        else:
            print(f"  ok {label}: all {len(specs)} spec file(s) collected by a project")
    return ok


def cross_check_against_list() -> bool | None:
    """Compare the static model to `npx playwright test --list` for the WHOLE suite.

    `--list` is the ground truth; this exists because a hand-maintained glob
    model is exactly the kind of thing that drifts. The comparison is an
    equivalence in both directions — every file the model calls collected must
    appear in `--list`, and every file `--list` reports must be one the model
    calls collected. A one-directional check would miss the model silently
    *dropping* a spec (which is the failure this whole guard is about).

    Returns None (skip) when the toolchain is absent: the invariants job has no
    Node, so this must never be the thing that makes the guard fail.
    """
    checked = 0
    for label, rel_dir in ACTIVE_SUITES:
        e2e_dir = suite_dir(rel_dir)
        if not (e2e_dir / "node_modules").is_dir():
            print(f"  .. {label}: skipped (no node_modules — cannot run --list)")
            continue
        try:
            proc = subprocess.run(
                ["npx", "playwright", "test", "--list", "--reporter=json"],
                cwd=e2e_dir, capture_output=True, text=True, timeout=600,
            )
        except (OSError, subprocess.SubprocessError) as e:
            print(f"  .. {label}: skipped (--list unavailable: {e})")
            continue
        try:
            data = json.loads(proc.stdout)
        except json.JSONDecodeError:
            print(f"  .. {label}: skipped (--list produced no JSON)")
            continue

        listed: set[str] = set()

        def walk(suites: list[dict]) -> None:
            for suite in suites:
                if suite.get("file"):
                    listed.add(suite["file"])
                for spec in suite.get("specs", []):
                    if spec.get("file"):
                        listed.add(spec["file"])
                walk(suite.get("suites", []))

        walk(data.get("suites", []))
        if not listed:
            print(f"  .. {label}: skipped (--list reported no tests)")
            continue

        specs = specs_on_disk(e2e_dir)
        model_collected = set(specs) - set(
            uncovered_specs((e2e_dir / "playwright.config.ts").read_text(encoding="utf8"), specs)
        )

        def norm(paths: set[str]) -> set[str]:
            return {p.split("tests/e2e/")[-1] for p in paths}

        listed_n, model_n = norm(listed), norm(model_collected)
        if listed_n != model_n:
            print(f"  ✗ {label}: model and --list disagree")
            for f in sorted(model_n - listed_n):
                print(f"        model says collected, --list disagrees: {f}")
            for f in sorted(listed_n - model_n):
                print(f"        --list ran it, model says uncovered: {f}")
            return False
        checked += 1
        print(f"  ok {label}: static model == --list ({len(listed_n)} file(s))")
    return True if checked else None


def self_test() -> bool:
    """Prove the guard goes red on each regression it claims to catch."""
    ok = True
    _, rel_dir = ACTIVE_SUITES[0]
    e2e_dir = suite_dir(rel_dir)
    config_src = (e2e_dir / "playwright.config.ts").read_text(encoding="utf8")
    specs = specs_on_disk(e2e_dir)
    victim = "pemakaian-monitoring.spec.ts"

    if uncovered_specs(config_src, specs):
        print("  ✗ self-test: baseline already red")
        return False
    print("  ok detects: nothing (healthy baseline is green)")
    if victim not in specs:
        print(f"  ✗ self-test: expected {victim} on disk to use as the canary")
        return False

    def expect(name: str, got: list[str], want: list[str]) -> None:
        nonlocal ok
        if got == want:
            print(f"  ok detects: {name}")
        else:
            print(f"  ✗ self-test: {name} — got {got}, want {want}")
            ok = False

    others = sorted(set(specs) - {victim})

    # 1. The original bug: the project's testMatch goes away.
    expect(
        f"a spec whose project's testMatch was removed ({victim})",
        uncovered_specs(
            re.sub(r"testMatch\s*:\s*\[\s*'\*\*/" + re.escape(victim) + r"'\s*\]",
                   "testMatch: []", config_src),
            specs,
        ),
        [victim],
    )

    # 2. A brand-new spec file no project mentions.
    expect(
        "a newly added spec file with no project",
        uncovered_specs(config_src, specs + ["brand-new-orphan.spec.ts"]),
        ["brand-new-orphan.spec.ts"],
    )

    # 3. testIgnore wins over testMatch — the spec is selected AND excluded.
    expect(
        f"a spec excluded by a project-level testIgnore ({victim})",
        uncovered_specs(
            config_src.replace(
                f"      testMatch: ['**/{victim}'],",
                f"      testMatch: ['**/{victim}'],\n      testIgnore: ['**/{victim}'],",
                1,
            ),
            specs,
        ),
        [victim],
    )

    # 4. A top-level testIgnore that swallows the spec (inherited by the project).
    expect(
        f"a spec excluded by the inherited top-level testIgnore ({victim})",
        uncovered_specs(
            config_src.replace("  testIgnore: [\n", f"  testIgnore: [\n    '**/{victim}',\n", 1),
            specs,
        ),
        [victim],
    )

    # 5. The project is line-commented out (must NOT count as configuration).
    #    Line comments rather than /* */ because the block contains a glob with
    #    `*/` in it, which would terminate a block comment early and make the
    #    mutation invalid JS rather than a disabled project.
    block = re.search(
        r"    \{\s*\n\s*name: 'perlengkapan-pemakaian-monitoring',.*?\n    \},", config_src, re.S)
    expect(
        "a commented-out project (its spec is no longer collected)",
        uncovered_specs(
            config_src.replace(
                block.group(0),
                "\n".join("// " + ln for ln in block.group(0).splitlines()),
            ) if block else config_src,
            specs,
        ),
        [victim],
    )

    # 6. A `//`-commented testMatch must not be read as live configuration.
    expect(
        "a `//`-commented testMatch (line comments are not configuration)",
        uncovered_specs(
            config_src.replace(
                f"      testMatch: ['**/{victim}'],",
                f"      // testMatch: ['**/{victim}'],", 1),
            specs,
        ),
        [victim],
    )

    # 7. A quoted pattern inside an array comment is text, not a pattern.
    #    The quoted glob here deliberately has no `**/` — a `*/` inside a block
    #    comment would end the comment, which is valid JS but a different case.
    expect(
        "a quoted pattern inside an array comment (text, not configuration)",
        uncovered_specs(
            config_src.replace(
                f"      testMatch: ['**/{victim}'],",
                f"      testMatch: [/* '{victim}' */],", 1),
            specs,
        ),
        [victim],
    )

    # 8. A second project may still collect what the first one ignores.
    expect(
        "an unrelated project ignoring a spec another project collects",
        uncovered_specs(
            "export default defineConfig({ projects: ["
            "{ name: 'p1', testMatch: ['**/*.spec.ts'], testIgnore: ['**/b.spec.ts'] },"
            "{ name: 'p2', testMatch: ['**/b.spec.ts'] },"
            "]});",
            ["a.spec.ts", "b.spec.ts"],
        ),
        [],
    )

    # 9. A project-level `testIgnore: []` overrides the inherited top-level one.
    expect(
        "a project-level testIgnore overriding the top-level ignore",
        uncovered_specs(
            "export default defineConfig({ testIgnore: ['**/a.spec.ts'], projects: ["
            "{ name: 'p', testMatch: ['**/a.spec.ts'], testIgnore: [] },"
            "]});",
            ["a.spec.ts"],
        ),
        [],
    )

    # 10. Bare-string testMatch is still a testMatch.
    expect(
        "a bare-string testMatch",
        uncovered_specs(
            "export default defineConfig({ projects: [{ name: 'p', testMatch: '**/a.spec.ts' }] });",
            ["a.spec.ts", "b.spec.ts"],
        ),
        ["b.spec.ts"],
    )

    # 11. A top-level testMatch is NOT coverage.
    expect(
        "a top-level testMatch (only `projects:` collects)",
        uncovered_specs(
            "export default defineConfig({ testMatch: ['**/*.spec.ts'], projects: ["
            "{ name: 'p' }] });",
            ["a.spec.ts"],
        ),
        ["a.spec.ts"],
    )

    # 12. A pattern inside a STRING is not configuration.
    expect(
        "a testMatch-looking pattern inside a string literal",
        uncovered_specs(
            "export default defineConfig({ projects: [{ name: 'p', "
            "use: { note: \"testMatch: ['**/a.spec.ts']\" } }] });",
            ["a.spec.ts"],
        ),
        ["a.spec.ts"],
    )

    del others  # kept for readability of the expectations above
    return ok


def main() -> int:
    global ACTIVE_SUITES
    argv = list(sys.argv[1:])
    if "--e2e-dir" in argv:
        i = argv.index("--e2e-dir")
        try:
            ACTIVE_SUITES = [("e2e", argv[i + 1])]
        except IndexError:
            print("--e2e-dir needs a path", file=sys.stderr)
            return 2
        del argv[i : i + 2]

    if "--self-test" in argv:
        return 0 if self_test() else 1

    ok = check()
    if "--cross-check" in argv:
        cross = cross_check_against_list()
        if cross is False:
            ok = False
    if ok:
        print("check-playwright-projects-cover-specs: every spec is collected by a project.")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
