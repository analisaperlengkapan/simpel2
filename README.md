# Simpelv2

Simpelv2 adalah platform web modern dengan pemisahan **Backend** (REST API) dan **Frontend** (Single‑Page App), memanfaatkan:

* **Backend**: Go (Gin) + sqlc sebagai API server ringan, modular, dan performa tinggi
* **Frontend**: React + Vite sebagai SPA dengan hot‑reload dan bundling efisien
* **Database**: PostgreSQL, migrasi manual via SQL
* **Reverse Proxy & Static Files**: Nginx
* **Container**: Docker Compose untuk Dev & Prod
* **Otomasi**: Makefile dan skrip setup Docker

---

## 📖 Alasan Pemilihan Teknologi

1. **Go (Gin)**: performa tinggi, modular, statically compiled, dan sangat cocok untuk layanan API skala besar.
2. **React + Vite**: pengembangan UI cepat dengan DX unggul.
3. **PostgreSQL**: fitur indexing, JSONB, dan reliabilitas tinggi.
4. **sqlc**: query SQL eksplisit → kode Go aman & cepat.
5. **Nginx (Reverse Proxy & Static)**: load balancing, proxy, dan serving aset statis.
6. **Docker Compose**: isolasi dan konsistensi antar lingkungan.
7. **Makefile**: menyatukan workflow dev/test/deploy dalam satu perintah.

---

## 🔧 Prasyarat

1. Git
2. Docker Engine & Docker Compose V2
3. Make

> Jika Docker Compose V2 belum terpasang:
>
> ```bash
> ./scripts/setup-docker.sh
> ```

---

## 📂 Struktur Proyek

```
simpelv2/
├── backend/
│   ├── cmd/                    # Entry point main.go
│   ├── db/                     # schema.sql, query.sql, sqlc.yaml
│   ├── internal/
│   │   ├── api/                # Gin handlers
│   │   ├── config/             # DB loader & env config
│   │   └── db/                 # Output hasil sqlc generate
│   ├── Dockerfile.backend
├── frontend/
│   ├── public/, src/, dist/
│   ├── Dockerfile.frontend
│   ├── package.json
│   └── vite.config.js
├── docker/
│   └── nginx/{dev.conf, prod.conf}
├── scripts/
│   └── setup-docker.sh
├── .env
├── docker-compose.dev.yml
├── docker-compose.prod.yml
├── Makefile
└── README.md
```

---

## 👥 Kolaborasi Tim dengan GitLab

Tetap berlaku seperti penjelasan sebelumnya (branching, MR, CI/CD, dsb).

gar proses migrasi dan pengembangan Simpelv2 terstruktur dan kolaboratif, berikut panduan kerja tim menggunakan **GitLab**:

### 1. **Membuat Repository GitLab**

* Buat repository baru bernama `simpelv2` di GitLab (bisa disetel private/public).
* Tambahkan anggota tim:

  * **Maintainer**: tim core (setup CI/CD, proteksi branch).
  * **Developer**: kontributor/pengembang fitur.

### 2. **Struktur Branching**

Gunakan strategi branching standar:

| Branch      | Fungsi                                       |
| ----------- | -------------------------------------------- |
| `main`      | Versi stabil/produksi, hanya via Merge       |
| `dev`       | Integrasi semua fitur baru sebelum ke `main` |
| `feature/*` | Fitur spesifik (ex: `feature/user-login`)    |
| `bugfix/*`  | Perbaikan bug                                |
| `docs/*`    | Dokumentasi                                  |

Contoh:

```bash
git checkout -b feature/migrasi-model-user
# lakukan migrasi model User
git add .
git commit -m "Migrasi model User dari Laravel ke Buffalo"
git push origin feature/migrasi-model-user
```

Lalu, buka **Merge Request** ke `dev` melalui GitLab UI.

### 3. **Manajemen Tugas (Issues & Milestone)**

