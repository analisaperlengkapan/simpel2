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

> **There is no `secreton` CLI anywhere.** The Dockerfile builds exactly one
> binary (`cargo build -p secreton-api --bin api_server`) and the runtime stage
> copies only that. Verified on the live staging pod: `/app` contains
> `api_server` and nothing else, and no `secreton` binary is on `PATH`. Every
> `secreton operator …` / `secreton kv …` line that used to be in this file, in
> `infra/AGENTS.md` and in `RUNBOOK.md` was **procedure that had never been
> run** — it fails with `command not found`. Drive the REST API instead. The
> image does ship `curl` (the container healthcheck uses it).

Operate over a port-forward so `jq` and your key-capture tooling stay on the
workstation — this is the same transport `infra/helm/bootstrap-secreton.sh`
already uses for the rest of the bootstrap chain.

```bash
kubectl -n simpelv2-<env> port-forward svc/secreton 8200:8200 >/dev/null 2>&1 &
PF=$!; trap 'kill $PF' EXIT
until curl -fsS http://127.0.0.1:8200/v1/sys/seal-status >/dev/null 2>&1; do sleep 1; done

# 1. Confirm it is actually uninitialized before you generate key material.
curl -fsS http://127.0.0.1:8200/v1/sys/seal-status | jq
# => {"initialized":false,"sealed":true,"t":3,"n":5,...}

# 2. Init. This is the ONLY moment the unseal keys and root token ever exist.
#    Capture them OFFLINE (password manager / 5 separate holders) — they are
#    not recoverable and are not stored in k8s.
curl -fsS -X POST http://127.0.0.1:8200/v1/sys/init \
  -H 'Content-Type: application/json' \
  -d '{"secret_shares":5,"secret_threshold":3}' | jq   # => {keys:[...], root_token:"..."}

# 3. Unseal with 3 of the 5 keys (one request each).
curl -fsS -X POST http://127.0.0.1:8200/v1/sys/unseal \
  -H 'Content-Type: application/json' -d '{"key":"<key-n>"}' | jq -r '.sealed'
# repeat until .sealed == false

curl -fsS http://127.0.0.1:8200/v1/sys/seal-status | jq -r '.initialized, .sealed'
```

⚠️ Do **not** run init through `kubectl exec … curl`: the keys and root token
would come back through the exec session into terminal scrollback and shell
history. Port-forward keeps the response in one place you control.

`infra/scripts/secreton-ci-bootstrap.sh` is the reference implementation of
this exact sequence (real Shamir init + unseal, `jq`-parsed). It is CI-only
because it *prints* the keys — a fresh throwaway DB every run — but the HTTP
calls it makes are the same ones above.

- **Unseal keys + root token: NEVER stored in k8s** — offline only (password
  manager / separate KMS). After bootstrap, **revoke root**.
- A restarted/rescheduled `secreton-0` comes up **sealed** → must be unsealed again
  (auto-unseal via KMS is the F6 hardening).

## CI / docker-compose bootstrap (ephemeral, no kubectl)

Same REST API as above, reached directly instead of through a port-forward. CI
drives the **REST API on :8200** (`/v1/sys/init` → `{keys[], root_token}`,
`/v1/sys/unseal {key}`, `/v1/sys/seal-status {initialized, sealed}` — all
whitelisted while sealed in `middleware.rs::is_whitelisted`). The `/health`
route is seal-independent; `/v1/health` is behind `seal_check_middleware`.

- Script: `infra/scripts/secreton-ci-bootstrap.sh` (real Shamir init+unseal).
- One-shot compose service `secreton-bootstrap` + secreton `/health` healthcheck.
- CI job `e2e-secreton-gateway` (gated `E2E_INTEGRATION_ENABLED` / `run_e2e`).
- **Seed via the gateway `PUT /v1/secrets/{path}`**, not secreton REST: REST
  `put_secret` (`state.engine`) and gRPC `get_secret` (`self.storage`,
  `HashMap<String,String>` envelope) are different code paths — seeding through
  the gateway (→ gRPC `StoreSecret`) guarantees the write matches the gRPC read
  simpelv1 uses. **CI-only** — keys are throwaway (fresh DB per run), never
  offline-captured secrets.

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

**Status/impl (F-GW PR-D):** the gRPC handlers for
`ConfigureDatabaseConnection` / `CreateDatabaseRole` / `GenerateDatabaseCredentials` /
`RenewLease` are LIVE — `SecretonGrpcService` (`layanan/secreton/crates/grpc/src/server.rs`)
delegates to the real engine `secreton-core …/secrets/database.rs` (creates a
Postgres role via `CREATE ROLE … PASSWORD` templated on `{{username}}`/`{{password}}`,
`SECRETON_DB_TLS_MODE=disable` for plaintext PG). A single shared engine instance
holds the connection/role/lease state, so provisioning + issuance MUST hit the
**same** secreton process.
- **Provisioning path:** steps 1–2 are gRPC-only. Reach them from a script via the
  gateway's env-gated admin routes `POST /v1/database/config/{name}` +
  `POST /v1/database/roles/{role}` — mounted **only** when the gateway runs with
  `GATEWAY_ENABLE_DB_ADMIN=1` (off in the prod simpelv1 sidecar; on in e2e). Prod
  provisioning is an operator action, not a runtime sidecar capability.
- **e2e:** `infra/scripts/secreton-ci-bootstrap.sh` step 6 (opt-in
  `SECRETON_DB_PROVISION=1`) configures the connection, creates the role, leases a
  credential, and proves it authenticates against PG. The engine↔PG path also has
  an `#[ignore]` integration test `dynamic_lease_pg_it` (run with `SECRETON_IT_DB_URL`).
- **Prod caveat:** e2e uses a `SUPERUSER` creation statement (throwaway DB); prod
  MUST use a scoped `GRANT` in the role's `creation_statements`.

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
