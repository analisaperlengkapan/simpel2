# 🤖 AGENTS.md — `layanan/gateway` (REST→gRPC sidecar)

> **Peran:** sidecar **penerjemah** untuk **simpelv1** (Laravel/php-fpm). Ia
> memegang channel gRPC **long-lived** ke authenc/secreton/integrasi dan
> mengekspos REST kecil di **localhost** yang dikonsumsi PHP client
> (`monolith/simpelv1/app/Services/Grpc/*GrpcClient.php`). Lihat
> `monolith/simpelv1/AGENTS.md` untuk rasionalnya (php-fpm tak boleh buka mTLS
> gRPC per request).

## Kenapa service ini ada (bukan Envoy/Istio transcoder)

Istio/Envoy menangani **mTLS + routing** (transport). Gateway ini menangani
**lapisan aplikasi** yang TIDAK bisa dilakukan `grpc_json_transcoder`:
1. **Reshape/rename field** — `ValidateToken{valid,user_id,realm_roles}` →
   `{claims:{sub,…}}`; `GetSecret{data}` → `{value}`.
2. **Sintesis HTTP status dari boolean** — `valid:false` → **401** (fail-closed);
   error transport → **5xx** (fail-open, agar 1 blip tak mass-logout). Middleware
   `EnforceTokenRevocation` simpelv1 bergantung persis pada semantik ini.
3. **Lookup tanpa RPC by-id** — `/v1/siman/inventory/{id}` = list+filter.
4. **Endpoint dormant** — MonSAKTI → 501 (migrasi ke MyIntress).

Transcoder = passthrough 1:1 + butuh anotasi `google.api.http` di proto (tak
ada) + tetap tak cover #1–#4. Gateway = ~komponen kecil, **0 perubahan PHP**.

## Kontrak REST (DITURUNKAN dari PHP client, JANGAN ubah tanpa ubah PHP-nya)

| REST | upstream gRPC |
|---|---|
| `POST /v1/tokens/verify {token}` | authenc `ValidateToken` → `{claims}` / 401 |
| `GET /v1/users/{id}` | authenc `GetUser` |
| `GET/PUT/DELETE /v1/secrets/{*path}` | secreton `GetSecret/Store/Delete` |
| `GET /v1/database-credentials/{db}` | secreton `GenerateDatabaseCredentials` |
| `GET /v1/api-keys/{name}` | secreton `GetSecret` (`api-keys/…`) |
| `GET /v1/mysimkari/employees/{nip}` | integrasi `GetMysimkariPegawai` |
| `GET /v1/siman/inventory/{id}` | integrasi `GetSimanAssets` (+filter) |
| `GET /v1/monsakti/*` | **501** (dormant) |
| `GET /healthz` | liveness |
| `POST /v1/database/config/{name}` † | secreton `ConfigureDatabaseConnection` |
| `POST /v1/database/roles/{role}` † | secreton `CreateDatabaseRole` |

† **Privileged, env-gated.** These provisioning routes are mounted **only** when
the gateway runs with `GATEWAY_ENABLE_DB_ADMIN=1` (`AppState::enable_db_admin`) —
OFF in the base compose and in production, ON only for the e2e secreton bootstrap.
They accept an admin DSN + raw SQL, so they must not exist on the production
simpelv1 localhost sidecar; when disabled the routes are absent from the router
(404), not merely 403'd. See `secreton-ops` SKILL for the dynamic-DB flow.

## Struktur

- `src/lib.rs` — proto includes (reuse `layanan/{authenc,secreton,integrasi}/proto`
  via `build.rs`, sama dgn perlengkapan), `GatewayConfig`, `AppState`
  (channel `connect_lazy` — boot walau upstream sempat down), `build_router`.
- `src/handlers.rs` — handler + **helper mapping murni** (`map_validate`,
  `map_get_secret`, …) yang **bebas I/O** → diuji unit tanpa server gRPC. Round-trip
  nyata diuji job CI `e2e-simpelv1-integration`.
- `src/main.rs` — entrypoint axum.

## Aturan AI

- **Kontrak REST = cermin PHP client.** Tambah/ubah route HANYA berbarengan ubah
  `*GrpcClient.php` + tesnya. Field-mapping baru → tambah helper murni + unit test.
- **Fail-closed vs fail-open:** reject definitif (`valid:false`, 4xx upstream) →
  4xx; transport/`Unavailable` → 5xx. Jangan dibalik (lihat `upstream_status`).
- **Jangan log token/secret.** Hanya log gRPC code + service.
- gRPC ke upstream = **plaintext** (mTLS oleh sidecar Istio); gateway tak terminasi TLS.
- Port tunggal `GATEWAY_PORT` (default 8090); ketiga `*_GATEWAY_URL` simpelv1
  menunjuk ke port ini.
- Test: `cargo test -p layanan-gateway` (unit mapping) wajib hijau.