Gunakan fitur `Issues` GitLab untuk memecah migrasi menjadi sub-tugas:

| Milestone          | Issue Contoh                         |
| ------------------ | ------------------------------------ |
| Database Migration | Migrasi struktur tabel SIMAN         |
| Auth               | Migrasi login Laravel ke Buffalo     |
| Frontend UI        | Konversi Blade → React               |
| CRUD Aset          | Modul Aset pada frontend dan backend |

Setiap developer dapat assign dirinya sendiri pada issue.

### 4. **Code Review & Merge Request**

* Setiap **Merge Request (MR)** minimal direview oleh 1 anggota tim.
* ## Gunakan checklist untuk review:

### 5. **Automated Test & CI/CD**

* Tambahkan file `.gitlab-ci.yml` untuk menjalankan test otomatis:

  * Cek linting Go & JS
  * Build frontend
  * Test endpoint (opsional)

Contoh:

```yaml
stages:
  - test
  - build

test_backend:
  image: golang:1.23
  script:
    - cd backend
    - go test ./...

test_frontend:
  image: node:22
  script:
    - cd frontend
    - npm install
    - npm run lint
```

### 6. **Dokumentasi Tim**

* Gunakan fitur **Wiki** di GitLab untuk mencatat:

  * Mapping struktur Laravel → Buffalo
  * Referensi REST API
  * Standar folder & file
  * Langkah build manual & otomatis

---

## 🏠 Menjalankan di Localhost (Tanpa Docker)

Jika Anda ingin menjalankan Simpelv2 secara lokal tanpa Docker, ikuti langkah berikut:

1. **Install Prasyarat**:

   * Go >=1.23
   * Buffalo CLI: `go install github.com/gobuffalo/cli/cmd/buffalo@latest`
   * Node.js >=18 & npm
   * PostgreSQL: buat database `simpelv2_dev` dan user sesuai `.env`
2. **Backend**:

   ```bash
   cd backend
   buffalo dev    # Menjalankan Buffalo dengan hot-reload di port 3000
   ```
3. **Frontend**:

   ```bash
   cd frontend
   npm install
   npm run dev -- --host    # Menjalankan Vite di port 5173
   ```
4. **Konfigurasi Proxy (opsional)**:

   * Jika ingin proxy API di localhost:8080, jalankan Nginx dengan `dev.conf` atau gunakan Buffalo proxy.

Setelah berhasil, akses:

