# 🤖 AGENTS.md - Layanan Perlengkapan

> **Service Context**: Backend service untuk manajemen Barang Milik Negara (BMN) Kejaksaan RI.

## Inherited Global Rules

- This file extends the global rules in `AGENTS.md`.
- Use this document only for **local deltas** specific to `layanan/perlengkapan`.
- If guidance here conflicts with root policy, root policy takes precedence unless an explicit local override is documented.

---

## 🌍 Service Context

Layanan Perlengkapan adalah backend service utama untuk sistem informasi perlengkapan Kejaksaan RI. Service ini menangani:

- Manajemen data BMN (Barang Milik Negara)
- Dokumentasi dan pelaporan perlengkapan
- Notifikasi terkait perlengkapan
- Sistem bantuan/ticket untuk perlengkapan

Service ini berkomunikasi dengan:

- **Authenc** (via gRPC) untuk autentikasi dan otorisasi
- **Secreton** (via gRPC) untuk manajemen secrets
- **Integrasi** (via gRPC) untuk integrasi dengan sistem eksternal (MySIMKARI, SIMAN)
- **Microfrontends** (via REST API) untuk antarmuka pengguna

---

## 🔑 Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| **Language** | Rust (Edition 2024) | 1.93+ |
| **HTTP Framework** | Axum | 0.8.x |
| **Database** | PostgreSQL | tokio-postgres + deadpool |
| **Cache** | Redis | - |
| **gRPC** | Tonic + Prost | 0.14.x |
| **Async Runtime** | Tokio | 1.x |
| **Validation** | validator | 0.18.x |
| **Metrics** | Prometheus | - |
| **Scheduler** | tokio-cron-scheduler | - |

---

## 🏗️ Architecture

### Crate Structure

Per refactor `refactor/perlengkapan-unified`: a single package owns every
domain. The previous `crates/{api,dokumen,notifikasi,bantuan}` split is
gone — modules now sit as siblings under `src/` and depend on each other
through ports & adapters (traits in `lib-perlengkapan::contracts`).

```
layanan/perlengkapan/
├── Cargo.toml              # unified package: layanan-perlengkapan
├── Dockerfile
├── build.rs                # compiles authenc/secreton/integrasi protos
├── migrations/             # V001..V0XX SQL, embedded via refinery
└── src/
    ├── main.rs             # bin entrypoint, wires AppState
    ├── lib.rs              # module declarations
    ├── state.rs            # AppState (carries Arc<dyn DocumentGenerator>
    │                       #            + Arc<dyn NotificationSender>)
    ├── routes.rs           # axum router composition
    ├── migrations.rs       # refinery::embed_migrations! runner
    │                       # (NB: no top-level handlers/models/services/
    │                       #  repository.rs — the old catch-all was
    │                       #  dissolved into feature modules, see below)
    │
    ├── shared/             # infra lintas-fitur: db, grpc, middleware,
    │                       #   cache, events, error, health, pagination, …
    ├── analisis/           # CRUD analisis_kebutuhan (eks catch-all)
    ├── export/             # generic xlsx export jobs (eks catch-all)
    ├── audit/              # audit-log query surface
    │
    ├── bank_aset/          # BMN catalog
    ├── kebutuhan_bmn/      # needs assessment + SIMAN integration
    ├── pakaian_dinas/      # uniform allocation
    ├── pemakaian_bmn/      # usage permits + scheduler
    ├── penghapusan_bmn/    # asset disposal
    ├── roadmap_sarpras/    # facilities roadmap
    ├── mapping_kodefikasi/ # asset coding
    ├── dashboard/          # KPI dashboard + WebSocket
    ├── admin/              # user/role admin (local to perlengkapan)
    ├── workflow/           # approval state machine + SLA monitor
    │   ├── engine/         # split: mod (types+ctors) + transition +
    │   │                   #   documents + notifications submodules
    │   ├── sla.rs
    │   ├── sla_scheduler.rs
    │   └── notification_types.rs  # WorkflowNotificationType +
    │                                # to_notification_message() adapter
    │
    ├── dokumen/            # templates + PDF/Excel generators + storage
    │   └── service.rs      # impl DocumentGenerator (real prod wiring)
    ├── notifikasi/         # email/SMS/whatsapp/push/in-app + queue
    │   └── service.rs      # impl NotificationSender (in-app real, others stub)
    └── bantuan/            # FAQ, ticketing, knowledge base, chatbot
        └── ticket.rs       # injects NotificationSender for tiket events
```

### Layout per modul fitur (konvensi F0-A)

Tiap modul fitur memakai `handlers` · `models` · `services` · `repository`
(+ `mod.rs`). **Bila satu berkas tumbuh besar (>~30KB / ratusan baris), pecah
menjadi direktori submodul kohesif** dengan `mod.rs` yang me-`pub use <grp>::*`
sehingga path publik lama (`crate::<modul>::models::*`, dll) tetap utuh:

