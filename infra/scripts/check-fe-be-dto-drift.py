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
#
# The canonical envelope file is named explicitly rather than globbing all of
# `lib/`. `lib/*/src/**` would also pair the frontend's wire *requests* with
# lib-perlengkapan's same-named *domain* models — `CreateKebutuhanBmnRequest`
# exists in both, deliberately with different fields — and every such pair
# reports as drift. Only the envelope is genuinely shared across the boundary.
BE_GLOBS = (
    "layanan/*/src/**/*.rs",
    "layanan/*/crates/*/src/**/*.rs",
    "lib/perlengkapan/src/response.rs",
)

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
ALLOWLIST: dict[tuple[str, str], str] = {
    # The wire object is the backend's `DelegationView` = `Delegation` flattened
    # (`#[serde(flatten)]`) + these two fields. The frontend struct mirrors the
    # whole view under the shorter name, and this guard does not follow a
    # flatten on the BACKEND side, so it sees `Delegation` without them.
    ("Delegation", "berlaku"): "added by DelegationView (flatten + berlaku)",
    ("Delegation", "catatan_berlaku"): "added by DelegationView (flatten + catatan_berlaku)",
}


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


# ─────────────────────────────────────────────────────────────────────────────
# Duplicate wire envelopes.
#
# The field-level pass above UNIONS every backend struct sharing a name, on the
# assumption that two same-named definitions are an entity and a response view
# of it. That assumption is right for those, and it is exactly what makes it
# blind here: when a service copies the wire envelope locally and the copy
# drifts, the union quietly absorbs the difference. Every field the frontend
# asks for is present in *some* declaration, so nothing is reported.
#
# `penghapusan_bmn` shipped that way. Its local `PaginatedResponse` omitted
# `total_pages`, so serde rejected the entire response while the API answered
# 200 — the page rendered its error arm, and this guard passed. The union
# looked at the shared `lib_perlengkapan` twin (which has `total_pages`) and
# concluded all was well.
#
# So envelopes are checked for EXACT parity instead: same name, same fields,
# no union. A shared envelope exists precisely so its shape is defined once;
# a second declaration of it is the bug, whatever it currently contains.
ENVELOPE_NAMES = {"ApiResponse", "PaginatedResponse", "ListResponse", "ErrorBody"}


def _service_of(path: Path) -> str:
    """Which deployable owns this file: a service name, or the shared `lib`.

    Sub-crates are folded into their service: `layanan/authenc/crates/api` and
    `layanan/authenc/crates/core` are one deployable, so an envelope declared
    in both is the duplicate this check exists to find.
    """
    try:
        rel = path.relative_to(ROOT)
    except ValueError:
        return "?"
    parts = rel.parts
    if parts and parts[0] == "layanan" and len(parts) > 1:
        return f"layanan/{parts[1]}"
    if parts and parts[0] == "lib" and len(parts) > 1:
        return f"lib/{parts[1]}"
    return parts[0] if parts else "?"


def lib_deps_of_service(service: str) -> set[str]:
    """Shared `lib` crates the service actually depends on, from its Cargo.toml.

    This is what makes the comparison honest. Every service declaring its own
    envelope is not a defect — secreton's `ApiResponse` carries
    `error`/`metadata` where perlengkapan's carries `message`, and the two are
    unrelated types in unrelated domains. What IS a defect is a service that
    depends on a shared crate defining the envelope and declares a second copy
    anyway, because then two definitions of the same wire type exist in one
    deployable and only one compiler ever sees both.
    """
    manifest = ROOT / service / "Cargo.toml"
    if not manifest.exists():
        return set()
    try:
        text = manifest.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return set()
    return set(re.findall(r"^\s*(lib-\w+)\s*=", text, re.M))


def _lib_crate_of(service: str) -> str | None:
    """`lib/perlengkapan` -> `lib-perlengkapan` (the Cargo package name)."""
    if not service.startswith("lib/"):
        return None
    return "lib-" + service.split("/", 1)[1]


