# Simpelv2

Simpelv2 adalah platform web modern dengan pemisahan **Backend** (REST API) dan **Frontend** (Single‑Page App), memanfaatkan:

* **Backend**: Buffalo (Go) sebagai API server
* **Frontend**: React + Vite sebagai SPA dengan hot‑reload
* **Database**: PostgreSQL, migrasi via `pop`/`fizz`
* **Reverse Proxy & Static Files**: Nginx
* **Container**: Docker Compose untuk Dev & Prod
* **Otomasi**: Makefile dan skrip setup Docker

---

## 📖 Alasan Pemilihan Teknologi

1. **Go (Buffalo)**: performa tinggi, mudah di-deploy, statically compiled sehingga meminimalisir dependensi runtime.
2. **React + Vite**: pengembangan UI responsif dengan hot-reload cepat dan bundling minimal.
3. **PostgreSQL**: reliabilitas, dukungan fitur lanjutan (JSONB, indexing), dan komplementer dengan Pop ORM.
4. **Nginx (Reverse Proxy & Static)**: mengelola routing ke backend/API, meneruskan request statis dengan cepat, serta SSL termination.
5. **Docker Compose**: menyederhanakan setup lingkungan Dev/Prod, konsistensi antar tim, isolasi layanan.
6. **Makefile & Skrip Otomasi**: menyatukan perintah kompleks dalam satu baris (`make dev`, `make release`), mempercepat onboarding pengembang baru.

---

## 🔧 Prasyarat

1. Git
2. Docker Engine & Docker Compose V2
3. Make

> Jika Docker Compose V2 belum terpasang, jalankan:
>
> ```bash
> ./scripts/setup-docker.sh
> ```

---

## 📂 Struktur Proyek

```
simpelv2/
├── backend/
│   ├── actions/         # Handler HTTP & definitions
│   ├── models/          # Struct Pop ORM
│   ├── migrations/      # Fizz migration files
│   └── ...
├── frontend/
│   ├── public/          # Static assets (html, images, css)
│   └── src/             # React components, router, API helper
├── docker/
│   ├── backend.Dockerfile
│   ├── frontend.Dockerfile
│   └── nginx/{dev.conf,prod.conf}
├── scripts/
│   └── setup-docker.sh
├── .env                  # Env vars
├── docker-compose.dev.yml
├── docker-compose.prod.yml
├── Makefile
└── README.md
```

---

## 👥 Kolaborasi Tim dengan GitLab

Agar proses migrasi dan pengembangan Simpelv2 terstruktur dan kolaboratif, berikut panduan kerja tim menggunakan **GitLab**:

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

### Menjalankan di Localhost dengan Docker

Untuk menjalankan di localhost menggunakan Docker (tanpa installing manual setiap komponen), ikuti:

1. Pastikan Docker Engine & Docker Compose V2 terpasang.
2. Dari folder root proyek:

   ```bash
   docker compose -f docker-compose.dev.yml up --build
   ```
