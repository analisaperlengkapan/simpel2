#!/usr/bin/env python3
"""Deteksi dokumen Kubernetes yang DITELAN oleh pemisah `---` yang hilang.

Kalau sebuah aksi Go-template ditutup dengan trailing dash (`-}}`), newline
SESUDAHNYA ikut terpangkas. Pemisah `---` di baris berikutnya lalu menempel ke
baris sebelumnya:

    app.kubernetes.io/name: postgres---
    apiVersion: apps/v1
    kind: StatefulSet

YAML membaca itu sebagai SATU dokumen dengan kunci ganda — `kind`, `metadata`,
dan `spec` yang belakangan menang — sehingga objek yang pertama LENYAP dari
manifest. Tidak ada error di mana pun: `helm template` sukses, `helm lint`
hijau, kube-linter hijau, karena hasilnya tetap YAML yang sah. Yang terjadi di
cluster: `helm upgrade` menghapus objek yang hilang itu.

Terukur 2026-08-25 pada `templates/infrastructure/postgres.yaml`: Service
`postgres-primary` raib dari render, dan diff terhadap rilis live menampilkannya
sebagai penghapusan — padahal DestinationRule `postgres-primary-mtls` masih
menunjuk host itu.

Guard ini TIDAK mencocokkan pola di template (pola bisa berpindah bentuk).
Ia menurunkan jawabannya dari hasil render: jumlah baris `kind:` di kolom 0
harus sama dengan jumlah dokumen yang benar-benar ter-parse.

Pakai: check-chart-document-separators.py <label> <file.yaml> [file2.yaml ...]
"""
import sys

try:
    import yaml
except ImportError:
    sys.exit("PyYAML tidak tersedia")


def main() -> int:
    if len(sys.argv) < 3:
        return int(bool(sys.stderr.write(f"usage: {sys.argv[0]} <label> <file...>\n")))
    label, paths = sys.argv[1], sys.argv[2:]
    failed = False
    for path in paths:
        text = open(path, encoding="utf-8", errors="replace").read()
        # `kind:` di kolom 0 = kind tingkat-dokumen. Kind yang bersarang
        # (mis. di dalam `spec.template`) selalu terindentasi.
        declared = sum(1 for ln in text.split("\n") if ln.startswith("kind:"))
        try:
            parsed = sum(1 for d in yaml.safe_load_all(text) if d)
        except yaml.YAMLError as exc:
            print(f"FAIL [{label}] {path}: YAML tak bisa di-parse: {exc}")
            failed = True
            continue
        if declared != parsed:
            print(
                f"FAIL [{label}] {path}: {declared} baris `kind:` tapi hanya "
                f"{parsed} dokumen ter-parse — {declared - parsed} dokumen "
                f"DITELAN oleh pemisah `---` yang menempel ke baris sebelumnya. "
                f"Cari aksi template yang ditutup `-}}}}` tepat sebelum `---`."
            )
            failed = True
        else:
            print(f"ok [{label}] {path}: {parsed} dokumen, tak ada yang ditelan")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
