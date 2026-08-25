#!/usr/bin/env python3
"""Deteksi dokumen Kubernetes yang DITELAN oleh pemisah `---` yang hilang.

Kalau sebuah aksi Go-template ditutup dengan trailing dash, newline SESUDAHNYA
ikut terpangkas. Pemisah `---` di baris berikutnya lalu menempel ke baris
sebelumnya:

    app.kubernetes.io/name: postgres---
    apiVersion: apps/v1
    kind: StatefulSet

YAML membaca itu sebagai SATU dokumen dengan kunci ganda — `kind`, `metadata`,
dan `spec` yang belakangan menang — sehingga objek yang pertama LENYAP dari
manifest. Tidak ada error di mana pun: `helm template` sukses, `helm lint`
hijau, kube-linter hijau, karena hasilnya tetap YAML yang sah. Yang terjadi di
cluster: `helm upgrade` MENGHAPUS objek yang hilang itu.

Terukur 2026-08-25 pada `templates/infrastructure/postgres.yaml`: Service
`postgres-primary` raib dari render, dan diff terhadap rilis live menampilkannya
sebagai penghapusan — padahal DestinationRule `postgres-primary-mtls` masih
menunjuk host itu.

Guard ini TIDAK mencocokkan pola di template (pola bisa berpindah bentuk). Ia
menurunkan jawabannya dari hasil render: jumlah baris `kind:` di kolom 0 harus
sama dengan jumlah dokumen yang benar-benar ter-parse.

Input: JSON array dokumen di stdin (dari `yq -o=json eval-all '[.]'`) + path
berkas YAML aslinya sebagai argumen, untuk menghitung baris `kind:`-nya.
Sengaja TIDAK memakai PyYAML: image runner ARC tak menyediakannya, dan itu
sudah menjatuhkan job ini sebelumnya.

Pakai: yq -o=json eval-all '[.]' < f.yaml | check-chart-document-separators.py f.yaml
"""
import json
import sys


def main() -> int:
    if len(sys.argv) != 2:
        sys.stderr.write(f"usage: {sys.argv[0]} <rendered.yaml>  (JSON on stdin)\n")
        return 1
    path = sys.argv[1]

    raw = sys.stdin.read()
    try:
        docs = json.loads(raw)
    except json.JSONDecodeError as exc:
        print(f"FAIL {path}: stdin bukan JSON yang sah ({exc}). "
              f"Guard ini mengharap keluaran `yq -o=json eval-all '[.]'`.")
        return 1
    if not isinstance(docs, list):
        print(f"FAIL {path}: stdin bukan array dokumen.")
        return 1

    parsed = sum(1 for d in docs if d)

    with open(path, encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    # `kind:` di kolom 0 = kind tingkat-dokumen. Kind yang bersarang
    # (mis. di dalam `spec.template`) selalu terindentasi.
    declared = sum(1 for ln in text.split("\n") if ln.startswith("kind:"))

    if declared != parsed:
        print(
            f"FAIL {path}: {declared} baris `kind:` tapi hanya {parsed} dokumen "
            f"ter-parse — {declared - parsed} dokumen DITELAN oleh pemisah `---` "
            f"yang menempel ke baris sebelumnya. Cari aksi template yang ditutup "
            f"dengan trailing dash tepat sebelum pemisah dokumen."
        )
        return 1

    print(f"ok {path}: {parsed} dokumen, tak ada yang ditelan")
    return 0


if __name__ == "__main__":
    sys.exit(main())
