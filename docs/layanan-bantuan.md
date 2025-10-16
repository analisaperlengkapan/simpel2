# Layanan Bantuan – SIMPelv2

Layanan **Bantuan** menyediakan pusat pembelajaran, tanya jawab, dan sistem dukungan pengguna (helpdesk) dalam sistem SIMPelv2. Layanan ini bertujuan untuk meningkatkan pengalaman pengguna dan mempercepat penyelesaian masalah melalui bantuan interaktif dan terintegrasi AI.

---

## 🎯 Tujuan

Mendukung pengguna SIMPelv2 dengan dokumentasi, FAQ, video tutorial, serta kanal pengajuan tiket bantuan yang dikelola secara terstruktur, cepat, dan informatif.

---

## 🧱 Fitur Utama

- ❓ Modul FAQ dan dokumentasi teknis
- 🎥 Video tutorial (dari `layanan-dokumen`)
- 💬 Chatbot AI untuk tanya jawab BMN
- 🎫 Tiket bantuan: buat, lacak, dan respon
- 🔁 Feedback otomatis ke tim layanan terkait
- 🧠 Knowledge Base dengan pencarian semantik (AI RAG)

---

## ⚙️ Teknologi

- Backend: Go (Gin) + sqlc
- Database: PostgreSQL (skema `bantuan`)
- Frontend: Komponen Antarmuka SIMPelv2 (FAQ dan tiket)
- AI Q&A: LLaMA3-3B + Qdrant (Retrieval-Augmented Generation)
- Notifikasi: Email via `authenc`, webhook Telegram

---

## 📁 Struktur Folder

```
layanan-bantuan/
├── api/                  # Endpoint bantuan & tiket
├── model/                # FAQ, tiket, respon
├── repository/           # Query SQL
├── service/              # Logika pengelolaan tiket & FAQ
├── ai/                   # Integrasi AI Q&A (RAG)
├── handler/              # Routing & middleware
├── job/                  # Notifikasi, feedback, sinkronisasi
├── main.go
└── go.mod / go.sum
```

---

## 🔄 Contoh Endpoint

| Metode | Endpoint                      | Keterangan                                |
|--------|-------------------------------|-------------------------------------------|
| GET    | `/faq`                        | Ambil daftar pertanyaan umum              |
| GET    | `/faq/:id`                    | Detail FAQ tertentu                       |
| POST   | `/tiket`                      | Buat tiket bantuan baru                   |
| GET    | `/tiket/:id`                  | Lihat detail tiket dan status             |
| POST   | `/faq/chat`                   | Tanya jawab interaktif berbasis AI        |
| GET    | `/feedback`                   | Feedback pengguna untuk evaluasi layanan  |

---

## 🤖 Kontribusi AI

| Pendekatan AI           | Implementasi di Layanan Bantuan                                 |
|-------------------------|-----------------------------------------------------------------|
| Retrieval-Augmented Gen | Q&A berbasis dokumen FAQ menggunakan LLaMA3 + Qdrant           |
| Supervised Learning     | Klasifikasi kepuasan dari feedback pengguna                    |
| Summarization           | Ringkasan otomatis dari tanggapan tiket                        |
| Clustering              | Pengelompokan tiket berdasarkan topik dan frekuensi masalah     |
| Feedback Analyzer       | Sentimen dan topik analisis dari teks pengguna                 |

---

## 🔐 Akses & Validasi

- Semua pengguna dapat membuat tiket (perlu login)
- Admin layanan atau PIC hanya dapat merespon tiket
- Semua aksi dicatat oleh `layanan-audit`
- Validasi input wajib melalui middleware `authenc`

---

## 📌 Integrasi Terkait

- `layanan-dokumen`: untuk dokumen/video pembelajaran
- `layanan-ai`: menyuplai Q&A model dan retriever
- `layanan-konfigurasi`: untuk kategori tiket, FAQ, prioritas
- `authenc`: validasi user & pengiriman notifikasi

---

## 📦 .env Konfigurasi

```env
AI_QA_MODEL=llama3-3b
QDRANT_URL=http://qdrant:6333
FAQ_INDEX_PATH=data/faq_embeddings.json
FEEDBACK_WEBHOOK_URL=https://telegram.me/simpelv2_bot
```

---

## 📚 Tips Pengembangan

- FAQ dikelola sebagai file markdown dan diindeks secara otomatis
- Dataset FAQ ditraining ulang secara berkala berdasarkan tiket masuk
- Tambahkan tag-topik untuk FAQ dan tiket untuk mempermudah klasifikasi
- Gunakan job periodik untuk re-clustering topik bantuan
- Pastikan antarmuka pengguna intuitif dan mudah digunakan

---

## 📝 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia – Internal Use Only.

Layanan ini merupakan bagian dari sistem internal SIMPelv2 dan tidak untuk disebarluaskan atau digunakan di luar organisasi tanpa izin resmi tertulis.
