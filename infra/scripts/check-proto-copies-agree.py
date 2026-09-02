#!/usr/bin/env python3
"""Fail when two copies of the same .proto file disagree.

Three service contracts exist twice in this tree. `authenc/proto/` holds its
own copy of `integrasi.proto`, `secreton.proto` and `common.proto`, because
`layanan/authenc/Dockerfile` copies only `layanan/authenc` into the build
context, so its `build.rs` cannot reach the owning service's directory.
(`layanan/perlengkapan/build.rs` reads the owners' files directly and needs no
copy — the duplication is authenc's alone.)

A gRPC contract that exists twice drifts silently in ONE direction: add a field
to the owner, regenerate, and the owning service compiles and serves it while
every consumer built from the stale copy has no such field. That is not a
hypothetical — adding `MysimkariPegawai.foto` to the owner compiled fine and
broke authenc with `no field 'foto' on type MysimkariPegawai`, which is the
LUCKY case. A field ADDED to a message that authenc only reads would not break
any build: prost decodes unknown fields by ignoring them, so authenc would
simply never see the value, and every screen fed from it would show nothing
with no error anywhere.

So: as long as the copies exist, they must agree. Comments and blank lines are
ignored (the copies are commented differently and always have been); anything
that reaches the generated code is compared.

The real fix is to delete the copies and point authenc's `build.rs` at the
owners, which needs `layanan/*/proto` added to its Docker build context. Until
then this guard is what stops the drift.

No YAML, no third-party imports: the ARC runner image carries neither PyYAML
nor yq (see project_arc_runner_image_missing_tools).
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

PROTO_GLOB = "layanan/*/proto/*.proto"
SKIP_DIRS = {"target", "node_modules", ".git"}

# A `//` comment, but not one inside a string literal. Proto string literals in
# this tree are option values (`option go_package = "...";`) and contain no
# `//`, but the pattern is written to leave quoted runs alone anyway.
STRING_LITERAL_RE = re.compile(r'"(?:[^"\\]|\\.)*"')
LINE_COMMENT_RE = re.compile(r"//[^\n]*")
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.S)


def strip_comments(src: str) -> str:
    out: list[str] = []
    idx = 0
    for m in STRING_LITERAL_RE.finditer(src):
        chunk = BLOCK_COMMENT_RE.sub(" ", src[idx : m.start()])
        out.append(LINE_COMMENT_RE.sub("", chunk))
        out.append(m.group(0))
        idx = m.end()
    tail = BLOCK_COMMENT_RE.sub(" ", src[idx:])
    out.append(LINE_COMMENT_RE.sub("", tail))
    return "".join(out)


def significant(path: Path) -> list[str]:
    """The lines of `path` that actually reach the generated code."""
    src = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
    return [line.rstrip() for line in src.splitlines() if line.strip()]


def main() -> int:
    by_name: dict[str, list[Path]] = defaultdict(list)
    for path in sorted(ROOT.glob(PROTO_GLOB)):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        by_name[path.name].append(path)

    if not by_name:
        print(
            f"FAIL: no .proto matched {PROTO_GLOB} — the derivation is broken, "
            "not the code.",
            file=sys.stderr,
        )
        return 2

    problems: list[str] = []
    compared = 0
    for name, paths in sorted(by_name.items()):
        if len(paths) < 2:
            continue
        compared += 1
        first, *rest = paths
        base = significant(first)
        for other in rest:
            lines = significant(other)
            if lines == base:
                continue
            diff = [
                f"        {a!r} != {b!r}"
                for a, b in zip(base, lines)
                if a != b
            ][:3]
            if len(base) != len(lines):
                diff.append(
                    f"        {len(base)} significant line(s) vs {len(lines)}"
                )
            problems.append(
                f"  {name}\n"
                f"      {first.relative_to(ROOT)}\n"
                f"      {other.relative_to(ROOT)}\n" + "\n".join(diff)
            )

    if not compared:
        print(
            "FAIL: no duplicated .proto found — either the copies were removed "
            "(delete this guard and point build.rs at the owners) or the scan "
            "is broken.",
            file=sys.stderr,
        )
        return 2

    if problems:
        print(
            "Copies of the same .proto disagree. A consumer built from the "
            "stale copy\nwill not see the owner's field — and for an ADDED "
            "field nothing fails to\ncompile: prost ignores unknown fields, so "
            "the value simply never arrives.\n",
            file=sys.stderr,
        )
        print("\n".join(problems), file=sys.stderr)
        return 1

    print(f"OK: {compared} duplicated .proto file(s), every copy in agreement.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