* Backend langsung: [http://localhost:3000/api/](http://localhost:3000/api/)
* Frontend langsung: [http://localhost:5173](http://localhost:5173)
* Full App (jika proxy): [http://localhost:8080](http://localhost:8080)
---

## 🏠 Menjalankan Lokal (Tanpa Docker)

**Prasyarat**:
- Go >= 1.22
- Node.js >= 18
- PostgreSQL aktif dan `DATABASE_URL` diset

**Backend**:
```bash
cd backend && go run ./cmd/main.go
```

**Frontend**:
```bash
cd frontend && npm install && npm run dev -- --host
```

---

## 🐳 Menjalankan Dengan Docker

```bash
make dev
```

- SPA: http://localhost:5173
- API: http://localhost:8080/api/
- Full App (proxy via Nginx): http://localhost:8080

Hentikan: `make stop-dev`, Log: `make logs-dev`

---

## 🎯 Production

```bash
make build
make prod
```

- http://localhost

Stop: `make stop-prod`, Log: `make logs-prod`

---

## 🚀 Rilis Sekaligus

```bash
make release
```

---

## 📦 Daftar Perintah Make

| Perintah         | Deskripsi                                  |
|------------------|---------------------------------------------|
| make sqlc        | Generate kode Go dari SQL                  |
| make envcheck    | Validasi `.env` penting                    |
| make migrate-up  | Jalankan migrasi SQL                       |
| make migrate-down| Rollback migrasi                           |
| make seed        | Jalankan seeder data awal                  |
| make dev         | Jalankan full stack development            |
| make prod        | Jalankan full stack production             |
| make release     | Jalankan sqlc → build → prod               |

---

## 🔄 Migrasi Laravel → Simpelv2

## 🔄 Migrasi dari Simpelv1 (Laravel) ke Simpelv2

**Mengapa Migrasi?**

* **Performa dan Skalabilitas**: Go (Gin) memberikan eksekusi sangat cepat dan rendah latensi dibandingkan PHP.
* **Pengalaman Pengembang Modern**: React + Vite menawarkan workflow frontend dinamis dengan hot‑reload instan.
* **Konsistensi Lintas Layanan**: PostgreSQL memiliki fitur lengkap (JSONB, indexing) yang terintegrasi baik dengan sqlc dan Go.
* **DevOps & Deployment**: Docker Compose menyederhanakan provisioning environment dan deployment multistage.
* **Arsitektur Terpisah**: Pemisahan backend API dan frontend SPA memudahkan maintenance dan kerja tim paralel.
* **Keamanan & Stabilitas**: Middleware dan proxy Nginx membantu menjaga kestabilan serta keamanan arsitektur microservice.

### A. Struktur Simpelv1 (Laravel)

```
simpel_web-main/
├── app/Http/Controllers/
├── app/Models/
├── database/migrations/
├── resources/views/
├── resources/js/, css/
├── routes/web.php
├── routes/api.php
└── public/
```

### B. Ekspor Schema MySQL

```bash
mysqldump --no-data simpelv1 > schema_v1.sql
```

### C. Mapping File Laravel ke Simpelv2

| Simpelv1 Path                     | Simpelv2 Path                   | Keterangan                                  |
|----------------------------------|----------------------------------|---------------------------------------------|
| `app/Http/Controllers/*.php`     | `backend/internal/api/*.go`      | Handler API                                 |
| `app/Models/*.php`               | `backend/db/query.sql`           | Model SQL + generate via sqlc               |
| `resources/views/*.blade.php`    | `frontend/src/pages/`            | Komponen halaman React                      |
| `resources/js/`, `resources/css/`| `frontend/src/`                  | Frontend logic                              |
| `routes/api.php`                 | `main.go`                        | Router Gin                                  |
| `public/`                        | `frontend/public/`               | Aset statis                                 |
| `.env`                           | `.env`                           | Lingkungan dan koneksi                      |

### D. Migrasi Database

1. Salin struktur dari `schema_v1.sql` ke `backend/db/schema.sql` (konversi ke PostgreSQL jika perlu).
2. Tambahkan query ke `query.sql`, contoh:

```sql
-- name: ListUsers :many
SELECT id, name FROM users ORDER BY id;
```

3. Jalankan `make sqlc` untuk generate kode.

### E. Migrasi Model + Handler

* Buat file handler di `backend/internal/api/user.go`:

```go
func GetUsers(c *gin.Context) {
  users, err := db.Queries.ListUsers(c)
  if err != nil {
    c.JSON(500, gin.H{"error": "internal error"})
    return
  }
  c.JSON(200, users)
}
```

### F. Migrasi View ke React

```jsx
function Users() {
  const [data, setData] = useState([]);
  useEffect(() => {
    fetch('/api/users').then(r => r.json()).then(setData);
  }, []);
  return <ul>{data.map(u => <li key={u.id}>{u.name}</li>)}</ul>;
}
```

Tambahkan ke router:

```jsx
<Route path="/users" element={<Users />} />
```

### G. Jalankan

```bash
make dev
```

Akses:

- SPA: http://localhost:5173
- Full App: http://localhost:8080
- API: http://localhost:8080/api/users

---

## 🤝 Kontribusi

1. Fork & clone
2. Buat branch baru
3. Commit & push
4. Buat Merge Request

---

## 📝 Lisensi

Copyright © 2025 Kejaksaan Republik Indonesia