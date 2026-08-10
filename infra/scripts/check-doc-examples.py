#!/usr/bin/env python3
"""Fail when a ```ignore'd doc example names an API the crate does not have.

#115 made `cargo test --doc` a gate. ```ignore is the hole in it: rustdoc parses
the fence, sees `ignore`, and never compiles the body — so an example can go on
documenting types that were renamed or deleted years ago and nothing complains.
That is not a cosmetic problem. `secreton_core::utils` shipped ~300 lines of
module docs describing an `LruCache`, a `SecureMemory` and a `ConfigLoader` while
the real exports were `SecretLruCache`, `SecureSecretMemory` and `Config`; a
reader (or an agent) following those docs writes code that cannot compile, and
the more confident the prose, the longer they look for the mistake in themselves.

The rule this enforces is narrow on purpose:

    An example may opt out of EXECUTION. It may not name APIs that do not exist.

`ignore` stays legitimate for examples that need a live server, a cluster or a
database. Those still have to spell real type names. Anything that can actually
run should lose the `ignore` instead and let rustdoc check it properly — that is
strictly better than passing this guard.

Method: for each ```ignore fence, take every `use <crate>::…` item it names and
look it up in the set of names the crate really exports. `pub use X as Y` exports
Y and not X, which is exactly the `LruCache as SecretLruCache` case above, so the
alias handling is not incidental.

This is a LOWER bound and deliberately so — it checks names in `use` statements,
not method calls or signatures. An example calling `LruCache::new(cap, ttl)` when
`new` takes one argument passes here. Only compilation catches that, which is the
argument for un-`ignore`ing rather than for making this guard cleverer.

Stdlib only (tomllib + re): the ARC runner image has no PyYAML and no yq.
"""

from __future__ import annotations

import glob
import re
import sys
import tomllib
from pathlib import Path

FENCE = re.compile(r"^\s*(?:///|//!)\s*```(.*)$")
DOCLINE = re.compile(r"^\s*(?:///|//!) ?(.*)$")
USE = re.compile(r"^\s*(?:#\s*)?use\s+([A-Za-z_][\w:]*)\s*(?:::\{([^}]*)\})?\s*;")

DECL = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)"
    r"(?:async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(?:struct|enum|fn|trait|type|mod|const|static|union)\s+([A-Za-z_]\w*)"
)
PUB_USE = re.compile(r"^[ \t]*pub\s+use\s+([^;]+);", re.M | re.S)
MACRO = re.compile(r"^\s*macro_rules!\s+([A-Za-z_]\w*)")


def exported_names(src: Path) -> set[str]:
    """Every name the crate makes reachable, by declaration or re-export."""
    names: set[str] = set()
    for path in sorted(src.rglob("*.rs")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for line in text.splitlines():
            m = DECL.match(line) or MACRO.match(line)
            if m:
                names.add(m.group(1))
        # `pub use a::b::{C, D as E};` reaches C and E — NOT D. Crediting both
        # sides of `as` is what let `LruCache as SecretLruCache` vouch for a doc
        # example saying `utils::cache::LruCache`, a path that does not resolve.
        for m in PUB_USE.finditer(text):
            for item in m.group(1).replace("{", ",").replace("}", ",").split(","):
                item = item.strip()
                if not item:
                    continue
                if " as " in item:
                    item = item.split(" as ")[-1].strip()
                leaf = item.split("::")[-1].strip()
                if leaf and leaf != "*" and leaf not in {"crate", "self", "super"}:
                    names.add(leaf)
    return names


def ignored_examples(text: str):
    """Yield (line_number, body_lines) for each ```ignore fence."""
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        fence = FENCE.match(lines[i])
        if not fence or "ignore" not in re.split(r"[,\s]+", fence.group(1).strip()):
            i += 1
            continue
        start, body = i + 1, []
        i += 1
        while i < len(lines) and not FENCE.match(lines[i]):
            doc = DOCLINE.match(lines[i])
            body.append(doc.group(1) if doc else lines[i])
            i += 1
        yield start, body
        i += 1


def named_crate_items(body: list[str], prefix: str) -> list[str]:
    """Items an example imports from its own crate."""
    items: list[str] = []
    for line in body:
        use = USE.match(line)
        if not use:
            continue
        head, braces = use.group(1), use.group(2)
        if not head.startswith((prefix, "crate")):
            continue
        for item in (braces.split(",") if braces else [head.split("::")[-1]]):
            item = item.split(" as ")[0].strip()
            if item:
                items.append(item)
    return items


def workspace_crates() -> list[tuple[str, Path]]:
    root = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    crates = []
    for pattern in root["workspace"]["members"]:
        for member in sorted(glob.glob(pattern)):
            manifest = Path(member) / "Cargo.toml"
            if not manifest.is_file():
                continue
            name = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]["name"]
            src = Path(member) / "src"
            if src.is_dir():
                crates.append((name, src))
    return crates


def check(name: str, src: Path, overrides: dict[Path, str] | None = None) -> list[str]:
    """Problems found in one crate. `overrides` swaps file contents in memory."""
    overrides = overrides or {}
    known = exported_names(src)
    prefix = name.replace("-", "_")
    problems = []
    for path in sorted(src.rglob("*.rs")):
        text = overrides.get(path, path.read_text(encoding="utf-8", errors="replace"))
        for line, body in ignored_examples(text):
            missing = sorted(
                {item for item in named_crate_items(body, prefix) if item not in known}
            )
            if missing:
                problems.append(
                    f"::error file={path},line={line}::```ignore example names "
                    f"{', '.join(missing)}, which {'is' if len(missing) == 1 else 'are'} "
                    f"not exported by {name} — fix the name, or delete the example if "
                    f"the API it documents was never built"
                )
    return problems


def self_test(crates: list[tuple[str, Path]]) -> int:
    """Break one example on purpose and confirm the guard notices.

    A guard is unproven until it has been shown to fail on the regression it
    claims to catch (#115: a first cut of the crate-coverage guard passed a
    deleted `cargo test --doc` step because the step's *name* still matched).
    Mutate the first ```ignore fence found anywhere into one that imports a type
    no crate could possibly export.
    """
    for name, src in crates:
        prefix = name.replace("-", "_")
        for path in sorted(src.rglob("*.rs")):
            text = path.read_text(encoding="utf-8", errors="replace")
            hit = next(iter(ignored_examples(text)), None)
            if hit is None:
                continue
            line, _ = hit
            lines = text.splitlines(keepends=True)
            lines.insert(line, f"//! use {prefix}::ThisTypeWasNeverBuilt;\n")
            mutated = "".join(lines)
            if check(name, src, {path: mutated}):
                print(f"  ok   detects: a fictional import added to {path}:{line}")
                return 0
            print(
                f"::error::self-test FAILED — guard did not flag an obviously "
                f"fictional import injected into {path}:{line}"
            )
            return 1
    print("::error::self-test is stale: no ```ignore example left to mutate")
    return 1


def main() -> int:
    crates = workspace_crates()
    failures = self_test(crates)

    total = 0
    problems: list[str] = []
    for name, src in crates:
        for path in sorted(src.rglob("*.rs")):
            total += sum(1 for _ in ignored_examples(path.read_text(encoding="utf-8", errors="replace")))
        problems.extend(check(name, src))

    for problem in problems:
        print(problem)

    if problems or failures:
        print(
            f"\n{len(problems)} ```ignore example(s) document APIs that do not exist."
            if problems
            else ""
        )
        return 1

    print(
        f"{total} ```ignore doc example(s) across {len(crates)} crates; "
        f"every crate item they name really exists"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