- `kebutuhan_bmn/` — `handlers/` (per-handler-group + `params`), `models/`
  (`status`/`entities`/`requests`/`responses`), `services/`, `repository/`
  (`mod` = trait, `pg` = impl).
- `pemakaian_bmn/` — `models/`, `services/`, `repository/` (per-fitur).
- `pakaian_dinas/` — `models/`, `repository/`.
- `bantuan/` — `handlers/` (per-fitur; `mod.rs` menyimpan `routes()`).
- `workflow/engine/` — `transition`/`documents`/`notifications` (inherent
  impl boleh dipecah lintas-berkas selama submodul = descendant module;
  method privat yang dipanggil lintas-berkas → `pub(crate)`). **Trait impl
  TIDAK dipecah** (koherensi: satu blok — lihat `repository/pg.rs`).

Aturan saat memecah: submodul pakai path absolut `crate::<modul>::…` (bukan
`super::…`, sebab makna `super` berubah saat berkas turun satu level); `mod.rs`
mempertahankan re-export glob. Murni pemindahan kode — tanpa ubah perilaku.

### Why this shape

- **Ports & adapters**: cross-module calls go through trait contracts in
  `lib-perlengkapan::contracts` (`DocumentGenerator`, `NotificationSender`,
  `AuditSink`, `DocumentStorage`). The workflow engine no longer takes a
  concrete `DokumenClient` — it takes `Arc<dyn DocumentGenerator>`, so the
  module can be replaced or extracted to its own crate later without
  touching call sites.
- **No internal gRPC**: the old workflow → dokumen / workflow → notifikasi
  proto-over-Tonic clients are gone. Direct in-process trait dispatch
  removes the serialization hop and the second binary.
- **Migrations**: `migrations/V*.sql` (baseline `V001__baseline.sql` +
  `V002__seed.sql`) is the source of truth, applied by `refinery` on startup
  (`migrations::run`), then `add_essential_indexes`. The old hand-coded
  `Database::migrate` bootstrap was removed in F4 — refinery is the only path.

### Communication Flow

```mermaid
flowchart TB
    subgraph Frontend["🌐 Microfrontends"]
        Portal["Portal"]
        PerlengkapanUI["Perlengkapan UI"]
    end

    subgraph Perlengkapan["⚙️ Layanan Perlengkapan"]
        API["HTTP/Router (src/)"]
        DOC["dokumen module"]
        NOTIF["notifikasi module"]
        BANTUAN["bantuan module"]
    end

    subgraph Core["🔐 Core Services"]
        AUTH["Authenc"]
        SEC["Secreton"]
        INT["Integrasi"]
    end

    subgraph Data["💾 Data Layer"]
        PG[(PostgreSQL)]
        RD[(Redis)]
    end

    Portal -->|"REST API"| API
    PerlengkapanUI -->|"REST API"| API
    API --> DOC
    API --> NOTIF
    API --> BANTUAN
    API -->|"gRPC"| AUTH
    API -->|"gRPC"| SEC
    API -->|"gRPC"| INT
    API --> PG
    API --> RD
    DOC --> PG
    NOTIF --> RD
    BANTUAN --> PG

    style Frontend fill:#e1f5fe
    style Perlengkapan fill:#fff3e0
    style Core fill:#fce4ec
    style Data fill:#e8f5e9
```

---

## 📏 Critical Conventions

### 1. Dependency Management

- **ALL dependencies** MUST be defined in root `Cargo.toml` `[workspace.dependencies]`
- Member crates MUST use `dependency_name = { workspace = true }`
- **NEVER** specify versions in member `Cargo.toml` files

### 2. Configuration

- Use environment variables with `dotenvy` for local development
- Use Secreton gRPC for production secrets
- Config validation at startup with clear error messages
- Feature flags for optional functionality

### 3. Database Layer

- Use `deadpool-postgres` for connection pooling
- **Schema `perlengkapan` di shared `dbsimpelv2`** (bersama `authenc` + `integrasi`);
  baca data master **cross-schema** dari `integrasi.*` & `authenc.*` (JOIN/FK nyata,
  mis. `batch_operation_log.user_id → authenc.users`). JANGAN duplikasi data master —
  fetch-at-read/baca cross-schema (SSoT). Sub-domain `dokumen`/`notifikasi`/`cache`
  dibuat oleh migrasi perlengkapan. Lihat `layanan/AGENTS.md` → Database Architecture.
