# 🔔 Layanan Notifikasi - SIMPEL

**Layanan Notifikasi** adalah microservice dalam SIMPEL untuk pengiriman notifikasi multi-channel (email, WhatsApp, push), template dinamis, audit log, rate limit, dan observability. Dibangun dengan Rust (Axum), PostgreSQL, Redis, dan siap integrasi cloud.

## 🚀 Fitur Utama

- Multi-channel: Email (SMTP), WhatsApp (Business API), Push (FCM)
- Batch & single send
- Template dinamis & versioning
- Delivery tracking & analytics
- Rate limiting per channel
- Queue (Redis)
- API key & audit log
- Consent & opt-out
- Observability: Prometheus, tracing
- Health check endpoint

## 🏗️ Struktur Folder

```
layanan/notifikasi/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── models.rs
│   ├── email.rs
│   ├── whatsapp.rs
│   ├── push.rs
│   ├── template.rs
│   ├── audit.rs
│   ├── queue.rs
│   ├── security.rs
│   └── handlers.rs
├── db/
│   └── schema.sql
├── Cargo.toml
├── .env.example
├── README.md
```

## ⚙️ Dependensi Utama

- Rust (Axum, SQLx, Tokio, Serde, Tracing, Prometheus, Tower)
- PostgreSQL
- Redis (queue)
- SMTP, WhatsApp API, FCM
- Docker (opsional)

## 🔧 Environment Variable

Lihat `.env.example` untuk semua variabel yang didukung:

- DATABASE_URL, REDIS_URL, SMTP_*, WHATSAPP_*, FCM_*, API_KEY, CORS_ORIGINS, RATE_LIMIT_*

## 📡 API Endpoint

### Email

- `POST   /notifications/email/send`
- `POST   /notifications/email/batch`
- `GET    /notifications/email/status/:id`

### WhatsApp

- `POST   /notifications/whatsapp/send`
- `POST   /notifications/whatsapp/batch`
- `GET    /notifications/whatsapp/status/:id`

### Push

- `POST   /notifications/push/send`
- `POST   /notifications/push/batch`
- `GET    /notifications/push/status/:id`

### Template

- `GET    /templates`
- `POST   /templates`
- `PUT    /templates/:id`
- `DELETE /templates/:id`

### Audit & Health

- `GET    /audit/logs?notification_id=...&user_id=...&limit=...`
- `GET    /health`
- `GET    /metrics`

## 🔒 Keamanan

- API key: Semua endpoint dilindungi header `x-api-key`
- Rate limiting: Per channel, per IP, configurable
- Audit log otomatis setiap aksi penting
- Semua secret dari env
- CORS configurable

## 📊 Observability

- Prometheus metrics (`/metrics`)
- Tracing (OpenTelemetry)
- Health check endpoint

## 🧪 Testing

- Jalankan: `cargo test`
- Test email, whatsapp, push, template, dsb.

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

- Email: SMTP (Gmail, SendGrid, dsb)
- WhatsApp: Meta WhatsApp Business API
- Push: FCM (Firebase Cloud Messaging)
- Security Service: User/role (untuk audit, consent)

## 📑 Contoh Request

### Kirim Email

```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"recipient":"user@email.com","subject":"Test","body":"Hello"}' \
  http://localhost:3004/notifications/email/send
```

### Kirim WhatsApp

```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"phone_number":"+628123456789","template_name":"otp","variables":[{"type":"text","text":"123456"}]}' \
  http://localhost:3004/notifications/whatsapp/send
```

### Kirim Push

```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"device_token":"...","title":"Hi","body":"Pesan","data":{}}' \
  http://localhost:3004/notifications/push/send
```

## 📝 Catatan

- Setiap endpoint utama sudah terhubung audit log.
- Rate limit per channel dan API key wajib diaktifkan di produksi.
- Untuk integrasi production, pastikan semua service eksternal (SMTP, WhatsApp, FCM, Redis, dsb) sudah siap.

---
**Dibangun dengan ❤️ dan Rust untuk notifikasi yang andal dan scalable**
