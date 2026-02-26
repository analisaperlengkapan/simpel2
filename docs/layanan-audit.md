# layanan-audit

**Layanan Audit SIMPEL** bertanggung jawab atas pencatatan otomatis seluruh aktivitas pengguna dan sistem, guna memastikan integritas, pelacakan histori, dan dukungan audit internal/eksternal. Layanan ini juga terintegrasi dengan deteksi ancaman berbasis AI/ML dan menjadi komponen utama keamanan sistem.

---

## 🌟 Tujuan

- Menyediakan **jejak audit immutable** terhadap seluruh aktivitas dalam SIMPEL.
- Mendukung **forensik digital**, pelacakan perubahan, dan deteksi penyalahgunaan akses.
- Menjadi dasar **compliance ISO 27001**, **PCI DSS**, dan pengawasan internal kejaksaan.

---

## 🧹 Fungsionalitas

- Mencatat seluruh **aksi kritikal** pengguna dan layanan (C/U/D, login, approval).
- Menyimpan histori entitas (aset, usulan, hibah, pemakaian, dll).
- Menyediakan **query log** terstruktur via REST API.
- Menghasilkan notifikasi atau alert atas aktivitas mencurigakan.
- Mendukung integrasi dengan **SIEM/UEBA AI** dan visualisasi audit (Grafana/ELK).

---

## 🧱 Teknologi

| Komponen      | Teknologi Digunakan              |
|---------------|----------------------------------|
| Bahasa        | Go + Gin                         |
| Arsitektur    | Event-driven via Pub/Sub         |
| Database      | PostgreSQL (skema `audit`)       |
| Audit Bus     | Redis Streams / NATS (opsional)  |
| Integrasi AI  | UEBA: Rule-based + ML            |

---

## 📂 Struktur Direktori

```
layanan-audit/
├── cmd/               # Entrypoint dan server
├── api/               # Handler dan routing
├── middleware/        # Fiber middleware untuk intercept audit
├── events/            # Event subscriber untuk menerima log dari layanan lain
├── service/           # Logika bisnis: simpan, validasi, query log
├── model/             # Skema data & entitas log
├── db/                # Kueri SQL (sqlc)
├── config/            # Konfigurasi YAML/ENV
└── README.md
```

---

## 🔐 Keamanan & Kepatuhan

- **Audit log immutable** (tidak bisa dihapus pengguna biasa)
- Setiap log mengandung:
  - UUID v4
  - Timestamps UTC
  - ID pengguna dan role
  - Layanan sumber
  - Jenis aksi (`create`, `delete`, `export`, dll)
  - Metadata (ID objek, nilai sebelum/sesudah)
- Seluruh komunikasi **dienkripsi (TLS v1.3)**
- Mendukung **verifikasi hash log** untuk integritas

---

## 🧠 Integrasi AI UEBA

Layanan ini dapat didukung oleh **AI User Entity Behavior Analytics (UEBA)**:

| Fitur AI               | Pendekatan                           |
|------------------------|--------------------------------------|
| Deteksi Anomali        | Supervised Learning (login, akses)   |
| Threshold Alerting     | Rule-based policy                    |
| Deteksi Abuse Patterns | Unsupervised Clustering              |
| Visualisasi            | Integrasi ke Grafana atau Kibana     |

Model AI ini berada di `layanan-ai`, dengan inferensi dipanggil secara terpisah.

---

## 🧪 Contoh Endpoint

```http
GET /api/audit/logs?user_id=42&from=2025-07-01&to=2025-07-15
```

```json
[
  {
    "id": "log-ax3920",
    "timestamp": "2025-07-10T13:02:11Z",
    "user_id": 42,
    "aksi": "update",
    "entitas": "usulan",
    "metadata": {
      "id_usulan": "U-5822",
      "field": "jumlah_barang",
      "sebelum": 12,
      "sesudah": 8
    }
  }
]
```

---

## 📌 Best Practice Integrasi Layanan Lain

Layanan-layanan wajib mengirim event atau log ke `layanan-audit`:

| Event                  | Wajib Dicatat? | Keterangan                     |
|------------------------|----------------|--------------------------------|
| Login/Logout           | ✅             | Termasuk status gagal/berhasil |
| CRUD Aset              | ✅             | Catat field penting            |
| Export/PDF             | ✅             | Tracking untuk keamanan        |
| Penilaian & Hibah      | ✅             | Termasuk hasil perhitungan     |
| Notifikasi Manual      | ⚠️             | Bisa dicatat untuk pelacakan   |

---

## 🥮 Pengujian & Validasi

```bash
make test-audit
```

Meliputi:
- Simulasi penyimpanan log
- Uji penanganan input tidak sah
- Benchmark penulisan 10.000 log
- Validasi struktur metadata per event

---

## 📊 Monitoring & Visualisasi

- Data log dapat di-push ke **Prometheus + Grafana**, atau **ELK Stack**
- Rekomendasi: gunakan Loki + Tempo untuk tracing lengkap lintas layanan

---

## 🛡️ Catatan Khusus Keamanan

Layanan ini berada di lingkup **Zero Trust**, dan **tidak boleh diakses langsung** oleh pengguna luar. Semua akses hanya via gateway yang terautentikasi.

---

## 📝 Lisensi

Hak Cipta © 2025 Kejaksaan Republik Indonesia – **Proyek Internal Tertutup**

> Layanan ini memproses data sensitif. Segala pelanggaran keamanan, penyalahgunaan, atau publikasi tanpa izin tertulis akan dikenakan sanksi hukum dan pemutusan akses permanen.
