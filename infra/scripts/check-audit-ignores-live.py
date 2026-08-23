#!/usr/bin/env python3
"""Menolak entri ignore di .cargo/audit.toml yang advisory-nya tak lagi menyala.

Sebuah ignore yang basi terbaca persis seperti ignore yang masih perlu, dan itu
bukan sekadar kerapian: ia adalah penekanan yang tetap berlaku diam-diam kalau
crate-nya suatu saat kembali. Ketika penjaga ini ditulis (2026-08-23), 9 dari 15
entri di .cargo/audit.toml sudah basi — rustls-webpki, lopdf, quick-xml dan
kawan-kawan sudah lama di-upgrade melewati versi rentannya, sebagian bahkan
memenuhi syarat pencabutan yang ditulis di komentarnya sendiri ("un-ignore when
printpdf bumps lopdf >= 0.42" — lopdf sudah 0.44).

cargo-deny punya `-D advisory-not-detected` untuk hal yang sama; cargo-audit
tidak punya padanannya, jadi bagian ini ditutup di sini.

CARA PAKAI (lihat security.yml): jalankan `cargo audit --json` dari direktori
yang TIDAK memuat .cargo/audit.toml — supaya daftar ignore-nya tak ikut
terpakai dan kita melihat himpunan advisory yang sebenarnya menyala — lalu
pipe JSON-nya ke sini bersama path berkas konfigurasi aslinya.

    tmp=$(mktemp -d); cp Cargo.lock "$tmp/"
    (cd "$tmp" && cargo audit --json -f Cargo.lock) \\
      | python3 infra/scripts/check-audit-ignores-live.py .cargo/audit.toml

CATATAN soal cara menurunkan himpunan itu: JANGAN meng-grep ID RUSTSEC dari
keluaran teks alat mana pun. Deskripsi sebuah advisory bisa MENYEBUT ID
advisory lain, dan grep akan menghitungnya sebagai "menyala". Persis itu yang
sempat terjadi saat penjaga ini dibuat: RUSTSEC-2024-0370 terbaca menyala
padahal `proc-macro-error` sudah lama hilang dari pohon. Karena itu di sini
sumbernya adalah JSON terstruktur, bukan teks.
"""

import json
import re
import sys


def configured_ignores(path):
    """Ambil ID advisory dari daftar `ignore` di berkas audit.toml.

    Sengaja regex, bukan parser TOML: berkas ini memuat komentar panjang yang
    ingin kita pertahankan apa adanya, dan kita hanya butuh ID-nya.
    """
    text = open(path, encoding="utf-8").read()
    m = re.search(r"ignore\s*=\s*\[(.*?)\n\]", text, re.S)
    if not m:
        return None
    return set(re.findall(r'"(RUSTSEC-\d{4}-\d+)"', m.group(1)))


def fired_advisories(report):
    """Himpunan ID advisory yang benar-benar dilaporkan cargo-audit."""
    out = set()
    for v in (report.get("vulnerabilities") or {}).get("list") or []:
        adv = (v or {}).get("advisory") or {}
        if adv.get("id"):
            out.add(adv["id"])
    for entries in ((report.get("warnings") or {}).values()):
        for w in entries or []:
            adv = (w or {}).get("advisory") or {}
            if adv.get("id"):
                out.add(adv["id"])
    return out


def main():
    if len(sys.argv) != 2:
        print("::error::pakai: check-audit-ignores-live.py <path audit.toml> < laporan.json")
        return 2
    cfg_path = sys.argv[1]

    try:
        report = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        print(f"::error::gagal mem-parse laporan JSON cargo-audit: {exc}")
        return 1

    configured = configured_ignores(cfg_path)
    if configured is None:
        print(f"::error::tak menemukan daftar `ignore` di {cfg_path}.")
        return 1

    fired = fired_advisories(report)

    # Laporan yang tak memeriksa apa pun terbaca persis seperti laporan yang
    # lulus. Kalau nol advisory menyala, jauh lebih mungkin probe-nya yang
    # rusak (konfigurasi ikut terbaca, lockfile salah, skema JSON berubah)
    # daripada seluruh pohon dependensi mendadak bersih.
    if not fired:
        print("::error::nol advisory menyala — probe-nya yang rusak, bukan pohon "
              "dependensi yang bersih. Pastikan `cargo audit --json` dijalankan "
              "dari direktori TANPA .cargo/audit.toml.")
        return 1

    stale = sorted(configured - fired)
    if stale:
        for a in stale:
            print(f"::error file={cfg_path}::{a} di-ignore tapi tak lagi menyala — "
                  f"crate-nya sudah hilang atau di-upgrade melewati versi rentannya. "
                  f"Cabut entri ini; ignore basi tetap menekan advisory-nya kalau "
                  f"crate itu suatu saat kembali.")
        print(f"{len(stale)} ignore basi dari {len(configured)} entri "
              f"({len(fired)} advisory menyala).")
        return 1

    print(f"OK {cfg_path}: {len(configured)} ignore, semuanya masih menyala "
          f"(dari {len(fired)} advisory aktif).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
