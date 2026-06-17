# Retensi Event - Authenc

Dokumen ini mendeskripsikan kebijakan dan mekanisme retensi data event (log aktivitas) dalam layanan Authenc.

## Kebijakan Retensi

Kebijakan retensi menentukan berapa lama data event disimpan di database aktif sebelum diarsipkan atau dihapus.

- **Event Pengguna**: Default 90 hari (login, logout, operasi MFA).
- **Event Admin**: Default 365 hari (tindakan administratif, perubahan konfigurasi).

## Mekanisme Pengarsipan (Cold Storage)

Sebelum dihapus dari database, event akan diarsipkan ke penyimpanan dingin (S3/MinIO):

- **Pemrosesan Batch**: Event diarsipkan dalam kelompok (batch) untuk efisiensi.
- **Format JSON**: Data disimpan dalam format JSON agar mudah diambil kembali.
- **Struktur Terorganisir**: Arsip disusun berdasarkan tanggal dan jenis event.

## Konfigurasi

Contoh konfigurasi retensi dalam `events.toml`:

```toml
[events]
enabled = true
user_event_retention_days = 90
admin_event_retention_days = 365
archive_before_delete = true

[events.cold_storage]
enabled = true
storage_type = "s3"
region = "ap-southeast-1"
bucket = "kejaksaan-authenc-events"
```

## Monitoring

Metrik Prometheus tersedia untuk memantau proses retensi:
- `authenc.event_retention.events_deleted_total`: Total event yang dihapus.
- `authenc.event_retention.events_archived_total`: Total event yang diarsipkan.
