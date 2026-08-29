#!/usr/bin/env python3
"""The frontend must not keep its own copy of the workflow status vocabulary.

A workflow status has exactly one authority: the Rust enum in
`layanan/perlengkapan` that writes the column. Its `label()` names the state for
humans, its `to_state_name()` names it for the database. When the frontend
restates that mapping —

    match status {
        "DRAFT"  => ("Draft", …),
        "ACTIVE" => ("Aktif", …),
        _        => ("Lainnya", …),
    }

— the copy does not stay in step, and its drift is invisible: it compiles, it
renders, it has no HTTP status of its own. Measured on 2026-08-29 there were
FOUR such copies in `antarmuka/perlengkapan`, each with different gaps:

  * `pemakaian_bmn/list_page.rs` and `pemakaian_bmn/detail_page.rs` had no arm
    for `SUBMITTED_APPROVER_SATKER` or `REVISI_OPERATOR` — the two states the
    whole V035 three-step approval runs through. A permit waiting on the
    Approver Satker displayed as **"Lainnya"**.
  * `penghapusan_bmn/list_page.rs` and `.../detail_page.rs` had no arm for
    `KONSEP_SK_WILAYAH_GENERATED` or `SK_SIGNED_WILAYAH` — the entire wilayah
    authority branch (V029) displayed as "Lainnya", and its detail page said
    "Fase tidak dikenal." about a proposal proceeding perfectly normally.
  * `components/pemakaian_bmn_monitoring.rs` mapped nothing at all and printed
    the raw `ACTIVE` into the table.

The fix was not to add the missing arms — that only postpones the fifth copy.
The vocabulary moved to the enum, which is the only thing that actually knows
the list, and the backend now ships `status_label` + `status_tone` per row. The
frontend maps five tones to CSS and holds no state names.

The rule
--------
A `match` in frontend code with three or more arms whose **pattern** is a
workflow state-name literal is a vocabulary copy: it reads a state and produces
something to show. Three is the threshold that separates a table from logic —
`status == "ACTIVE"` and `.filter(|t| t.status != "REVOKED")` are single
comparisons that decide behaviour, not tables that name states for the reader.

The direction matters, and the guard checks it. A state name on the **right** of
`=>` is the opposite construction: a filter enum turning a UI choice into a query
value (`Self::Aktif => "ACTIVE"`). That is a curated subset by intent, not a
mapping that claims to cover the workflow, so it is deliberately not flagged.
(Those lists have gaps of their own — the pemakaian filter offers no chip for
`SUBMITTED_APPROVER_SATKER` or `REVISI_OPERATOR` — but the fix there is a
server-provided option list, a different change from this one.)

The vocabulary itself is DERIVED from the backend enums' `to_state_name()`, not
listed here. A hand-written list would need to grow when a workflow gains a
state — which is the same failure this guard exists to catch.

Comments are stripped before matching. A guard that reads its own subject matter
out of a comment has been three separate bugs in this repo already.

Exit 0 clean, 1 on findings. Stdlib only — the ARC runner image has no PyYAML.
"""

import re
import sys
from pathlib import Path

BACKEND = "layanan/perlengkapan/src"
FRONTEND_ROOTS = ["antarmuka"]
SKIP_DIRS = {"target", "node_modules", "dist", ".git"}

# How many state-name arms make a match a vocabulary copy rather than logic.
ARM_THRESHOLD = 3

STATE_FN = re.compile(r"fn to_state_name\(&self\)[^{]*\{(.*?)\n    \}", re.S)
STATE_ARM = re.compile(r'=>\s*"([A-Z][A-Z0-9_]*)"')
STRING = re.compile(r'"(?:\\.|[^"\\])*"')
# `"A"` or `"A" | "B" | …` — nothing else.
BARE_STATE_PATTERN = re.compile(r'"[^"]*"(?:\s*\|\s*"[^"]*")*')


def strip_comments(src: str) -> str:
    """Drop // and /* */ comments, leaving string literals intact."""
    out = []
    i, n = 0, len(src)
    while i < n:
        ch = src[i]
        if ch == '"':
            m = STRING.match(src, i)
            if m:
                out.append(m.group(0))
                i = m.end()
                continue
            out.append(ch)
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif src.startswith("/*", i):
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append("".join(c if c == "\n" else " " for c in src[i:j]))
            i = j
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def backend_state_names(repo: Path) -> set[str]:
    names: set[str] = set()
    base = repo / BACKEND
    if not base.is_dir():
        return names
    for path in sorted(base.rglob("*.rs")):
        src = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        for body in STATE_FN.findall(src):
            names.update(STATE_ARM.findall(body))
    return names


