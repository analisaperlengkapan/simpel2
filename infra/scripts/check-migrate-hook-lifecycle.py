#!/usr/bin/env python3
"""Hook Job yang menyentuh database WAJIB `post-install,pre-upgrade`.

Kenapa dua fase berbeda, dan kenapa ini pernah salah:

* **Install pertama harus `post-install`.** StatefulSet postgres + Secret
  kredensialnya adalah resource biasa: Helm membuatnya SETELAH fase
  `pre-install` dan SEBELUM `post-install`. Hook `pre-install` karena itu
  menunggu database yang belum pernah dibuat — deadlock.
* **Upgrade harus `pre-upgrade`.** Skema harus mendarat SEBELUM pod baru
  rollout.

Chart ini sempat memakai `post-upgrade` untuk ketiga Job, dengan alasan
tertulis "safe under the expand/contract migration discipline". Alasan itu
membalik arah jaminannya. Expand/contract membuat kode LAMA tahan terhadap
skema BARU — itulah yang membuat `helm rollback` tak perlu restore DB
(`infra/AGENTS.md` → "Migrasi WAJIB expand/contract"). Ia tidak menjanjikan
apa pun tentang kode BARU bertemu skema LAMA, dan `post-upgrade` menghasilkan
persis itu: pod ber-image baru melayani trafik selama jendela sebelum Job
migrasi jalan.

Terukur 2026-08-26: rc28 menjalankan seluruh lapisan scoping RBAC lewat
`integrasi.v_satker_wilayah` (satker_scope, kebutuhan_bmn, pakaian_dinas,
pemakaian_bmn, bank_aset), sementara view itu BELUM ADA di staging — ia dibuat
migrasi integrasi 005. Dengan `post-upgrade`, setiap pembacaan ber-scope gagal
sepanjang jendela rollout. `infra/AGENTS.md:474` bahkan sudah menulis
"(pre-upgrade)"; chart-nya yang tak ikut.

CAKUPAN DITURUNKAN, BUKAN DAFTAR TULIS TANGAN. Guard ini tidak menghafal tiga
nama berkas — ia memindai SEMUA template Job ber-anotasi `helm.sh/hook`, lalu
menandai yang menyentuh database (menyebut host `postgres`, `pg_isready`, atau
`DATABASE_URL`). Job hook baru yang bicara ke DB otomatis ikut terjaga; itu
pola yang sama dengan penjaga lain di direktori ini, dan alasannya sama —
daftar tulis-tangan tidak ikut tumbuh bersama chart.

Sengaja TIDAK memakai PyYAML: image runner ARC tak menyediakannya dan itu
sudah menjatuhkan job lain sebelumnya.

Pakai: python3 infra/scripts/check-migrate-hook-lifecycle.py
"""
import pathlib
import re
import sys

WANT = "post-install,pre-upgrade"
TEMPLATES = pathlib.Path("infra/helm/simpel/templates")
DB_SIGNALS = ("pg_isready", "DATABASE_URL", "host: postgres", "-h postgres")


def hook_value(text: str) -> str | None:
    m = re.search(r'^\s*"helm\.sh/hook":\s*(.+?)\s*$', text, re.M)
    return m.group(1).strip().strip('"') if m else None


def touches_db(text: str) -> bool:
    # Buang komentar Go-template dan komentar YAML supaya prosa penjelasan
    # (yang menyebut "postgres") tidak dihitung sebagai sinyal.
    body = re.sub(r"\{\{-?\s*/\*.*?\*/\s*-?\}\}", "", text, flags=re.S)
    body = "\n".join(l for l in body.split("\n") if not l.lstrip().startswith("#"))
    return any(s in body for s in DB_SIGNALS)


def main() -> int:
    if not TEMPLATES.is_dir():
        sys.stderr.write(f"tidak menemukan {TEMPLATES} — jalankan dari root repo\n")
        return 1

    checked, bad = [], []
    for path in sorted(TEMPLATES.rglob("*.yaml")):
        text = path.read_text()
        if "kind: Job" not in text:
            continue
        hook = hook_value(text)
        if hook is None or not touches_db(text):
            continue
        rel = path
        checked.append(rel)
        if hook != WANT:
            bad.append((rel, hook))

    if not checked:
        sys.stderr.write(
            "GAGAL: tidak ada satu pun hook Job penyentuh-DB yang terdeteksi.\n"
            "Itu bukan 'bersih' — itu penjaga yang buta. Sinyal deteksinya\n"
            f"({', '.join(DB_SIGNALS)}) kemungkinan sudah tak cocok lagi.\n"
        )
        return 1

    for rel, hook in bad:
        sys.stderr.write(
            f"{rel}: helm.sh/hook = {hook!r}, harus {WANT!r}\n"
            f"  `post-upgrade` menjalankan migrasi SESUDAH pod baru rollout;\n"
            f"  skema harus mendarat lebih dulu. Lihat docstring berkas ini.\n"
        )
    if bad:
        return 1

    print(f"OK: {len(checked)} hook Job penyentuh-DB, semuanya {WANT}")
    for rel in checked:
        print(f"  {rel}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
