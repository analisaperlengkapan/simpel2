# AGENTS.md - Secreton

> AI Agent Guide for working with the Secreton codebase.

## Service Context

> **Skill:** untuk operasi/insiden Secreton (init/unseal, bootstrap role/policy,
> provisioning dynamic DB creds, recovery saat sealed) pakai Skill **`secreton-ops`**
> (`.claude/skills/`).

**Secreton** is the secrets management, transit encryption, and PKI service for SIMPEL. It is a Rust binary in the simpelv2 monorepo (`layanan/secreton/`). It exposes a REST API (Axum 0.8), a gRPC API (Tonic 0.14), and a CLI tool (clap).

**Binary:** `api_server` (crate `secreton-api`)  
**Default ports:** 8200 (HTTP), 8201 (gRPC), 8300 (cluster)  
**Config file:** `secreton.toml` (loaded from working directory or `--config` flag)

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust Edition 2024, MSRV 1.97.1 |
| HTTP | Axum 0.8.x |
| gRPC | Tonic 0.14 + Prost |
| CLI | clap (derive) |
| Database | PostgreSQL (tokio-postgres + deadpool-postgres) |
| Symmetric Crypto | AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305 |
| Asymmetric Crypto | Ed25519, ECDSA-P256, ECDSA-secp256k1, X25519 |
| Post-Quantum | ML-KEM, ML-DSA (via pqcrypto, hybrid mode) |
| Hashing | SHA-256, SHA-384, SHA-512, SHA3-256, SHA3-384, SHA3-512, BLAKE3 |
| Key Derivation | Argon2id, PBKDF2, HKDF, scrypt |
| Secret Sharing | Shamir Secret Sharing |
| HSM | PKCS#11 (optional) |
| Consensus | Raft (feature-gated: `raft-consensus`) |

## Crate Map

```
crates/
├── types/              # secreton-types    — Shared enums, newtypes (SecurityLevel, ResourceId, Metadata, Tags)
├── core/               # secreton-core     — Business logic, config, audit, auth, namespace, PKI, resilience
├── crypto/             # secreton-crypto   — RustCrypto wrappers: encryption, FPE, hashing, Shamir, transit, hybrid PQ
├── storage/            # secreton-storage  — StorageBackend trait + impls: Raft, PostgreSQL, memory, file, encrypted, cached
├── api/                # secreton-api      — Axum HTTP server, route definitions, middleware, services, metrics
├── grpc/               # secreton-grpc     — Tonic gRPC server (secreton.v1 package, 36 RPCs)
├── cli/                # secreton-cli      — CLI binary (clap): seal, secret, transit, policy, audit, backup, replication
├── agent/              # secreton-agent    — Sidecar: auto-auth, token renewal, template rendering
├── hsm/                # secreton-hsm      — HSM via PKCS#11: Thales Luna, AWS CloudHSM, SoftHSM2, YubiHSM2
├── k8s-operator/       # secreton-k8s-operator — K8s CRD controller for SecretSync
├── auto-unseal/        # secreton-auto-unseal  — Transit, AWS KMS, GCP KMS, Azure Key Vault unseal providers
├── backup/             # secreton-backup   — Backup/restore scheduler, S3/local storage
├── health/             # secreton-health   — Health check registry (Healthy/Degraded/Unhealthy)
└── replication/        # secreton-replication — DR replication, failover, conflict resolution
```

### Dependency graph (simplified)

```
api ─→ core, crypto, storage, grpc, health
core ─→ types, crypto
crypto ─→ types
storage ─→ types
grpc ─→ core, crypto, storage
cli ─→ (HTTP client to api)
agent ─→ (HTTP client to api)
auto-unseal ─→ core, crypto
backup ─→ core, storage, crypto
k8s-operator ─→ (gRPC client to grpc)
```

## Application State

The central state is `ServiceContainer` (in `crates/api/src/services/mod.rs`), wrapped in `Arc` as `AppState`.

Key fields: `config`, `storage`, `secret_storage`, `pool` (deadpool-postgres), `crypto`, `auth`, `engine` (SecretService), `admin`, `audit`, `seal`, `namespace`, `transit_engine`, `pki_engine`, `lease_manager`, `policy_service`, `hsm` (optional), plus the surviving secret engines: `aws_engine`, `rotation_engine`, `wrapping`, `mfa`. The GCP/Azure/LDAP/RabbitMQ/Kafka/KMIP engines were deleted in #103 — see "Secrets engines" below.

### `storage` vs `secret_storage` — pick the right one

There are **two** storage handles and they are not interchangeable:

| Handle | What it is | Use it for |
|---|---|---|
| `storage` | the RAW backend (postgres/file/raft) | engine state, sessions, OAuth state, namespace metadata, injection sessions, stats |
| `secret_storage` | `storage` wrapped in `EncryptedStorage`, keyed off the seal master key | **every secret payload**, REST and gRPC alike |

Anything written through `storage` lands in Postgres unencrypted. Until #127 that
was the only handle, which is how KV secrets ended up sitting in
`vault_entries.encrypted_data` as plaintext JSON under a metadata stamp that
claimed `"algorithm": "aes-256-gcm"`. If you are adding a path that persists
secret material, it goes through `secret_storage`.

