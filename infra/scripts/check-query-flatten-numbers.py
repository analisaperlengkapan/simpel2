#!/usr/bin/env python3
"""Numbers inside a `#[serde(flatten)]`ed query type must use a tolerant deserializer.

`serde_urlencoded` parses `?page=2` into an `i32` by asking the field for its
type. `#[serde(flatten)]` takes that away: serde buffers the input into its
internal `Content` first, and everything in the buffer is a string. Numeric
fields INSIDE the flattened type then fail with

    invalid type: string "2", expected i32

and axum turns the rejection into `400 Bad Request` before the handler runs.

What makes this class survive review, CI and staging alike:

  * `cargo check` cannot see it — the types are all valid.
  * Unit tests cannot see it — they construct the struct, they do not parse a
    query string.
  * The endpoint answers **200** when the client sends no pagination, because
    the `#[serde(default)]` path never touches the buffer. So it looks healthy
    right up until a page control is used.

Found live on 2026-08-24: six pakaian-dinas endpoints answering 400 to
`?page=1&per_page=100`, plus the same shape latent in authenc's audit export
and secreton's policy list.

The boundary is measured, not assumed (see `lib_core::serde_query`):

    inside the flattened type   -> broken (Option and default make no odds)
    sibling of the flattened    -> fine
    no flatten anywhere         -> fine

so this guard only looks inside flattened types.

Scope is derived: query DTOs come from the `Query<…>` extractors in the tree,
and their flattened types are followed transitively. Nothing here is a list of
known-bad sites, so a new one is caught the day it is written.

Exit 0 clean, 1 on findings. Stdlib only — the ARC runner image has no PyYAML.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOTS = ("layanan", "lib", "antarmuka")

# Path-qualified forms count: `Query<crate::shared::pagination::Foo>` is the
# same extractor. The first draft matched bare names only and sailed past a
# deliberately-broken canary DTO — which is the whole reason this guard
# carries one.
QUERY_EXTRACTOR_RE = re.compile(
    r"\bQuery<\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*>"
)
STRUCT_RE = re.compile(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", re.M)

# Types that cannot round-trip through serde's buffered `Content` from text.
NUMERIC = {
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "bool",
}

FIELD_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,]+?)\s*,?\s*$"
)
FLATTEN_ATTR_RE = re.compile(r"serde\s*\(([^)]*)\)")
TOLERANT_RE = re.compile(r"deserialize_with\s*=\s*\"[^\"]*serde_query::")


def struct_body(text: str, name: str) -> str | None:
    """Return the brace-balanced body of `struct <name> { … }`, or None."""
    m = re.search(r"\bstruct\s+" + re.escape(name) + r"\s*\{", text)
    if not m:
        return None
    depth, i = 0, m.end() - 1
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[i + 1 : j]
    return None


def base_type(ty: str) -> str:
    """Unwrap `Option<T>` / whitespace to the inner type name."""
    ty = ty.strip()
    m = re.fullmatch(r"Option\s*<\s*(.+?)\s*>", ty)
    if m:
        return base_type(m.group(1))
    return ty


def fields_of(body: str) -> list[tuple[str, str, str]]:
    """(attrs, field name, type) for each field, attrs being the lines above it.

    Attributes are accumulated until their brackets balance: rustfmt splits a
    long `#[serde(...)]` across lines, and reading only the first line loses
    exactly the `deserialize_with` this guard is looking for. That mistake made
    the first draft report the sites it had just fixed.
    """
    out: list[tuple[str, str, str]] = []
    pending: list[str] = []
    depth = 0
    for line in body.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("//"):
            continue
        if depth > 0:
            pending.append(stripped)
            depth += stripped.count("[") - stripped.count("]")
            continue
        if stripped.startswith("#["):
            pending.append(stripped)
            depth = stripped.count("[") - stripped.count("]")
            continue
        m = FIELD_RE.match(line)
        if m:
            out.append(("\n".join(pending), m.group(1), m.group(2)))
        pending = []
    return out


USE_RE = re.compile(r"^\s*(?:pub\s+)?use\s+([^;]+);", re.M)


def crate_root(path: Path, repo: Path) -> Path:
    """Nearest ancestor holding a Cargo.toml — the crate this file belongs to."""
    d = path.parent
    while d != repo and d != d.parent:
        if (d / "Cargo.toml").exists():
            return d
        d = d.parent
    return repo


def import_hint(text: str, type_name: str) -> str | None:
    """The module path a file imports `type_name` from, if any.

    Name resolution matters here: this repo has FOUR distinct `PaginationQuery`
    types, three of them in one crate, and only one of each is the flattened
    one. Matching by bare name reported the other three as broken — a guard
    that red-flags correct code is worse than no guard, so resolve the import.
    """
    for use_path in USE_RE.findall(text):
        flat = " ".join(use_path.split())
        if type_name not in flat:
            continue
        # `use a::b::Type;` or `use a::b::{X, Type, Y};`
        head = flat.split("{")[0].rstrip(":").strip()
        if flat.endswith("::" + type_name) or flat == type_name:
            head = flat[: -(len(type_name) + 2)] or head
        return head
    return None


def collect(
    paths: list[Path], repo: Path
) -> tuple[dict[str, list[tuple[Path, Path, str]]], list[tuple[Path, str]]]:
    """struct name -> [(crate root, file, body)], plus [(file, Query<T> name)]."""
    defs: dict[str, list[tuple[Path, Path, str]]] = {}
    queries: list[tuple[Path, str]] = []
    for p in paths:
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        root = crate_root(p, repo)
        for qt in QUERY_EXTRACTOR_RE.findall(text):
            queries.append((p, qt))
        for name in STRUCT_RE.findall(text):
            body = struct_body(text, name)
            if body is not None:
                defs.setdefault(name, []).append((root, p, body))
    return defs, queries


def resolve(
    defs: dict[str, list[tuple[Path, Path, str]]],
    type_name: str,
    from_file: Path,
    repo: Path,
) -> list[tuple[Path, str]]:
    """Definitions of `type_name` visible from `from_file`, most specific first."""
    candidates = defs.get(type_name, [])
    if not candidates:
        return []
    # 1. defined in the same file
    same_file = [(f, b) for _r, f, b in candidates if f == from_file]
    if same_file:
        return same_file
    # 2. the module path the file imports it from
    try:
        text = from_file.read_text(encoding="utf-8", errors="replace")
    except OSError:
        text = ""
    hint = import_hint(text, type_name)
    if hint is None:
        # Glob imports (`use super::params::*;`) never name the type, and this
        # repo uses them — `kebutuhan_bmn/handlers/satker.rs` picks up its
        # `PaginationQuery` that way. Try each glob's module in turn.
        for use_path in USE_RE.findall(text):
            flat = " ".join(use_path.split())
            if not flat.endswith("::*"):
                continue
            segs = [
                x
                for x in flat[:-3].split("::")
                if x not in ("crate", "super", "self", "")
            ]
            if not segs:
                continue
            frag = "/".join(segs)
            by_glob = [
                (f, b)
                for _r, f, b in candidates
                if frag in f.with_suffix("").as_posix()
                or segs[-1] == f.with_suffix("").name
            ]
            if by_glob:
                return by_glob
    if hint:
        # `crate::shared::pagination` -> path fragment `shared/pagination`
        segs = [x for x in hint.split("::") if x not in ("crate", "super", "self", "")]
        if segs:
            frag = "/".join(segs)
            by_path = [
                (f, b)
                for _r, f, b in candidates
                if frag in f.with_suffix("").as_posix()
                or segs[-1] in f.with_suffix("").as_posix()
            ]
            if by_path:
                return by_path
    # 3. same crate, and unambiguous there
    root = crate_root(from_file, repo)
    same_crate = [(f, b) for r, f, b in candidates if r == root]
    if len(same_crate) == 1:
        return same_crate
    return []


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    paths = [
        p
        for root in ROOTS
        for p in (repo / root).rglob("*.rs")
        if "target" not in p.parts
    ]
    if not paths:
        print("check-query-flatten-numbers: found no .rs files to scan", file=sys.stderr)
        return 1

    defs, queries = collect(paths, repo)
    if not queries:
        # "Found nothing" is a broken scan, not a clean tree — this repo has
        # been bitten by guards that silently matched zero things.
        print(
            "check-query-flatten-numbers: no `Query<…>` extractors found at all; "
            "the scan is broken, not the tree",
            file=sys.stderr,
        )
        return 1

    findings: list[str] = []

    def visit(type_name: str, from_file: Path, via: str, depth: int = 0) -> None:
        if depth > 4:
            return
        for def_file, body in resolve(defs, type_name, from_file, repo):
            rel = def_file.relative_to(repo)
            for attrs, fname, ftype in fields_of(body):
                inner = base_type(ftype)
                if inner in NUMERIC and not TOLERANT_RE.search(attrs):
                    findings.append(
                        f"{rel}: `{type_name}.{fname}: {ftype.strip()}` is flattened "
                        f"into the query DTO `{via}` "
                        f"({from_file.relative_to(repo)}) but has no tolerant "
                        f"deserializer — every request naming it answers 400.\n"
                        f"    fix: #[serde(..., deserialize_with = "
                        f'"lib_core::serde_query::de_{inner}")]'
                    )
                elif "flatten" in attrs and inner not in NUMERIC:
                    visit(inner, def_file, via, depth + 1)

    checked = 0
    external = 0
    unresolved: list[str] = []
    for qfile, qt in queries:
        resolved = resolve(defs, qt, qfile, repo)
        if not resolved:
            if qt in defs:
                # Defined somewhere but not pinnable from here. Skipping would
                # be a silent blind spot in exactly the place a broken DTO would
                # hide, so this fails instead of passing quietly.
                unresolved.append(f"{qfile.relative_to(repo)}: Query<{qt}>")
            else:
                # No definition in the tree at all (e.g. serde_json::Value) —
                # nothing here to check.
                external += 1
            continue
        for _def_file, body in resolved:
            checked += 1
            for attrs, _fname, ftype in fields_of(body):
                if "flatten" not in attrs:
                    continue
                m = FLATTEN_ATTR_RE.search(attrs.replace("\n", " "))
                if not m or "flatten" not in m.group(1):
                    continue
                visit(base_type(ftype), qfile, qt)

    # Dedupe: the same DTO can appear in several handlers.
    findings = sorted(set(findings))

    if unresolved:
        print(
            "Query DTOs that could not be resolved to a definition (a blind spot, "
            "not a pass) — add an explicit import so the type is unambiguous:\n"
        )
        for u in sorted(set(unresolved)):
            print(f"  - {u}")
        return 1

    if findings:
        print("Numeric fields inside a flattened query type (these answer 400):\n")
        for f in findings:
            print(f"  - {f}\n")
        return 1

    print(
        f"OK: {len(queries)} `Query<…>` extractors — {checked} resolved, "
        f"{external} external; every numeric field reachable through "
        f"`#[serde(flatten)]` uses a tolerant deserializer."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