def envelope_field_sets(backend: dict[str, list[dict]]) -> dict[str, list[set[str]]]:
    """Field sets of every envelope declaration that has a duplicate to answer for.

    Two ways a declaration qualifies:

    * **same service** — a service has one wire envelope; a second declaration
      inside it is necessarily a stale copy, whatever it currently contains.
    * **shadowing its own shared crate** — the service depends on a `lib` crate
      that defines this envelope, so the shared definition is the authority and
      the local one must match it exactly.

    The second is the `penghapusan_bmn` case: it re-exported the shared
    `PaginatedResponse` but kept a local `ApiResponse`, and the local one was
    the shape that drifted.
    """
    out: dict[str, list[set[str]]] = {}

    for name, entries in backend.items():
        shaped: list[tuple[str, set[str]]] = []
        for e in entries:
            fields = set(e["fields"])
            # Shape-based too, so a locally-invented envelope name is caught:
            # `success` + `data` is the wire envelope, whatever it is called.
            if name in ENVELOPE_NAMES or {"success", "data"} <= fields:
                shaped.append((_service_of(e["path"]), fields))
        if not shaped:
            continue

        by_service: dict[str, list[set[str]]] = {}
        for service, fields in shaped:
            by_service.setdefault(service, []).append(fields)

        # (a) Duplicates inside one service.
        for service, variants in by_service.items():
            if len(variants) > 1:
                out.setdefault(f"{name} [{service}]", []).extend(variants)

        # (b) A local declaration shadowing a shared crate the service depends
        #     on. Compare against that crate's declared shape, not a union, so
        #     a local copy that GROWS a field is reported too.
        for service, variants in by_service.items():
            if service.startswith("lib/"):
                continue
            for dep in lib_deps_of_service(service):
                canonical = by_service.get(f"lib/{dep.removeprefix('lib-')}")
                if canonical:
                    # The canonical declaration is the one the crate exports;
                    # `lib` crates keep a single envelope per name.
                    out.setdefault(
                        f"{name} [{service} shadows {dep}]", []
                    ).extend([canonical[0], *variants])
                    break

    return out


def check_duplicate_envelopes(backend: dict[str, list[dict]]) -> list[str]:
    """Report every envelope declared more than once for the same authority.

    A duplicate is reported whether or not the copies currently agree. Two
    identical declarations are not a benign redundancy — they are the setup for
    the drift, and this repository's own record is that a rule or shape living
    in two places eventually holds in only one of them (`secreton/AGENTS.md`,
    the two-authorizer note). The drift is the symptom; the copy is the cause,
    so the fix is always the same: delete it and re-export the shared type.

    When the copies have already diverged, the differing fields are named so
    the report says which side is wrong. For a shadowed shared crate the
    authority is that crate; for same-service duplicates it is the union.
    """
    drift: list[str] = []
    for label, variants in sorted(envelope_field_sets(backend).items()):
        if len(variants) < 2:
            continue

        shadow = "shadows" in label
        if shadow:
            authority, candidates = variants[0], variants[1:]
        else:
            authority, candidates = set().union(*variants), variants

        labels = ", ".join(f"#{i}" for i in range(1, len(candidates) + 1))
        for i, fields in enumerate(candidates, 1):
            missing = sorted(authority - fields)
            extra = sorted(fields - authority)
            detail = []
            if missing:
                detail.append(f"omits {', '.join(missing)}")
            if extra:
                detail.append(f"adds {', '.join(extra)}")
            what = " and ".join(detail) if detail else "matches the authority"
            drift.append(
                f"  DUPLICATE-ENVELOPE {label} — {len(candidates)} local "
                f"declaration(s) ({labels}); #{i} {what}"
            )
    return drift


