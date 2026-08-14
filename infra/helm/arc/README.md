# ARC — Ephemeral Autoscaling Runners (Actions Runner Controller)

Migrasi runner CI dari **self-hosted statis** (host `simple02`, yang ikut leak
buildx di `/var`) ke **runner ephemeral autoscaling** di microk8s. Tiap job CI
mendapat **pod baru**; workspace, dockerd (dind), dan buildx hidup di dalam pod
dan **dibuang saat job selesai** → akar masalah leak hilang secara struktural.

## Isi folder

| File | Fungsi |
|------|--------|
| `values-controller.yaml` | Values untuk chart `gha-runner-scale-set-controller`. |
| `values-runner-set.yaml`  | Values untuk satu `gha-runner-scale-set` (runner repo simpel2). |
| `install.sh` | Pemasangan idempotent (`helm upgrade --install`) controller + runner set. |

Chart-nya **upstream OCI** (`ghcr.io/actions/actions-runner-controller-charts/*`),
tidak divendor — kita hanya menyimpan values + skrip agar reproducible & ter-version.

## Prasyarat

1. `kubectl` & `helm` menjangkau cluster: `kubectl get nodes` harus sukses.
2. **PAT GitHub** (akun pemilik repo / student personal — Anda admin repo):
   - Classic PAT: scope `repo`, atau
   - Fine-grained PAT (repo `analisaperlengkapan/simpel2`): **Administration: RW**,
     **Actions: RW**, **Metadata: R**.
   - PAT **tidak** disimpan di repo — hanya jadi Secret `arc-github-token` di cluster.
   - Boleh PAT yang sama dengan `AUTOFIX_PAT`? Sebaiknya PAT terpisah (scope ARC
     berbeda). `AUTOFIX_PAT` cukup Contents/PR/Workflows; ARC butuh Administration.

## Pasang

```bash
GITHUB_PAT=<pat-untuk-arc> ./infra/helm/arc/install.sh
```

Verifikasi:

```bash
kubectl get pods -n arc-systems                       # controller Running
kubectl get autoscalingrunnerset -n arc-runners       # set 'arc-simpel' terdaftar
# GitHub → Settings → Actions → Runners → ada runner scale set 'arc-simpel'
```

Uji: jalankan workflow dengan `runs-on: arc-simpel` → pod `*-runner-*` muncul di
`arc-runners` saat job jalan, lalu hilang setelah selesai.

## Cutover `runs-on` (setelah ARC tervalidasi)

Nama scale set = label `runs-on`. Sengaja **bukan** `self-hosted` (nama reserved &
ambigu dengan runner lama). Setelah ARC terbukti hijau di PR uji:

1. Bulk-ganti `runs-on: self-hosted` → `runs-on: arc-simpel` di `.github/workflows/*`.
2. Merge → pastikan CI berjalan di pod ARC.
3. **Dekomisi runner lama** di `simple02`:

   ```bash
   sudo systemctl stop  actions.runner.* 2>/dev/null || true
   sudo systemctl disable actions.runner.* 2>/dev/null || true
   # lalu hapus registrasi runner di GitHub → Settings → Actions → Runners
   ```

4. Setelah runner lama mati, reaper buildx host-level di `ci.yml`/`maintenance.yml`/
   `release.yml` boleh disederhanakan (sudah tak relevan).

## Keamanan dind privileged (PENTING)

Mode `containerMode: dind` menjalankan sidecar dockerd **privileged** = praktis root
di kernel **node**. Risiko: bila build pod dikompromikan (PR jahat, dependency
beracun), bisa **escape ke node** dan menyentuh workload lain (termasuk produksi).

Mitigasi diterapkan / disarankan:
- **Batasi PR fork**: `autofix.yml` & build sudah `if head.repo == repository`.
- **Isolasi node** (disarankan): taint node build agar runner terpisah dari prod.

  ```bash
  kubectl taint nodes simple02 dedicated=ci:NoSchedule
  ```

  `values-runner-set.yaml` sudah punya `nodeSelector` + `toleration` cocok. Pastikan
  workload prod **tidak** men-toleransi taint ini (default tidak). Bila cluster
  2-node tak bisa benar-benar memisahkan, terima risiko terukur + batasi fork.
