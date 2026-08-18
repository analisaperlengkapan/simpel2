#!/usr/bin/env python3
"""Menolak referensi image yang tak bisa dipulihkan, atas manifest TER-RENDER.

Dijalankan oleh job `helm-lint` di .github/workflows/security.yml terhadap
keluaran `helm template` — jadi cakupannya TURUT TUMBUH dengan chart: workload
baru ikut diperiksa tanpa ada yang perlu menambahkannya ke daftar mana pun.

Dua aturan, keduanya diturunkan dari satu insiden nyata.

`imagePullPolicy: Never` + tag yang bisa bergerak adalah kombinasi yang membuat
sebuah pod TAK BISA DIPULIHKAN, bukan sekadar tidak rapi:

  - `Never` berarti kubelet tak pernah menarik; pod hanya bisa jalan kalau
    image-nya kebetulan masih ada di disk node itu.
  - Tag yang bisa bergerak (`:prod`, `:latest`) berarti tak ada catatan isi
    mana yang seharusnya ada di sana.

Sekali image itu hilang dari disk node — garbage collect kubelet, node
di-drain, disk dibersihkan — pod itu tak akan pernah bisa dijadwalkan lagi,
DAN tak ada cara mengetahui apa yang harus dibangun ulang. Ini bukan hipotesis:
namespace `simpelv2-production` hari ini berjalan persis begitu
(`localhost:32000/...:prod` + `Never`), delapan dari sepuluh workload-nya nol
replica, dan root `simpel.kejaksaan.go.id` sudah 503 selama ~30 hari.

kube-linter TIDAK menangkap keduanya — diuji dengan kenari 2026-08-18: sebuah
Deployment ber-`Never` dan bertag `:prod` lolos bersih dari check set default.
Karena itu guard ini ada, bukan karena kurang percaya pada kube-linter.

Baca daftar objek K8s (array JSON, hasil `yq -o=json eval-all '[.]'`) dari
stdin. yq HANYA mengubah YAML→JSON; seluruh logika ada di sini, di pustaka
standar — pelajaran dari guard emptyDir yang versi pertamanya menaruh logika di
ekspresi yq dan LOLOS pada input yang sengaja dirusak.
"""

import json
import re
import sys

# Nama tag yang isinya boleh berubah di bawah kaki kita. Ini bukan daftar
# vendor atau daftar image — ini daftar KATA, jadi ia berlaku untuk image apa
# pun yang muncul di chart, termasuk yang belum ada hari ini.
MOVING_TAGS = {
    "latest",
    "prod",
    "production",
    "stag",
    "staging",
    "stable",
    "main",
    "master",
    "edge",
    "dev",
    "devel",
    "development",
    "test",
    "nightly",
    "snapshot",
}

# Field yang memuat daftar container di sebuah PodSpec.
CONTAINER_FIELDS = ("initContainers", "containers", "ephemeralContainers")


def pod_specs(obj):
    """Hasilkan (jalur, podSpec) untuk tiap PodSpec di dalam sebuah objek.

    Menyisir bentuk-bentuk yang benar-benar dipakai chart ini: Pod telanjang,
    workload ber-`.spec.template`, dan CronJob yang menyarangkannya dua tingkat.
    """
    kind = obj.get("kind", "?")
    if kind == "Pod":
        yield "spec", obj.get("spec") or {}
        return
    if kind == "CronJob":
        spec = (
            ((obj.get("spec") or {}).get("jobTemplate") or {}).get("spec") or {}
        ).get("template") or {}
        yield "spec.jobTemplate.spec.template.spec", spec.get("spec") or {}
        return
    tmpl = (obj.get("spec") or {}).get("template")
    if isinstance(tmpl, dict):
        yield "spec.template.spec", tmpl.get("spec") or {}


def split_ref(image):
    """Pisahkan referensi image jadi (repository, tag, digest).

    Titik dua di dalam bagian host (`localhost:32000/x`) BUKAN pemisah tag —
    itu port. Karena itu pemisahan dilakukan setelah garis miring terakhir.
    """
    if "@" in image:
        repo, _, digest = image.partition("@")
        return repo, None, digest
    head, _, last = image.rpartition("/")
    if ":" in last:
        name, _, tag = last.partition(":")
        repo = f"{head}/{name}" if head else name
        return repo, tag, None
    return image, None, None


def main():
    who = sys.argv[1] if len(sys.argv) > 1 else "manifest"
    try:
        docs = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        print(f"::error::[{who}] gagal mem-parse JSON manifest: {exc}")
        return 1

    problems = []
    checked = 0

    for obj in docs:
        if not isinstance(obj, dict):
            continue
        name = (obj.get("metadata") or {}).get("name", "?")
        kind = obj.get("kind", "?")
        for path, spec in pod_specs(obj):
            for field in CONTAINER_FIELDS:
                for container in spec.get(field) or []:
                    checked += 1
                    where = f"{kind}/{name} {path}.{field}[{container.get('name', '?')}]"

                    policy = container.get("imagePullPolicy")
                    if policy == "Never":
                        problems.append(
                            f"{where}: imagePullPolicy: Never — pod ini mati permanen "
                            f"begitu image-nya hilang dari disk node."
                        )

                    image = container.get("image")
                    if not image:
                        problems.append(f"{where}: tak punya field image.")
                        continue

                    repo, tag, digest = split_ref(image)
                    if digest:
                        continue
                    if tag is None:
                        problems.append(
                            f"{where}: image {image!r} tanpa tag — ini berarti "
                            f"`:latest` implisit."
                        )
                    elif tag.lower() in MOVING_TAGS:
                        problems.append(
                            f"{where}: image {image!r} memakai tag yang bisa bergerak "
                            f"({tag!r}) — tak ada catatan isi mana yang seharusnya jalan. "
                            f"Pakai tag semver immutable atau digest."
                        )

    # Laporan yang tak pernah memeriksa apa pun terbaca persis seperti laporan
    # yang lulus. Kalau selektor di atas tak lagi cocok dengan bentuk chart,
    # itu harus terlihat sebagai merah, bukan sebagai hijau.
    if checked == 0:
        print(f"::error::[{who}] nol container diperiksa — selektor guard ini tak lagi cocok.")
        return 1

    if problems:
        for p in problems:
            print(f"::error::[{who}] {p}")
        print(f"[{who}] {len(problems)} masalah pada {checked} container.")
        return 1

    print(f"[{who}] OK — {checked} container, semua bertag immutable & tak ada pullPolicy Never.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