The wrapper resolves its key per call via `SealMasterKeyProvider`
(`crates/core/src/storage/seal_key.rs`), so **secret I/O fails while the engine
is sealed** — that is the ceremony working, not a bug. Seal state itself stays on
the raw backend; encrypting it under the key it protects would deadlock unseal.

Entries carry `encryption_metadata.envelope = "aes256gcm-v1"`. Rows without it
predate encryption: they still read (with a warning) and are sealed on next
write. Do not "fix" that passthrough away until staging and production have been
re-written; removing it strands every existing secret.

`AppState` = `ApiState { services: Arc<ServiceContainer>, transit, kv, pki, prometheus_handle, metrics }` — passed to Axum via `.with_state(state)`.

## Router & Middleware

`create_api_router()` in `crates/api/src/lib.rs` builds the full router:

```
Top-level (unauthenticated):
  GET  /health
  GET  /ready
  GET  /live
  GET  /version
  GET  /metrics
  GET  /metrics/prometheus
  GET  /metrics/tls

/v1 (protected):
  Merged from:
    1. Legacy routers:
       /v1/transit  — create_transit_router()
       /v1/pki      — create_pki_router()
    2. Handler routers:
       create_protected_router()   — auth, secret, admin, sys/*, dynamic, raft, etc.
       create_unprotected_router() — sys/seal endpoints (init, unseal, seal-status)
```

**Middleware stack (outer → inner):**

1. `standard_cors` (lib-backend) — CORS
2. `correlation_id_middleware` (lib-backend) — X-Correlation-ID
3. `RequestLogger` (lib-backend) — structured request logging
4. `security_headers_middleware` (lib-backend) — HSTS, CSP, X-Frame-Options
5. `request_rate_middleware` — per-IP rate limiting
6. _(on /v1 only)_
   - `metrics_middleware` — per-endpoint latency/counters
   - `auth_middleware` — JWT validation, inserts `RequestContext`
   - `seal_check_middleware` — returns 503 if vault sealed
   - `policy_check_middleware` — **authorization**; 403 if no policy allows it

The `/v1` four are listed in execution order, which is the REVERSE of the
`.layer()` calls in `create_api_router` — tower makes the last layer added the
outermost. Authorization is innermost by design: it needs the identity
`auth_middleware` put in the request.

### Authorization is enforced, and it fails closed

