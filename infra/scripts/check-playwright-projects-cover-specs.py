#!/usr/bin/env python3
"""Every Playwright spec on disk must actually be collected by some project.

`npx playwright test` only executes specs matched by a project's `testMatch`
*and not excluded by a `testIgnore`*. A spec file that no project collects is
skipped silently, and a suite that never runs is indistinguishable from a suite
that passes. Nothing else notices: the file compiles (TypeScript is only checked
when the file is loaded), the route-coverage gate greps spec *text* for route
strings (it never asks whether the spec is collected), and `cargo` does not know
the file exists.

Demonstrated, not hypothesised: `antarmuka/perlengkapan/tests/e2e/
pemakaian-monitoring.spec.ts` — 8 assertions over the six per-role monitoring
read endpoints (the exact scoping bug class the file was written to catch) — sat
on disk matched by no project. Same failure shape as the orphaned `tests/`
sub-directory suites that `lint-orphan-tests` guards for Cargo, one toolchain
over.

Derives both sides rather than trusting either to be maintained:
  * specs on disk = `*.spec.ts` / `*.setup.ts` under each e2e dir
  * collected     = (union of project `testMatch`) minus `testIgnore`, where a
                    project inherits the top-level `testIgnore` when it does not
                    declare its own

Three details decide whether this guard is real or decorative, each learned from
a way it was wrong:

  1. **`testMatch` alone is not "collected".** Playwright drops files matching a
     `testIgnore` even when a project's `testMatch` selects them, so a guard
     reading only `testMatch` calls an ignored spec covered. Both scopes must be
     modelled. (Top-level `testMatch` is still *not* coverage — treating it as
     such would let `**/*.spec.ts` mark everything covered, which is the bug.)
  2. **Comments are not configuration.** A commented-out project must not count.
     The config source is run through a scanner that blanks comments *and string
     literals* before anything is parsed, so `'**/a.spec.ts'` cannot be mistaken
     for code and a `// { name: ... }` cannot be mistaken for a project.
  3. **The set is only trustworthy if it is verifiable.** `--list` is the ground
     truth, but the invariants job that runs this has no Node toolchain and a
     checkout with no `node_modules` cannot install one. So the static model is
     pinned by an independent cross-check against `--list` when the toolchain
     happens to be present (see `cross_check_against_list`).

Usage:  check-playwright-projects-cover-specs.py [--self-test] [--cross-check]
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# (label, e2e dir relative to repo root)
SUITES = [
    ("perlengkapan", "antarmuka/perlengkapan/tests/e2e"),
    ("portal", "antarmuka/portal/tests/e2e"),
]

SPEC_GLOBS = ("*.spec.ts", "*.setup.ts")
SKIP_PREFIXES = ("node_modules/", "results/", "test-results/", "playwright-report/")


# ── source scanning ───────────────────────────────────────────────────────────


def strip_comments_and_strings(src: str) -> str:
    """Blank comments and string literals, preserving offsets and newlines.

    Comments are blanked to spaces (so `// { name: ... }` cannot be read as a
    project) and strings to NUL (so `'**/a.spec.ts'` cannot be read as code, and
    — crucially — whitespace-skipping after a `key:` cannot walk *through* a
    blanked literal and miss a bare-string value like `testMatch: '**/x.ts'`).

    Both matter. Comments because a commented-out project must not be read as
    configuration. Strings because the patterns being parsed *are* strings:
    scanning raw text for `testMatch` finds the ones inside a `//` note, and
    scanning for project braces finds the ones inside a `'**/{a,b}.spec.ts'`.
    """
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
            out[i] = "\x00"
            i += 1
            while i < n and src[i] != quote:
                if src[i] == "\\" and i + 1 < n:
                    out[i] = out[i + 1] = "\x00"
                    i += 2
                    continue
                out[i] = "\n" if src[i] == "\n" else "\x00"
                i += 1
            if i < n:
                out[i] = "\x00"
                i += 1
            continue
        i += 1
    return "".join(out)


def _match_bracket(src: str, open_idx: int) -> int:
    """Index just past the bracket matching `src[open_idx]` (strings pre-blanked)."""
    pairs = {"[": "]", "{": "}", "(": ")"}
    close = pairs[src[open_idx]]
    depth = 0
    i = open_idx
    while i < len(src):
        if src[i] == src[open_idx]:
            depth += 1
        elif src[i] == close:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(src)


def _object_spans(src: str) -> list[tuple[int, int]]:
    """Spans of the brace objects that sit *directly* in the `projects: [...]` array.

    Deliberately shallow: only the array's own elements are projects. Scanning
    for `name:` anywhere would also pick up the nested `use: { ... }`, and
    scanning for every `{` would pick up objects inside a project.
    """
    m = re.search(r"\bprojects\s*:\s*\[", src)
    if not m:
        return []
    arr_start = src.index("[", m.start())
    arr_end = _match_bracket(src, arr_start)
    spans, i = [], arr_start + 1
    while i < arr_end:
        if src[i] == "{":
            end = _match_bracket(src, i)
            spans.append((i, end))
            i = end
            continue
        i += 1
    return spans


def _key_value_extent(scan: str, key: str, start: int, end: int) -> tuple[int, int] | None:
    """Span of the value assigned to `key` inside [start, end) of the scan.

    Handles both `key: ['a','b']` and `key: 'a'`. Returns offsets into `scan`,
    which line up with the raw source because blanking preserves offsets.
    """
    m = re.search(r"\b" + re.escape(key) + r"\s*:\s*", scan[start:end])
    if not m:
        return None
    vstart = start + m.end()
    while vstart < end and scan[vstart] in " \t\r\n":
        vstart += 1
    if vstart >= end:
        return None
    if scan[vstart] in "[{(":
        return vstart, _match_bracket(scan, vstart)
    # A bare string literal: run to the end of that literal in the raw source.
    return vstart, vstart


def _literal_extent(raw_src: str, vstart: int, vend: int) -> list[str]:
    """String literals of a value, taken from the RAW source at the same offsets."""
    if vend <= vstart:
        # Bare literal: scan forward to the closing quote in the raw text.
        if vstart < len(raw_src) and raw_src[vstart] in "'\"":
            quote = raw_src[vstart]
            j = vstart + 1
            while j < len(raw_src) and raw_src[j] != quote:
                j += 2 if raw_src[j] == "\\" else 1
            vend = j + 1
        else:
            return []
    return [a or b for a, b in re.findall(r"'([^'\\]*)'|\"([^\"\\]*)\"", raw_src[vstart:vend])]


def config_model(raw_src: str) -> tuple[list[re.Pattern[str]], list[re.Pattern[str]]]:
    """(project match globs, effective ignore globs) — comments/strings respected.

    A project that declares its own `testIgnore` replaces the inherited one;
    otherwise the top-level `testIgnore` applies. Both scopes are honoured, and
    ignore globs are applied across the collected set (what Playwright does).
    """
    scan = strip_comments_and_strings(raw_src)

    top_ignore: list[str] = []
    m = re.search(r"\btestIgnore\s*:\s*\[", scan)
    if m:
        arr_start = scan.index("[", m.start())
        top_ignore = _literal_extent(raw_src, arr_start, _match_bracket(scan, arr_start))

    matches: list[str] = []
    ignores: list[str] = list(top_ignore)
    for start, end in _object_spans(scan):
        tm = _key_value_extent(scan, "testMatch", start, end)
        if tm is None:
            continue
        matches.extend(_literal_extent(raw_src, *tm))
        ti = _key_value_extent(scan, "testIgnore", start, end)
        if ti is not None:
            ignores.extend(_literal_extent(raw_src, *ti))

    return (
        [glob_to_regex(p) for p in _dedupe(matches)],
        [glob_to_regex(p) for p in _dedupe(ignores)],
    )


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


# ── the model of what Playwright collects ─────────────────────────────────────


def _dedupe(xs: list[str]) -> list[str]:
    seen, out = set(), []
    for x in xs:
        if x not in seen:
            seen.add(x)
            out.append(x)
    return out


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
    matches, ignores = config_model(raw_src)
    out = []
    for s in specs:
        if not any(r.match(s) for r in matches):
            out.append(s)
        elif any(r.match(s) for r in ignores):
            out.append(s)
    return out


# ── checks ────────────────────────────────────────────────────────────────────


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
    for label, rel_dir in SUITES:
        e2e_dir = REPO / rel_dir
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

        # --list can report a nested path prefix; compare on the file basename set
        # plus relative path so either form matches.
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
    _, rel_dir = SUITES[0]
    e2e_dir = REPO / rel_dir
    config_src = (e2e_dir / "playwright.config.ts").read_text(encoding="utf8")
    specs = specs_on_disk(e2e_dir)

    if uncovered_specs(config_src, specs):
        print("  ✗ self-test: baseline already red")
        return False
    print("  ok detects: nothing (healthy baseline is green)")

    def caught(name: str, mutated: str, victim: str) -> None:
        nonlocal ok
        if mutated == config_src:
            print(f"  ✗ self-test: could not construct mutation for {name}")
            ok = False
            return
        if victim in uncovered_specs(mutated, specs):
            print(f"  ok detects: {name}")
        else:
            print(f"  ✗ self-test: guard did NOT notice {name}")
            ok = False

    victim = "pemakaian-monitoring.spec.ts"
    if victim not in specs:
        print(f"  ✗ self-test: expected {victim} on disk to use as the canary")
        return False

    # 1. The original bug: the project's testMatch goes away.
    caught(
        f"a spec whose project's testMatch was removed ({victim})",
        re.sub(r"testMatch\s*:\s*\[\s*'\*\*/" + re.escape(victim) + r"'\s*\]",
               "testMatch: []", config_src),
        victim,
    )

    # 2. A brand-new spec file no project mentions.
    if "brand-new-orphan.spec.ts" in uncovered_specs(config_src, specs + ["brand-new-orphan.spec.ts"]):
        print("  ok detects: a newly added spec file with no project")
    else:
        print("  ✗ self-test: guard did NOT notice a newly added orphan spec")
        ok = False

    # 3. testIgnore wins over testMatch — the spec is selected AND excluded.
    caught(
        f"a spec excluded by a project-level testIgnore ({victim})",
        config_src.replace(
            f"      testMatch: ['**/{victim}'],",
            f"      testMatch: ['**/{victim}'],\n      testIgnore: ['**/{victim}'],",
            1,
        ),
        victim,
    )

    # 4. A top-level testIgnore that swallows the spec (inherited by the project).
    caught(
        f"a spec excluded by the inherited top-level testIgnore ({victim})",
        config_src.replace(
            "  testIgnore: [\n",
            f"  testIgnore: [\n    '**/{victim}',\n", 1),
        victim,
    )

    # 5. The project is commented out (must NOT count as configuration).
    block = re.search(
        r"\{\s*\n\s*name: 'perlengkapan-pemakaian-monitoring',.*?\n    \},", config_src, re.S)
    caught(
        "a commented-out project (its spec is no longer collected)",
        config_src.replace(block.group(0), "/* " + block.group(0) + " */") if block else config_src,
        victim,
    )

    # 6. A `//`-commented testMatch must not be read as live configuration.
    caught(
        "a `//`-commented testMatch (line comments are not configuration)",
        config_src.replace(
            f"      testMatch: ['**/{victim}'],",
            f"      // testMatch: ['**/{victim}'],", 1),
        victim,
    )

    return ok


def main() -> int:
    if "--self-test" in sys.argv:
        return 0 if self_test() else 1

    ok = check()
    if "--cross-check" in sys.argv:
        cross = cross_check_against_list()
        if cross is False:
            ok = False
    if ok:
        print("check-playwright-projects-cover-specs: every spec is collected by a project.")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
