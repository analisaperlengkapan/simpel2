# Otorisasi Satker (Satuan Kerja)

Dokumen ini mendeskripsikan mekanisme otorisasi berbasis Satker di layanan Authenc.

## Ringkasan

Sistem ini memastikan bahwa pengguna hanya dapat mengakses data atau melakukan tindakan yang relevan dengan Satker (Satuan Kerja) tempat mereka bernaung.

## Fitur Utama

- **Hierarki Satker**: Mendukung struktur organisasi Kejaksaan RI yang bertingkat.
- **Role-Based Access Control (RBAC)**: Kombinasi peran pengguna dengan batasan Satker.
- **Middleware Otorisasi**: Validasi otomatis ID Satker pada setiap request API.

## Atribut Satker dalam Token JWT

Setiap token yang diterbitkan oleh Authenc mencakup klaim `satker_id`:

```json
{
  "sub": "123456789",
  "name": "Ahmad",
  "satker_id": "JA-001",
  "roles": ["operator_aset"]
}
```

## Validasi di Sisi Layanan

Layanan hilir (seperti `layanan-aset`) wajib memvalidasi bahwa data yang diminta sesuai dengan `satker_id` dalam token:

1. Ambil `satker_id` dari klaim JWT.
2. Tambahkan filter `WHERE satker_id = ?` pada query database.
3. Tolak akses jika Satker tidak cocok, kecuali pengguna memiliki peran `admin_pusat`.
