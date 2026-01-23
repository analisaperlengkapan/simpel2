# Secreton Storage & Database HA Implementation Plan

## 1. Latar Belakang & Kondisi Saat Ini

Komponen relevan di `infra/secreton`:

- **Storage Abstraction**
  - `crates/storage/src/lib.rs` — trait `StorageBackend` & `KvBackend`, error model `StorageError`, `HealthStatus`, `StorageStats`, dsb.
  - `crates/storage/src/factory.rs` — `StorageFactory`, `StorageBackendType` (`Memory`, `File`, `Consul`, `Raft`, `S3`, `Postgres`).
- **Backends Fisik**
  - File: `crates/storage/src/backends/file.rs`
    - Single-node, non-HA, permission & fsync sudah cukup baik untuk dev/small deployments.
  - Consul: `crates/storage/src/backends/consul.rs`
    - Storage HA via cluster Consul (leader election, KV, health checks), sudah ada health check & metrics.
    - `max_retries` ada di `ConsulConfig` tapi belum digunakan di path `get/put/delete/list`.
    - Default `scheme = "http"` (non-TLS), `tls_skip_verify` bisa diaktifkan.
  - S3: `crates/storage/src/backends/s3.rs`
    - Struktur config sudah siap (bucket, region, access key, SSE, versioning), tetapi **seluruh operasi `get/put/delete/list` masih TODO** dan mengembalikan error placeholder.
  - Postgres: `crates/storage/src/backends/postgres.rs`
    - Menggunakan `deadpool_postgres::Pool` dengan `NoTls`.
    - Dideskripsikan sebagai **opsional, tidak untuk HA** (HA diserahkan ke cluster Postgres eksternal).
- **Raft Cluster (HA internal)**
  - `crates/storage/src/raft/mod.rs` dan submodul (network, state_machine, metrics, dll.).
  - Tersedia `RaftCluster`, `RaftClusterConfig`, `RaftMetrics`, dan integrasi dengan `openraft`.
  - `StorageBackendType::Raft` di factory saat ini mengembalikan error konfigurasi dan menginstruksikan penggunaan `RaftCluster::new()` langsung.
- **Wiring ke API & gRPC**
  - `crates/api/src/services/mod.rs`
    - Fungsi pemilihan backend storage masih:
      - "raft" / "integrated" → warning, fallback ke `MemoryBackend`.
      - "memory" / "mock" → `MemoryBackend`.
      - "postgres", "redis", "file" → hanya warning, fallback ke `MemoryBackend`.
    - Akibatnya, **runtime default tidak memakai storage persisten/HA sama sekali**, hanya in-memory.
  - `crates/grpc/src/server.rs`
    - gRPC RAFT cluster management (`AddNode`, `RemoveNode`, `health_check`) mengharapkan `storage.as_any()` dapat di-downcast ke `secreton_storage::raft::RaftCluster` ketika fitur `raft-consensus` aktif.
- **Database Pool (non-engine DB)**
  - `crates/api/src/services/mod.rs::create_database_pool`
    - Menggunakan `deadpool_postgres::Config`, `ManagerConfig{ RecyclingMethod::Fast }`, dan `NoTls`.
    - Tidak ada konfigurasi eksplisit `max_size`, `connection_timeout`, `max_lifetime`, dsb (mengandalkan default).
- **Health Check**
  - `crates/core/src/services/health.rs` saat ini masih banyak yang bersifat simulasi (sleep), belum sepenuhnya terhubung ke DB nyata maupun RAFT cluster.

**Kesimpulan singkat:**

- Secara **arsitektur**, Secreton sudah memiliki fondasi untuk storage HA (Consul, RAFT, S3) dan observability (metrics, health status).
- Namun, di jalur **runtime utama**:
  - Storage backend HA/persisten **belum dipakai** (fallback ke memory).
  - S3 backend belum diimplementasikan.
  - RAFT backend belum disambungkan sebagai `StorageBackend` default.
  - Koneksi Postgres (baik engine-storage maupun API DB) masih menggunakan **NoTls** dan konfigurasi pool dasar.