`policy_check_middleware` loads the caller's policies (`RequestContext::
policy_names`, derived from the token's `policies`, else metadata, else
`{role}-policy`) and evaluates them against the request path and the HTTP verb
mapped to a capability. Deny overrides allow; **no matching rule is a denial**.
If the policy store cannot be reached it answers 500 — never "allow".

Two consequences to know before debugging a 403:

- **With no policies seeded, every non-root principal is refused.** That is the
  posture, not a misconfiguration. Seed policies via `/v1/sys/policies`.
- **`root` is implicit** (`middleware::ROOT_POLICY`), as in Vault: it grants
  everything and has no stored rules, because a freshly initialised engine holds
  no policies and the token from `/v1/sys/init` has to be able to write the
  first one.

Paths exempt from authorization are exactly the paths exempt from
authentication — `policy_check_middleware` calls the same `is_whitelisted()`
predicate `auth_middleware` uses. It used to keep its own copy, which had
already drifted (it was missing both `/sys/rekey/*` entries).

## Handler Modules

All handlers in `crates/api/src/handlers/`:

| Module | Prefix | Description |
|--------|--------|-------------|
| `health` | `/health`, `/ready`, `/live` | Health probes (unauthenticated) |
| `seal` | `/v1/sys` | Init, seal, unseal, rekey (unauthenticated) |
| `auth` | `/v1/auth` | Login, logout, token refresh/verify, MFA, OAuth, sessions |
| `secret` | `/v1/secret` | CRUD, keys, encrypt/decrypt/sign/verify/hash, backup, audit |
| `admin` | `/v1/admin` | Users, roles, config, maintenance, security scan, metrics |
| `namespace` | `/v1/sys/namespaces` | Multi-tenant namespace CRUD + stats |
| `lease` | `/v1/sys/leases` | Renew, revoke, lookup, stats |
| `policy` | `/v1/sys/policies` | Policy CRUD + test — the ONLY policy API |
| `wrapping` | `/v1/sys/wrapping` | Wrap, unwrap, lookup, rewrap |
| `rotation` | `/v1/sys/rotation` | Rotation policies, scheduler, history |
| `totp` | `/v1/sys/totp` | TOTP key management, generate, validate |
| `crypto` | `/v1/sys/crypto` | HMAC, random bytes, re-encrypt |
| `transform` | `/v1/sys/transform` | FPE roles, encode/decode, tokenize/detokenize |
| `pki` | `/v1/sys/pki` | Extended PKI: intermediate CA, OCSP, templates, renewal |
| `ssh` | `/v1/sys/ssh` | SSH CA, roles, creds, sign user/host, OTP |
| `aws` | `/v1/sys/aws` | Dynamic AWS IAM/STS credentials |
| `key_hierarchy` | `/v1/sys/key-hierarchy` | Key hierarchy management |
| `inject` | `/v1/sys/inject` | Environment variable injection |
| `webhook` | `/v1/sys/webhooks` | Webhook subscriptions and delivery |
| `classification` | `/v1/secret` | Secret classification endpoints |
| `revocation` | `/v1/secret` | Certificate/credential revocation |
| `dynamic` | `/v1/dynamic` | Dynamic database credentials |
| `metrics` | `/v1/metrics` (also `/metrics`) | JSON metrics snapshot |

> **Trimmed 2026-07-18 (#103).** The `gcp`, `azure`, `identity`, `kmip`, `ldap`,
> `rabbitmq` and `kafka` engines were **deleted**. None had a client library —
> they generated a credential locally, persisted a lease, and returned success
> **without ever contacting the target system**, so callers would have received
> fabricated credentials as if valid. `identity` was a second OIDC provider,
> which contradicts the codified split (authenc = IAM, secreton = secrets only).
> None had a consumer. Do **not** re-add an engine without a real client library
> and an integration test that proves the remote object was actually created.

The same audit found the same disease on the authentication side:

> **Auth methods trimmed 2026-07-19 (#103).** The whole
> `core/src/services/auth/` tree (9 modules, 3.544 baris) was **deleted**: it had
> **zero consumers anywhere in the repo** — the live authentication path is
> `core/src/auth/authenc_provider.rs` (delegasi ke authenc) plus the API crate's
> own `api/src/services/auth.rs`, which is what the binary actually constructs.
>
> Five of them were simulated, and one was actively dangerous: `ldap.rs` `bind()`
> returned `Ok(())` for **any non-empty password** and `search_user()` fabricated
> a user for **any username** — an accept-anything authenticator sitting behind
> the `AuthMethod` trait, i.e. exactly the shape something gets wired into later.
> `github`/`oidc`/`kubernetes`/`aws` were likewise simulated ("Simulate parsing
> JWT", "return mock identity"). `userpass`/`certificate` were genuinely
> implemented (Argon2id, x509-parser) but duplicated the live API-crate service.
>
> **Rule: an auth method is security-critical dead code.** Never add one without
> a real client library, a mounted route, and a test proving a _wrong_ credential
> is **rejected**.
>
> This is not theoretical. `ldap.rs` shipped 3 passing tests; all 3 fed a
> valid-looking credential and asserted success:
>
> ```rust
> let result = ldap.authenticate("testuser", "password123").await;
> assert!(result.is_ok());          // passes BECAUSE bind() accepts anything
> ```
>
> Not one fed a wrong password and asserted failure. The suite was green and
> encoded the bug as expected behaviour. **For anything that decides "is this
> caller who they claim to be", the negative case is the test that matters** —
> a happy-path assertion cannot distinguish a working authenticator from one
> that returns `Ok` unconditionally.

And a third instance, this time a duplicated layer rather than a fake one:

> **Shadow service layer deleted 2026-07-19 (#103).** `core/src/services/`
> carried a second, unreachable copy of the service layer —
> `admin_service`, `auth_service`, `secret_service`, `policy_service`,
> `dynamic_role_service`, `rbac`, `key_manager` (4.280 baris incl. its test).
> All had zero external references (`auth_service` only via the equally-dead
> `admin_service`), while `api/src/services/{admin,auth,secret_engine,policy}.rs`
> are the ones `ServiceContainer` constructs. `key_manager` was not even compiled
> (`// pub mod key_manager;`). Git history confirms which copy is real: the API
> copies have 5–8 commits, the core copies only 2–3, all mechanical clippy sweeps.
> Before "moving" a service into `core`, switch the consumer over in the same
> change — otherwise both copies survive and only one is real.

There is also a `raft` handler (feature-gated behind `raft-consensus`):

| Module | Prefix | Description |
|--------|--------|-------------|
| `raft` | `/v1/raft` | Join, peers, status, election-stats, snapshot |

## CLI Structure

Binary: `secreton-cli` (in `crates/cli/src/main.rs`)

Top-level commands (from `Commands` enum):

| Command | Type | Description |
|---------|------|-------------|
| `status` | standalone | System health check |
| `login` | standalone | Authenticate (--method userpass\|token) |
| `logout` | standalone | End session |
| `config` | subcommand | `set`, `get`, `show` |
| `policy` | subcommand | `list`, `read`, `write`, `delete`, `fmt`, `validate`, `test` |
| `token` | subcommand | `create`, `lookup`, `renew`, `revoke`, `capabilities` |
| `seal` | subcommand | `init`, `seal`, `unseal`, `status`, `rekey` (init/update/cancel/status) |
| `auto-unseal` | subcommand | `configure`, `status`, `test`, `disable` |
| `backup` | subcommand | `create`, `restore`, `verify`, `list` |
| `operator` | subcommand | `diagnose` |
| `audit` | subcommand | `list` (with `--user`, `--operation`, `--path`, `--start_time`, `--end_time`, `--limit`, `--format`) |
| `replication` | subcommand | `enable`, `disable`, `status`, `promote`, `add-secondary`, `remove-secondary`, `lag` |
| `transit` | subcommand | `create-key`, `list-keys`, `encrypt`, `decrypt` |
| `secret` | subcommand | `put`, `get`, `list`, `delete` |

CLI modules (in `crates/cli/src/`):

```
main.rs            — Cli struct, Commands enum, dispatch
auth.rs            — login_command, logout_command
seal.rs            — SealCommand enum, execute_seal_command
token.rs           — TokenCommand enum, execute_token_command
policy.rs          — PolicyCommand enum, execute_policy_command
policy_parser/     — TOML policy file parsing and formatting
  mod.rs
  toml_parser.rs
  formatter.rs
audit.rs           — AuditCommand enum, execute_audit_command
backup.rs          — BackupCommand enum, execute_backup_command
operator.rs        — OperatorCommand enum, execute_operator_command
replication.rs     — ReplicationCommand enum, execute_replication_command
auto_unseal.rs     — AutoUnsealCommand enum, execute_auto_unseal_command
config.rs          — CliConfig (server_url, default_namespace, token storage)
http_client.rs     — AuthenticatedClient (adds auth header to requests)
middleware.rs       — SealChecker (pre-flight seal status check)
token_store.rs     — Persistent token storage (~/.secreton/token)
```

## gRPC API

Proto files in `proto/`:

- `secreton.proto` (817 lines) — package `secreton.v1`, service `SecretonService` with 36 RPCs
- `common.proto` (128 lines) — shared message types

RPCs grouped:

| Category | RPCs |
|----------|------|
| Secrets | StoreSecret, GetSecret, DeleteSecret, ListSecrets |
| Transit | CreateKey, Encrypt, Decrypt, Sign, Verify, RotateKey |
| Cluster | GetClusterStatus, ListPeers, AddNode, RemoveNode |
| Snapshots | CreateSnapshot, ListSnapshots, RestoreSnapshot |
| Namespaces | ListNamespaces, CreateNamespace, GetNamespace, UpdateNamespace, DeleteNamespace, GetNamespaceStats |
| Dynamic | GenerateDatabaseCredentials, CreateDatabaseConnection, GetDatabaseConnection, DeleteDatabaseConnection, CreateDatabaseRole, GetDatabaseRole, DeleteDatabaseRole |
| Leases | RenewLease, RevokeLease, RevokeLeasePrefix, LookupLease, ListLeases, GetLeaseStats |
| Policies | ListPolicies, CreatePolicy, GetPolicy, UpdatePolicy, DeletePolicy, TestPolicy |
| Wrapping | WrapData, UnwrapData, LookupWrappingToken, RewrapData |
| Health | HealthCheck, GetMetrics |

Generated code goes to `crates/grpc/src/generated/`.

### Client authentication is enforced, and it fails closed

Every one of those 36 RPCs reads or mutates secret material, so the listener
authenticates its callers with **mTLS client certificates** — not bearer tokens.
Bringing it up:

1. `resolve_listener_security(GrpcTlsConfig::from_env(), allow_insecure_from_env())`
   decides. Without mTLS material it returns an **error**, and the process
   refuses to start. The only way past is `GRPC_ALLOW_INSECURE=true`.
2. `serve_with_mtls(addr, tls, authorizer)` attaches `ClientAuthLayer` to
   `Server::builder()` — one layer covering every method, so a new RPC is
   guarded the moment it is routed.
3. `serve_insecure(addr, reason)` is the opt-out. It logs the reason at WARN on
   every startup.

| Variable                 | Meaning                                                    |
|--------------------------|------------------------------------------------------------|
| `GRPC_TLS_CERT_PATH`     | server certificate (PEM)                                    |
| `GRPC_TLS_KEY_PATH`      | server private key (PEM)                                    |
| `GRPC_TLS_CA_PATH`       | CA signing client certs — its presence turns mTLS on         |
| `GRPC_ALLOW_INSECURE`    | `true` serves anonymous callers; production sets `false`     |
| `GRPC_ALLOWED_CLIENT_CNS`| comma-separated CN allowlist; unset ⇒ CA trust is the rule   |

Same names as authenc's gRPC boundary — one convention configures both.

**mTLS authenticates a workload, it does not authorize a request.** A valid
`gateway` certificate proves the caller is the gateway; it says nothing about
which paths the gateway may read. Do not read "mTLS is on" as "secrets are
access-controlled".

Per-path scoping on this boundary is **still open** (task #129). The REST side
got it — see "Authorization is enforced, and it fails closed" — but the tower
layer cannot do the same job here, because the secret path travels in the
request body, not in the method name. That check belongs in the handler,
reading `PeerIdentity` out of the request extensions.

Istio does **not** cover this hop: `values.yaml` `mtls.disableForHosts`
excludes authenc, secreton and postgres, and both authenc and secreton run
`istioInjection: false` — including under production `STRICT`. The mesh cannot
substitute for the layer.

## Configuration

Primary config file: `secreton.toml` (see `secreton.toml.example` for all options).

### Two-phase config loading

1. **BootstrapConfig** (`crates/core/src/config/mod.rs`) — loaded first:
   - `storage` (backend type, raft config, postgres URL)
   - `listener.http` (bind address, TLS)
   - `listener.grpc` (bind address, enabled)
   - `seal` (type, shamir params, KMS config)

2. **ApplicationConfig** — loaded after bootstrap:
   - `encryption`, `authentication`, `policies`, `audit`

3. **ApiConfig** (`crates/api/src/config.rs`) — derived from both:
   - HTTP/gRPC server settings, auth, rate-limit, TLS, monitoring, CORS, logging, HSM, database, storage

### Key config sections in `secreton.toml`

```toml
log_level = "info"       # trace|debug|info|warn|error
log_format = "json"      # json|pretty

[storage]
backend = "raft"         # raft|postgres|file|memory

[storage.raft]
path = "/var/lib/secreton/raft"
node_id = "node1"

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = false

[listener.grpc]
enabled = true
address = "0.0.0.0:8201"

[seal]
type = "shamir"
[seal.shamir]
shares = 5
threshold = 3

[telemetry]
prometheus_enabled = true
```

### Environment variable override

Environment variables use prefix `SECRETON__` (double underscore as separator):

```
SECRETON__STORAGE__BACKEND=postgres
SECRETON_STORAGE_URL=postgres://user:pass@host:5432/secreton
SECRETON__LISTENER__HTTP__ADDRESS=0.0.0.0:8200
```

## Database

PostgreSQL migrations in `migrations/` (12 files):

| Migration | Tables/Changes |
|-----------|---------------|
| `create_audit_logs` | Audit log tables |
| `create_secret_manager_state` | Secret manager state |
| `create_namespaces` | Namespace isolation |
| `create_dynamic_roles` | Dynamic secret roles |
| `create_leases` | Lease management |
| `create_policies` | Policy storage |
| `create_wrapping_tokens` | Response wrapping tokens |
| `create_raft_snapshots` | Raft snapshot metadata |
| `use_secreton_schema` | Schema namespacing |
| `dynamic_role_system` | Dynamic role system |
| `create_sealed_master_keys` | Sealed master key storage |

Initialize: `psql -f scripts/init-db.sql`

## Build & Run

```bash
# Build API server / CLI
cargo build -p secreton-api --bin api_server
cargo build -p secreton-cli --bin secreton-cli

# With Raft support
cargo build -p secreton-api --bin api_server --features raft-consensus

# Run (dev, in-memory storage)
cargo run -p secreton-api --bin api_server

# Test
cargo test -p secreton-core
cargo test -p secreton-crypto
cargo test -p secreton-api

# Format & lint
cargo fmt --all && cargo clippy --workspace
```

### Docker

4-stage Dockerfile using cargo-chef. Ports: 8200, 8201, 8300

## Deployment

Deploy **hanya** via umbrella Helm chart `infra/helm/simpel/` (GitOps; plain-YAML/Kustomize
lama sudah dihapus). Image: `ghcr.io/analisaperlengkapan/simpel2/secreton:vX.Y.Z`
(SemVer immutable — dilarang `latest`/`stag`/`prod`). Alur deploy WAJIB staging→promote→production.
Detail: [`infra/AGENTS.md`](../../infra/AGENTS.md) + Skill `deploy-to-environment` / `secreton-ops`.

## Coding Patterns

### Adding a new handler module

1. Create `crates/api/src/handlers/myfeature.rs`
2. Add `pub mod myfeature;` in `crates/api/src/handlers/mod.rs`
3. Add routes in `create_protected_router()` or `create_unprotected_router()` (in `crates/api/src/handlers/mod.rs`)
4. Inject dependencies via `AppState` → `ServiceContainer`

### Handler pattern

```rust
use axum::{Json, extract::State};
use crate::{AppState, ApiResult, response::ApiResponse};

pub async fn my_handler(
    State(state): State<AppState>,
    Json(req): Json<MyRequest>,
) -> ApiResult<Json<ApiResponse<MyResponse>>> {
    let result = state.services.engine.do_thing(&req).await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    Ok(Json(ApiResponse::success(result)))
}
```

### Adding a new engine/service

1. Implement in `crates/core/src/services/` or a new crate
2. Add `Arc<MyEngine>` field to `ServiceContainer`
3. Initialize in `ServiceContainer::new()` (`crates/api/src/services/mod.rs`)
4. Create handler module in `crates/api/src/handlers/`
5. Wire routes

### Error handling

- `ApiError` / `ApiResult<T>` in `crates/api/src/error.rs`
- `CoreError` in `crates/core/src/error.rs`
- All handlers return `ApiResult<Json<ApiResponse<T>>>`
- Standard HTTP status codes: 200, 201, 400, 401, 403, 404, 409, 500

### Response format

All responses use `ApiResponse<T>`:

```json
{
  "success": true,
  "data": { ... },
  "metadata": {
    "request_id": "...",
    "timestamp": "..."
  }
}
```

Error responses:

```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "..."
  }
}
```

## Testing

```bash
# Unit/integration tests
cargo test --workspace

# Specific crate
cargo test -p secreton-crypto
cargo test -p secreton-api

# E2E (requires running instance)
bash test_e2e_v3.sh

# Benchmarks
cargo bench -p secreton --bench transit_bench
cargo bench -p secreton --bench storage_bench
```

Test suites in `tests/`:

| File | Coverage |
|------|----------|
| `test_compliance.rs` | Government compliance checks |
| `test_config_deployment.rs` | Config loading, deployment validation |
| `test_engine.rs` | Secret engine operations |
| `test_crypto*.rs` | Crypto roundtrips, key management |
| `test_integration*.rs` | API integration tests |
| `test_lease*.rs` | Lease lifecycle |
| `test_performance*.rs` | Performance benchmarks |
| `test_pki*.rs` | PKI/certificate tests |
| `test_post_quantum*.rs` | Hybrid PQ crypto |
| `test_raft*.rs` | Raft consensus |
| `test_seal*.rs` | Seal/unseal |
| `test_security*.rs` | Security validation |
| `test_storage*.rs` | Storage backend tests |

## Pitfalls

**DO NOT:**

- Log plaintext secrets. Use `tracing::info!("Stored at path: {}", path)`, never log the value.
- Specify dependency versions in member `Cargo.toml`. Use `dependency = { workspace = true }`.
- Call Secreton directly from microfrontends. Route through layanan REST API → gRPC.
- Use environment variables for production secrets. Use the seal/unseal mechanism.
- Set Shamir shares=1, threshold=1. Minimum recommended: shares=5, threshold=3.
- Skip the seal check middleware for authenticated endpoints.
- Persist secret material through `ServiceContainer::storage`. That handle is the
  raw backend — use `secret_storage` (see "Application State").
- Write an `encryption_metadata` claim by hand. The layer that does the
  encryption owns that field; a hard-coded `"algorithm"` is how #127 stayed
  invisible for as long as it did.
- Add a `Server::builder()` call of your own, or reach for `.with_interceptor`.
  Client auth is a tower layer because `PeerAuthorizer::authorize` is async and
  a tonic interceptor cannot await. Go through `serve_with_mtls`.
- Check the caller per-RPC. One forgotten line in a 36-RPC service is an
  unauthenticated hole; the layer is what makes that impossible.
- Assert a security property with a round-trip. `store` then `get` passes
  identically whether or not anything is encrypted, and whether or not anything
  is enforced. Reach past the wrapper:
  `encrypted_storage::tests::encryption_at_rest_is_real` reads the inner backend
  directly, `crates/grpc/tests/mtls_enforcement.rs` mints a CA and drives a real
  handshake, and `crates/api/tests/policy_enforcement.rs` drives the real router
  rather than one it assembled itself.
- Mount a second policy API. `/v1/sys/policies` is guarded by `require_admin`;
  an unguarded duplicate at `/v1/secret/policies` called the same
  `policy_service` with no check at all, so the guard could be skipped by
  changing the URL. One route, one guard.
- Write a middleware and leave the mounting for later. `policy_check_middleware`
  was complete, documented, and referenced in comments — and layered nowhere.
  It is the **third** control found in that state, after the `rbac_middleware`
  citation removed in #771 and `interceptor::auth_interceptor` deleted in #772.
  If you add a middleware, add the `.layer()` call in the same change, plus a
  test that fails when the layer is removed
  (`crates/api/tests/policy_enforcement.rs` is the worked example).
- Give the policy check its own copy of the auth whitelist. Call
  `is_whitelisted()`. Two lists drift, and the drift stays silent until the day
  something finally mounts the second one.
- Back a secrets route with process-local state. `/v1/kv` was a `HashMap`
  behind an `RwLock`, rebuilt in `main()` on every boot: writes answered `200`
  with a version number, reads were correct for the life of the pod, and the
  whole store was gone on restart. Nothing in the API surface said so, and no
  round-trip test could tell (#130). Secrets go through
  `ServiceContainer::secret_storage`; `crates/api/tests/no_inmemory_kv.rs`
  reads that handle directly after an HTTP write and fails if a route ever
  answers out of memory again.
- Hand the policy engine an HTTP URI. Policies name **resources**
  (`sys/capabilities-self`, `secret/data/simpelv1/*`) — that is what the
  migration seeds, what the Helm block renders, and what Vault does. It is also
  the only vocabulary the gRPC boundary can share, since a gRPC call carries the
  secret path in its body and has no URI. `PolicySet::path_matches` is a plain
  glob with no normalisation, so `/v1/sys/capabilities-self` matched nothing and
  authorization silently collapsed to "deny everything that is not root" —
  fail-closed, but unable to grant anything either. Convert with
  `middleware::resource_path` before evaluating. Note the ordering:
  `is_whitelisted` still takes the raw URI, because that list is about routes.
- Narrow a secret path to a single URL segment. Real paths are hierarchical
  (`simpelv1/app`, `postgres/simpelv2`), so `/data/{path}` 404s on every caller
  that matters while `/data/{*path}` works. The 404 reads as "no such secret",
  not "no such route" — which is why this is worth stating rather than
  rediscovering.

**DO:**

- Add new workspace dependencies in the root `Cargo.toml` `[workspace.dependencies]` first.
- Run `cargo fmt --all && cargo clippy --workspace` before commits.
- Use `ApiResponse::success(data)` for all handler responses.
- Audit all secret access (the `AuditLogger` handles this when using `ServiceContainer` methods).
- Use the existing `ServiceContainer` initialization pattern when adding new engines.
- Feature-gate experimental or heavy features (e.g., `raft-consensus`).

## Key File Reference

| Purpose | File |
|---------|------|
| API server entry point | `crates/api/src/bin/api_server.rs` |
| Router setup | `crates/api/src/lib.rs` → `create_api_router()` |
| All handler modules | `crates/api/src/handlers/mod.rs` |
| ServiceContainer | `crates/api/src/services/mod.rs` |
| API config | `crates/api/src/config.rs` |
| Middleware | `crates/api/src/middleware/` |
| Core config | `crates/core/src/config/mod.rs` |
| CryptoEngine | `crates/crypto/src/lib.rs` |
| StorageBackend trait | `crates/storage/src/lib.rs` |
| CLI entry point | `crates/cli/src/main.rs` |
| gRPC server | `crates/grpc/src/server.rs` |
| Proto definitions | `proto/secreton.proto`, `proto/common.proto` |
| Config example | `secreton.toml.example` |
| DB init script | `scripts/init-db.sql` |
| Migrations | `migrations/` |

## Related Docs

| Document | Path |
|----------|------|
| Root AGENTS.md | `/AGENTS.md` |
| Authenc AGENTS.md | `/layanan/authenc/AGENTS.md` |
| lib-backend / lib-core | `/lib/AGENTS.md` |
| Secreton README | `/layanan/secreton/README.md` |
| Config reference | `/layanan/secreton/secreton.toml.example` |
| Deployment manifests | `/layanan/secreton/deploy/` |

## 🔐 Secret Fetching by Other Services (Consumer Side)

Secreton itu sendiri **tidak fetch dari Secreton** (avoid circular). Tapi dokumentasikan di sini cara service lain (authenc, layanan-perlengkapan, layanan-integrasi, simpelv1) consume:

### Kubernetes Auth Backend — ⚠️ BELUM ADA SISI SERVER (jangan diaktifkan)

> **Terverifikasi 2026-07-19 (#103, task #105). Mekanisme ini TIDAK berfungsi.**
> Yang di bawah adalah desain yang direncanakan, bukan perilaku nyata:
>
> - **Tidak ada route `/v1/auth/kubernetes/login`** di API. Yang ada hanya
>   `/v1/auth/login` dan `/oauth/{provider}`.
> - Bekas "implementasi server-side" (`core/src/services/auth/kubernetes.rs`)
>   tidak pernah di-mount (nol konsumen) dan **dihapus di #103** karena
>   disimulasi (`// Simulate parsing JWT`) — seperti 5 auth method lain.
> - `infra/helm/bootstrap-secreton.sh` ditulis untuk **API HashiCorp Vault**,
>   bukan Secreton: memanggil `/sys/auth/kubernetes`, `/auth/kubernetes/config`,
>   `/auth/kubernetes/role/<svc>`, dan contoh token-nya `hvs.xxxx` (prefix Vault).
>   Dengan `set -euo pipefail` + `curl -fsS`, script **abort di langkah 4/6**.
>
> **Tidak memutus apa pun hari ini** — `secretonAuth.enabled: false` di
> `values.yaml:125` dan tak ada override staging/production. **JANGAN flip
> `secretonAuth.enabled=true`** (termasuk sebagai prasyarat F6-A) sebelum task
> #105 diputuskan: bangun sisi server, atau buang mekanisme ini dan pertahankan
> jalur token/gRPC (`secretonGrpcUrl`) sebagai satu-satunya cara konsumsi.
>
> Sisi klien memang ada (`crates/agent/src/auth/kubernetes.rs` + policy per-service
> di `values.yaml`) — itulah yang membuat gap ini tampak seperti fitur jadi.

### Login Flow

1. Pod consumer project SA token via `projected.serviceAccountToken` (audience `secreton`, TTL 3600s).
2. `POST /v1/auth/kubernetes/login {"role": "<service>", "jwt": "<sa-jwt>"}` → return `client_token` (TTL 1 jam, max 24 jam dengan renew).
3. Use `client_token` di header `X-Secreton-Token` untuk fetch dari `kv/data/<allowed-path>`.
4. Auto-renew via `POST /v1/auth/token/renew-self` setiap 30 menit di background task.

### Audit

- Semua secret access **WAJIB** ter-log di Secreton audit backend dengan field: `time, request_id, client_token (hash), path, capability, source_ip, user_agent`.
- **NEVER log plaintext value** (bug if ditemukan — refer audit policy `layanan/secreton/crates/core/src/audit/`).
- Query log: `secreton audit list --from=<ts>` atau `kubectl exec secreton-0 -- secreton audit list ...`.

### TLS / mTLS

- Production: client (consumer pod) WAJIB pakai gRPC mTLS (port 9000) atau HTTPS (port 8200 dengan cert dari Secreton PKI engine).
  Ini kini **ditegakkan, bukan dianjurkan**: `backendConfig.grpcAllowInsecure`
  = `"false"` di `values-production.yaml`, jadi pod secreton **menolak start**
  sampai PKI menerbitkan cert/key/CA-nya (prasyarat F6-A). Lihat
  "Client authentication is enforced" di bagian gRPC API.
- Staging: HTTP plain port 8200 boleh untuk debugging (tapi mtls.mode=PERMISSIVE harus tetap aktif via Istio sidecar).
  gRPC staging jalan lewat opt-out (`grpcAllowInsecure: "true"`) selama PKI
  belum ada — tercatat di log startup tiap boot, bukan diam-diam.
- Konsumen yang akan memegang cert: gateway sidecar
  (`layanan/gateway/src/lib.rs` — sekarang `SecretonServiceClient::new(ch)`
  tanpa kredensial) dan layanan-perlengkapan. Menyalakan mTLS tanpa
  menerbitkan cert untuk mereka akan mematahkan `e2e-simpelv1-integration`.

## 📋 Common Tasks (Operator Perspective)

### 1. Onboard a new consumer service

Misal service `layanan-arsip` baru perlu baca KV path `kv/arsip/*`:

1. **Helm values** — `infra/helm/simpel/values.yaml`:

   ```yaml
   secretonAuth:
     policies:
       layanan-arsip:
         - "kv/data/arsip/*"
         - "kv/data/postgres/arsip"
   ```

   `policies.yaml` ConfigMap template otomatis render policy ACL.

2. **Kubernetes auth role** — sama file, di
   `secretonAuth.kubernetesRoles` (cek `templates/secreton/auth-config.yaml`):

   ```yaml
   kubernetesRoles:
     layanan-arsip:
       boundServiceAccountNames: ["layanan-arsip"]
       boundServiceAccountNamespaces: ["simpelv2-{{ env }}"]
       policies: ["layanan-arsip"]
       tokenTTL: 3600
   ```

3. **Service workload** — `_workload.tpl` injects env var
   (`SECRETON_AUTH_ROLE=layanan-arsip`) saat secretonAuth enabled.
   Service code pakai `SecretonClient::connect()` lalu Kubernetes
   auth login flow.

4. **Seed secrets**:

   ```bash
   ./infra/helm/seed-secrets.sh staging arsip
   # atau manual:
   kubectl exec secreton-0 -- secreton kv put kv/arsip/api api_key=xxx
   ```

5. **Verify**: `kubectl logs deploy/layanan-arsip | grep -i "fetched secret"`.

### 2. Rotate a secret without downtime

Skenario: `kv/perlengkapan/notifikasi/smtp` perlu rotate password
karena bocor.

1. **Tulis nilai baru** ke Secreton — TIDAK menghapus yang lama
   dulu:

   ```bash
   kubectl exec secreton-0 -- secreton kv put kv/perlengkapan/notifikasi/smtp \
       username=current_user password=NEW_PASSWORD host=smtp.kejaksaan.go.id port=587
   ```

   Secreton KV-v2 menyimpan **versi**; consumer terakhir baca versi
   N+1 setelah cache TTL expired (default 60 detik).

2. **Force re-fetch** di consumer pods (kalau tidak mau tunggu cache):

   ```bash
   kubectl rollout restart deploy/layanan-perlengkapan -n simpelv2-prod
   ```

   Pod baru baca versi terbaru saat start; pod lama exit graceful.

3. **Update sistem eksternal** (mis. SMTP provider) supaya menerima
   password baru — bisa sebelum atau sesudah langkah 1 selama window
   transisi pendek.

4. **Audit verifikasi**:

   ```bash
   kubectl exec secreton-0 -- secreton kv metadata get kv/perlengkapan/notifikasi/smtp
   # → cek current_version naik, created_time match
   ```

5. **Cleanup**: setelah 24-48 jam tanpa regression, hapus versi lama:

   ```bash
   kubectl exec secreton-0 -- secreton kv metadata delete-versions \
       -versions=N kv/perlengkapan/notifikasi/smtp
   ```

   (N = versi lama yang mau dihapus). Versi tersedia untuk
   rollback selama tidak di-delete eksplisit.

### 3. Bootstrap di environment baru

Lihat `infra/AGENTS.md` → Common Tasks #1 untuk flow lengkap
microk8s + Helm + `bootstrap-secreton.sh`. Highlight Secreton-specific:

- **Shamir share**: init dengan `-shamir-shares=5 -shamir-threshold=3`.
  Simpan 5 share di **5 lokasi terpisah** (per personnel terpisah
  juga ideal). Threshold 3 = quorum untuk unseal.
- **Root token**: revoke setelah K8s auth backend di-setup dan
  policies sudah live. Root cuma untuk bootstrap, JANGAN dipakai
  untuk runtime.
- **Audit backend**: enable sejak hari pertama (`secreton audit enable file path=/secreton/logs/audit.log`)
  — log gak bisa di-replay retroaktif.
