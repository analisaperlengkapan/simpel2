# ADR-0001: Secret Management Zero-Trust via Secreton + Kubernetes Auth Backend

- **Status**: Accepted
- **Date**: 2026-05-05
- **Deciders**: SIMPEL DevOps & SecOps
- **Technical context**: Helm refactor + production deployment perdana ke `simpel.kejaksaan.go.id`

## Konteks

Sebelum keputusan ini, secret produksi & staging tinggal di:

- `.env` di working tree (tidak masuk repo karena `.gitignore`, tapi bisa terkopi ke history kalau lupa).
- `k8s Secret` object di namespace (rendered via Helm `secrets.bootstrap=true` flag, nilai diambil dari `values-secrets.yaml` lokal).
- Hardcoded di `values.yaml` di kasus-kasus saat developer terburu-buru (anti-pattern).

Masalah yang muncul:

1. **Rotasi sulit**: tidak ada single source of truth — secret tersebar antara file lokal, k8s Secret, dan kadang env literal di template.
2. **Audit gap**: tidak ada log siapa baca secret apa.
3. **Compliance**: standar Zero-Trust pemerintah (Kejaksaan RI) menuntut secret tidak boleh disimpan di etcd plaintext atau di config file.
4. **Token API eksternal** (SIMAN/MonSAKTI/MySIMKARI) butuh rotation berkala dan kontrol access ketat.

## Pertimbangan

| Opsi | Pros | Cons |
|------|------|------|
| **A. Tetap k8s Secret + Sealed Secrets** | Simpel, GitOps-friendly | Tidak audit-log, tidak ada policy granular, master key bisa hilang |
| **B. External Secrets Operator + cloud KMS** | Standar industri | Vendor lock-in cloud (kita pakai bare-metal microk8s), butuh internet egress |
| **C. HashiCorp Vault + Kubernetes Auth** | Mature, fitur lengkap | Lisensi commercial untuk fitur enterprise (replication, HSM), bukan in-house |
| **D. Secreton + Kubernetes Auth Backend** ★ | In-house (sudah ada di `layanan/secreton`), policy granular, audit log built-in, support gRPC + REST, no external dependency | Implementasi auth backend baru saja stable |

Kita sudah punya **Secreton** (Vault-like service) di workspace dengan:

- KV store, transit engine, PKI engine
- Kubernetes Auth Backend (`layanan/secreton/crates/core/src/services/auth/kubernetes.rs`)
- Client agent (`layanan/secreton/crates/agent/src/auth/kubernetes.rs`)
- Audit log dengan field lengkap (request_id, client_token hash, path, capability, source_ip, user_agent)

## Keputusan

Pakai **Opsi D**: Secreton sebagai source of truth untuk semua secret produksi & staging, dengan **Kubernetes Auth Backend** sebagai mekanisme auth (bukan static token).

### Implementasi (lihat PR 3 di Helm refactor)

1. **ServiceAccount per service** dengan annotation `secreton.simpel.io/role: <name>`.
2. **ProjectedServiceAccountToken** di pod (audience `secreton`, TTL 3600s, path `/var/run/secrets/tokens/secreton-token`).
3. **Helper `_workload.tpl`** otomatis inject volume + env (`SECRETON_ADDR`, `SECRETON_AUTH_METHOD=kubernetes`, `SECRETON_AUTH_ROLE`, `SECRETON_K8S_TOKEN_PATH`) saat `secretonAuth.enabled=true`.
4. **ConfigMap `secreton-policies`** (HCL ACL per service) dan **`secreton-auth-config`** (role definitions JSON) di-render dari `values.yaml` `secretonAuth.policies`.
5. **`bootstrap-secreton.sh`** one-shot post-install: enable kubernetes auth method, configure TokenReview API, apply policies + roles dari ConfigMap.
6. **simpelv1 (Laravel/PHP)**: tidak punya client native → init container `fetch-secrets` (script `monolith/simpelv1/fetch-secrets.sh`) generate `/app/runtime/.env` dari Secreton sebelum main container start.
7. **Migrasi**: `scripts/migrate-env-to-secreton.sh` untuk migrasi `.env` existing ke `kv/data/<prefix>/*`.

### Yang tetap k8s Secret (tidak bisa via Secreton)

- `ghcr-pull` (image pull secret) — kubelet butuh sebelum pod start.
- `simpel-tls` di `istio-system` — Istio gateway baca dari k8s Secret.
- Bootstrap unseal keys → **TIDAK** disimpan di k8s; offline-only (Shamir 3-of-5).

## Konsekuensi

### Positif

- Single source of truth: `kv/<service>/...` di Secreton.
- Audit log lengkap (siapa, kapan, akses path apa).
- Policy granular per service (read-only ke path tertentu).
- Token short-lived (1 jam, renewable) — leak token risk berkurang drastis.
- Rotasi secret tanpa restart pod (consumer fetch ulang saat token renewal).
- Zero-trust selaras compliance Kejaksaan RI.

### Negatif / Risiko

- Bootstrap kompleks: init+unseal Secreton, bootstrap auth backend, lalu flip `secretonAuth.enabled=true`. Butuh runbook (`infra/helm/RUNBOOK.md`) yang dijaga.
- Cold start lebih lambat: pod startup butuh 1 round-trip ke Secreton + token exchange.
- Secreton menjadi single point of failure — perlu HA (replicate, raft consensus). Sudah ada di `layanan/secreton/crates/replication/` tapi belum tested di production.
- simpelv1 init container menambah ~5-10 detik startup per pod.

### Mitigasi

- Bootstrap script idempotent (bisa di-rerun aman).
- Secreton auto-renew token di background (1× per 30 menit) sebelum expire.
- Secreton HA dengan StatefulSet replicas: 3 di production (TODO setelah replication crate stable).
- simpelv1 init container failure → readiness probe gagal → main container tidak terima traffic (graceful).

## Referensi

- `layanan/secreton/crates/core/src/services/auth/kubernetes.rs` (server)
- `layanan/secreton/crates/agent/src/auth/kubernetes.rs` (client)
- `infra/helm/simpel/templates/secreton/` (bootstrap config & policies)
- `infra/helm/bootstrap-secreton.sh` (post-install setup)
- `scripts/migrate-env-to-secreton.sh` (migration helper)
- `monolith/simpelv1/fetch-secrets.sh` (init container script)
- HashiCorp Vault Kubernetes Auth (referensi pattern): https://developer.hashicorp.com/vault/docs/auth/kubernetes