Rencana berikut memfokuskan pada menjadikan storage & DB:

- **HA-ready** (bisa memanfaatkan cluster Consul/RAFT/DB eksternal), dan
- **aman untuk production** (TLS, secrets handling, health checks, observability).

---

## 2. Tujuan & Prinsip Desain

- **Tujuan utama**
  - Mengaktifkan **storage persisten dan HA** sebagai default untuk deployment production Secreton.
  - Memastikan koneksi ke storage dan database menggunakan **TLS** dan konfigurasi pool yang sehat.
  - Menyediakan **health check & metrics** yang merepresentasikan status cluster sebenarnya (storage, RAFT, DB).

- **Prinsip desain**
  - *Config-driven*: memilih backend melalui konfigurasi (`secreton.toml`, env) tanpa perubahan kode aplikasi.
  - *Secure by default*: profil production harus menggunakan TLS, retry, dan konfigurasi yang aman sebagai default.
  - *Graceful degradation*: fallback hanya digunakan secara eksplisit (misalnya mode `development`/`mock`).
  - *Observability first*: setiap backend wajib memiliki health check dan metrics yang terintegrasi ke gRPC/HTTP health endpoints.

---

## 3. Ruang Lingkup

- **Termasuk**
  - Storage backend untuk engine entries (`StorageBackend`/`KvBackend`): Consul, File, Postgres, RAFT, S3.
  - Connection pooling, TLS, retry, dan failover di tingkat storage & DB.
  - Health check & metrics untuk storage dan cluster RAFT.
- **Tidak termasuk (lintas proyek)**
  - Provisioning infrastruktur fisik (cluster Consul, cluster Postgres, bucket S3, K8s, dsb.) — diasumsikan disiapkan oleh tim infra.

---

## 4. Arsitektur Target (High-Level)

- **Secret Vault storage**
  - **Mode HA utama (on-premise)**: Consul backend atau RAFT cluster.
  - **Mode cloud-native**: S3 backend (setelah implementasi penuh) dengan SSE dan versi.
  - **Mode legacy**: Postgres backend yang bergantung pada cluster Postgres eksternal untuk HA.
- **API & gRPC layer**
  - Membaca konfigurasi storage dari `core::config::StorageConfig` / `secreton.toml`.
  - Menggunakan `StorageFactory::create` atau wiring RAFT cluster sebagai `StorageBackend`.
  - Health check gRPC/HTTP mengagregasi status storage, RAFT, dan DB pool.
- **Database non-engine (metadata, audit, dsb.)**
  - Menggunakan `deadpool_postgres` dengan TLS dan konfigurasi pool yang eksplisit.
  - HA dilakukan melalui endpoint/load balancer cluster Postgres, dengan tambahan retry di level aplikasi jika diperlukan.

---

## 5. Fase Implementasi

### Fase 1 — Enable Persistent Storage & TLS Basic

**Tujuan:** Menghilangkan fallback in-memory untuk mode production dan mengaktifkan storage persisten yang bisa dikonfigurasi.

**Langkah:**

1. **Wiring StorageFactory ke API service**
   - Lokasi: `crates/api/src/services/mod.rs`.
   - Ganti blok pemilihan backend storage:
     - Untuk `backend_type = "file"`, `"consul"`, `"postgres"` (dan ke depan `"raft"`, `"s3"`), gunakan konfigurasi dari `core::config::StorageConfig` / `secreton.toml`.
     - Panggil `StorageFactory::create(StorageFactoryConfig)` dengan mapping dari config global.
     - Hanya gunakan `MemoryBackend` untuk profil `development` atau jika `backend_type = "memory"/"mock"` secara eksplisit.

