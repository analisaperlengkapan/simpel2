#!/usr/bin/env python3
"""Setiap emptyDir wajib dikecualikan dari File System Backup Velero.

Velero berjalan `defaultVolumesToFsBackup: true` (opt-out) — pilihan yang benar,
karena volume baru otomatis ter-backup tanpa perlu diingat siapa pun. Harganya:
volume yang TAK PERNAH berisi data ikut tersalin juga.

Terukur pada latihan restore 2026-08-18: dari 74 volume, 19 di antaranya scratch
(sidecar Istio, cache nginx, direktori soket). Semuanya gagal disalin saat
klaster sibuk, dan restore dilaporkan `PartiallyFailed` — padahal SELURUH volume
data berhasil; checksum dbsimpelv2 hasil restore identik dengan sumbernya.
Sinyal merah yang tidak menandakan kehilangan data adalah sinyal yang akan
diabaikan orang, dan itu jauh lebih mahal daripada 19 volume.

Chart menurunkan sebagian daftar pengecualian (emptyDir dari values, lewat
`_workload.tpl`) dan menulis sebagian literal (workload yang merender pod-spec
sendiri: postgres/redis/simpelv1). Skrip ini yang menjaga bagian tulis-tangan
itu tidak tertinggal saat emptyDir baru ditambahkan.

Masukan: JSON array berisi seluruh objek dari `helm template`, mis.
    yq -o=json eval-all '[.]' rendered/staging.yaml | check-velero-volume-excludes.py staging

Stdlib saja — image runner ARC tidak punya PyYAML (karena itu konversi
YAML→JSON dilakukan yq, dan logikanya di sini).
"""

import json
import sys

ANNOTATION = "backup.velero.io/backup-volumes-excludes"
KINDS = {"Deployment", "StatefulSet", "DaemonSet"}


def main() -> int:
    label = sys.argv[1] if len(sys.argv) > 1 else "<stdin>"
    docs = json.load(sys.stdin)

    problems = []
    checked = 0
    for doc in docs:
        if not isinstance(doc, dict) or doc.get("kind") not in KINDS:
            continue
        checked += 1
        name = doc.get("metadata", {}).get("name", "<tanpa-nama>")
        tpl = doc.get("spec", {}).get("template", {})
        ann = (tpl.get("metadata") or {}).get("annotations") or {}
        excluded = {v for v in (ann.get(ANNOTATION) or "").split(",") if v}
        empty_dirs = {
            v["name"] for v in (tpl.get("spec", {}).get("volumes") or []) if "emptyDir" in v
        }
        for missing in sorted(empty_dirs - excluded):
            problems.append(f"{doc['kind']}/{name}: {missing}")

    if checked == 0:
        print(f"::error::{label}: tidak ada workload yang diperiksa — guard ini pasti salah jalan.")
        return 1

    if problems:
        print(f"::error::{label}: emptyDir tanpa pengecualian backup Velero:")
        for p in problems:
            print(f"  {p}")
        print("")
        print(f"Tambahkan namanya ke anotasi `{ANNOTATION}` pada pod template")
        print("workload tersebut (lihat infra/helm/simpel/templates/_workload.tpl).")
        return 1

    print(f"OK {label}: {checked} workload diperiksa, semua emptyDir dikecualikan.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
