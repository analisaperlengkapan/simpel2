#!/usr/bin/env python3
"""Fail when a frontend DTO names a field its backend twin never sends.

The frontend declares its own copy of every wire type in
`antarmuka/*/src/api/*.rs`. Nothing compares those copies to the backend
structs they mirror, and they have drifted three times so far -- every time
silently, because the failure is a deserialise error inside a resource, not an
HTTP status:

  * `Ukuran` invented id/size/created_at/updated_at (#117) -- whole page
    stuck in ErrorState.
  * `PegawaiPakaianDinas` invented id/pegawai_id/pegawai_nama/... (#117) --
    the self-service form never prefilled.
  * `PengajuanPakaianDinas` named five fields that do not exist (#818) -- the
    campaign list AND the report page's period dropdown both dead, against a
    backend answering 200 every time.

The fourth drift was not a struct at all, so this guard could not see it:
`KebutuhanBmnStatus` is mirrored as an ENUM WITH DISCRIMINANTS, and the two
copies disagreed from 2004 down (frontend 2004 = AnalisisKelayakan, backend
2004 = SubmitPusat). Nothing failed to deserialise -- the code is just an
integer -- so every badge on a request queued for Validator Pusat announced
that Pusat had already analysed it. Enum parity is therefore checked too, by
variant name AND value.

Two severities, both failures:

  MISSING-REQUIRED  the frontend field is neither `Option<...>` nor
                    `#[serde(default)]`, so serde rejects the ENTIRE response
                    and every consumer of that endpoint breaks.

  ALWAYS-EMPTY      the field is optional, so it deserialises to `None`
                    forever. Softer, and worse to find: the page renders, just
                    without the data. All three cases live when this guard was
                    written were real (`JenisPakaianDinas.keterangan` was
                    `deskripsi` upstream, so a description typed into the create
                    form was dropped AND the list showed "-").

Pairing is by struct NAME, which is the convention this repo already follows.
Frontend types with no same-named backend struct are skipped -- they are
view-models, not wire types. The reverse direction is NOT checked: a frontend
DTO may deliberately be a subset, since serde ignores unknown fields, and
several are documented as such.

No YAML, no third-party imports: the ARC runner image carries neither PyYAML
nor yq (see project_arc_runner_image_missing_tools).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

FE_GLOB = "antarmuka/*/src/api/*.rs"
# Enums are mirrored in the same api modules, but the backend keeps its
# status enums under `models/`, which BE_GLOBS already covers.
BE_GLOBS = ("layanan/*/src/**/*.rs", "layanan/*/crates/*/src/**/*.rs")

SKIP_DIRS = {"target", "node_modules", "dist", ".git"}

# Struct with its preceding doc-comment / attribute lines, then its body.
STRUCT_RE = re.compile(
    r"((?:^[ \t]*(?:///[^\n]*|//![^\n]*|#\[[^\n]*\])[ \t]*\n)*)"
    r"^[ \t]*pub struct (\w+)[^\n{;]*\{(.*?)^\}",
    re.M | re.S,
)

# `pub name: Type` with its own attribute lines.
FIELD_RE = re.compile(
    r"((?:^[ \t]*(?:///[^\n]*|#\[[^\n]*\])[ \t]*\n)*)" r"^[ \t]*pub (\w+)[ \t]*:[ \t]*([^,\n]+)",
    re.M,
)

SERDE_RENAME_RE = re.compile(r'serde\s*\(\s*rename\s*=\s*"([^"]+)"')
SERDE_FLATTEN = "flatten"
SERDE_DEFAULT_RE = re.compile(r"serde\s*\([^)]*\bdefault\b")
SERDE_SKIP_RE = re.compile(r"serde\s*\([^)]*\bskip\b")

# Frontend fields allowed to have no backend counterpart. Every entry needs a
# written reason; an empty allowlist is the healthy state.
ALLOWLIST: dict[tuple[str, str], str] = {}


# `pub enum X { Variant = 1000, ... }` -- only the explicit-discriminant kind.
# A mirrored enum whose values are positional carries no wire meaning to compare.
ENUM_RE = re.compile(
    r"^[ \t]*pub enum (\w+)[ \t]*\{(.*?)^\}",
    re.M | re.S,
)
ENUM_VARIANT_RE = re.compile(r"^[ \t]*(\w+)[ \t]*=[ \t]*(-?\d+)[ \t]*,", re.M)


def parse_enums(path: Path) -> dict[str, dict[str, int]]:
    """Name -> {variant: discriminant} for enums that spell their values out."""
    try:
        src = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return {}
    found: dict[str, dict[str, int]] = {}
    for m in ENUM_RE.finditer(src):
        name, body = m.group(1), m.group(2)
        variants = {v: int(d) for v, d in ENUM_VARIANT_RE.findall(body)}
        if variants:
            found[name] = variants
    return found


def collect_enums(globs) -> dict[str, dict[str, int]]:
    out: dict[str, dict[str, int]] = {}
    for glob in globs:
        for path in ROOT.glob(glob):
            if not _relevant(path):
                continue
            for name, variants in parse_enums(path).items():
                out.setdefault(name, {}).update(variants)
    return out


def _relevant(path: Path) -> bool:
    return not any(part in SKIP_DIRS for part in path.parts)


def _wire_name(field: str, attrs: str) -> str | None:
    """Serialised name of a field, or None when it never reaches the wire."""
    if SERDE_SKIP_RE.search(attrs):
        return None
    renamed = SERDE_RENAME_RE.search(attrs)
    return renamed.group(1) if renamed else field


def parse_structs(path: Path) -> dict[str, list[dict]]:
    try:
        src = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return {}
    found: dict[str, list[dict]] = {}
    for m in STRUCT_RE.finditer(src):
        attrs, name, body = m.group(1), m.group(2), m.group(3)
        line = src.count("\n", 0, m.start()) + 1
        fields: dict[str, dict] = {}
        flattened = False
        for fm in FIELD_RE.finditer(body):
            fattrs, fname, ftype = fm.group(1), fm.group(2), fm.group(3).strip()
            if SERDE_FLATTEN in fattrs:
                # A flattened field contributes its inner struct's keys, which
                # this parser does not resolve. Mark the struct open-ended so it
                # is not judged on names it may legitimately absorb.
                flattened = True
                continue
            wire = _wire_name(fname, fattrs)
            if wire is None:
                continue
            fields[wire] = {
                "declared": fname,
                "type": ftype,
                "optional": ftype.startswith("Option<"),
                "default": bool(SERDE_DEFAULT_RE.search(fattrs)),
                "line": line + body.count("\n", 0, fm.start()),
            }
        found.setdefault(name, []).append(
            {
                "path": path,
                "attrs": attrs,
                "fields": fields,
                "flattened": flattened,
                "line": line,
            }
        )
    return found


def collect(globs, derive: str) -> dict[str, list[dict]]:
    out: dict[str, list[dict]] = {}
    for glob in globs:
        for path in ROOT.glob(glob):
            if not _relevant(path):
                continue
            for name, entries in parse_structs(path).items():
                for entry in entries:
                    if derive in entry["attrs"]:
                        out.setdefault(name, []).append(entry)
    return out


def main() -> int:
    frontend = collect((FE_GLOB,), "Deserialize")
    backend = collect(BE_GLOBS, "Serialize")

    if not frontend:
        # "Found nothing" must never read as "all clean" -- the same failure
        # mode that made check-ci-summary-scope.py treat a renamed job as tidy.
        print("check-fe-be-dto-drift: parsed ZERO frontend DTOs — the glob or "
              "the struct pattern is broken, not the repo.", file=sys.stderr)
        return 1

    required: list[str] = []
    empty: list[str] = []
    compared = 0

    for name in sorted(frontend):
        if name not in backend:
            continue
        fe = frontend[name][0]
        if fe["flattened"]:
            continue
        compared += 1

        # A name can be defined more than once upstream (an entity and a
        # response view of it). Union their fields: any of them may be what the
        # endpoint returns, and flagging a field one of them carries would be a
        # false positive.
        be_fields: set[str] = set()
        for entry in backend[name]:
            be_fields |= set(entry["fields"])

        rel = fe["path"].relative_to(ROOT)
        for wire, meta in fe["fields"].items():
            if wire in be_fields or (name, wire) in ALLOWLIST:
                continue
            where = f"{rel}:{meta['line']}  {name}.{meta['declared']}"
            if meta["optional"] or meta["default"]:
                empty.append(f"  ALWAYS-EMPTY     {where}")
            else:
                required.append(f"  MISSING-REQUIRED {where}")

    # Enum discriminants: same idea, different shape. A mirrored status enum
    # that renumbers itself does not fail to parse -- it silently relabels.
    fe_enums = collect_enums((FE_GLOB,))
    be_enums = collect_enums(BE_GLOBS)
    enum_drift: list[str] = []
    enums_compared = 0
    for name in sorted(fe_enums):
        if name not in be_enums:
            continue
        enums_compared += 1
        fe_v, be_v = fe_enums[name], be_enums[name]
        # Compare by VALUE, which is what crosses the wire. A frontend variant
        # is free to be absent; one that claims a value the backend gives to a
        # different state is the bug.
        be_by_value = {v: k for k, v in be_v.items()}
        for variant, value in sorted(fe_v.items(), key=lambda kv: kv[1]):
            upstream = be_by_value.get(value)
            if upstream is None:
                enum_drift.append(
                    f"  UNKNOWN-CODE     {name}::{variant} = {value} — the backend has no state with this value"
                )
            elif upstream != variant:
                enum_drift.append(
                    f"  RENUMBERED       {name}::{variant} = {value} — upstream {value} is {name}::{upstream}"
                )

    if not required and not empty and not enum_drift:
        print(
            f"check-fe-be-dto-drift: {compared} frontend DTOs and "
            f"{enums_compared} mirrored enums match their backend twin."
        )
        return 0

    if enum_drift:
        print("Frontend enum values that mean something else upstream:\n", file=sys.stderr)
        for line in enum_drift:
            print(line, file=sys.stderr)
        print(
            "\nA renumbered mirror does not fail to deserialise — the code is an\n"
            "integer either way. It relabels: KebutuhanBmnStatus 2004 was\n"
            "AnalisisKelayakan here and SubmitPusat upstream, so every request\n"
            "merely queued for Validator Pusat was shown as already analysed.\n",
            file=sys.stderr,
        )
        if not required and not empty:
            return 1

    print("Frontend DTO fields the backend never sends:\n", file=sys.stderr)
    for line in required:
        print(line, file=sys.stderr)
    for line in empty:
        print(line, file=sys.stderr)
    print(
        "\nMISSING-REQUIRED rejects the whole response — every consumer of that\n"
        "endpoint renders its error arm against a backend answering 200.\n"
        "ALWAYS-EMPTY deserialises to None forever: the page renders without the\n"
        "data, which is harder to notice and has shipped three times.\n\n"
        "Rename the field to what the backend actually serialises. Only add to\n"
        "ALLOWLIST with a written reason, and only when the two really are\n"
        "different types that happen to share a name.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
