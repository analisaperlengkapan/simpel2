# 🆘 Layanan Bantuan - SIMPEL

**Layanan Bantuan** adalah microservice dalam SIMPEL untuk sistem bantuan, FAQ, ticket, chatbot AI, knowledge base, analytics, GDPR, webhook, export/import, RBAC granular, rate limit, captcha, dan audit log. Dibangun dengan Rust (Axum), PostgreSQL, Redis, dan siap integrasi AI.

## 🚀 Fitur Utama
- FAQ Management (CRUD, search, versioning)
- Ticket System (CRUD, komentar, status, anti-spam)
- AI Chatbot (integrasi AI, feedback loop)
- Knowledge Base (CRUD, search, related, export/import)
- Analytics (statistik, satisfaction)
- Audit log (query/filter)
- RBAC granular (per endpoint/resource)
- Rate limiting per user/IP
- CAPTCHA (anti-spam endpoint publik)
- Webhook (integrasi eksternal)
- GDPR endpoint (hapus/unduh data user)
- Observability: Prometheus, tracing, Sentry
- Health check endpoint

## 🏗️ Struktur Folder
```
layanan/bantuan/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── models.rs
│   ├── faq.rs
│   ├── ticket.rs
│   ├── chatbot.rs
│   ├── knowledge.rs
│   ├── analytics.rs
│   ├── audit.rs
│   ├── webhook.rs
│   ├── export_import.rs
│   ├── gdpr.rs
│   ├── rbac.rs
│   ├── rate_limit.rs
│   ├── captcha.rs
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
- Redis (rate limit, captcha)
- AI Service (chatbot)
- Docker (opsional)

## 🔧 Environment Variable
Lihat `.env.example` untuk semua variabel yang didukung:
- DATABASE_URL, AI_SERVICE_URL, API_KEY, CORS_ORIGINS, SEARCH_INDEX_PATH, LOG_LEVEL, SENTRY_DSN, RATE_LIMIT_*, CAPTCHA_SECRET, REDIS_URL

## 📡 API Endpoint (Utama)
### FAQ
- `GET/POST   /faq/categories`
- `GET/PUT/DELETE /faq/categories/:id`
- `GET    /faq/categories/:id/articles`
- `GET/POST   /faq/articles`
- `GET/PUT/DELETE /faq/articles/:id`
- `GET    /faq/search?q=...`

### Ticket
- `GET/POST   /tickets`
- `GET/PUT/DELETE /tickets/:id`
- `GET/POST   /tickets/:id/comments`
- `PUT    /tickets/:id/status`
- `GET    /tickets/search?q=...`

### Chatbot
- `POST   /chatbot/query`
- `GET    /chatbot/history?conversation_id=...`
- `POST   /chatbot/feedback`
- `GET    /chatbot/suggestions?user_id=...`

### Knowledge Base
- `GET/POST   /knowledge/articles`
- `GET/PUT/DELETE /knowledge/articles/:id`
- `GET    /knowledge/search?q=...`
- `GET    /knowledge/related/:id?limit=...`
- `GET    /knowledge/export`
- `POST   /knowledge/import`

### Analytics
- `GET    /analytics/tickets?start=...&end=...`
- `GET    /analytics/faq?category_id=...&period=...`
- `GET    /analytics/chatbot?user_id=...&period=...`
- `GET    /analytics/satisfaction?start=...&end=...`

### Audit & Admin
- `GET    /audit/logs?user_id=...&action=...&resource=...&limit=...`
- `GET/POST   /webhook/events`
- `POST   /webhook/events/:id/deliver`
- `POST   /webhook/events/:id/retry`
- `GET    /export/:resource`
- `POST   /import/:resource`
- `POST   /gdpr/request_delete`
- `POST   /gdpr/request_download`
- `GET    /gdpr/status?user_id=...`
- `POST   /gdpr/process/:id`
- `GET    /health`
- `GET    /metrics`

## 🔒 Keamanan
- RBAC granular: Role & permission per endpoint/resource
- API key: Semua endpoint dilindungi header `x-api-key`
- Rate limiting: Per user/IP, configurable
- CAPTCHA: Verifikasi Google reCAPTCHA untuk endpoint publik
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
- Test FAQ, ticket, chatbot, knowledge, dsb.

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
- AI Service: Chatbot via HTTP
- Security Service: User/role (RBAC, audit)
- Notifikasi, webhook, export/import, dsb

## 📑 Contoh Request
### Buat FAQ
```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"name":"General","description":"Info umum"}' \
  http://localhost:3006/faq/categories
```
### Buat Tiket
```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"user_id":"<uuid>","subject":"Butuh bantuan","description":"..."}' \
  http://localhost:3006/tickets
```
### Query Chatbot
```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"user_id":"<uuid>","message":"Halo"}' \
  http://localhost:3006/chatbot/query
```
### Export Knowledge
```bash
curl -H 'x-api-key: <API_KEY>' http://localhost:3006/knowledge/export
```
### GDPR Request Delete
```bash
curl -X POST -H 'x-api-key: <API_KEY>' -H 'Content-Type: application/json' \
  -d '{"user_id":"<uuid>","details":{}}' \
  http://localhost:3006/gdpr/request_delete
```

## 📝 Catatan
- Setiap endpoint utama sudah terhubung audit log.
- RBAC, rate limit, captcha, dan API key wajib diaktifkan di produksi.
- Untuk integrasi production, pastikan semua service eksternal (AI, Redis, dsb) sudah siap.
- Webhook, export/import, GDPR, dan audit log siap untuk kebutuhan compliance dan integrasi enterprise.

---
**Dibangun dengan ❤️ dan Rust untuk sistem bantuan yang cerdas, aman, dan scalable** 