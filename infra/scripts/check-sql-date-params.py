#!/usr/bin/env python3
"""Cari parameter SQL tanpa cast di dalam aritmatika tanggal/waktu.

Postgres menyelesaikan tipe operator dari operan-nya. Placeholder telanjang
(`$1`) bertipe `unknown`, jadi `CURRENT_DATE + $1` punya banyak kandidat
(`date + integer`, `date + interval`, `date + time`, ...) dan Postgres menolak
memilih. Yang membuat kelas bug ini lolos ke produksi: **kedua arah gagalnya
tidak sama**, dan yang satu tidak terlihat sampai runtime.

    SELECT CURRENT_DATE + $1     -- ERROR saat PREPARE: operator is not unique
    SELECT CURRENT_DATE - $1     -- PREPARE SUKSES; Postgres menebak $1 = date
                                 -- (date - date -> integer), lalu bind gagal:
                                 -- "parameter $1 of type integer cannot be
                                 --  coerced to the expected type date"

Bentuk minus itulah alasan pemeriksaan "apakah SQL-nya parse" tidak cukup —
ia lulus PREPARE dan baru meledak saat query benar-benar dijalankan dengan
argumen. Dua-duanya ditemukan hidup di repo ini (2026-08-24): jalur notifikasi
izin pemakaian BMN yang kedaluwarsa, dan statistik API per-modul di integrasi.

Perbaikannya selalu sama: beri cast eksplisit pada placeholder-nya, `$1::int`.

Keluar 1 kalau menemukan pelanggaran. Hanya pustaka standar.
"""
from __future__ import annotations

import pathlib
import re
import sys

# Ekspresi yang jelas bertipe tanggal/waktu di sisi kiri operator.
_TEMPORAL = r"(?:CURRENT_DATE|CURRENT_TIMESTAMP|LOCALTIMESTAMP|NOW\(\))"
# `$N` yang TIDAK diikuti `::` (cast) — spasi sebelum `::` tetap dihitung aman.
_BARE_PARAM = r"\$\d+(?!\s*::)"

PATTERN = re.compile(rf"{_TEMPORAL}\s*[-+]\s*{_BARE_PARAM}", re.IGNORECASE)

SKIP_DIRS = {
    ".git", "target", "node_modules", "dist", "vendor",
    "__pycache__", ".venv", "test-results", "playwright-report",
}
SUFFIXES = {".rs", ".sql"}


def scan(root: pathlib.Path) -> list[tuple[pathlib.Path, int, str]]:
    hits: list[tuple[pathlib.Path, int, str]] = []
    for path in root.rglob("*"):
        if path.suffix not in SUFFIXES:
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if "$" not in text:
            continue
        for lineno, line in enumerate(text.splitlines(), start=1):
            if PATTERN.search(line):
                hits.append((path.relative_to(root), lineno, line.strip()))
    return hits


def main() -> int:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    hits = scan(root)
    if not hits:
        print("OK: tak ada placeholder tanpa cast di aritmatika tanggal/waktu.")
        return 0
    print(
        f"GAGAL: {len(hits)} placeholder tanpa cast di aritmatika tanggal/waktu.\n"
        "Postgres tak bisa memilih operator untuk `date + unknown`; bentuk minus\n"
        "malah lolos PREPARE lalu gagal saat bind. Beri cast eksplisit, mis. `$1::int`.\n",
        file=sys.stderr,
    )
    for path, lineno, line in hits:
        print(f"  {path}:{lineno}: {line}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
