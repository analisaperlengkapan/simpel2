# Panduan Pengguna Multi-Factor Authentication (MFA)
## Sistem Informasi Manajemen Perlengkapan v2 (SIMPelv2)

### Daftar Isi
1. [Pengenalan MFA](#pengenalan-mfa)
2. [Pengaturan Awal MFA](#pengaturan-awal-mfa)
3. [Menggunakan Aplikasi Authenticator](#menggunakan-aplikasi-authenticator)
4. [Proses Login dengan MFA](#proses-login-dengan-mfa)
5. [Mengatasi Masalah Umum](#mengatasi-masalah-umum)
6. [Kode Cadangan](#kode-cadangan)
7. [Bantuan dan Dukungan](#bantuan-dan-dukungan)

---

## Pengenalan MFA

Multi-Factor Authentication (MFA) adalah lapisan keamanan tambahan yang melindungi akun Anda dengan memerlukan dua bentuk verifikasi:
1. **Sesuatu yang Anda ketahui**: Password/kata sandi
2. **Sesuatu yang Anda miliki**: Kode 6 digit dari aplikasi authenticator di ponsel

### Mengapa MFA Penting?
- **Keamanan Berlapis**: Melindungi data sensitif pemerintah
- **Compliance**: Memenuhi standar keamanan Kejaksaan RI
- **Perlindungan Identitas**: Mencegah akses tidak sah meskipun password bocor

---

## Pengaturan Awal MFA

### Langkah 1: Login Pertama Kali
1. Masuk ke portal SIMPelv2 dengan NIP dan password Anda
2. Setelah login berhasil, Anda akan diarahkan ke halaman "Setup MFA"
3. Jangan tutup halaman ini sampai setup selesai

### Langkah 2: Persiapan Ponsel
Pastikan Anda memiliki salah satu aplikasi authenticator berikut di ponsel:
- **Google Authenticator** (Direkomendasikan)
- **Microsoft Authenticator**
- **FreeOTP**
- **Authy**

### Langkah 3: Scan QR Code
1. Buka aplikasi authenticator di ponsel Anda
2. Pilih "Tambah Akun" atau ikon "+"
3. Pilih "Scan QR Code" atau "Scan Barcode"
4. Arahkan kamera ke QR code yang ditampilkan di layar
5. Tunggu hingga akun "SIMPelv2 Kejaksaan RI" muncul di aplikasi

### Langkah 4: Verifikasi Setup
1. Lihat kode 6 digit yang muncul di aplikasi authenticator
2. Masukkan kode tersebut di kolom "Kode Verifikasi" di portal
3. Klik "Verifikasi dan Selesaikan Setup"
4. Jika berhasil, Anda akan diarahkan ke dashboard

### Alternatif: Input Manual
Jika tidak bisa scan QR code:
1. Klik "Tidak bisa scan? Masukkanal"
2. Salin kode rahasia yang ditampilkan
3. Di aplikasi authenticator, pilih "Input Manual"
4. Masukkan informasi berikut:
   - **Nama Akun**: [NIP Anda]@kejaksaan.go.id
   - **Kode Rahasia**: [Kode yang disalin]
   - **Issuer**: SIMPelv2 Kejaksaan RI

---

## Menggunakan Aplikasi Authenticator

### Google Authenticator
1. **Download**: Tersedia di Google Play Store dan App Store
2. **Setup**: Buka app → Tap "+" → Pilih "Scan QR code"
3. **Penggunaan**: Kode berubah setiap 30 detik
4. **Backup**: Aktifkan backup Google untuk sinkronisasi antar device

### Microsoft Authenticator
1. **Download**: Tersedia di Google Play Store dan App Store
2. **Setup**: Buka app → Tap "+" → Pilih "Other account" → Scan QR
3. **Penggunaan**: Kode berubah setiap 30 detik
4. **Backup**: Login dengan akun Microsoft untuk sinkronisasi

### FreeOTP (Open Source)
1. **Download**: Tersedia di F-Droid dan App Store
2. **Setup**: Buka app → Tap "+" → Scan QR code
3. **Penggunaan**: Tap akun untuk generate kode baru
4. **Backup**: Export/import manual melalui menu settings

### Authy
1. **Download**: Tersedia di Google Play Store dan App Store
2. **Setup**: Daftar dengan nomor telepon → Tap "+" → Scan QR
3. **Penggunaan**: Kode berubah setiap 30 detik
4. **Backup**: Otomatis tersinkronisasi dengan nomor telepon

---

## Proses Login dengan MFA

### Login Rutin
1. **Masukkan Kredensial**:
   - NIP: [Nomor Induk Pegawai Anda]
   - Password: [Kata sandi Anda]
   - Klik "Login"

2. **Verifikasi MFA**:
   - Anda akan diarahkan ke halaman "Verifikasi MFA"
   - Buka aplikasi authenticator di ponsel
   - Lihat kode 6 digit untuk akun SIMPelv2
   - Masukkan kode di kolom verifikasi
   - Klik "Verifikasi"

3. **Akses Dashboard**:
   - Jika kode benar, Anda akan masuk ke dashboard
   - Jika salah, coba lagi dengan kode yang baru

### Tips Login
- **Waktu Kode**: Kode berubah setiap 30 detik, pastikan menggunakan kode terbaru
- **Koneksi Internet**: Pastikan ponsel dan komputer terhubung internet
- **Sinkronisasi Waktu**: Pastikan waktu di ponsel dan komputer akurat

---

## Mengatasi Masalah Umum

### Masalah 1: QR Code Tidak Bisa Di-scan
**Gejala**: Kamera tidak bisa membaca QR code

**Solusi**:
1. Pastikan kamera ponsel bersih dan fokus
2. Atur jarak yang tepat (10-15 cm dari layar)
3. Pastikan pencahayaan cukup
4. Gunakan opsi "Input Manual" sebagai alternatif

### Masalah 2: Kode Verifikasi Selalu Salah
**Gejala**: Kode 6 digit selalu ditolak sistem

**Solusi**:
1. **Periksa Waktu**:
   - Pastikan waktu di ponsel akurat
   - Sinkronkan waktu otomatis di pengaturan ponsel
   - Periksa zona waktu (WIB/WITA/WIT)

2. **Tunggu Kode Baru**:
   - Jangan gunakan kode yang hampir habis (detik ke-25-30)
   - Tunggu kode baru muncul sebelum input

3. **Periksa Akun**:
   - Pastikan menggunakan kode dari akun SIMPelv2 yang benar
   - Hapus akun duplikat jika ada

### Masalah 3: Aplikasi Authenticator Hilang/Rusak
**Gejala**: Ponsel hilang, rusak, atau aplikasi terhapus

**Solusi**:
1. **Gunakan Kode Cadangan**:
   - Masukkan salah satu kode cadangan 8 digit
   - Setiap kode hanya bisa digunakan sekali

2. **Hubungi Administrator**:
   - Jika kode cadangan habis atau hilang
   - Minta reset MFA oleh admin IT
   - Siapkan dokumen identitas untuk verifikasi

### Masalah 4: Lupa Password
**Gejala**: Tidak ingat password login

**Solusi**:
1. Gunakan fitur "Lupa Password" di halaman login
2. Ikuti instruksi reset via email
3. Setelah password direset, MFA tetap aktif
4. Hubungi helpdesk jika email tidak diterima

### Masalah 5: Akun Terkunci
**Gejala**: Pesan "Akun terkunci karena terlalu banyak percobaan"

**Solusi**:
1. **Tunggu Otomatis**: Akun akan terbuka otomatis setelah 15 menit
2. **Hubungi Admin**: Untuk unlock manual jika urgent
3. **Pencegahan**: Jangan coba-coba input kode sembarangan

---

## Kode Cadangan

### Apa itu Kode Cadangan?
Kode cadangan adalah 10 kode 8-digit yang bisa digunakan sebagai pengganti kode authenticator dalam situasi darurat.

### Kapan Menggunakan?
- Ponsel hilang atau rusak
- Aplikasi authenticator bermasalah
- Tidak bisa akses ponsel sementara waktu

### Cara Menggunakan:
1. Di halaman verifikasi MFA, klik "Gunakan Kode Cadangan"
2. Masukkan salah satu kode 8-digit
3. Klik "Verifikasi"
4. Kode yang sudah digunakan tidak bisa dipakai lagi

### Penyimpanan Kode Cadangan:
⚠️ **PENTING**: Simpan kode cadangan dengan aman!

**Cara Aman**:
- Cetak dan simpan di tempat aman (Secreton, laci terkunci)
- Simpan di password manager yang terenkripsi
- Foto dan simpan di cloud storage pribadi yang aman

**JANGAN**:
- Simpan di email kerja
- Tulis di sticky note di meja
- Bagikan dengan orang lain
- Simpan di file tidak terenkripsi

### Regenerasi Kode Cadangan:
- Kode baru bisa diminta melalui menu "Pengaturan MFA"
- Kode lama akan otomatis tidak berlaku
- Lakukan regenerasi jika kode lama terkompromi

---

## Bantuan dan Dukungan

### Kontak Helpdesk
- **Email**: helpdesk-simipelv2@kejaksaan.go.id
- **Telepon**: (021) 123-4567 ext. 890
- **Jam Kerja**: Senin-Jumat, 08:00-16:00 WIB

### Informasi yang Perlu Disiapkan:
1. **NIP** (Nomor Induk Pegawai)
2. **Nama Lengkap**
3. **Satker** (Satuan Kerja)
4. **Deskripsi Masalah** yang detail
5. **Screenshot** error jika ada

### FAQ Cepat

**Q: Apakah MFA wajib untuk semua pegawai?**
A: Ya, MFA wajib untuk semua pegawai Kejaksaan RI sesuai kebijakan keamanan.

**Q: Bisakah menggunakan SMS sebagai pengganti authenticator app?**
A: Tidak, sistem hanya mendukung TOTP melalui aplikasi authenticator untuk keamanan maksimal.

**Q: Berapa lama kode MFA berlaku?**
A: Kode berlaku selama 30 detik dan berubah otomatis.

**Q: Bisakah setup MFA di beberapa device?**
A: Ya, Anda bisa scan QR code yang sama di beberapa device sebagai backup.

**Q: Apa yang terjadi jika salah input kode berkali-kali?**
A: Akun akan terkunci sementara (15 menit) setelah 5 kali percobaan gagal.

### Keamanan dan Privasi
- Jangan pernah bagikan kode MFA dengan siapa pun
- Laporkan segera jika mencurigai akun terkompromi
- Logout selalu setelah selesai menggunakan sistem
- Gunakan jaringan internet yang aman (hindari WiFi publik)

---

**Dokumen ini terakhir diperbarui**: [Tanggal Update]
**Versi**: 1.0
**Kontak**: Tim IT Kejaksaan RI

*Untuk informasi teknis lebih lanjut, silakan merujuk ke dokumentasi administrator atau hubungi tim IT.*
