#!/usr/bin/env python3
"""The e2e fixtures must not invent a name for a barang code that exists.

Measured against the staging snapshot on 2026-08-30, the SIMAN codification is a
strict FUNCTION: over 624 533 rows, 2 033 distinct `kd_brg` values map to exactly
2 033 names — **zero** codes carry two. (The reverse is not a function: seven
names, "Genset" and "Helmet" among them, are shared by two codes each. That is
why a picker has to return a code, never a name.)

The e2e seed broke that function and nothing noticed, because each individual
pair looked plausible and every code it used was real:

    3050104001  is "Lemari Besi/Metal"  (7 428 rows) — the seed filed a Toyota
                                                       Avanza under it
    3050201002  is "Meja Kerja Kayu"   (43 791 rows) — the seed filed a Honda
                                                       Vario under it
    3100102003  is "Note Book"          (2 141 rows) — the seed called it
                                                       "Personal Computer Unit"
    3100105010  carries no real asset at all

One fixture row against 43 791 real ones is still a counter-example, and it made
the codification look many-to-many — the exact property a codification picker
must be able to rely on. Worse, ~10 e2e assertions asserted the invented strings,
so the suite was certifying the fiction rather than catching it.

CI has no real SIMAN data, so this guard cannot compare names against the source.
What it CAN enforce is the shape the source has, plus the agreement between the
two seed files that a human currently maintains by hand:

  (1) `kd_brg -> ur_sskel` is a function across every seeded siman_aset row.
  (2) Every permit in `izin_pemakaian_bmn` names a seeded asset: its
      (`bmn_nup`, `bmn_kode_barang`) pair must match that asset's
      (`no_aset`, `kd_brg`). A permit that disagrees is a permit for a
      different asset than the one the picker and the availability check
      resolve, and the two seed files drift apart silently.

`penghapusan_bmn` is deliberately EXCLUDED from (2): its H2 fixture exists to
exercise the "NUP found here but under a different barang code" branch, so a
mismatch there is the point. Excluding it is a decision, not an oversight — a
new mismatch outside that table is a bug.

Run: python3 infra/scripts/check-e2e-fixture-codification.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MULTISATKER = ROOT / "tests/fixtures/e2e/seed-multisatker.sql"
WORKFLOW = ROOT / "tests/fixtures/e2e/seed-perlengkapan-workflow.sql"

# Every barang code the e2e fixtures use, with the name SIMAN gives it and the
# number of real assets carrying it. NOT hand-written — read off the source:
#
#   SELECT kd_brg, ur_sskel, COUNT(*) FROM integrasi.siman_aset
#    WHERE no_aset NOT LIKE 'E2E-%' GROUP BY 1, 2 ORDER BY 3 DESC;
#
# A code the fixture uses but this table does not list is a HARD failure, and
# that is the point: adding a fixture asset forces the author to run the query
# above rather than invent a plausible-looking name. The counts are recorded so
# a later reader can tell a real code from one that only ever existed here —
# the previous fixture used 3100105010, which carried no real asset at all.
VERIFIED_AGAINST = "staging snapshot, 624 533 rows, 2026-08-30"
SIMAN_CODIFICATION: dict[str, tuple[str, int]] = {
    "3020101003": ("Station Wagon", 785),
    "3020104001": ("Sepeda Motor", 4060),
    "3100102003": ("Note Book", 2141),
    "2010104001": ("Tanah Bangunan Kantor Pemerintah", 906),
    "3100203003": ("Printer (Peralatan Personal Komputer)", 15654),
}


def strip_comments(sql: str) -> str:
    """Comments are parsed as code often enough in this repo to be a rule.

    A `--` inside a string literal is not a comment, so quotes are tracked.
    """
    out, i, in_str = [], 0, False
    while i < len(sql):
        ch = sql[i]
        if in_str:
            out.append(ch)
            if ch == "'":
                # '' is an escaped quote, not the end of the literal.
                if sql[i + 1 : i + 2] == "'":
                    out.append("'")
                    i += 2
                    continue
                in_str = False
            i += 1
            continue
        if ch == "'":
            in_str = True
            out.append(ch)
            i += 1
            continue
        if sql.startswith("--", i):
            j = sql.find("\n", i)
            i = len(sql) if j < 0 else j
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def split_top_level(text: str, sep: str) -> list[str]:
    """Split on `sep`, ignoring separators inside quotes or nested parens."""
    parts, buf, depth, in_str = [], [], 0, False
    i = 0
    while i < len(text):
        ch = text[i]
        if in_str:
            buf.append(ch)
            if ch == "'":
                if text[i + 1 : i + 2] == "'":
                    buf.append("'")
                    i += 2
                    continue
                in_str = False
            i += 1
            continue
        if ch == "'":
            in_str = True
            buf.append(ch)
        elif ch == "(":
            depth += 1
            buf.append(ch)
        elif ch == ")":
            depth -= 1
            buf.append(ch)
        elif ch == sep and depth == 0:
            parts.append("".join(buf))
            buf = []
        else:
            buf.append(ch)
        i += 1
    parts.append("".join(buf))
    return parts


def unquote(value: str) -> str | None:
    v = value.strip()
    if v.upper() == "NULL":
        return None
    if v.startswith("'") and v.endswith("'") and len(v) >= 2:
        return v[1:-1].replace("''", "'")
    return v


def parse_insert(sql: str, table: str) -> list[dict[str, str | None]]:
    """Rows of the first `INSERT INTO <table> (cols) VALUES (...), (...)`."""
    m = re.search(
        rf"INSERT\s+INTO\s+{re.escape(table)}\s*\((?P<cols>[^)]*)\)\s*VALUES\s*",
        sql,
        re.IGNORECASE,
    )
    if not m:
        raise SystemExit(f"::error::INSERT INTO {table} not found — did the seed move?")
    cols = [c.strip() for c in m.group("cols").split(",")]

    # The VALUES list ends at the statement's `;`, or at ON CONFLICT.
    rest = sql[m.end() :]
    end = len(rest)
    for stop in re.finditer(r";|ON\s+CONFLICT", rest, re.IGNORECASE):
        end = stop.start()
        break
    body = rest[:end]

    rows = []
    for tup in split_top_level(body, ","):
        t = tup.strip()
        if not (t.startswith("(") and t.endswith(")")):
            continue
        values = [unquote(v) for v in split_top_level(t[1:-1], ",")]
        if len(values) != len(cols):
            raise SystemExit(
                f"::error::{table}: row has {len(values)} values for {len(cols)} columns"
            )
        rows.append(dict(zip(cols, values)))
    if not rows:
        raise SystemExit(f"::error::{table}: no VALUES rows parsed")
    return rows


def norm_kode(kode: str | None) -> str:
    """SIMAN stores ten undotted digits; the rest of the system writes dots."""
    return re.sub(r"[^0-9]", "", kode or "")


def check(multisatker_sql: str, workflow_sql: str) -> list[str]:
    problems: list[str] = []

    aset = parse_insert(strip_comments(multisatker_sql), "integrasi.siman_aset")

    # (1) one code, one name.
    by_code: dict[str, set[str]] = {}
    by_nup: dict[str, str] = {}
    for row in aset:
        code = norm_kode(row.get("kd_brg"))
        name = row.get("ur_sskel")
        if code:
            by_code.setdefault(code, set()).add(name or "")
            by_nup[row.get("no_aset") or ""] = code
    for code, names in sorted(by_code.items()):
        if len(names) > 1:
            problems.append(
                f"kd_brg {code} carries {len(names)} names in the fixture "
                f"({', '.join(sorted(repr(n) for n in names))}); in SIMAN a code "
                f"has exactly one."
            )

    # (3) every seeded pair matches the codification SIMAN actually publishes.
    for row in aset:
        code = norm_kode(row.get("kd_brg"))
        name = row.get("ur_sskel")
        pinned = SIMAN_CODIFICATION.get(code)
        if pinned is None:
            problems.append(
                f"kd_brg {code!r} (NUP {row.get('no_aset')!r}) is not in "
                f"SIMAN_CODIFICATION. Look the code up in integrasi.siman_aset and "
                f"pin its real name here — do not invent one."
            )
        elif name != pinned[0]:
            problems.append(
                f"kd_brg {code} is {pinned[0]!r} in SIMAN ({pinned[1]} real assets), "
                f"but the fixture calls it {name!r}."
            )

    # (2) every permit names a seeded asset.
    permits = parse_insert(strip_comments(workflow_sql), "perlengkapan.izin_pemakaian_bmn")
    for row in permits:
        nup = row.get("bmn_nup") or ""
        kode = norm_kode(row.get("bmn_kode_barang"))
        seeded = by_nup.get(nup)
        if seeded is None:
            problems.append(
                f"izin {row.get('nomor_izin')!r} names NUP {nup!r}, which no seeded "
                f"siman_aset row carries — the permit is for an asset that does not exist."
            )
        elif seeded != kode:
            problems.append(
                f"izin {row.get('nomor_izin')!r} claims kode barang {kode!r} for NUP "
                f"{nup!r}, but the seeded asset carries {seeded!r} — the two seed files "
                f"disagree about which asset this permit is for."
            )
    return problems


def main() -> int:
    problems = check(MULTISATKER.read_text(), WORKFLOW.read_text())
    if problems:
        for p in problems:
            print(f"::error::{p}")
        return 1
    aset = parse_insert(strip_comments(MULTISATKER.read_text()), "integrasi.siman_aset")
    permits = parse_insert(
        strip_comments(WORKFLOW.read_text()), "perlengkapan.izin_pemakaian_bmn"
    )
    print(
        f"check-e2e-fixture-codification: {len(aset)} seeded assets, one name per "
        f"barang code; {len(permits)} permits all name a seeded asset "
        f"(penghapusan_bmn excluded by design); names pinned to the "
        f"{VERIFIED_AGAINST}"
    )
    return 0


# --- canaries: the guard must fire, and must NOT fire, on demand -------------
def _canaries() -> int:
    ms = MULTISATKER.read_text()
    wf = WORKFLOW.read_text()
    failures = 0

    def expect(label: str, sql_ms: str, sql_wf: str, should_fail: bool) -> None:
        nonlocal failures
        found = bool(check(sql_ms, sql_wf))
        ok = found == should_fail
        print(f"  {'ok  ' if ok else 'FAIL'} {label}")
        if not ok:
            failures += 1

    expect("the seeds as committed pass", ms, wf, should_fail=False)
    expect(
        "detects: the exact bug — a real code under another item's name",
        ms.replace("'E2E-A-2', 'Note Book'", "'E2E-A-2', 'Personal Computer Unit'"),
        wf,
        should_fail=True,
    )
    expect(
        "detects: a barang code nobody looked up",
        ms.replace("'3020101003'", "'3050104001'"),
        wf.replace("'E2E-A-1', '3020101003'", "'E2E-A-1', '3050104001'"),
        should_fail=True,
    )
    expect(
        "detects: two names for one code (the invariant, once a code repeats)",
        ms.replace("'E2E-C-1', 'Printer (Peralatan Personal Komputer)'", "'E2E-C-1', 'Note Book Palsu'")
          .replace("'3100203003'", "'3100102003'"),
        wf.replace("'E2E-C-1', '3100203003'", "'E2E-C-1', '3100102003'"),
        should_fail=True,
    )
    expect(
        "detects: a permit whose kode barang drifts from its asset",
        ms,
        wf.replace(
            "'E2E-IZIN-002', 'LAPTOP', 'E2E-A-1', '3020101003'",
            "'E2E-IZIN-002', 'LAPTOP', 'E2E-A-1', '3050104001'",
        ),
        should_fail=True,
    )
    expect(
        "detects: a permit for a NUP no asset carries",
        ms,
        wf.replace("'E2E-IZIN-003', 'LAPTOP', 'E2E-B-1'", "'E2E-IZIN-003', 'LAPTOP', 'E2E-Z-9'"),
        should_fail=True,
    )
    # The dotted/undotted split must NOT be reported: both spellings are the
    # same code, and the system writes both.
    expect(
        "tolerates: the dotted presentation spelling of the same code",
        ms,
        wf.replace(
            "'E2E-IZIN-001', 'LAPTOP', 'E2E-A-2', '3100102003'",
            "'E2E-IZIN-001', 'LAPTOP', 'E2E-A-2', '3.10.01.02.003'",
        ),
        should_fail=False,
    )
    return failures


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(_canaries())
    sys.exit(main())