- **Scoping BMN per-satker (#43/#66):** `bank_aset` membaca `integrasi.siman_aset` (keyed
  `kdsatker_keu`). RBAC data-visibility ditegakkan **server-side** via
  `bank_aset::AsetScope` (`bank_aset/scope.rs`), diturunkan dari `Claims` di tiap handler dan
  diteruskan ke SEMUA query repository. Tier: cross-satker role (`is_cross_satker_role`) =
  `All` (tanpa filter); `validator_wilayah` = `Wilayah` (cocokkan `substring(kdsatker_keu,6,4)`);
  operator/`validator_satker` = `Satker` (cocokkan `kdsatker_keu`); tanpa satker identity =
  `Denied` (**fail-closed**, 0 baris). Pemetaan dari caller `kode_satker` (MySIMKARI) → SIMAN
  `kdsatker_keu`/`wilayah_kode` lewat `integrasi.v_satker_code_map` (mapping kanonik, pemilik =
  integrasi) — **bukan** name-match. Satker yg belum ter-mapping ⇒ fail-closed (butuh override
  `verified` di staging, P2/F5-E). Enrichment read internal (pemakaian/penghapusan auto-fill)
  sengaja pakai `AsetScope::All` — kepemilikan satker BMN dijaga oleh workflow, bukan lookup ini.
- Migrasi `refinery embed_migrations!` (self-migrate saat boot, `main.rs`); baseline
  `V001__baseline.sql` + `V002__seed.sql`. Boot perlengkapan **setelah** integrasi &
  authenc migrate selesai (urutan bring-up). Penamaan: `satker_id`/`satker_nama`/
  `nama_barang`/`nama_pegawai` (bukan `ms_satker_id`/`nm_satker`/`nama` generik).
