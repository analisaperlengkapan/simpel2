# Panduan Aplikasi Authenticator untuk MFA SIMPEL

## Daftar Isi

1. [Google Authenticator](#google-authenticator)
2. [Microsoft Authenticator](#microsoft-authenticator)
3. [FreeOTP](#freeotp)
4. [Authy](#authy)
5. [Perbandingan Aplikasi](#perbandingan-aplikasi)
6. [Troubleshooting Khusus Aplikasi](#troubleshooting-khusus-aplikasi)

---

## Google Authenticator

### Instalasi

**Android**: [Google Play Store](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2)
**iOS**: [App Store](https://apps.apple.com/app/google-authenticator/id388497605)

### Setup Langkah Demi Langkah

#### Untuk Pengguna Baru

1. **Download dan Install**
   - Buka Google Play Store atau App Store
   - Cari "Google Authenticator"
   - Tap "Install" atau "Get"
   - Tunggu hingga instalasi selesai

2. **Buka Aplikasi**
   - Tap ikon Google Authenticator
   - Pilih "Get Started" atau "Mulai"
   - Baca dan setujui terms of service

3. **Tambah Akun SIMPEL**
   - Tap tombol "+" di pojok kanan bawah
   - Pilih "Scan a QR code"
   - Izinkan akses kamera jika diminta
   - Arahkan kamera ke QR code di layar komputer
   - Tunggu hingga muncul "SIMPEL Kejaksaan RI" di daftar

4. **Verifikasi Setup**
   - Lihat kode 6 digit yang muncul
   - Masukkan kode di portal SIMPEL
   - Jika berhasil, setup selesai

#### Setup Manual (Jika QR Code Tidak Bisa Di-scan)

1. Tap "+" → "Enter a setup key"
2. Isi informasi:
   - **Account**: [NIP Anda]@kejaksaan.go.id
   - **Key**: [Salin kode rahasia dari portal]
3. Tap "Add"

### Penggunaan Sehari-hari

- **Melihat Kode**: Buka app, kode langsung terlihat
- **Refresh Kode**: Kode berubah otomatis setiap 30 detik
- **Multiple Accounts**: Scroll untuk melihat akun lain

### Backup dan Restore

- **Export**: Settings → Transfer accounts → Export accounts
- **Import**: Settings → Transfer accounts → Import accounts
- **Cloud Backup**: Aktifkan di Settings → Google Account sync

### Tips Google Authenticator

✅ **Kelebihan**:

- Sederhana dan mudah digunakan
- Tidak perlu koneksi internet
- Backup otomatis dengan Google Account
- Gratis tanpa iklan

⚠️ **Perhatian**:

- Backup hanya tersedia di versi terbaru
- Tidak ada sinkronisasi real-time antar device

---

## Microsoft Authenticator

### Instalasi

**Android**: [Google Play Store](https://play.google.com/store/apps/details?id=com.azure.authenticator)
**iOS**: [App Store](https://apps.apple.com/app/microsoft-authenticator/id983156458)

### Setup Langkah Demi Langkah

#### Setup Awal

1. **Download dan Install**
   - Install dari store resmi
   - Buka aplikasi setelah instalasi

2. **Setup Akun Microsoft (Opsional)**
   - Tap "Sign in with Microsoft" untuk backup
   - Atau tap "Skip" untuk penggunaan lokal

3. **Tambah Akun SIMPEL**
   - Tap "+" di pojok kanan atas
   - Pilih "Other account (Google, Facebook, etc.)"
   - Pilih "Scan QR code"
   - Scan QR code dari portal SIMPEL

#### Setup Manual

1. Tap "+" → "Other account" → "Enter code manually"
2. Isi:
   - **Account name**: SIMPEL - [NIP Anda]
   - **Secret key**: [Kode rahasia dari portal]
3. Tap "Finish"

### Fitur Unggulan

- **Push Notifications**: Untuk akun Microsoft (tidak berlaku untuk SIMPEL)
- **Cloud Backup**: Otomatis jika login dengan akun Microsoft
- **Biometric Lock**: Kunci app dengan fingerprint/face ID
- **Dark Mode**: Tema gelap untuk kenyamanan mata

### Pengaturan Keamanan

1. **Aktifkan App Lock**:
   - Settings → App lock → On
   - Pilih PIN, fingerprint, atau face ID

2. **Backup Settings**:
   - Settings → Cloud backup → On
   - Login dengan akun Microsoft

### Tips Microsoft Authenticator

✅ **Kelebihan**:

- Interface modern dan intuitif
- Backup cloud otomatis
- Keamanan berlapis dengan app lock
- Mendukung multiple account types

⚠️ **Perhatian**:

- Memerlukan akun Microsoft untuk backup penuh
- Ukuran app lebih besar

---

## FreeOTP

### Instalasi

**Android**: [F-Droid](https://f-droid.org/packages/org.fedorahosted.freeotp/) atau [Google Play](https://play.google.com/store/apps/details?id=org.fedorahosted.freeotp)
**iOS**: [App Store](https://apps.apple.com/app/freeotp-authenticator/id872559395)

### Setup Langkah Demi Langkah

#### Setup Awal

1. **Download dan Install**
   - Pilih dari F-Droid (open source) atau Google Play
   - Install dan buka aplikasi

2. **Tambah Akun**
   - Tap ikon "+" atau kamera di pojok kanan atas
   - Arahkan kamera ke QR code
   - Akun akan otomatis ditambahkan

#### Setup Manual

1. Tap menu (3 garis) → "Add Token Manually"
2. Isi:
   - **Issuer**: SIMPEL Kejaksaan RI
   - **Label**: [NIP Anda]
   - **Secret**: [Kode rahasia dari portal]
   - **Type**: TOTP
   - **Algorithm**: SHA1
   - **Digits**: 6
   - **Period**: 30
3. Tap "Add"

### Penggunaan

- **Generate Kode**: Tap pada akun untuk generate kode baru
- **Auto-refresh**: Kode berubah otomatis setiap 30 detik
- **Edit Token**: Long press untuk edit atau hapus

### Backup dan Export

1. **Export**:
   - Menu → Settings → Export
   - Pilih format (JSON recommended)
   - Simpan file backup

2. **Import**:
   - Menu → Settings → Import
   - Pilih file backup
   - Konfirmasi import

### Tips FreeOTP

✅ **Kelebihan**:

- Sepenuhnya open source
- Tidak ada tracking atau ads
- Ringan dan cepat
- Export/import manual

⚠️ **Perhatian**:

- Interface lebih sederhana
- Tidak ada cloud backup otomatis
- Perlu backup manual secara berkala

---

## Authy

### Instalasi

**Android**: [Google Play Store](https://play.google.com/store/apps/details?id=com.authy.authy)
**iOS**: [App Store](https://apps.apple.com/app/authy/id494168017)

### Setup Langkah Demi Langkah

#### Setup Awal

1. **Download dan Install**
   - Install dari store resmi
   - Buka aplikasi

2. **Registrasi Nomor Telepon**
   - Masukkan nomor telepon Indonesia (+62)
   - Verifikasi dengan SMS atau panggilan
   - Buat PIN untuk keamanan

3. **Tambah Token SIMPEL**
   - Tap "+" di pojok kanan bawah
   - Pilih "Scan QR Code"
   - Scan QR code dari portal

#### Setup Manual

1. Tap "+" → "Enter Key Manually"
2. Isi:
   - **Account Name**: SIMPEL - [NIP]
   - **Key**: [Kode rahasia dari portal]
   - **Digits**: 6 digits
3. Tap "Save"

### Fitur Unggulan

- **Multi-Device Sync**: Sinkronisasi antar device otomatis
- **Cloud Backup**: Backup terenkripsi ke cloud Authy
- **Desktop App**: Tersedia untuk Windows, Mac, Linux
- **Biometric Lock**: Fingerprint dan face ID support

### Pengaturan Keamanan

1. **Aktifkan Authenticator Backups**:
   - Settings → Accounts → Authenticator Backups → Enable

2. **Set Master Password**:
   - Settings → Accounts → Master Password
   - Buat password yang kuat

3. **Multi-Device**:
   - Settings → Devices → Allow Multi-device → On/Off

### Sinkronisasi Multi-Device

1. **Tambah Device Baru**:
   - Install Authy di device baru
   - Login dengan nomor telepon yang sama
   - Verifikasi dari device lama

2. **Hapus Device**:
   - Settings → Devices
   - Pilih device yang ingin dihapus
   - Tap "Remove"

### Tips Authy

✅ **Kelebihan**:

- Sinkronisasi multi-device terbaik
- Desktop app tersedia
- Backup cloud terenkripsi
- Interface user-friendly

⚠️ **Perhatian**:

- Memerlukan nomor telepon untuk registrasi
- Backup tergantung layanan Authy
- Ukuran app lebih besar

---

## Perbandingan Aplikasi

| Fitur | Google Auth | Microsoft Auth | FreeOTP | Authy |
|-------|-------------|----------------|---------|-------|
| **Gratis** | ✅ | ✅ | ✅ | ✅ |
| **Open Source** | ❌ | ❌ | ✅ | ❌ |
| **Cloud Backup** | ✅ | ✅ | ❌ | ✅ |
| **Multi-Device** | ✅ | ✅ | Manual | ✅ |
| **Offline** | ✅ | ✅ | ✅ | ✅ |
| **Desktop App** | ❌ | ❌ | ❌ | ✅ |
| **Biometric Lock** | ❌ | ✅ | ❌ | ✅ |
| **Export/Import** | ✅ | ❌ | ✅ | ❌ |

### Rekomendasi Berdasarkan Kebutuhan

**Untuk Pemula**: Google Authenticator

- Paling sederhana dan mudah digunakan
- Backup otomatis dengan Google Account

**Untuk Keamanan Maksimal**: Microsoft Authenticator

- App lock dengan biometric
- Backup cloud yang aman

**Untuk Privacy**: FreeOTP

- Open source, no tracking
- Kontrol penuh atas data

**Untuk Multi-Device**: Authy

- Sinkronisasi terbaik antar device
- Desktop app tersedia

---

## Troubleshooting Khusus Aplikasi

### Google Authenticator

**Masalah**: Kode tidak muncul setelah scan QR
**Solusi**:

1. Pastikan kamera memiliki izin akses
2. Coba scan ulang dengan pencahayaan lebih baik
3. Gunakan setup manual jika QR tidak terbaca

**Masalah**: Backup tidak berfungsi
**Solusi**:

1. Update ke versi terbaru
2. Login dengan Google Account
3. Aktifkan sync di Settings

### Microsoft Authenticator

**Masalah**: Push notification tidak muncul
**Solusi**:

1. Periksa pengaturan notifikasi di sistem
2. Pastikan app tidak dalam mode battery saver
3. Re-login akun Microsoft jika perlu

**Masalah**: Backup tidak tersinkronisasi
**Solusi**:

1. Pastikan login dengan akun Microsoft yang sama
2. Periksa koneksi internet
3. Force sync di Settings → Cloud backup

### FreeOTP

**Masalah**: Token hilang setelah update app
**Solusi**:

1. Restore dari backup file jika ada
2. Setup ulang jika tidak ada backup
3. Selalu export backup sebelum update

**Masalah**: Tidak bisa import backup
**Solusi**:

1. Periksa format file backup (harus JSON)
2. Pastikan file tidak corrupt
3. Coba import satu per satu jika batch import gagal

### Authy

**Masalah**: Tidak bisa login di device baru
**Solusi**:

1. Pastikan nomor telepon sama persis
2. Periksa SMS/panggilan verifikasi
3. Pastikan multi-device enabled di device lama

**Masalah**: Desktop app tidak sync
**Solusi**:

1. Logout dan login ulang di desktop
2. Periksa koneksi internet
3. Restart aplikasi desktop

### Masalah Umum Semua Aplikasi

**Kode Selalu Salah**:

1. **Sinkronisasi Waktu**:
   - Android: Settings → Date & time → Automatic date & time
   - iOS: Settings → General → Date & Time → Set Automatically

2. **Zona Waktu**:
   - Pastikan zona waktu sesuai lokasi (WIB/WITA/WIT)
   - Restart aplikasi setelah ubah zona waktu

**App Crash atau Freeze**:

1. Force close dan buka ulang aplikasi
2. Restart ponsel
3. Update aplikasi ke versi terbaru
4. Clear cache aplikasi (Android)

**Tidak Bisa Scan QR Code**:

1. Bersihkan lensa kamera
2. Atur jarak 10-15 cm dari layar
3. Pastikan pencahayaan cukup
4. Gunakan setup manual sebagai alternatif

---

**Tips Keamanan Umum**:

- Selalu download dari store resmi
- Aktifkan screen lock di ponsel
- Jangan screenshot kode MFA
- Backup aplikasi authenticator secara berkala
- Gunakan kode cadangan jika app bermasalah

**Kontak Bantuan**: helpdesk-simipelv2@kejaksaan.go.id
