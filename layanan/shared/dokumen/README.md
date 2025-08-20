# 📄 Layanan Dokumen - SIMPelv2

**Layanan Dokumen** adalah microservice dalam SIMPelv2 untuk manajemen dokumen digital, upload/download terenkripsi, preview, OCR, tagging, arsip, dan audit log. Dibangun dengan Rust (Axum), PostgreSQL, dan siap integrasi AI.

## 🚀 Fitur Utama
- Upload/download dokumen terenkripsi (AES-256)
- Preview dokumen (PDF/Word, file as-is)
- OCR (integrasi AI service)
- Klasifikasi/tagging dokumen (AI/manual)
- Manajemen arsip & koleksi
- Kontrol akses RBAC (Owner, Editor, Viewer)
- Audit log lengkap
- Virus scan (opsional)
- Observability: Prometheus, tracing, Sentry
- Health check endpoint

## 🏗️ Struktur Folder
```
layanan/dokumen/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── models.rs
│   ├── storage.rs
│   ├── security.rs
│   ├── audit.rs
│   ├── ocr.rs
│   ├── classify.rs
│   ├── archive.rs
│   └── handlers.rs
├── db/
│   └── schema.sql
├── Cargo.toml
├── .env.example
├── README.md
```

## ⚙️ Dependensi Utama
- Rust (Axum, SQLx, Tokio, Serde, Tracing, Prometheus, Sentry, Tower)
- PostgreSQL
- AI Service (OCR/tagging)
- Docker (opsional)

## 🔧 Environment Variable
Lihat `.env.example` untuk semua variabel yang didukung:
- DATABASE_URL, AI_SERVICE_URL, STORAGE_PATH, ENCRYPTION_KEY, CORS_ORIGINS, LOG_LEVEL, SENTRY_DSN, dsb.

## 📡 API Endpoint
### Dokumen
- `POST   /documents/upload` (multipart: file, metadata)
- `GET    /documents/:id` (metadata)
- `DELETE /documents/:id`
- `GET    /documents/:id/download`
- `GET    /documents/:id/preview`

### OCR
- `POST   /documents/:id/ocr`
- `GET    /documents/:id/ocr/status`
- `GET    /documents/:id/ocr/text`

### Tagging
- `POST   /documents/:id/classify`
- `GET    /documents/:id/tags`
- `PUT    /documents/:id/tags` (body: {"tags": ["tag1", ...]})

### Arsip
- `POST   /documents/:id/archive` (body: {"collection_id": "uuid"})
- `GET    /archive/collections`
- `GET    /archive/search?q=...&limit=...`

### Audit & Health
- `GET    /audit/logs?document_id=...&user_id=...&limit=...`
- `GET    /health`
- `GET    /health/storage`
- `GET    /health/ai`
- `GET    /metrics`

## 🔒 Keamanan
- File terenkripsi AES-256-GCM
- RBAC: Owner, Editor, Viewer (cek permission di setiap endpoint utama)
- Audit log otomatis setiap aksi penting
- Semua secret dari env
- CORS configurable
- Sentry error tracking (opsional)

## 📊 Observability
- Prometheus metrics (`/metrics`)
- Tracing (OpenTelemetry, Sentry)
- Health check endpoint

## 🧪 Testing
- Jalankan: `cargo test`
- Test upload, download, OCR, tagging, dsb.

## 🛠️ Build & Run
```bash
# Build
cargo build --release
# Jalankan migrasi DB
psql -d simpelv2 -f db/schema.sql
# Jalankan service
cargo run
```

## 🔗 Integrasi
- AI Service: OCR/tagging via HTTP
- Security Service: User/role (integrasi dengan layanan keamanan untuk user_id)
- Notifikasi, virus scan, cloud storage (opsional)

## 📑 Contoh Request
### Upload Dokumen
```bash
curl -F "file=@contoh.pdf" -F 'metadata={"judul":"Contoh"};type=application/json' http://localhost:3003/documents/upload
```
### Download Dokumen
```bash
curl -OJ http://localhost:3003/documents/<id>/download
```
### OCR
```bash
curl -X POST http://localhost:3003/documents/<id>/ocr
```
### Tagging
```bash
curl -X POST http://localhost:3003/documents/<id>/classify
```
### Arsip
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"collection_id":"<uuid>"}' http://localhost:3003/documents/<id>/archive
```

## 📝 Catatan
- Setiap endpoint utama sudah terhubung audit log.
- RBAC: Untuk produksi, pastikan user_id diambil dari JWT/auth header dan permission dicek di setiap handler.
- Preview dokumen saat ini return file as-is, bisa dikembangkan ke PDF/image preview.
- Untuk integrasi production, pastikan AI service dan security service sudah siap.

---
**Dibangun dengan ❤️ dan Rust untuk manajemen dokumen yang aman dan efisien** 