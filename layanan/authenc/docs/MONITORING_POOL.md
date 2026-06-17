# Monitoring Pool Koneksi Database

Dokumen ini menjelaskan sistem pemantauan untuk pool koneksi database di Authenc.

## Metrik Utama

Kami melacak metrik berikut untuk memastikan performa database yang optimal:

- **Penggunaan Pool**: Persentase koneksi yang sedang digunakan vs total kapasitas.
- **Waktu Tunggu**: Berapa lama request harus menunggu untuk mendapatkan koneksi.
- **Tingkat Penggunaan Kembali**: Seberapa sering koneksi yang ada dipakai ulang dibandingkan pembuatan koneksi baru.
- **Kegagalan Koneksi**: Jumlah kegagalan saat mencoba menghubungkan ke database.

## Ambang Batas Peringatan

- **Peringatan**: Penggunaan pool > 80% atau waktu tunggu > 100ms.
- **Kritis**: Penggunaan pool > 90% atau waktu tunggu > 500ms.

## Konfigurasi Default

```toml
[database]
max_connections = 50
min_connections = 10
connection_timeout = 30
idle_timeout = 600
```

## Langkah Pemecahan Masalah

Jika penggunaan pool tinggi:
1. Periksa query yang lambat.
2. Tingkatkan `max_connections` jika resource server mencukupi.
3. Pastikan koneksi dilepaskan (release) segera setelah selesai digunakan.