- **Scoping list workflow (#66 part 2):** `izin_pemakaian_bmn` + `penghapusan_bmn` punya
  kolom `satker_code` (MySIMKARI, V003) yang **di-derive dari claims saat create** (bukan
  client `satker_id` UUID yg tak ber-FK). List endpoint memakai `shared::satker_scope::SatkerScope`
  (sibling `bank_aset::AsetScope`, tier sama: operator=satker sendiri, validator_wilayah=wilayah
  via `integrasi.v_satker_wilayah`, pusat/admin=semua, tanpa identitas=fail-closed).
  **JANGAN** memakai `integrasi.mysimkari_satker.wilayah` untuk tier ini: kolom itu berisi
  gugus pengawasan JAM `I`/`II`/`III` — 15 Kejati (238 satker) per nilai — bukan Kejati.
  Baris lama ber-`satker_code` NULL hanya terlihat role pusat/admin (default aman).
- **Identitas aset BMN = kode satker + kode barang + NUP.** NUP (nomor urut
  pendaftaran) adalah nomor urut DI DALAM satu satker untuk satu kode barang, jadi ia
  berulang di seluruh negeri: pada snapshot SIMAN nyata, 14.142 nilai NUP menampung
  624.533 aset, dan NUP `1` sendirian dipakai **44.017 aset di 553 satker**. Bahkan di
  dalam satu satker, 41,6% pasangan (satker, NUP) menunjuk lebih dari satu kode barang.
  Setiap query yang menjawab "aset yang mana" — cek tabrakan pemakaian, lookup SIMAN,
  verifikasi penghapusan, agregasi riwayat — **WAJIB ber-key pada ketiganya**. Ber-key
  NUP saja tidak menghasilkan jawaban yang salah sesekali; ia menyatukan puluhan ribu
  aset berbeda menjadi satu, dan efek sampingnya menyeberangi batas satker (nama
  pemegang dari satker lain muncul di pesan error). Lihat header
  `pemakaian_bmn/repository/lookup.rs` + `tests/integration/pemakaian_asset_identity_test.rs`.
- Use prepared statements for frequently executed queries
- Implement proper transaction handling

### 4. Error Handling

- Use `thiserror` for error enums
- Implement `From` conversions for common errors
- Return structured error responses with error codes
- Log errors with appropriate context using `tracing`

### 5. Authentication & Authorization

**Otentikasi = lapisan router deny-by-default.** `main.rs::build_router` memasang
`shared::middleware::require_authentication` di atas SELURUH `/api/v1/perlengkapan`:
tanpa bearer token yang valid → `401` sebelum handler (juga untuk path yang tak ada —
tak ada oracle 404-vs-401). Satu-satunya pengecualian: handshake WebSocket
`/dashboard/ws` (peramban tak bisa memasang header; handlernya memvalidasi token query
sendiri) dan probe kubelet `/health*` (di luar `api_routes`). Ekstraktor `Claims` membaca klaim yang
sudah divalidasi middleware (tanpa panggilan gRPC kedua) dan tetap mengotentikasi
sendiri bila dipasang tanpa middleware (tes).

> Otentikasi **bukan** otorisasi. Sebelum lapisan ini, "publik" = handler tanpa `Claims`,
> dan 18 handler tulis hanya memakai `Claims` untuk user id. Sekarang dua hal dijaga
> terpisah: middleware (siapa kamu) dan gerbang di handler (boleh apa).

- **Setiap handler menerima `Claims`** dan **setiap handler TULIS memuat gerbang**
  (`claims.require_*`, `Policy.authorize(_as)`, `enforce_*`). Handler BACA memakai scope
  (`SatkerScope`/`AsetScope::from_claims`) atau gerbang. Ini DIPAKSA CI:
  `infra/scripts/check-authz-guards.py` (matriks rute + `--self-test`). Allowlist di skrip
  itu hanya untuk endpoint swalayan/referensi dan **wajib beralasan**; entri basi = merah.
- **`Claims` membawa SEMUA role** (`claims.roles`, terurut, huruf kecil) + `claims.role` =
  role primer deterministik (`RoleSet::primary`, urutan `PRIMARY_ROLE_PRIORITY`), bukan
  "role pertama yang dicantumkan token". Guard mencocokkan **persis** ke semua role:
  `require_any_role`, `require_role`, `holds_role`, `require_capability(Capability::…)`.
  Role `system` (aktor internal engine) **tidak bisa** diklaim token.
- **TIDAK ADA bypass admin.** `admin`/`admin_pusat`/`superadmin` mengurus aplikasi
  (data master, template, audit, pemantauan, auto-expire: `Capability::Administer`/
  `ViewAudit`) — mereka **tidak** menyetujui, menolak, mencabut, atau menyelesaikan
  keputusan bisnis (segregation of duties). `WorkflowPolicy::authorize`, `require_any_role`
  dan `WorkflowEngine::validate_approver_role` tidak lagi punya jalan pintas admin.
  Endpoint yang memang boleh dilayani admin memakai `require_any_role_or_admin` /
  `require_capability`, bukan literal.
- **Peran aktor diteruskan ke engine = role yang MENGOTORISASI aksi**
  (`authorize_as` / `Claims::acting_role` / `role_for_transition`), bukan role primer —
  pengguna ber-banyak-role tidak boleh lolos policy sebagai operator lalu ditolak engine
  sebagai validator.
- **Maker-checker** (`shared::policy::enforce_maker_checker`): pengusul izin pemakaian tak
  boleh memvalidasi usulannya; approver bukan pengusul maupun validator.
- **Transisi generik Pemakaian** hanya untuk `Submit` & `Cancel`; forward/return/approve/
  revoke wajib lewat endpoint khususnya (kunci versi + stempel validator/approver).
- **Break-glass** = satu-satunya override admin atas alur bisnis:
  `POST /admin/break-glass/{modul}/{id}/transition` (khusus `ADMIN_ROLES`, alasan ≥ 20
  karakter, baris `perlengkapan.break_glass_log` ditulis **sebelum** aksi [V012], dijalankan
  sebagai aktor internal dengan user id admin, catatan diberi tag `[BREAK-GLASS]`).
  `GET /admin/break-glass` terbuka bagi admin **dan** validator_pusat (pihak yang keputusannya
  ditimpa). Break-glass bergerak sepanjang edge yang didefinisikan alur; ia tak menciptakan edge.
- **Delegasi** (`/workflow/delegations`) divalidasi (hanya role yang dipegang, bukan admin,
  maks 30 hari, alasan, penerima ada) tetapi **belum berlaku sebagai hak akses** — setiap
  respons memuat `berlaku: false`. Menjadikannya efektif butuh keputusan desain (token vs
  lookup per-request + jejak audit "bertindak atas nama").
- **Ekspor**: `ExportQuery.scope` (`#[serde(skip)]`, default `Denied`) diisi dari klaim;
  ekspor pakaian dinas (NIP+nama) & kebutuhan dibatasi satker pemanggil; roadmap/riwayat
  (kunci uuid legacy) hanya lintas-satker. Job ekspor milik pembuatnya (`created_by`).
- **Unggahan** (`shared::upload`): allowlist ekstensi (pdf/png/jpg/jpeg/docx/xlsx), magic bytes
  wajib cocok, `Content-Type` diturunkan server, ≤ 10 MiB & ≤ 20 file/permintaan. Tautan dokumen
  yang dikirim klien hanya `http(s)://` atau path situs (`validate_document_url`).
- **CORS deny-by-default** di luar dev (`CORS_ALLOWED_ORIGINS` tak diset ⇒ same-origin saja);
  `SKIP_AUTHENC=true` hanya bila `APP_ENV` ∈ dev/development/local/test.
- **IP klien** (`ClientIp`) dari `lib_backend::client_ip`: header `X-Forwarded-For`/`X-Real-IP`
  hanya dipercaya dari peer di `TRUSTED_PROXY_CIDRS` (dibaca dari kanan). Server harus jalan
  dgn `into_make_service_with_connect_info` (sudah).
- **Rate limiter** per-user berjalan SETELAH otentikasi (kunci = user id dari klaim).
- **Jangan ambil `satker_id` dari query string** untuk menentukan data siapa yang
  dibaca — itu melewati scoping. Turunkan dari token via `SatkerScope`/`AsetScope::from_claims`
  (fail-closed bila token tak punya satker). Objek yang dijangkau lewat id-nya sendiri
  (barang, job, analisis) wajib diperiksa terhadap scope pemiliknya (404 bila di luar scope).
- Daftar peran-literal (`"admin" | "superadmin"`, `== "admin"`) **dilarang** di luar
  `lib/core/src/authz.rs` (dipaksa `check-authz-guards.py`, aturan R5).
- Menutup gerbang di BE **hampir selalu menuntut perubahan FE**: pemanggil yang pakai
  `gloo_net::http::Request` mentah atau `window.open` tak membawa header
  `Authorization` dan akan 401. Pakai `api_get`/`auth_get_binary`; untuk unduhan
  bergerbang pakai pola fetch → blob → anchor sintetis.
- RBAC per-peran & lintas-satker ditegakkan **di server**, bukan dgn menyembunyikan
  tombol di FE.
- Env baru: `TRUSTED_PROXY_CIDRS`, `PENGHAPUSAN_WILAYAH_MAX_NILAI` (rupiah; plafon kewenangan
  Wilayah, kosong = tak ditegakkan — angka regulasi PMK 83/2016 dan perubahannya **harus
  dikonfirmasi** sebelum diisi).

---

## 🦀 Code Patterns

### Axum Handler Pattern

```rust
use axum::{
    extract::{Path, Query, State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use uuid::Uuid;

use crate::shared::middleware::Claims;

pub async fn get_perlengkapan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    // WAJIB. Ekstraktor inilah gerbang auth-nya — tak ada middleware auth.
    // Tulis `_claims: Claims` bila nilainya tak dipakai; menghapusnya
    // membuat endpoint ini publik & terjangkau internet.
    claims: Claims,
) -> Result<Json<PerlengkapanResponse>, AppError> {
    // Scoping diturunkan dari TOKEN, bukan dari query string.
    // `push_condition` mengembalikan predikat + mendaftarkan param `$n`-nya;
    // `None` = peran lintas-satker (tanpa batasan), dan `AsetScope::Denied`
    // (token tanpa identitas satker) menghasilkan `FALSE` ⇒ nol baris, jadi
    // jalur ini fail-closed dengan sendirinya.
    let scope = AsetScope::from_claims(&claims);
    let mut params: Vec<BoxedParam> = vec![Box::new(id)];
    let scope_clause = match scope.push_condition(&mut params) {
        Some(cond) => format!(" AND {cond}"),
        None => String::new(),
    };

    // Batas satker ikut masuk ke WHERE — biar DB yang menegakkannya,
    // bukan disaring belakangan di Rust.
    let sql = format!("SELECT * FROM perlengkapan WHERE id = $1{scope_clause}");
    let perlengkapan = state.db_pool
        .get()
        .await?
        .query_one(&sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>())
        .await?
        .try_into()?;

    Ok(Json(perlengkapan))
}
```

### gRPC Client Pattern

```rust
use tonic::transport::Channel;

pub async fn create_authenc_client() -> Result<AuthServiceClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static("https://authenc.internal:50051")
        .tls_config(tonic::transport::ClientTlsConfig::new())?
        .connect()
        .await?;
    Ok(AuthServiceClient::new(channel))
}
```

### Database Operation Pattern

```rust
use deadpool_postgres::Pool;

pub async fn create_perlengkapan(
    pool: &Pool,
    request: CreatePerlengkapanRequest,
) -> Result<Perlengkapan> {
    let client = pool.get().await?;
    let tx = client.transaction().await?;

    let row = tx.query_one(
        "INSERT INTO perlengkapan (kode_barang, nama, satker_id, created_by) 
         VALUES ($1, $2, $3, $4) 
         RETURNING *",
        &[&request.kode_barang, &request.nama, &request.satker_id, &request.created_by]
    ).await?;

    tx.commit().await?;
    Ok(row.try_into()?)
}
```

---

## 📋 Common Tasks

### 1. Add New API Endpoint

Perlengkapan is a *single-crate* service (flat `src/`, no `crates/api/` —
that pattern lives in authenc, not here). End-to-end:

1. **Route** — register in `src/routes.rs`. Example layout (see the
   `Workflow Delegation Routes` block circa line 510 for a fresh
   reference):

   ```rust
   .route(
       "/workflow/delegations",
       get(crate::workflow::delegation_handlers::list_delegations_handler)
           .post(crate::workflow::delegation_handlers::create_delegation_handler),
   )
   ```

2. **Handler** — drop the function in `<module>/handlers.rs`, or in the
   relevant `<module>/handlers/<group>.rs` when that module's handlers are
   split into a directory (e.g. `kebutuhan_bmn`, `bantuan`). The
   re-exporting `mod.rs` keeps the `crate::<module>::<fn>` path stable.
   Extractor ordering matters in axum: `State` first, then custom
   extractors (`Claims`), then `Path` / `Query`, with `Json(body)` last.

3. **Service / repository** — keep the handler thin; business rules and
   DB I/O go in `<module>/services(.rs|/…)` and
   `<module>/repository(.rs|/…)` — single file for small modules, a
   directory of cohesive submodules (re-exported via `mod.rs`) for large
   ones. The handler wires through `AppState`.

4. **Migration** — schema lives under `migrations/` as `V###__name.sql`
   and runs through `refinery` from `main.rs` (`migrations::run()`).
   Never `ALTER` an existing migration after merge — append a new file.

   **Semua DDL WAJIB di migrasi — DILARANG DDL saat runtime/boot.** Bukan
   gaya, tapi hasil dua insiden: `add_essential_indexes` (dihapus V007) dan
   `ensure_snapshot_table` (dipindah ke V008) sama-sama membuat skema jadi
   efek samping dari melayani request — tak terlihat di `migrations/`, tak
   bisa di-review, dan menambah round-trip DDL tiap panggilan.

   **Setiap relasi yang di-query WAJIB ada di migrasi.** Ditegakkan secara
   mekanis oleh `tests/integration/sql_relations_exist_test.rs`: test itu
   memindai raw string literal di `src/**` untuk tiap `<schema>.<relasi>`
   pada posisi FROM/JOIN/INTO/UPDATE, lalu meng-assert relasi tsb ada di DB
   yang baru dimigrasi. Alasannya `cargo check` buta terhadap isi string SQL
   — kelas bug "tabel hantu" ini sudah muncul tiga kali (#113, #118, #123)
   dan sebagian besar tersembunyi karena error-nya ditelan (`transition()`
   hanya `warn!`) atau kodenya tak pernah dipanggil. Kalau test ini merah,
   perbaiki query/tambah migrasi — **jangan** tambahkan pengecualian.

5. **Test** — unit tests next to the service; cross-module flows go in
   `tests/`. Backend lib tests must run serial (`--test-threads=1`)
   because they share a Docker Postgres.

### 2. Add Validation Rule

1. Add validation function in modul fitur terkait (mis. `src/<fitur>/models.rs`):

```rust
use validator::Validate;

#[derive(Validate)]
pub struct CreatePerlengkapanRequest {
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,
    
    #[validate(length(min = 1, max = 255))]
    pub nama: String,
}
```

2. Apply validation in handler:

```rust
request.validate().map_err(AppError::Validation)?;
```

### 3. Add a Workflow Definition or Delegation

The workflow engine lives in `src/workflow/`. Two ways to extend it:

- **New workflow definition (a new approval flow)** — add a `default_*`
  fn to `workflow/config.rs::WorkflowConfig` that returns
  `WorkflowConfig { states, transitions, sla_minutes, roles, ... }`,
  then wire `WorkflowEngine::for_<entity>` in `main.rs`. Once shipped,
  the admin UI at `/admin/workflow` displays it for inspection without
  any frontend change. Add a refinery migration only if the new
  workflow needs entity-specific columns (most don't — the engine
  tables are polymorphic).

- **Delegation (validator hands off to another user)** — the model is
  already wired (`workflow/delegation.rs` + `delegation_handlers.rs`
  routes at `/workflow/delegations`). The hot path is:
  `POST /workflow/delegations` → `DelegationManager::create_delegation`
  inserts into `perlengkapan.workflow_delegations`. Status transitions
  (`Scheduled` → `Active` → `Expired`) are driven by
  `DelegationManager::process_delegation_lifecycle()`; hook it into
  the SLA scheduler if you add a separate cron loop. Frontend lives at
  `antarmuka/perlengkapan/src/pages/workflow/delegation.rs`.

### 4. Add a Secreton-backed Service Credential

`main.rs` already creates a single `SecretonClient` and keeps it alive
for the duration of the process — reuse it instead of opening a new
gRPC channel per service. Worked example: SMTP credentials for
`notifikasi::email::EmailService`:

```rust
// 1. Construct service with `new_with_secreton`, passing the live
//    Secreton client and a KV path. The path resolves under
//    `kv/data/<path>` in Secreton's REST API.
let email = EmailService::new_with_secreton(
    notif_config,
    db.pool().clone(),
    secreton_client_ref,         // &SecretonClient (kept across boot)
    "perlengkapan/notifikasi/smtp", // bundle with `username` + `password` keys
).await?;

// 2. Production hardening: under `APP_ENV=production` the Err arm
//    must abort start-up (anyhow::bail!) instead of degrading silently.
```

The KV bundle must be readable by the SA role declared in
`infra/helm/simpel/values.yaml` under
`secretonAuth.policies.layanan-perlengkapan`. Add new globs there for
new bundles — current entries:

- `kv/data/postgres/perlengkapan`
- `kv/data/postgres/integrasi`
- `kv/data/perlengkapan/notifikasi/*`

### 5. Add gRPC Service Method

1. Update proto file in `proto/`
2. Regenerate code: `cargo build -p layanan-perlengkapan`
3. Implement service in `src/shared/grpc/`
4. Add to router

---

## ⚠️ Common Pitfalls

❌ **DON'T:**

- Call Authenc/Secreton directly from microfrontends
- Store secrets in environment variables in production
- Use raw SQL queries without prepared statements
- Forget to validate user input
- Ignore error context in error handling
- Use blocking I/O in async functions
- Hardcode database connection strings
- Skip transaction handling for multi-step operations

✅ **DO:**

- Use gRPC for inter-service communication
- Fetch secrets from Secreton via gRPC
- Use prepared statements with parameterized queries
- Validate all input with `validator` crate
- Include context in errors using `tracing`
- Use async/await throughout
- Use connection pooling with `deadpool`
- Wrap multi-step operations in transactions

---

## 🔍 Troubleshooting

### Database Connection Issues

- **Symptom**: "Connection refused" or timeout errors
- **Check**: PostgreSQL is running, connection string is correct
- **Solution**: Verify `DATABASE_URL` and network connectivity

### gRPC Connection Failures

- **Symptom**: "Failed to connect to authenc"
- **Check**: mTLS certificates are valid, service is running
- **Solution**: Verify certificate paths and service discovery

### Validation Errors

- **Symptom**: "Validation error" without details
- **Check**: Validation rules are properly defined
- **Solution**: Add custom error messages to validation attributes

### Performance Issues

- **Symptom**: Slow API responses
- **Check**: Database query performance, connection pool size
- **Solution**: Add indexes, optimize queries, increase pool size

---

## 📚 Key Files Reference

| File | Purpose |
|------|---------|
| `src/main.rs` | Bin entrypoint, wires `AppState` |
| `src/lib.rs` | Module declarations |
| `src/routes.rs` | Router composition (gabung `routes()` per fitur) |
| `src/state.rs` | `AppState` + `FromRef` |
| `src/<fitur>/handlers.rs` | HTTP request handlers (per fitur) |
| `src/<fitur>/models.rs` | Data models & DTOs (per fitur) |
| `src/shared/` | Infra lintas-fitur (db, grpc, cache, middleware, dll) |
| `build.rs` | Proto file compilation |
| `src/dokumen/` | Document management module |
| `src/notifikasi/` | Notification module |
| `src/bantuan/` | Help/ticket module |

---

## 🧪 Testing

### Unit Tests

```bash
cargo test -p layanan-perlengkapan
```

### Integration Tests

```bash
cargo test -p layanan-perlengkapan --features integration-tests
```

### Build

```bash
cargo build -p layanan-perlengkapan --release
```

---

## 🚀 Build & Run

### Development

```bash
cd layanan/perlengkapan
cargo run
```

### Production Build

```bash
cargo build -p layanan-perlengkapan --release
```

### Docker Build

```bash
docker build -t layanan-perlengkapan:latest .
```

---

## 🔗 End-to-End Integration Map

### Frontend → Backend Flow

```
antarmuka/perlengkapan (WASM)
  → reads JWT from localStorage key `auth_token`
  → calls REST API at /api/v1/perlengkapan/*
  → layanan-perlengkapan (Axum)
    → validates JWT via gRPC → authenc-grpc
    → fetches secrets via gRPC → secreton-grpc
    → queries PostgreSQL via deadpool-postgres
    → returns JSON response
```

> **Canonical API prefix (WAJIB): `/api/v1/perlengkapan/*`.** The Axum router
> mounts `api_routes` at `/api/v1/perlengkapan` (`src/main.rs`), the FE client
> sets `API_BASE = "/api/v1/perlengkapan"` (`antarmuka/perlengkapan/src/api/client.rs`),
> and Istio routes `prefix: /api/v1/perlengkapan` → `layanan-perlengkapan:3020`
> **without rewrite** in all three `infra/helm/simpel/values*.yaml`. This sits in
> authenc's `/api/v1/*` namespace alongside `/api/v1/{auth,iam,oauth2}` (authenc),
> partitioning portal vs perlengkapan-v1 (Laravel, `/perlengkapan/simpel/v1`) vs
> perlengkapan-v2 (this service). The legacy `/api/pembinaan/perlengkapan` +
> rewrite-from-`/api/perlengkapan/` prefixes are **removed** (pre-prod, no
> back-compat). Any new endpoint is reached at `/api/v1/perlengkapan/<route>`;
> FE calls must go through the `api/client.rs` helpers (which prepend `API_BASE`)
> or use the full canonical path — never a bare `/api/v1/<domain>`.

### Cross-Microfrontend Auth

Portal and Perlengkapan share the same `auth_token` localStorage key.
When Portal logs out, it:

1. Calls `POST /api/v1/auth/logout` to invalidate server session
2. Clears `auth_token`, `refresh_token` from localStorage
3. Fires `logout_event` storage event for cross-tab sync
4. Does a full page reload to clear WASM memory

Perlengkapan listens for `auth_token` and `logout_event` storage events
to sync session state across tabs.

See root `AGENTS.md` → "Canonical localStorage Keys" for the full key table.

### Known Integration Gaps

Snapshot as of the May 2026 stabilization sweep (Phase 1–2 of the
`tolong-bantu-saya-saya-tender-flamingo` plan).

| Gap | Status | Notes |
|-----|--------|-------|
| Cross-MFE token refresh (Portal ↔ Perlengkapan) | ✅ Wired | `features::session_monitor::spawn_refresh_loop` in Perlengkapan refreshes ahead of expiry; Portal listener now reacts to `auth_token` storage events too. ADR `docs/adr/0003-cross-mfe-token-sync.md`. |
| Email notification via SMTP | ✅ Wired | `EmailService::new_with_secreton` reads `perlengkapan/notifikasi/smtp` bundle; `NotifikasiService::with_email()` dispatches the Email channel. Helm policy adds `kv/data/perlengkapan/notifikasi/*`. |
| MFA TOTP REST endpoints | ✅ Wired | `services::LocalMfaApi` adapter in authenc-api drives `setup`/`verify`/`disable` against Postgres-backed stores (migration `049_totp_backup_codes.sql`). TOTP secrets stored as base32 plaintext — wrap with `authenc-crypto` envelope encryption in a follow-up. |
| Hardcoded `pegawai_id="0"` on `/pakaian-dinas/ukuran` | ✅ Removed | Route now reads `UserSession` from context. `fetch_pegawai_ukuran` no longer appends a path-param; backend resolves pegawai from JWT NIP claim. |
| Portal admin: Roles CRUD | ✅ Wired | Create + Delete modals in `antarmuka/portal/src/pages/admin/roles.rs`. |
| Portal admin: Realm General settings | ✅ Wired | Editable form bound to `iam_update_realm`. Other 8 tabs carry a "Pratinjau — belum tersambung" banner pending backend endpoints. |
| Workflow delegation (`/workflow/delegations`) | ✅ Wired | Backend handlers + REST routes added; `delegation.rs` table refs corrected to V007 schema (`workflow_delegations`). Frontend at `/admin/workflow-delegation`. |
| SMS / WhatsApp / Push channels | 🟡 Deferred | `NotifikasiService` still logs-and-skips these; SMTP-style adapter scaffolding can be ported once provider creds are minted into Secreton. |
| Authorization Services (Keycloak parity: resource servers, scopes, policies, evaluate) | 🔴 Pending epic | `antarmuka/portal/src/pages/admin/permissions.rs` shows a labelled "Pratinjau" UI; no backend endpoints yet. |
| Realm Settings tabs other than General | 🔴 Pending epic | Login/Email/Themes/Keys/Sessions/Tokens/Security/Localization need authenc settings endpoints. |
| TOTP secret encryption-at-rest | 🟡 Follow-up | Tracked in migration 049 comment; wrap with envelope encryption from `authenc-crypto`. |
| Pengadaan tender, BMN Wasdal | 🟡 Stays on v1 | Per the plan, v1 PHP keeps these — bug fixes in `monolith/simpelv1/`, no v2 rebuild scheduled. |

---

## 🔐 Secret Fetching (Zero-Trust)

Saat `secretonAuth.enabled=true` di Helm values:

- Pod `layanan-perlengkapan` punya projected SA token di `/var/run/secrets/tokens/secreton-token` (audience `secreton`).
- Pakai `secreton-agent` Kubernetes auth backend (lihat `layanan/secreton/crates/agent/src/auth/kubernetes.rs`).
- **Path policy** (`secretonAuth.policies.layanan-perlengkapan` di `infra/helm/simpel/values.yaml`):
  - `kv/data/postgres/perlengkapan` — DATABASE_URL untuk DB perlengkapan.
  - `kv/data/postgres/integrasi` — read-only ke DB integrasi (untuk join data BMN ↔ sync).
- **Env auto-injected** oleh `_workload.tpl`: `SECRETON_ADDR`, `SECRETON_AUTH_METHOD=kubernetes`, `SECRETON_AUTH_ROLE=layanan-perlengkapan`, `SECRETON_K8S_TOKEN_PATH=/var/run/secrets/tokens/secreton-token`.
- **DILARANG** pakai `SECRETON_TOKEN` env di production. Token statis hanya untuk dev lokal (docker-compose).

---

> **Catatan Akhir**: Layanan Perlengkapan adalah service kritis untuk operasional BMN Kejaksaan RI. Pastikan semua perubahan melalui code review dan testing yang menyeluruh sebelum deployment ke production.