- **Alternatif lebih aman** (effort lebih besar): rootless dind, `buildkitd` pod,
  atau `containerMode: kubernetes` (tanpa docker daemon — perlu ganti `buildx`→
  `buildkit`/`kaniko` di workflow). Evaluasi bila kebijakan keamanan menuntut.

PSA: namespace `arc-runners` dilabeli `pod-security.kubernetes.io/enforce=privileged`
oleh `install.sh` agar dind diizinkan (cluster ini tak memaksa PSA `restricted`).

## Cache build (opsional, disarankan)

Runner ephemeral kehilangan `~/.cargo` & `target/` tiap job → build Rust lambat.
Lihat catatan di akhir `values-runner-set.yaml`: buildx **registry cache**
(`--cache-to/from type=registry`) paling robust untuk image; `sccache` untuk objek
Rust; PVC **RWX** (longhorn) bila ingin cache cargo bersama. Aktifkan setelah ARC
stabil.

## CI tidak mengambil job (job menggantung `queued`)

Kegagalan paling berbahaya di sini **tidak terlihat merah**. Tanpa runner, job
tidak gagal — ia menggantung `queued`, lalu GitHub membatalkannya di jam ke-24
sehingga run terbaca `cancelled`, tak terbedakan dari pembatalan manusia.
`required_status_checks` tak bisa menangkapnya: check-nya tak pernah MULAI.
Penjaganya karena itu ada di luar sistem yang sakit —
`.github/workflows/ci-heartbeat.yml`, satu-satunya workflow yang berjalan di
runner GitHub-hosted.

Diagnosis berurutan, dari yang paling sering:

**1. Versi runner di-deprecate → seluruh scale set terkunci mati.**
Ini yang mematikan CI 4 hari pada 2026-08-10. Runner yang lewat batas keluar
dengan **exit code 7**; ARC merantainya jadi
`EphemeralRunner → EphemeralRunnerSet → AutoscalingRunnerSet` berfase `Outdated`,
dan fase terakhir itu **terkunci permanen**: controller membacanya di awal
`Reconcile`, merobohkan listener + ERS + runner scale set, lalu `return` sebelum
baris yang mengembalikannya ke `Running`.

```bash
kubectl get autoscalingrunnerset arc-simpel -n arc-runners -o jsonpath='{.status.phase}'
# "Outdated" ⇒ terkunci
```

Satu-satunya jalan keluar adalah **perubahan spec ARS** (hash berubah ⇒ controller
mereset fase ke `Pending`). Jadi bump versi runner sekaligus obat kaskadenya:

```bash
# 1) infra/helm/arc/runner-image/Dockerfile: FROM ...actions-runner:<versi-baru>
# 2) build + push tag image BARU (jangan timpa tag lama — immutable)
# 3) infra/helm/arc/values-runner-set.yaml: naikkan tag di KEDUA tempat
helm upgrade arc-simpel -n arc-runners <chart> -f infra/helm/arc/values-runner-set.yaml
```

**2. Listener menunjuk EphemeralRunnerSet yang sudah tiada.**
Pod listener crashloop ~5 detik sekali dengan `exit 1`; lognya berakhir di
`could not patch ephemeral runner set ... "arc-simpel-xxxxx" not found`.
Controller tak pernah memperbarui `spec.ephemeralRunnerSetName`, jadi ini tak
pulih sendiri. Bandingkan lalu hapus CR listener-nya (controller membuat ulang
menunjuk ERS yang benar):

```bash
kubectl get autoscalinglistener -n arc-systems -o jsonpath='{.items[0].spec.ephemeralRunnerSetName}'
kubectl get ephemeralrunnerset -n arc-runners -o jsonpath='{.items[0].metadata.name}'
# beda ⇒
kubectl delete autoscalinglistener -n arc-systems <nama>
```

**3. Runner macet ASSIGNED / JIT secret yatim.** Lihat `reaper.yaml`.

Catatan urutan: perbaiki (1) lebih dulu. Selama fase masih `Outdated`, melepas
listener di (2) hanya membeli ~45 detik — runner naik, exit 7 lagi, terkunci lagi.
