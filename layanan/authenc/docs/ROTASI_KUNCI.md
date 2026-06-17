# Rotasi Kunci Otomatis

## Ringkasan

Fitur rotasi kunci otomatis meningkatkan keamanan dengan mengganti kunci kriptografi yang digunakan dalam sistem Authenc secara berkala. Hal ini meminimalkan dampak jika terjadi kebocoran kunci.

## Fitur

- **Penjadwalan Otomatis**: Kunci dirotasi setiap 30 hari secara default.
- **Integrasi Secreton**: Menggunakan Secreton untuk pengelolaan dan penyimpanan kunci yang aman.
- **Audit Trail**: Setiap rotasi dicatat dalam tabel `key_rotation_audit`.
- **Masa Transisi (Grace Period)**: Kunci lama tetap valid selama 7 hari setelah rotasi untuk transisi yang mulus.

## Jenis Kunci yang Dirotasi

- Kunci penandatanganan JWT (Ed25519).
- Kunci enkripsi sesi (AES-GCM).
- Kunci enkripsi rahasia MFA.

## Konfigurasi

```toml
[key_rotation]
enabled = true
rotation_interval_days = 30
grace_period_days = 7
enable_notifications = true
```

## Monitoring

Gunakan metrik berikut untuk memantau status rotasi:
- `authenc_key_rotation_total`: Total rotasi yang berhasil.
- `authenc_key_rotation_failures`: Jumlah rotasi yang gagal.