def selftest() -> int:
    """Prove the duplicate-envelope check fires on the bug it exists for.

    Reproduces the `penghapusan_bmn` shape: a shared crate defines the
    envelope with `total_pages`, and a service that depends on that crate
    declares its own copy without the field. The field-level union pass sees
    nothing wrong — it merges same-named declarations and so absorbs the
    difference, which is precisely why the original bug shipped.
    """
    def entry(path: str, fields: tuple[str, ...]) -> dict:
        return {
            "path": ROOT / path,
            "attrs": "Serialize",
            "fields": {k: {} for k in fields},
            "flattened": False,
            "line": 1,
        }

    shared = entry(
        "lib/perlengkapan/src/response.rs",
        ("success", "data", "message", "total", "page", "per_page", "total_pages"),
    )
    local = entry(
        "layanan/perlengkapan/src/penghapusan_bmn/handlers.rs",
        ("success", "data", "message", "total", "page", "per_page"),
    )
    fake = {"PaginatedResponse": [shared, local]}

    drift = check_duplicate_envelopes(fake)
    if not drift:
        print("check-fe-be-dto-drift: SELFTEST FAILED — the duplicate-envelope "
              "check did not fire on the penghapusan_bmn shape.", file=sys.stderr)
        return 1
    if "total_pages" not in drift[0]:
        print("check-fe-be-dto-drift: SELFTEST FAILED — fired, but did not name "
              "the omitted field.", file=sys.stderr)
        return 1

    # A local copy that INVENTS a field is drift too: the frontend would be
    # written against a key the authority never sends.
    grown = dict(local)
    grown["fields"] = {**local["fields"], "invented": {}}
    if not check_duplicate_envelopes({"PaginatedResponse": [shared, grown]}):
        print("check-fe-be-dto-drift: SELFTEST FAILED — did not fire on a local "
              "copy that adds a field.", file=sys.stderr)
        return 1

    # Two identical declarations are still reported: the copy is the setup for
    # the next drift, and the fix (delete it) is the same either way.
    if not check_duplicate_envelopes({"PaginatedResponse": [shared, dict(shared)]}):
        print("check-fe-be-dto-drift: SELFTEST FAILED — did not fire on identical "
              "duplicate declarations.", file=sys.stderr)
        return 1

    # A service with its own unrelated envelope and no shared dependency is
    # NOT drift — that is secreton today, and flagging it would be the
    # false positive that gets a guard deleted.
    if check_duplicate_envelopes(
        {"ApiResponse": [entry("layanan/secreton/crates/api/src/response.rs",
                               ("success", "data", "error", "metadata"))]}
    ):
        print("check-fe-be-dto-drift: SELFTEST FAILED — flagged a standalone "
              "service envelope.", file=sys.stderr)
        return 1

    print("check-fe-be-dto-drift: selftest ok (duplicate-envelope drift detected)")
    return 0


def count_envelope_declarations(backend: dict[str, list[dict]]) -> int:
    """How many envelope-shaped declarations exist across all services."""
    n = 0
    for name, entries in backend.items():
        for e in entries:
            fields = set(e["fields"])
            if name in ENVELOPE_NAMES or {"success", "data"} <= fields:
                n += 1
    return n


def main() -> int:
    if "--selftest" in sys.argv:
        return selftest()

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

    env_drift = check_duplicate_envelopes(backend)

    if not required and not empty and not enum_drift and not env_drift:
        print(
            f"check-fe-be-dto-drift: {compared} frontend DTOs, "
            f"{enums_compared} mirrored enums and "
            f"{count_envelope_declarations(backend)} envelope declaration(s) "
            f"({len(envelope_field_sets(backend))} duplicated) match their backend twin."
        )
        return 0

    if env_drift:
        print("Backend envelopes declared more than once, with drift:\n", file=sys.stderr)
        for line in env_drift:
            print(line, file=sys.stderr)
        print(
            "\nThe shared envelope exists so its shape is defined once. A local\n"
            "copy is not compared field-by-field against the original by any\n"
            "compiler — the two live in different crates and never meet at a\n"
            "type boundary. The union pass above cannot see it either, because\n"
            "it merges same-named declarations and so absorbs the difference.\n"
            "Delete the copy and re-export the shared type.\n",
            file=sys.stderr,
        )
        if not required and not empty and not enum_drift:
            return 1

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
