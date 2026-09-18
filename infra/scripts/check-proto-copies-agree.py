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

import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

PROTO_GLOB = "layanan/*/proto/*.proto"
SKIP_DIRS = {"target", "node_modules", ".git"}

def strip_comments(src: str) -> str:
    """Remove `//` and `/* */` comments, preserving string literals.

    A single left-to-right scan, NOT a regex pass. The first version of this
    function found every quoted run in the file and treated each as a string
    literal to be protected — which meant a QUOTE INSIDE A COMMENT opened a
    fake literal, and the comment text after it survived as "code". Adding a
    comment that said `// False means "no photo on record"` made this guard
    report the two copies as disagreeing on lines that were both comments.

    That is the fourth time in this repo a comment has been read as code
    (see project_comments_are_parsed_as_code). A character scan cannot make
    the mistake: a quote inside a comment is never the start of a literal,
    because the scanner is already in the comment state when it reaches it.
    """
    out: list[str] = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '"':
            out.append(c)
            i += 1
            while i < n:
                out.append(src[i])
                if src[i] == "\\" and i + 1 < n:
                    out.append(src[i + 1])
                    i += 2
                    continue
                if src[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if src.startswith("//", i):
            while i < n and src[i] != "\n":
                i += 1
            continue
        if src.startswith("/*", i):
            end = src.find("*/", i + 2)
            i = n if end == -1 else end + 2
            out.append(" ")
            continue
        out.append(c)
        i += 1
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


def _canaries() -> int:
    """Prove the comment stripper on the cases that actually broke it.

    Every case below is a REGRESSION, not a description: the previous
    regex-pair implementation leaks the comment text of the first two and
    would report two identically-generated copies as disagreeing.
    """
    failures = 0

    def expect(label: str, src: str, keep: list[str], drop: list[str]) -> None:
        nonlocal failures
        out = strip_comments(src)
        bad = [f"missing {k!r}" for k in keep if k not in out]
        bad += [f"leaked {d!r}" for d in drop if d in out]
        print(f"  {'ok  ' if not bad else 'FAIL'} {label}" + ("" if not bad else f" — {'; '.join(bad)}"))
        if bad:
            failures += 1

    expect(
        "a quote inside a line comment does not open a string literal",
        '// False means "no photo on record", ordinary\nstring foto = 12;\n',
        ["string foto = 12;"],
        ["no photo on record", "False means"],
    )
    expect(
        "two consecutive quoted comments do not pair their quotes",
        '// a "x" b\n// c "y" d\nbool found = 1;\n',
        ["bool found = 1;"],
        ["x", "y"],
    )
    expect(
        "a real string literal survives, `//` inside it included",
        'option go_package = "github.com/x//y";\n',
        ['"github.com/x//y"'],
        [],
    )
    expect(
        "block comments go",
        "/* drop\n   this */\nint32 a = 1;\n",
        ["int32 a = 1;"],
        ["drop"],
    )
    expect(
        "an unterminated block comment does not swallow the file silently",
        "int32 a = 1;\n/* never closed\n",
        ["int32 a = 1;"],
        ["never closed"],
    )
    expect(
        "an escaped quote does not end the literal early",
        'option x = "a\\"//b";\nint32 a = 1;\n',
        ['"a\\"//b"', "int32 a = 1;"],
        [],
    )

    # The other direction: the copies as committed must PASS. A guard that
    # only ever proves it can fail is a guard nobody has proven can pass.
    rc = main()
    print(f"  {'ok  ' if rc == 0 else 'FAIL'} the copies as committed agree (exit {rc})")
    if rc != 0:
        failures += 1

    return failures


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(_canaries())
    sys.exit(main())
