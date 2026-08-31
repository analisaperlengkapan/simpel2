#!/usr/bin/env python3
"""Fail when a frontend URL sends a query parameter no backend `Query<T>` reads.

`check-fe-be-dto-drift.py` compares the frontend's copies of RESPONSE structs
against the backend's. Query parameters are the other half of the same wire
contract and nothing compared them, so they drifted the same silent way:

  * `GET /pakaian-dinas/spesifikasi` narrows by jenis. The backend named that
    parameter `jenis_id`; both frontend callers have always sent
    `jenis_pakaian_dinas_id` -- the name of the column and of the field in the
    create DTO. serde DROPS query keys it does not know, so the filter
    deserialised to `None` on every request and the endpoint answered 200 with
    every jenis's rows: the master drawer listed other jenis's spesifikasi, and
    the campaign form offered Toga rows under PDH -- into a campaign that then
    freezes whatever was picked.

Why nothing else catches it:

  * the status is 200 and the JSON envelope is correct, so no status-based
    check and no deserialise failure ever fires,
  * the two names live in different crates that never meet at a type boundary,
    so `cargo check` cannot see them,
  * an e2e test that merely reads the list back passes -- the wanted row IS in
    the unfiltered response, alongside the ones that should not be.

WHAT THIS CHECKS, precisely: every literal query-parameter name a frontend URL
builds must be declared by SOME backend struct that is extracted with
`Query<...>` (following `#[serde(flatten)]` and honouring `serde(rename)`).

WHAT IT DOES NOT CHECK: that the parameter belongs to the struct behind THAT
path. Pairing URL -> route -> handler -> query type is possible but needs the
route table parsed and the frontend's runtime-built paths resolved; a wrong
derivation is worse than none (project_gate_scope_must_be_derived). The union
form has no false positives and would have caught the drift above, since
`jenis_pakaian_dinas_id` was a field of a request BODY, never of any query.

No YAML, no third-party imports: the ARC runner image carries neither PyYAML
nor yq (see project_arc_runner_image_missing_tools).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# The WHOLE frontend tree, not `src/api/`: portal keeps its clients in
# `src/utils/` (`authenc_api.rs`), so a scan scoped to `src/api/` would have
# covered perlengkapan only and left every portal query parameter unguarded --
# the same hand-written-scope failure this guard exists to prevent
# (project_gate_scope_must_be_derived). Files that mention no `/api/` path are
# skipped below, which keeps the widened scan from reading query-like strings
# out of code that is not an API client.
FE_GLOBS = ("antarmuka/*/src/**/*.rs",)
BE_GLOBS = ("layanan/*/src/**/*.rs", "layanan/*/crates/*/src/**/*.rs")

SKIP_DIRS = {"target", "node_modules", "dist", ".git"}

# Query parameters the frontend may send that no handler declares, each with a
# written reason. An empty allowlist is the healthy state.
ALLOWLIST: dict[str, str] = {}

# `Query<Name>` and `Query<Option<Name>>` in a handler signature.
QUERY_EXTRACTOR_RE = re.compile(r"\bQuery<\s*(?:Option<\s*)?(\w+)")

# A struct with its body. Attributes are captured with each field instead.
STRUCT_RE = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?struct[ \t]+(\w+)[^\n{;]*\{(.*?)^\}",
    re.M | re.S,
)

# `pub name: Type` (or a private field) preceded by its own attribute lines.
FIELD_RE = re.compile(
    r"((?:^[ \t]*(?:///[^\n]*|#\[[^\n]*\])[ \t]*\n)*)"
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(\w+)[ \t]*:[ \t]*([^,\n]+)",
    re.M,
)

SERDE_RENAME_RE = re.compile(r'serde\s*\(\s*[^)]*rename\s*=\s*"([^"]+)"')
SERDE_ALIAS_RE = re.compile(r'serde\s*\(\s*[^)]*alias\s*=\s*"([^"]+)"')

# `name=` inside the query part of a Rust string literal. The frontend builds
# every URL as a literal or a `format!`, so the NAMES are always spelled out
# even when the values are placeholders.
FE_PARAM_RE = re.compile(r'[?&]([a-zA-Z_][a-zA-Z0-9_]*)=')

# Rust line and block comments. Stripped before scanning so a parameter named
# only in prose is not read as code (project_comments_are_parsed_as_code).
LINE_COMMENT_RE = re.compile(r"//[^\n]*")
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.S)
# String literals, so a `//` INSIDE a URL ("https://...") is not mistaken for
# the start of a comment.
STRING_LITERAL_RE = re.compile(r'"(?:[^"\\]|\\.)*"')


def strip_comments(src: str) -> str:
    """Remove comments, preserving string literals (URLs contain `//`)."""
    out: list[str] = []
    idx = 0
    for m in STRING_LITERAL_RE.finditer(src):
        chunk = src[idx : m.start()]
        chunk = BLOCK_COMMENT_RE.sub(" ", chunk)
        chunk = LINE_COMMENT_RE.sub("", chunk)
        out.append(chunk)
        out.append(m.group(0))
        idx = m.end()
    tail = src[idx:]
    tail = BLOCK_COMMENT_RE.sub(" ", tail)
    tail = LINE_COMMENT_RE.sub("", tail)
    out.append(tail)
    return "".join(out)


def _relevant(path: Path) -> bool:
    return not any(part in SKIP_DIRS for part in path.parts)


def _iter(globs) -> list[Path]:
    seen: set[Path] = set()
    for glob in globs:
        for path in ROOT.glob(glob):
            if path.is_file() and _relevant(path):
                seen.add(path)
    return sorted(seen)


def parse_structs(src: str) -> dict[str, list[tuple[str, str, str]]]:
    """Name -> [(field, attrs, type)] for every struct in the source."""
    out: dict[str, list[tuple[str, str, str]]] = {}
    for m in STRUCT_RE.finditer(src):
        name, body = m.group(1), m.group(2)
        out[name] = [
            (field, attrs, ty.strip())
            for attrs, field, ty in FIELD_RE.findall(body)
        ]
    return out


def wire_names(
    struct: str,
    structs: dict[str, list[tuple[str, str, str]]],
    seen: set[str] | None = None,
) -> set[str]:
    """Query keys `struct` accepts, descending through `#[serde(flatten)]`."""
    seen = seen or set()
    if struct in seen or struct not in structs:
        return set()
    seen.add(struct)
    names: set[str] = set()
    for field, attrs, ty in structs[struct]:
        if "flatten" in attrs:
            inner = re.sub(r"^Option<\s*|\s*>$", "", ty).strip()
            inner = inner.split("<")[0].split("::")[-1]
            names |= wire_names(inner, structs, seen)
            continue
        renamed = SERDE_RENAME_RE.search(attrs)
        names.add(renamed.group(1) if renamed else field)
        names |= set(SERDE_ALIAS_RE.findall(attrs))
    return names


def backend_query_keys() -> set[str]:
    """Every query key any handler can read, across the backend services."""
    sources = {p: strip_comments(p.read_text(encoding="utf-8", errors="replace")) for p in _iter(BE_GLOBS)}
    structs: dict[str, list[tuple[str, str, str]]] = {}
    for src in sources.values():
        structs.update(parse_structs(src))

    extracted: set[str] = set()
    for src in sources.values():
        extracted |= set(QUERY_EXTRACTOR_RE.findall(src))

    keys: set[str] = set()
    for name in extracted:
        keys |= wire_names(name, structs)
    return keys, extracted


def query_part(literal: str) -> str | None:
    """The query portion of a string literal, or None if it carries none.

    A literal counts when it contains `?` (a whole URL) or begins with `&`
    (a fragment appended to a URL built elsewhere -- which is exactly how the
    spesifikasi filter was sent, so dropping that form would blind the guard
    to the bug it exists for).

    A FORM BODY looks identical to a query string otherwise, and this is what
    separates them: `"grant_type=authorization_code&code={}&..."` is posted as
    `application/x-www-form-urlencoded` by the OAuth token exchange, has no
    `?`, and does not start with `&`. Without this rule its fields read as
    query parameters no `Query<...>` declares, and the guard would demand an
    allowlist entry for correct code.
    """
    body = literal[1:-1]  # strip the surrounding quotes
    if "?" in body:
        return body[body.index("?") :]
    if body.startswith("&"):
        return body
    return None


def frontend_params() -> dict[str, list[str]]:
    """Query key -> the frontend files that send it."""
    found: dict[str, list[str]] = {}
    for path in _iter(FE_GLOBS):
        src = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        for literal in STRING_LITERAL_RE.findall(src):
            query = query_part(literal)
            if query is None:
                continue
            for name in FE_PARAM_RE.findall(query):
                rel = str(path.relative_to(ROOT))
                found.setdefault(name, [])
                if rel not in found[name]:
                    found[name].append(rel)
    return found


def main() -> int:
    keys, extracted = backend_query_keys()
    if not extracted:
        print(
            "FAIL: no `Query<...>` extractor found in the backend — the "
            "derivation is broken, not the code.",
            file=sys.stderr,
        )
        return 2
    if not keys:
        print(
            "FAIL: `Query<...>` extractors found but no fields derived from "
            "them — the struct parser is broken.",
            file=sys.stderr,
        )
        return 2

    sent = frontend_params()
    if not sent:
        print(
            "FAIL: no query parameters found in any frontend api module — the "
            "derivation is broken, not the code.",
            file=sys.stderr,
        )
        return 2

    problems: list[str] = []
    for name in sorted(sent):
        if name in keys or name in ALLOWLIST:
            continue
        where = ", ".join(sent[name])
        problems.append(
            f"  {name}\n"
            f"      sent by: {where}\n"
            f"      read by: no backend struct behind a `Query<...>` extractor"
        )

    if problems:
        print(
            "Frontend sends query parameters the backend never reads.\n"
            "serde drops unknown query keys, so these requests answer 200 with\n"
            "the filter silently ignored — no status, no deserialise error.\n",
            file=sys.stderr,
        )
        print("\n".join(problems), file=sys.stderr)
        print(
            f"\nChecked {len(sent)} parameter name(s) against "
            f"{len(keys)} key(s) from {len(extracted)} query struct(s).",
            file=sys.stderr,
        )
        return 1

    print(
        f"OK: {len(sent)} frontend query parameter(s) all reach a backend "
        f"`Query<...>` struct ({len(keys)} key(s), {len(extracted)} struct(s))."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