2. **TLS untuk Postgres (storage & DB API)**
   - Lokasi storage: `crates/storage/src/backends/postgres.rs`.
   - Lokasi DB API: `crates/api/src/services/mod.rs::create_database_pool`.
   - Tindakan:
     - Tambahkan dukungan TLS (misalnya menggunakan `tokio_postgres::tls::MakeTlsConnect` dengan `native-tls` atau `rustls` sesuai kebijakan proyek).
     - Konfigurasikan koneksi via URL (misal `DATABASE_URL` / `SECRETON_STORAGE_URL`) dengan `sslmode=require` atau set `TlsConnector` eksplisit.
     - Sediakan opsi konfigurasi untuk menspesifikasikan CA/client cert jika diperlukan (mTLS melalui sidecar atau langsung di DB).

3. **Konfigurasi Pool & Timeout**
   - Manfaatkan `PoolSettings` di `crates/storage/src/lib.rs` untuk storage backend yang memakai pool (Postgres, nanti mungkin Redis).
   - Untuk `create_database_pool`:
     - Atur `max_size`, `timeout`, `max_lifetime` secara eksplisit berdasarkan environment (dev/staging/prod).

4. **Profil konfigurasi**
   - Tambah contoh config di `docs/CONFIG_QUICK_REFERENCE.md` dan/atau `DEPLOYMENT.md` untuk:
     - `backend = "consul"` (HA) vs `"file"` (single-node) vs `"postgres"` (legacy).
     - DB URL dengan TLS.

### Fase 2 — Hardening Storage HA (Consul & RAFT)

**Tujuan:** Menjadikan mode Consul & RAFT benar-benar siap untuk HA production.

**Langkah:**

1. **Retry & Backoff di ConsulBackend**
   - Lokasi: `crates/storage/src/backends/consul.rs`.
   - Implementasikan pemakaian `max_retries` + backoff (misalnya exponential + jitter) untuk operasi `get/put/delete/list` yang gagal karena network/5xx.
   - Tambahkan klasifikasi error (timeout vs 4xx vs 5xx) untuk keputusan retry.

2. **Secure-by-default untuk Consul**
   - Ubah default `ConsulConfig` di production profile:
     - `scheme = "https"`.
     - `tls_skip_verify = false`.
   - Pada saat parse config untuk mode prod:
     - Jika `scheme = "http"` atau `tls_skip_verify = true`, log `error`/`warn` dan anggap misconfig (gagal start atau hanya diizinkan jika `environment = development`).

3. **Integrasi RAFT cluster sebagai StorageBackend (opsional, fitur `raft-consensus`)**
   - Lokasi: `crates/storage/src/raft/*`, `crates/storage/src/factory.rs`, `crates/grpc/src/server.rs`.
   - Tindakan:
     - Definisikan adaptor yang mengimplementasikan `StorageBackend` menggunakan `RaftCluster` + `SecretonRaftStorage` sebagai state machine.
     - Perbaiki `StorageBackendType::Raft` agar bisa menghasilkan backend yang memanfaatkan `RaftCluster`, atau definisikan jalur terpisah yang jelas di API ketika `backend_type = "raft"`.
     - Pastikan gRPC server (`Raft management RPCs`) dan storage RAFT menggunakan config yang sama (`RaftClusterConfig` / `secreton.toml`).

4. **Health & Metrics untuk RAFT**
   - Manfaatkan `RaftMetrics` dan `HealthStatus` di `crates/storage/src/raft/metrics.rs`.
   - Tambahkan endpoint atau extend gRPC `health_check` untuk memasukkan status:
     - Apakah node adalah leader/follower.
     - Apakah cluster punya quorum (`has_quorum()`).
     - Replication lag.

### Fase 3 — S3 Backend Productionization

**Tujuan:** Menghadirkan backend S3 yang benar-benar usable untuk deployment cloud-native.

**Langkah:**

1. **Integrasi `aws-sdk-s3` / klien S3 resmi**
   - Lokasi: `crates/storage/src/backends/s3.rs`.
   - Implementasikan:
     - `verify_bucket` menggunakan HEAD/`GetBucketLocation`.
     - `get/put/delete/list` menggunakan API resmi S3 (bisa mendukung juga endpoint MinIO via `endpoint` custom).
   - Gunakan SigV4 signing (library resmi akan mengurus ini).