def match_blocks(src: str):
    """Yield (line_no, block_text) for every `match … { … }` in the source."""
    for m in re.finditer(r"\bmatch\b", src):
        brace = src.find("{", m.end())
        if brace < 0:
            continue
        # A `{` that far from the keyword is not this match's block.
        if "\n" in src[m.end():brace] and src[m.end():brace].count("\n") > 2:
            continue
        depth, i = 0, brace
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        yield src.count("\n", 0, m.start()) + 1, src[brace:i + 1]


def pattern_state_names(block: str, vocabulary: set[str]) -> set[str]:
    """State names used as match PATTERNS, i.e. to the left of `=>`.

    A name to the right of the arrow is the reverse mapping (a filter enum
    producing a query value) and is not a vocabulary copy.
    """
    names = set()
    for arrow in re.finditer(r"=>", block):
        # Walk back to the nearest arm boundary. Doing this per `=>` rather
        # than per LINE is deliberate: the previous guard in this repo was
        # written line-first and went blind to a match written on one line,
        # which is exactly where a copy hides from review.
        head = block[: arrow.start()]
        cut = max(head.rfind(c) for c in (",", "{", "}", ";", "\n"))
        pattern = head[cut + 1 :].strip().rstrip("|").strip()
        # A display table is keyed on the state ALONE: `"ACTIVE" =>` or
        # `"SUBMIT_WILAYAH" | "SUBMITTED" =>`. A tuple pattern such as
        # `("SUBMITTED", "REVISI_OPERATOR") =>` is keyed on a transition — that
        # is the frontend deciding which endpoint to call, not naming a state
        # for the reader, and it fails loudly on an unknown pair rather than
        # quietly printing a generic word.
        if not BARE_STATE_PATTERN.fullmatch(pattern):
            continue
        for literal in STRING.findall(pattern):
            name = literal[1:-1]
            if name in vocabulary:
                names.add(name)
    return names


def scan(path: Path, vocabulary: set[str]) -> list[tuple[int, list[str]]]:
    src = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
    findings = []
    for line_no, block in match_blocks(src):
        arms = pattern_state_names(block, vocabulary)
        if len(arms) >= ARM_THRESHOLD:
            findings.append((line_no, sorted(arms)))
    return findings


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    vocabulary = backend_state_names(repo)
    if len(vocabulary) < 10:
        print(
            "check-fe-status-vocabulary: derived only "
            f"{len(vocabulary)} state name(s) from {BACKEND}. The derivation is "
            "broken, not the repo — a near-empty vocabulary would pass every file.",
            file=sys.stderr,
        )
        return 1

    total, scanned = 0, 0
    for root in FRONTEND_ROOTS:
        base = repo / root
        if not base.is_dir():
            print(f"::error::guard scope {root} does not exist", file=sys.stderr)
            return 1
        for path in sorted(base.rglob("*.rs")):
            if any(p in SKIP_DIRS for p in path.parts):
                continue
            scanned += 1
            for line_no, arms in scan(path, vocabulary):
                print(f"{path.relative_to(repo)}:{line_no}  match over {len(arms)} state names")
                print(f"    {', '.join(arms)}")
                total += 1

    if not scanned:
        print(
            "check-fe-status-vocabulary: scanned ZERO files — the scope is "
            "broken, not the repo.",
            file=sys.stderr,
        )
        return 1

    if total:
        print(
            f"\n{total} frontend match(es) restating the workflow status vocabulary.\n"
            f"That copy drifts silently: a state the backend adds is absent here, and\n"
            f"the row renders as a generic fallback while the workflow runs normally.\n"
            f"Render the backend's `status_label` / `status_tone` instead; keep only\n"
            f"the tone → CSS mapping, which does not grow with the workflow.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-fe-status-vocabulary: {scanned} files, {len(vocabulary)} backend "
        f"state names, no frontend copy of the vocabulary"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
