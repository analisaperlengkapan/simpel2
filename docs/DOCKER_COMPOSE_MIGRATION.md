# Migrasi: `docker-compose` → `docker compose`

Dokumen singkat untuk membantu tim berpindah dari *standalone* `docker-compose` (v1 / paket terpisah) ke **Docker Compose v2** yang menjadi subcommand `docker compose` di Docker CLI.

💡 *Mengapa migrasi?* `docker compose` (v2) adalah plugin resmi dan terintegrasi dengan Docker CLI, memiliki kompatibilitas lebih baik, lebih sering diperbarui, dan menghindari konflik antara instalasi (pip/apt/manual) yang dapat menyebabkan perilaku tak terduga.

---

## 1. Verifikasi keadaan saat ini

Jalankan:

```bash
# Periksa plugin modern
docker compose version

# Jika masih terpasang, periksa legacy
docker-compose --version
```

Jika `docker compose version` sukses, Anda sudah menggunakan plugin Compose (disarankan).

Jika hanya `docker-compose --version` yang tersedia, itu berarti Anda memakai binary standalone (legacy).

## 2. Cara menghapus (uninstall) `docker-compose` legacy

Pilih cara sesuai bagaimana `docker-compose` terpasang di mesin Anda.

- Jika terpasang via apt (Debian/Ubuntu):

```bash
sudo apt remove -y docker-compose
```

- Jika terpasang via pip/pip3:

```bash
sudo pip3 uninstall docker-compose
# atau
sudo pip uninstall docker-compose
```

- Jika terpasang secara manual (mis. /usr/local/bin/docker-compose):

```bash
sudo rm -f /usr/local/bin/docker-compose
```

- Jika menggunakan Snap atau metode lain, gunakan alat paket yang sesuai (contoh: `snap remove docker-compose`).

> ⚠️ Pastikan tidak menghapus paket `docker` atau `docker-engine` — hanya paket `docker-compose` yang berdiri sendiri.

## 3. Instal/aktifkan Docker Compose (v2) — plugin `docker compose`

Pada banyak distribusi modern, plugin ini sudah termasuk saat Anda menginstal Docker Engine (Docker 20.10+). Jika belum tersedia, beberapa distribusi memiliki paket `docker-compose-plugin`:

```bash
# Debian/Ubuntu (jika tersedia di repo)
sudo apt update && sudo apt install -y docker-compose-plugin
```

Verifikasi:

```bash
docker compose version
```

## 4. Hal yang perlu diperhatikan di repo ini

- Nama file tetap `docker-compose.yml` (tidak diubah).
- Perintah yang harus digunakan adalah `docker compose ...` (spasi). Contoh: `docker compose up -d`.
- Jika Anda memiliki skrip yang memanggil `docker-compose` secara langsung, gantilah menjadi `docker compose` atau gunakan fallback yang memeriksa keberadaan plugin.
- CI/CD: periksa runner image/host dan pastikan `docker compose` tersedia. Jika runner masih memiliki `docker-compose` legacy, migrasi runner atau pasang `docker-compose-plugin`.

## 5. Contoh pengecekan cepat (CI lint/pipeline)

Tambahkan langkah pengecekan singkat di CI untuk memastikan `docker compose` tersedia:

```bash
if docker compose version &> /dev/null; then
  echo "docker compose plugin available"
else
  echo "docker compose plugin NOT available" >&2
  exit 1
fi
```

## 6. Pertanyaan umum

- Q: Apakah saya perlu mengganti file `docker-compose.yml`?  
  A: Tidak. Format file sama — hanya subcommand yang berubah (`docker compose`).

- Q: Apakah ada perbedaan perilaku?  
  A: Minor; untuk sebagian besar use-case perintah dan opsi tetap kompatibel. Jika Anda menggunakan plugin tambahan atau scripting kompleks, verifikasi di lingkungan staging.

---

Jika ada kebutuhan untuk menambahkan instruksi khusus OS atau langkah automatis (script uninstall), buat isu kecil di repo dan saya bantu menyiapkannya. ✅