3. Container yang akan berjalan:

   * **DB**: PostgreSQL di port internal 5432
   * **Backend**: Buffalo di [http://localhost:3000](http://localhost:3000) (via Nginx proxy di 8080)
   * **Frontend**: Vite di [http://localhost:5173](http://localhost:5173) (via Nginx proxy di 8080)
   * **Nginx**: Reverse proxy + static serving di [http://localhost:8080](http://localhost:8080)
4. Akses:

   * SPA langsung: [http://localhost:5173](http://localhost:5173)
   * Full App: [http://localhost:8080](http://localhost:8080)
   * API: [http://localhost:8080/api/](http://localhost:8080/api/)
5. Hentikan semua container:

   ```bash
   docker compose -f docker-compose.dev.yml down
   ```

## 🚀 Development

```bash
make dev
```

* Vite: [http://localhost:5173](http://localhost:5173)
* Nginx + Proxy: [http://localhost:8080](http://localhost:8080)
* API: [http://localhost:8080/api/](http://localhost:8080/api/)

Hentikan: `make stop-dev`, Log: `make logs-dev`

---

## 🛠 Production

```bash
make build
make prod
```

* Akses: [http://localhost](http://localhost)
  Hentikan: `make stop-prod`, Log: `make logs-prod`

---

## 🎉 Release

```bash
make release
```

*Perintah **`make release`** menjalankan **`make build`** dan **`make prod`** secara berurutan untuk mempersiapkan dan menjalankan aplikasi di mode production.*

---

## 📦 Daftar Perintah Make (Makefile)

Berikut ringkasan target Make yang tersedia:

| Target           | Deskripsi                                                                                   |
| ---------------- | ------------------------------------------------------------------------------------------- |
| `make dev`       | Jalankan seluruh layanan dalam mode **development** (backend Buffalo, frontend Vite, Nginx) |
| `make stop-dev`  | Hentikan seluruh container development                                                      |
| `make logs-dev`  | Tampilkan log real-time untuk development                                                   |
| `make build`     | Bangun aset frontend (Vite) untuk produksi                                                  |
| `make prod`      | Jalankan seluruh layanan dalam mode **production** (detached)                               |
| `make stop-prod` | Hentikan seluruh container production                                                       |
| `make logs-prod` | Tampilkan log real-time untuk production                                                    |
| `make release`   | Otomatisasi `make build` dan `make prod` untuk release sekali jalan                         |

---

## 🔄 Migrasi dari Simpelv1 (Laravel) ke Simpelv2

**Mengapa Migrasi?**

* **Performa dan Skalabilitas**: Go (Buffalo) memberikan eksekusi sangat cepat dan rendah latensi dibandingkan PHP, sehingga cocok untuk layanan berskala besar.
* **Pengalaman Pengembang Modern**: React + Vite menawarkan workflow frontend yang lebih dinamis dan interaktif, dengan hot‑reload instan.
* **Konsistensi Lintas Layanan**: Menggunakan PostgreSQL untuk fungsionalitas yang lengkap (JSONB, partial index) dan integrasi mulus dengan Pop ORM.
* **DevOps & Deployment**: Docker Compose menyederhanakan provisioning lingkungan, meminimalisir perbedaan antara development, staging, dan production.
* **Arsitektur Terpisah**: Pemisahan backend API dan frontend SPA memudahkan maintenance, tim terpisah, serta deployment independen.
* **Keamanan & Stabilitas**: Buffalo memiliki middleware bawaan (CSRF, parameter logging) dan Nginx mengamankan SSL/TLS serta cache static asset.

Agar jelas, berikut detil untuk pengguna awam: (Laravel) ke Simpelv2

Agar jelas, berikut detil untuk pengguna awam:

### A. Struktur Simpelv1 (Laravel)

```
simpel_web-main/
├── app/Http/Controllers/  # Logika request
├── app/Models/            # Model Eloquent
├── database/migrations/   # Migration MySQL
├── resources/views/       # Blade templates
├── resources/js/, css/    # Frontend assets
├── routes/web.php         # Halaman
├── routes/api.php         # API
└── public/                # File publik
```

### B. Ekspor Schema MySQL

1. Masuk folder v1, jalankan:

   ```bash
   mysqldump --no-data simpelv1 > schema_v1.sql
   ```
2. File `schema_v1.sql` berisi struktur tabel (nama & kolom).

### Mapping File V1 ➔ V2

| Simpelv1 Path                     | Simpelv2 Path                   | Keterangan                             |
| --------------------------------- | ------------------------------- | -------------------------------------- |
| `app/Http/Controllers/*.php`      | `backend/actions/`              | Laravel controllers ➔ Buffalo handlers |
| `app/Models/*.php`                | `backend/models/`               | Eloquent models ➔ Pop ORM structs      |
| `database/migrations/*.php`       | `backend/migrations/*.fizz`     | MySQL migrations ➔ Fizz migrations     |
| `routes/api.php`                  | `backend/actions/app.go`        | API routes ➔ Buffalo routes            |
| `resources/views/*.blade.php`     | `frontend/src/pages/`           | Blade templates ➔ React components     |
| `resources/js/`, `resources/css/` | `frontend/src/`                 | Frontend assets ➔ React source folder  |
| `public/`                         | `frontend/public/`              | Static assets (images, CSS, fonts)     |
| `routes/web.php`                  | `frontend/src/router.jsx`       | Web routes ➔ React Router              |
| `app/Http/Middleware/`            | `backend/actions/middleware.go` | Middleware custom ➔ Buffalo middleware |
| `.env`                            | `.env`                          | Environment variables                  |

### C. Migrasi Database

1. Buat migration:

   ```bash
   ```

docker compose -f docker-compose.dev.yml exec backend buff pop gen migration create\_nm\_table

````
2. Buka file `.up.fizz`, salin definisi tabel dari `schema_v1.sql`, contohnya:
```fizz
create_table("users") { t.Column("id","integer",{primary:true,auto:true}); ... }
````

3. Jalankan migrasi:

   ```bash
   ```

docker compose -f docker-compose.dev.yml exec backend buff pop migrate

````
4. Cek di Postgres:
```bash
docker compose -f docker-compose.dev.yml exec db psql -c "\d+ users"
````

### D. Migrasi Model

* Dari `app/Models/User.php`, buat `backend/models/user.go`:

  ```go
  type User struct{ ID int `db:"id"`; Name string `db:"name"` }
  ```

### E. Migrasi Logic Controller

* File handler `backend/actions/users.go`:

  ```go
  func UsersList(c Context) error{ var u []models.User; DB.All(&u); return c.Render(200,r.JSON(u)) }
  ```
* Daftar rute di `app.go`: `app.GET("/api/users",UsersList)`

### F. Migrasi View ke React

1. Folder `frontend/src/pages/`.
2. Buat `Users.jsx`:

   ```jsx
   function Users(){ const [u,s]=useState([]); useEffect(()=>fetch('/api/users').then(r=>r.json()).then(s),[]); return <ul>{u.map(x=> <li key={x.id}>{x.name}</li>)}</ul> }
   ```
3. Router `frontend/src/router.jsx`: `<Route path="/users" element={<Users/>}/>`

### G. Migrasi Aset Statis

* Copy `public/` dari v1 ke `frontend/public/`.

### H. (Opsional) Migrasi Data

1. Export CSV: `mysqldump --tab=/tmp ... users`
2. Import ke Postgres: `psql ... \copy users FROM '/tmp/users.txt' CSV`

### I. Verifikasi

1. `make dev`
2. Cek di browser:

   * SPA: localhost:5173
   * Full App: localhost:8080
   * API: localhost:8080/api/users

---

## 🤝 Kontribusi

1. Fork & clone
2. Branch baru
3. Commit & push
4. Pull Request

---

## 📝 Lisensi

MIT © 2025 Kejaksaan Republik Indonesia
