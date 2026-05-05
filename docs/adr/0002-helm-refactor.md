# ADR-0002: Refactor Total Helm Chart — Helper Library Pattern + Schema Validation

- **Status**: Accepted
- **Date**: 2026-05-05
- **Deciders**: SIMPEL DevOps
- **Technical context**: Production deployment perdana ke `simpel.kejaksaan.go.id`, post-migrasi Kustomize→Helm

## Konteks

Migrasi dari Kustomize ke Helm (commit `dc3d5770`, April 2026) menyalin manifest 1:1, menghasilkan ~1100 baris template Helm dengan duplikasi tinggi:

- 9 file workload template (`templates/{infrastructure,backend,frontend}/<svc>.yaml`) masing-masing 80-180 baris.
- Setiap file rewrite struktur yang sama: Service ClusterIP, Deployment/StatefulSet, podSecurityContext, containerSecurityContext, probes, volumes, dll.
- Beberapa best practice belum konsisten:
  - Probe tidak lengkap (mis. simpelv1 tidak punya liveness probe).
  - Image tag `stag` / `prod` (mutable) — risiko deploy tidak reproducible.
  - PDB & topologySpread tidak konsisten antar service.
  - Resource request kadang terlalu tinggi (request 100% dari limit → scheduling stuck).
  - Tidak ada validation schema → typo di values.yaml lolos `helm install`, gagal saat apply ke cluster.

## Pertimbangan

| Opsi | Pros | Cons |
|------|------|------|
| **A. Status quo (1100 baris duplikasi)** | Tidak ada effort | Maintenance pain, drift antar service, security gap |
| **B. Pisah ke sub-chart per service** | Clean separation | Boilerplate Chart.yaml × 9, dependency management berlebih |
| **C. Library chart (`type: library`)** | Reusable helpers, satu chart umbrella | Setup awal lebih kompleks |
| **D. Helper templates di chart yang sama** ★ | Helm idiomatic, mudah debug, single chart deploy | Helper harus disiplin |

## Keputusan

Pakai **Opsi D**: helper templates di `templates/_<area>.tpl` yang dipanggil oleh thin wrappers di `templates/services/<svc>.yaml`.

### Helper Library

| File | Fungsi |
|------|--------|
| `_security.tpl` | `simpel.podSecurityContext`, `simpel.containerSecurityContext` (baseline: drop ALL caps, RO root, runAsNonRoot, seccomp RuntimeDefault) |
| `_probes.tpl` | `simpel.probe` (http/grpc/tcp/exec) + `simpel.probes` (startup+liveness+readiness composite) |
| `_topology.tpl` | `simpel.topologySpreadConstraints` + `simpel.podAntiAffinity` (preferred/required) |
| `_workload.tpl` | `simpel.workload` — render Service + Deployment/StatefulSet + Secreton SA token volume |
| `_cronjob.tpl` | `simpel.cronjob` — render CronJob dengan SA token (untuk integrasi per provider) |
| `_helpers.tpl` (existing) | Naming, labels, image refs, service URLs |

### Schema Validation

`values.schema.json` memvalidasi:
- `global.imageTag` regex `^v\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$` (semver immutable)
- `global.environment` enum [`staging`, `production`]
- `mtls.mode` enum [DISABLE, PERMISSIVE, STRICT, UNSET]
- Per-workload: replicas ≥ 0, image, resources, probes, secretonAuth.

`helm lint` reject input invalid sebelum apply ke cluster.

### Refactored Workload Pattern

Setiap service jadi 5-baris wrapper:

```yaml
{{- include "simpel.workload" (dict "ctx" . "name" "authenc" "values" .Values.authenc) }}
```

Konfigurasi inline di `values.yaml` (`service.ports`, `env`, `probes`, `volumes`, dll) — lebih mudah review & override per env.

## Konsekuensi

### Positif
- **95% kompresi**: 94 baris → 5 baris per service template.
- **Consistency**: security context, probes, strategy RollingUpdate seragam.
- **Validation**: typo / mutable tag direject sebelum cluster impact.
- **Probe coverage**: simpelv1 sekarang punya liveness probe; service lain dapat startup probe (cold start protection).
- **DRY**: 1 fix di helper otomatis apply ke 9 service.
- **Schema dokumentasi**: developer paham field yang valid tanpa baca kode.

### Negatif / Risiko
- Learning curve helper Helm (`include`, `dict`, `nindent`) untuk kontributor.
- Helper bug = 9 service kena (vs 1 dengan refactor manual).
- Beberapa service kompleks (postgres dengan 3 Service, redis dengan 2 Service alias + PVC, simpelv1 dengan init container) **tidak** bisa pakai helper umum — tetap di template existing.

### Mitigasi
- Helper ditest via `helm template --debug` di kedua env (staging & production) sebelum merge.
- Schema strict prevent typo.
- Service kompleks (postgres/redis/simpelv1) dipertahankan template-nya hingga refactor follow-up dengan helper variant.

## Service yang Belum Dimigrasi (PR Follow-up)

- `templates/infrastructure/postgres.yaml` — multi-Service (headless, primary, conditional replicas), Patroni-aware.
- `templates/infrastructure/redis.yaml` — 2 Service alias (`redis` + `redis-service`) untuk Laravel compat, PVC standalone.
- `templates/backend/simpelv1.yaml` — sudah refactor sebagian (init container fetch-secrets di-add untuk Secreton zero-trust); restrukturisasi penuh defer ke PR berikutnya karena complexity initContainer db-migrate + ConfigMap inline simpelv1-env.

## Referensi

- `infra/helm/simpel/templates/_*.tpl` (helper library)
- `infra/helm/simpel/values.schema.json` (validation schema)
- `infra/helm/simpel/templates/services/*.yaml` (refactored wrappers)
- Helm best practices: https://helm.sh/docs/chart_best_practices/
- Helm template language: https://helm.sh/docs/chart_template_guide/
