---
name: secreton-ops
description: Operate & troubleshoot Secreton (SIMPel's Vault-like secret manager) — init/unseal (Shamir), bootstrap auth backend/roles/policies, dynamic database secrets-engine provisioning (ConfigureDatabaseConnection→CreateDatabaseRole→GenerateDatabaseCredentials/RenewLease), recovery when sealed, verify pods fetch secrets. Use during Secreton bootstrap, incidents (sealed/recovery), or onboarding a service to dynamic DB creds. NOT for app-side secret *consumption* conventions (those are in AGENTS.md).
---

# Secreton ops (SIMPel)

Secreton = Vault-like secret manager. **Read first:** `infra/AGENTS.md`
("Bootstrap", "Rotasi Secret Runtime"), `infra/helm/RUNBOOK.md`, and
`layanan/secreton/AGENTS.md`. This Skill = the operational procedures + diagnostics;
mutations to the *deployment* still go via Helm (see `deploy-to-environment`).

## Init + unseal (once per environment, fresh deploy)
```bash
kubectl exec secreton-0 -n simpelv2-<env> -- \
  secreton operator init -shamir-shares=5 -shamir-threshold=3   # save keys OFFLINE
secreton operator unseal <key>   # × 3 (threshold)
```
- **Unseal keys + root token: NEVER stored in k8s** — offline only (password
  manager / separate KMS). After bootstrap, **revoke root**.
- A restarted/rescheduled `secreton-0` comes up **sealed** → must be unsealed again
  (auto-unseal via KMS is the F6 hardening).

## Bootstrap auth backend + roles + policies
```bash
SECRETON_TOKEN=<root> ./infra/helm/bootstrap-secreton.sh <staging|production>
# ...then, after services are wired and healthy:
./infra/helm/bootstrap-secreton.sh <env> --revoke-root
```
Run **once per env**, BEFORE flipping `secretonAuth.enabled=true`. Each service has a
ServiceAccount annotated `secreton.simpel.io/role: <name>` (zero-trust SA-token auth).

## Dynamic database credentials (per-service onboarding)
Vault-style leases instead of a static `DATABASE_URL`:
1. `ConfigureDatabaseConnection` (point Secreton at the PG instance).
2. `CreateDatabaseRole` (the role the service will assume).
3. Set `SECRETON_DB_ROLE=<role>` in `values-<env>.yaml` (`<svc>.env`) — absent ⇒
   service keeps the static `DATABASE_URL` (legacy behavior).
4. Verify: service calls `GenerateDatabaseCredentials` at boot, builds the pool from
   the dynamic DSN; a background task `RenewLease` at **½ TTL**. At the rotation
   boundary the pod exits and k8s restarts it with fresh creds (see
   `layanan/perlengkapan/src/main.rs` `spawn_db_lease_renewal`).

## Incident playbook
- **Pods CrashLoop "sealed"/can't fetch secret** → `secreton-0` is sealed (restart?)
  → unseal ×threshold; confirm the SA role + policy exist (bootstrap ran?).
- **Service 500s after rotation** → lease expired without renewal; pod should have
  exited+restarted — check `kubectl logs` for the renewal task, and the DB role TTL.
- **Recovery when sealed & keys needed** → retrieve offline unseal keys; `unseal`.
  Raft snapshot/restore for data loss (see `layanan/secreton/AGENTS.md`).
- **Verify a pod sees its secret** → exec in, check the mounted/fetched value;
  confirm `secreton.simpel.io/role` annotation + SA token mount.

## Guardrails
- Production secrets: **source of truth = `kv/<service>/...` in Secreton**, never a
  k8s Secret object or repo `.env`.
- Never echo secret values into logs/CI output.