2. **Enkripsi & keamanan**
   - Konfigurasi SSE-S3 (`sse_s3`) dan SSE-KMS (`sse_kms_key_id`).
   - Pastikan kredensial (`access_key`, `secret_key`) diambil dari mekanisme aman (env, IAM role, dsb.) dan tidak ditulis ke log.

3. **Retry & observability**
   - Tambahkan retry/backoff untuk error 5xx, throttling, atau network timeout.
   - Integrasikan metrics I/O (reads, writes, bytes) yang sudah ada dengan label backend `"s3"`.

4. **Dokumentasi & Contoh Konfigurasi**
   - Update `DEPLOYMENT.md` / `CONFIG_QUICK_REFERENCE.md` dengan contoh konfigurasi S3/MinIO.

### Fase 4 — Health Check Nyata untuk Storage & DB

**Tujuan:** Menggantikan health check simulatif dengan pengecekan nyata ke storage & DB, serta memaparkan status cluster.

**Langkah:**

1. **Perluas `HealthService`**
   - Lokasi: `crates/core/src/services/health.rs`.
   - Tambahkan implementasi `HealthCheck` yang:
     - Memanggil `StorageBackend::health_check()`.
     - Melakukan query ringan ke DB menggunakan pool Postgres.
     - Untuk RAFT, membaca `RaftMetrics` (`has_quorum`, `health`).

2. **gRPC & HTTP Health Endpoint**
   - Lokasi: `crates/grpc/src/server.rs` (fungsi `health_check`) dan endpoint HTTP terkait.
   - Sertakan:
     - `storage` dependency dengan `HealthStatus` yang diisi dari backend nyata.
     - `database` dependency dengan status ping DB.
     - Untuk mode RAFT: status cluster (leader, size, quorum).

3. **Integrasi dengan K8s / monitoring**
   - Update `infra/k8s/*` (di luar repo ini) untuk:
     - Memakai endpoint health yang baru sebagai liveness & readiness.
     - Menambahkan alert untuk status `Unhealthy` / `Degraded`.

### Fase 5 — Operasional & Runbook

**Tujuan:** Memastikan tim operasi memiliki panduan jelas untuk mengelola storage HA Secreton.

**Langkah:**

1. **Update Dokumentasi**
   - Tambah atau update dokumen di `infra/secreton/docs/`:
     - `cluster-management.md` (sudah ada) → tambahkan bagian khusus RAFT storage & pengelolaan node.
     - `DEPLOYMENT.md` → tambahkan skenario Consul/RAFT/S3/Postgres.

2. **Runbook Failover & Recovery**
   - Contoh prosedur:
     - Menambah/menghapus node RAFT via gRPC.
     - Menangani node Consul yang down (mengacu ke dokumentasi Consul, tetapi dengan catatan spesifik Secreton).
     - Recovery dari backup (jika ada mekanisme backup storage yang sudah diimplementasikan di modul lain).

---

## 6. Prioritas & Tahapan Rekomendasi

**Tahap 1 (wajib sebelum production):**

- Wiring `StorageFactory` ke API (hilangkan fallback ke memory untuk mode prod).
- TLS dan konfigurasi pool yang eksplisit untuk Postgres (DB & storage jika dipakai).
- Profil konfigurasi yang jelas per environment (dev/staging/prod).

**Tahap 2 (HA kuat di on-premise):**

- Hardening Consul backend (retry, TLS strict, health checks kuat).
- Integrasi RAFT cluster sebagai opsi `StorageBackend` dengan health & metrics.

**Tahap 3 (cloud-native):**

- Penyelesaian implementasi S3 backend dengan aws-sdk.
- Dokumentasi & contoh deployment untuk S3/MinIO.

**Tahap 4 (operasional & observability):**

- Health check nyata untuk storage & DB.
- Integrasi dengan probe K8s dan sistem monitoring/alerting.

Dokumen ini menjadi referensi utama untuk pekerjaan lanjutan terkait HA & keamanan storage/database di Secreton. Setiap fase dapat dipecah lagi menjadi task ticket yang lebih kecil sesuai kebutuhan tim.
