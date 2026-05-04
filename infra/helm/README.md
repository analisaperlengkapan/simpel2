# SIMPEL — Helm Charts

Pengganti `infra/k8s/` (Kustomize) yang sebelumnya. Dua chart hidup berdampingan:

| Chart            | Lokasi                | Namespace target            | Fungsi                                                        |
| ---------------- | --------------------- | --------------------------- | ------------------------------------------------------------- |
| `simpel`         | `infra/helm/simpel`   | `simpelv2-staging` / `…-production` | Stack aplikasi: Postgres, Redis, Authenc, Secreton, layanan-*, simpelv1, portal, perlengkapan, Istio routing, NetworkPolicies, HPA, PDB, ServiceMonitors. |
| `simpel-metallb` | `infra/helm/metallb`  | `metallb-system`            | IPAddressPool & L2Advertisement. Cluster-scoped, dirilis terpisah. |

## Quickstart

### Prasyarat

- MicroK8s ≥ 1.27 dengan addon `dns`, `storage`, `istio`, `metallb`, `longhorn`.
- `kubectl` & `helm` 3.13+.

### Deploy staging

```bash
# 1. Render & inspect (tanpa apply)
./infra/helm/deploy.sh staging template | less

# 2. Apply chart aplikasi (Secret BELUM dibuat — pod akan CrashLoopBackOff dulu)
./infra/helm/deploy.sh staging install

# 3. Apply real secrets (sekali setelah install)
cp infra/helm/simpel/values-secrets.example.yaml /tmp/values-secrets.yaml
$EDITOR /tmp/values-secrets.yaml         # isi password / token asli
helm upgrade simpel infra/helm/simpel \
  -f infra/helm/simpel/values-staging.yaml \
  -f /tmp/values-secrets.yaml \
  --set secrets.bootstrap=true \
  --namespace simpelv2-staging --reuse-values

# 4. Verifikasi
kubectl get all -n simpelv2-staging
curl -fsS http://10.1.7.121/health
curl -fsS http://10.1.7.121/portal/
```

### Deploy production

```bash
./infra/helm/deploy.sh production diff       # bandingkan dengan cluster
./infra/helm/deploy.sh production install    # setelah review
```

Production menggunakan tag immutable `:prod` (atau `:v0.1.0` untuk simpelv1) dan
`STRICT` mTLS. Secret production WAJIB dikelola via Secreton SecretSync CRD;
lihat [`layanan/secreton/crates/k8s-operator/README.md`](../../layanan/secreton/crates/k8s-operator/README.md).
Bootstrap (`secrets.bootstrap=true`) hanya untuk first-install.

### MetalLB (sekali per cluster)

```bash
./infra/helm/deploy.sh staging metallb-install
# atau langsung:
helm upgrade --install simpel-metallb infra/helm/metallb \
  --namespace metallb-system
```

### Review apps (per-MR)

```bash
./infra/helm/deploy.sh review-feat-foo install
# uninstall:
./infra/helm/deploy.sh review-feat-foo uninstall
```

## Struktur values

`values.yaml` menyetel default staging-shaped (1 replica, PERMISSIVE mTLS, HPA off).
`values-<env>.yaml` hanya menulis perbedaan. Lihat key-key yang sering di-tweak:

| Knob                                | Lokasi                                           |
| ----------------------------------- | ------------------------------------------------ |
| Replicas, resources, HPA, PDB       | `<komponen>.{replicas,resources,hpa,pdb}`        |
| Image tag/repo                      | `global.imageTag` / `<komponen>.image.{name,tag}` |
| mTLS mode                           | `mtls.mode`                                      |
| Istio routing (Gateway, VS, DR)     | `istio.{gateway,virtualService,destinationRules}`|
| Network policies & ingress gateway  | `networkPolicies.*`                              |
| ResourceQuota & LimitRange          | `resourceQuota.enabled` / `limitRange.enabled`   |
| ServiceMonitors (PrometheusOperator)| `monitoring.serviceMonitors.enabled`             |

## Operasi umum

```bash
# Tambah kapasitas portal sementara
kubectl -n simpelv2-production scale deployment/portal --replicas=5
# Permanen: edit values-production.yaml lalu helm upgrade.

# Rollback
helm rollback simpel -n simpelv2-staging

# Status
helm status simpel -n simpelv2-staging

# Hapus stack (HATI-HATI di production)
helm uninstall simpel -n simpelv2-staging
```

## TLS staging

Self-signed cert untuk `10.1.7.121` (default valid 730 hari). Re-issue:

```bash
./infra/helm/renew-tls-cert.sh
```

Override via env: `STAGING_IP`, `STAGING_DNS`, `TLS_SECRET_NAME`, `CERT_DAYS`.

## Troubleshooting

### Pod CrashLoopBackOff dengan log "missing DATABASE_URL"

Secret belum di-apply. Lihat `Quickstart` step 3 — jalankan helm upgrade dengan
`-f /tmp/values-secrets.yaml --set secrets.bootstrap=true`.

### `helm template` error "image spec missing both repository and name"

Komponen di-enable tapi `image.name` kosong dan tidak ada `image.repository`.
Perbaiki di values: setiap komponen wajib punya `image.name` (akan digabung
dengan `global.registry`) atau `image.repository` (full path).

### Istio VirtualService tidak terpasang ke Gateway

Pastikan `istio.virtualService.gatewayRef` cocok dengan namespace/name Gateway
yang ada (default: `<istioNamespace>/<gateway.name>`). Untuk mensharing gateway
existing di `istio-system`, set `istio.gateway.name: simpelv2-dev-gateway`.

### Secret `simpelv1-db-backup` tidak ada

Setel `simpelv1.dbBackup.source: none` (default) untuk skip init container restore,
atau pre-create ConfigMap/Secret berisi `dbsimpelv1.sql.gz`.

## Dokumen terkait

- [`infra/AGENTS.md`](../AGENTS.md) — pedoman infrastruktur global.
- [`layanan/secreton/AGENTS.md`](../../layanan/secreton/AGENTS.md) — secrets vault.
- [`layanan/authenc/AGENTS.md`](../../layanan/authenc/AGENTS.md) — identity provider.
