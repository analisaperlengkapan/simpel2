#!/usr/bin/env python3
"""Job yang GAGAL harus meninggalkan pod-nya — pod itulah log-nya.

Klaster ini tidak punya pengumpul log. Satu-satunya tempat membaca MENGAPA sebuah Job gagal adalah
`kubectl logs` pada pod-nya, jadi apa pun yang menghapus pod gagal ikut menghapus jawabannya. Ada
tiga cara sebuah manifest melakukannya, dan ketiganya merender YAML yang sah (`helm lint`,
`helm template`, kube-linter hijau):

1. `restartPolicy: OnFailure`. Container diulang DI DALAM pod yang sama; begitu `backoffLimit`
   habis, Job controller menghapus pod itu. Ini perilaku terdokumentasi Kubernetes ("your Pod
   running the Job will be terminated once the job backoff limit has been reached"), bukan bug.
   Dengan `Never`, tiap percobaan adalah pod sendiri yang tetap ada dalam fase Failed.
2. CronJob dengan `failedJobsHistoryLimit: 0`: Job gagal dihapus saat itu juga, beserta pod-nya.
3. `ttlSecondsAfterFinished` yang pendek: Job selesai (sukses MAUPUN gagal) dihapus setelah TTL,
   beserta pod-nya. Untuk hook Helm ber-`hook-delete-policy: hook-succeeded`, TTL itu hanya pernah
   berlaku bagi hook yang GAGAL — persis yang ingin dibaca orang.

Kejadian nyata (rehearsal produksi rc47, 2026-11-11): tiga CronJob sinkronisasi dijalankan terhadap
URL `.invalid` dan gagal, seperti seharusnya. Sesudahnya `kubectl logs -l job-name=...` menjawab
"No resources found": pod-nya sudah dihapus (cara 1). `integrasi.api_log` 0 baris,
`integrasi.sync_status` tetap `pending`. Alasan kegagalannya hanya terbaca karena log diikuti
langsung saat Job berjalan — di produksi, pukul 02.00, tidak ada yang mengikuti.

Saat aturan ini diarahkan ke SEMUA Job (bukan hanya empat CronJob itu), ia menemukan cara 3 pada
kelima hook Job: `ttlSecondsAfterFinished: 600`, yaitu bukti kegagalan install/upgrade hilang
10 menit setelah gagal.

CAKUPAN DITURUNKAN: setiap objek `Job` dan `CronJob` di render yang diberikan, bukan daftar nama.

Sengaja TIDAK memakai PyYAML (image runner ARC tak menyediakannya); yq mengubah YAML -> JSON.

Pakai: yq -o=json eval-all '[.]' < rendered.yaml | check-failed-job-keeps-its-log.py <label> [--must-find]
  --must-find : gagal bila render ini tak berisi satu pun Job/CronJob (untuk render yang PASTI
                punya; tanpa itu penjaga yang buta akan melapor "bersih").
"""
import json
import sys

# Sehari: kegagalan tengah malam masih terbaca esok paginya.
MIN_TTL_SECONDS = 86400


def docs(blob):
    for d in blob:
        if isinstance(d, dict) and d.get("kind") in ("Job", "CronJob"):
            yield d


def job_spec(d):
    spec = d.get("spec") or {}
    if d["kind"] == "CronJob":
        return (spec.get("jobTemplate") or {}).get("spec") or {}
    return spec


def check(blob):
    """-> (pelanggaran, jumlah objek yang diperiksa)."""
    bad, n = [], 0
    for d in docs(blob):
        n += 1
        who = f"{d['kind']}/{(d.get('metadata') or {}).get('name')}"
        js = job_spec(d)
        pod = ((js.get("template") or {}).get("spec")) or {}

        rp = pod.get("restartPolicy")
        if rp != "Never":
            bad.append(
                f"{who}: restartPolicy = {rp!r}, harus 'Never' — dengan OnFailure pod (dan log-nya) "
                f"DIHAPUS saat backoffLimit habis"
            )

        if d["kind"] == "CronJob":
            # Default Kubernetes bila tak ditulis = 1.
            hist = (d.get("spec") or {}).get("failedJobsHistoryLimit", 1)
            if not isinstance(hist, int) or isinstance(hist, bool) or hist < 1:
                bad.append(
                    f"{who}: failedJobsHistoryLimit = {hist!r}, harus >= 1 — dengan 0 Job gagal "
                    f"dihapus saat itu juga beserta pod-nya"
                )

        if "ttlSecondsAfterFinished" in js:
            ttl = js["ttlSecondsAfterFinished"]
            if not isinstance(ttl, int) or isinstance(ttl, bool) or ttl < MIN_TTL_SECONDS:
                bad.append(
                    f"{who}: ttlSecondsAfterFinished = {ttl!r}, harus >= {MIN_TTL_SECONDS} (atau "
                    f"tidak ditulis) — Job GAGAL ikut dihapus setelah TTL, beserta pod dan log-nya"
                )
    return bad, n


def self_test():
    def pod(rp):
        return {"template": {"spec": ({"restartPolicy": rp} if rp is not None else {})}}

    def job(rp="Never", ttl=None):
        spec = pod(rp)
        if ttl is not None:
            spec["ttlSecondsAfterFinished"] = ttl
        return {"kind": "Job", "metadata": {"name": "j"}, "spec": spec}

    def cron(rp="Never", hist="absent", ttl=None):
        js = pod(rp)
        if ttl is not None:
            js["ttlSecondsAfterFinished"] = ttl
        spec = {"jobTemplate": {"spec": js}}
        if hist != "absent":
            spec["failedJobsHistoryLimit"] = hist
        return {"kind": "CronJob", "metadata": {"name": "c"}, "spec": spec}

    cases = [
        ("CronJob OnFailure (kasus nyata rc47)", [cron("OnFailure", 1)], 1, 1),
        ("CronJob Never + riwayat 1 lulus", [cron("Never", 1)], 0, 1),
        ("CronJob Never, riwayat tak ditulis (default 1) lulus", [cron("Never")], 0, 1),
        ("CronJob riwayat gagal 0", [cron("Never", 0)], 1, 1),
        ("CronJob TTL pendek di jobTemplate", [cron("Never", 1, ttl=600)], 1, 1),
        ("CronJob TTL seminggu lulus (reaper ARC)", [cron("Never", 10, ttl=604800)], 0, 1),
        ("Job hook TTL 600 (kasus nyata: lima hook)", [job("Never", 600)], 1, 1),
        ("Job TTL tepat sehari lulus", [job("Never", MIN_TTL_SECONDS)], 0, 1),
        ("Job TTL sedetik di bawah sehari", [job("Never", MIN_TTL_SECONDS - 1)], 1, 1),
        ("Job tanpa TTL lulus", [job("Never")], 0, 1),
        ("Job OnFailure", [job("OnFailure")], 1, 1),
        ("Job tanpa restartPolicy (jangan dianggap Never)", [job(None)], 1, 1),
        ("Job TTL 0 = hapus seketika", [job("Never", 0)], 1, 1),
        ("dua pelanggaran pada satu objek dihitung dua", [cron("OnFailure", 0)], 2, 1),
        ("Deployment restartPolicy Always bukan urusan penjaga ini",
         [{"kind": "Deployment", "metadata": {"name": "d"},
           "spec": {"template": {"spec": {"restartPolicy": "Always"}}}}], 0, 0),
        ("dokumen kosong / null diabaikan", [None, {}, job()], 0, 1),
    ]
    fail = 0
    for title, objs, want_bad, want_n in cases:
        bad, n = check(objs)
        if len(bad) != want_bad or n != want_n:
            print(f"  GAGAL: {title}: {len(bad)} pelanggaran / {n} objek, "
                  f"seharusnya {want_bad} / {want_n}", file=sys.stderr)
            fail += 1
    if fail:
        print("self-test check-failed-job-keeps-its-log GAGAL", file=sys.stderr)
        sys.exit(2)
    return len(cases)


def main():
    ncases = self_test()
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a for a in sys.argv[1:] if a.startswith("--")}
    unknown = flags - {"--must-find", "--self-test"}
    if unknown:
        print(f"argumen tak dikenal: {sorted(unknown)}", file=sys.stderr)
        return 2
    if "--self-test" in flags:
        print(f"self-test OK — {ncases} kasus")
        return 0
    label = args[0] if args else "render"
    blob = json.load(sys.stdin)
    if blob and isinstance(blob[0], list):
        blob = blob[0]
    bad, n = check(blob)
    if not n:
        if "--must-find" in flags:
            print(f"[{label}] GAGAL: tak ada satu pun Job/CronJob di render ini, padahal seharusnya "
                  f"ada. Itu bukan 'bersih' — itu penjaga yang buta.", file=sys.stderr)
            return 1
        print(f"[{label}] OK — tak ada Job/CronJob di render ini.")
        return 0
    if bad:
        print(f"[{label}] {len(bad)} pelanggaran — Job yang gagal tidak akan meninggalkan log:",
              file=sys.stderr)
        for b in bad:
            print(f"  - {b}", file=sys.stderr)
        return 1
    print(f"[{label}] OK — {n} Job/CronJob: semuanya restartPolicy Never, riwayat gagal >= 1, "
          f"TTL >= {MIN_TTL_SECONDS}s atau tak ada.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
