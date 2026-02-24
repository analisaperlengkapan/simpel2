# Panduan Troubleshooting MFA SIMPEL

## Daftar Isi
1. [Masalah Setup MFA](#masalah-setup-mfa)
2. [Masalah Login dengan MFA](#masalah-login-dengan-mfa)
3. [Masalah Aplikasi Authenticator](#masalah-aplikasi-authenticator)
4. [Masalah Kode Cadangan](#masalah-kode-cadangan)
5. [Masalah Teknis](#masalah-teknis)
6. [Recovery dan Reset](#recovery-dan-reset)
7. [Kontak Darurat](#kontak-darurat)

---

## Masalah Setup MFA

### 🔴 QR Code Tidak Muncul di Halaman Setup

**Gejala**: Halaman setup MFA kosong atau QR code tidak tampil

**Penyebab Umum**:
- Koneksi internet lambat
- Browser tidak mendukung
- JavaScript diblokir
- Cache browser bermasalah

**Solusi**:
1. **Refresh Halaman**:
   - Tekan Ctrl+F5 (Windows) atau Cmd+Shift+R (Mac)
   - Tunggu beberapa detik untuk loading

2. **Periksa Koneksi Internet**:
   - Test buka website lain
   - Pastikan koneksi stabil
   - Coba gunakan jaringan berbeda

3. **Ganti Browser**:
   - Coba Chrome, Firefox, atau Edge terbaru
   - Pastikan JavaScript enabled
   - Disable ad blocker sementara

4. **Clear Cache Browser**:
   - Chrome: Ctrl+Shift+Delete → Clear browsing data
   - Firefox: Ctrl+Shift+Delete → Clear recent history
   - Pilih "All time" dan centang semua opsi

### 🔴 QR Code Tidak Bisa Di-scan

**Gejala**: Aplikasi authenticator tidak bisa membaca QR code

**Penyebab Umum**:
- Kualitas kamera buruk
- Pencahayaan kurang
- Jarak terlalu dekat/jauh
- QR code terpotong di layar

**Solusi**:
1. **Optimasi Scanning**:
   - Bersihkan lensa kamera ponsel
   - Atur jarak 10-15 cm dari layar
   - Pastikan pencahayaan cukup terang
   - Pegang ponsel steady, jangan goyang

2. **Atur Tampilan Layar**:
   - Zoom out browser jika QR code terpotong
   - Atur brightness layar ke maksimal
   - Matikan mode gelap/dark mode browser

3. **Gunakan Setup Manual**:
   - Klik "Tidak bisa scan? Masukkan manual"
   - Salin kode rahasia yang ditampilkan
   - Input manual di aplikasi authenticator

### 🔴 Kode Verifikasi Pertama Selalu Salah

**Gejala**: Kode 6 digit dari authenticator ditolak saat setup

**Penyebab Umum**:
- Waktu ponsel tidak sinkron
- Zona waktu salah
- Kode sudah expired
- Setup authenticator salah

**Solusi**:
1. **Sinkronisasi Waktu**:
   - **Android**: Settings → Date & time → Automatic date & time (ON)
   - **iOS**: Settings → General → Date & Time → Set Automatically (ON)
   - Restart aplikasi authenticator

2. **Periksa Zona Waktu**:
   - Pastikan zona waktu sesuai lokasi Indonesia
   - WIB (UTC+7), WITA (UTC+8), WIT (UTC+9)

3. **Tunggu Kode Baru**:
   - Jangan gunakan kode yang hampir expired (detik 25-30)
   - Tunggu kode baru muncul (awal siklus 30 detik)
   - Input kode segera setelah muncul

4. **Verifikasi Setup Authenticator**:
   - Pastikan akun name benar: [NIP]@kejaksaan.go.id
   - Pastikan issuer: SIMPEL Kejaksaan RI
   - Hapus dan setup ulang jika perlu

---

## Masalah Login dengan MFA

### 🔴 Tidak Diarahkan ke Halaman MFA Setelah Login

**Gejala**: Setelah input password, langsung masuk dashboard (bypass MFA)

**Penyebab Umum**:
- MFA belum diaktifkan untuk akun
- Session lama masih aktif
- Browser cache masalah

**Solusi**:
1. **Logout Lengkap**:
   - Klik logout di dashboard
   - Clear cookies browser
   - Close semua tab browser
   - Buka browser baru dan login ulang

2. **Periksa Status MFA**:
   - Hubungi admin untuk cek status MFA akun
   - Pastikan MFA sudah diaktifkan di sistem

3. **Gunakan Incognito/Private Mode**:
   - Buka browser dalam mode incognito
   - Login ulang untuk test

### 🔴 Kode MFA Selalu Ditolak Saat Login

**Gejala**: Kode dari authenticator selalu salah saat verifikasi login

**Penyebab Umum**:
- Waktu tidak sinkron
- Menggunakan kode lama
- Akun authenticator salah
- Rate limiting aktif

**Solusi**:
1. **Periksa Waktu dan Zona**:
   - Sinkronkan waktu otomatis di ponsel
   - Pastikan zona waktu Indonesia
   - Restart aplikasi authenticator

2. **Gunakan Kode Fresh**:
   - Tunggu kode baru muncul (awal siklus)
   - Input segera setelah kode berubah
   - Jangan tunggu sampai hampir expired

3. **Verifikasi Akun Authenticator**:
   - Pastikan menggunakan akun SIMPEL yang benar
   - Hapus akun duplikat jika ada
   - Periksa nama akun dan issuer

4. **Tunggu Rate Limit**:
   - Jika terlalu banyak percobaan, tunggu 15 menit
   - Jangan coba input kode sembarangan

### 🔴 Akun Terkunci Karena Terlalu Banyak Percobaan

**Gejala**: Pesan "Akun terkunci, coba lagi dalam X menit"

**Penyebab**: Lebih dari 5 kali input kode MFA salah

**Solusi**:
1. **Tunggu Unlock Otomatis**:
   - Akun akan unlock otomatis setelah 15 menit
   - Jangan coba login selama masa tunggu
   - Gunakan waktu untuk troubleshoot masalah

2. **Persiapkan untuk Login Ulang**:
   - Pastikan waktu ponsel sudah sinkron
   - Siapkan kode cadangan sebagai backup
   - Pastikan koneksi internet stabil

3. **Hubungi Admin untuk Unlock Manual**:
   - Jika urgent dan tidak bisa tunggu
   - Siapkan identitas untuk verifikasi
   - Email: helpdesk-simipelv2@kejaksaan.go.id

---

## Masalah Aplikasi Authenticator

### 🔴 Aplikasi Authenticator Crash atau Freeze

**Gejala**: App authenticator tidak bisa dibuka atau hang

**Solusi**:
1. **Force Close dan Restart**:
   - **Android**: Recent apps → Swipe up aplikasi
   - **iOS**: Double tap home → Swipe up aplikasi
   - Buka ulang aplikasi

2. **Restart Ponsel**:
   - Matikan ponsel sepenuhnya
   - Tunggu 10 detik, nyalakan kembali
   - Coba buka aplikasi lagi

3. **Update Aplikasi**:
   - Buka Play Store/App Store
   - Cari aplikasi authenticator
   - Tap "Update" jika tersedia

4. **Clear Cache (Android)**:
   - Settings → Apps → [Authenticator App]
   - Storage → Clear Cache
   - Jangan pilih "Clear Data" (akan hapus akun)

### 🔴 Akun SIMPEL Hilang dari Authenticator

**Gejala**: Akun SIMPEL tidak muncul di daftar authenticator

**Penyebab Umum**:
- Aplikasi di-reinstall tanpa backup
- Akun terhapus tidak sengaja
- Restore backup gagal

**Solusi**:
1. **Periksa Backup/Sync**:
   - **Google Auth**: Periksa Google Account sync
   - **Microsoft Auth**: Login ulang dengan akun Microsoft
   - **Authy**: Periksa sinkronisasi nomor telepon

2. **Restore dari Backup**:
   - Cari file backup jika menggunakan FreeOTP
   - Import backup jika tersedia

3. **Setup Ulang MFA**:
   - Jika tidak ada backup, hubungi admin
   - Minta reset MFA untuk setup ulang
   - Gunakan kode cadangan untuk akses sementara

### 🔴 Kode Tidak Berubah atau Stuck

**Gejala**: Kode 6 digit tidak berubah setelah 30 detik

**Solusi**:
1. **Refresh Manual**:
   - **FreeOTP**: Tap pada akun untuk refresh
   - **Lainnya**: Pull down untuk refresh atau restart app

2. **Periksa Koneksi Waktu**:
   - Pastikan ponsel terhubung internet
   - Sinkronkan waktu otomatis
   - Restart aplikasi setelah sinkronisasi

3. **Reinstall Aplikasi**:
   - Backup akun terlebih dahulu
   - Uninstall dan install ulang aplikasi
   - Restore dari backup

---

## Masalah Kode Cadangan

### 🔴 Kode Cadangan Tidak Diterima

**Gejala**: Kode cadangan 8 digit ditolak sistem

**Penyebab Umum**:
- Kode sudah pernah digunakan
- Salah ketik kode
- Kode sudah expired/regenerated

**Solusi**:
1. **Periksa Kode yang Belum Digunakan**:
   - Setiap kode hanya bisa digunakan sekali
   - Coba kode cadangan lain yang belum dipakai
   - Coret kode yang sudah digunakan

2. **Periksa Format Input**:
   - Pastikan input 8 digit tanpa spasi
   - Jangan tambahkan karakter lain
   - Copy-paste jika perlu untuk menghindari typo

3. **Hubungi Admin untuk Regenerasi**:
   - Jika semua kode sudah habis
   - Minta generate kode cadangan baru
   - Kode lama akan otomatis invalid

### 🔴 Kode Cadangan Hilang atau Lupa Disimpan

**Gejala**: Tidak ingat atau tidak punya kode cadangan

**Solusi**:
1. **Cari di Tempat Penyimpanan**:
   - Periksa email saat setup MFA
   - Cari di password manager
   - Periksa foto di galeri ponsel
   - Cari di dokumen/file komputer

2. **Hubungi Admin untuk Reset**:
   - Email: helpdesk-simipelv2@kejaksaan.go.id
   - Siapkan identitas untuk verifikasi
   - Jelaskan situasi kehilangan kode

3. **Pencegahan di Masa Depan**:
   - Simpan kode di tempat aman
   - Buat backup di beberapa lokasi
   - Catat di password manager

---

## Masalah Teknis

### 🔴 Error "Server Tidak Merespon" Saat MFA

**Gejala**: Pesan error koneksi saat verifikasi MFA

**Solusi**:
1. **Periksa Koneksi Internet**:
   - Test buka website lain
   - Coba ganti jaringan WiFi/mobile data
   - Restart router jika perlu

2. **Periksa Status Server**:
   - Tanya rekan kerja apakah mengalami masalah sama
   - Hubungi helpdesk untuk konfirmasi status server

3. **Coba Lagi Nanti**:
   - Tunggu beberapa menit dan coba lagi
   - Server mungkin sedang maintenance

### 🔴 Halaman MFA Tidak Loading atau Blank

**Gejala**: Halaman verifikasi MFA kosong atau tidak muncul

**Solusi**:
1. **Refresh dan Clear Cache**:
   - Hard refresh: Ctrl+F5
   - Clear browser cache dan cookies
   - Restart browser

2. **Ganti Browser**:
   - Coba browser lain (Chrome, Firefox, Edge)
   - Pastikan browser versi terbaru
   - Disable extension yang mungkin mengganggu

3. **Periksa JavaScript**:
   - Pastikan JavaScript enabled di browser
   - Disable ad blocker sementara
   - Coba mode incognito/private

### 🔴 Session Expired Terus Menerus

**Gejala**: Harus login ulang setiap beberapa menit

**Solusi**:
1. **Periksa Pengaturan Browser**:
   - Pastikan cookies enabled
   - Jangan gunakan mode private/incognito untuk kerja
   - Disable auto-clear cookies

2. **Periksa Koneksi**:
   - Pastikan koneksi internet stabil
   - Jangan ganti jaringan saat sedang login

3. **Hubungi Admin**:
   - Mungkin ada pengaturan session timeout yang terlalu pendek
   - Laporkan masalah untuk investigasi

---

## Recovery dan Reset

### 🔴 Ponsel Hilang/Rusak - Tidak Bisa Akses Authenticator

**Situasi Darurat**: Ponsel hilang, rusak, atau dicuri

**Langkah Darurat**:
1. **Gunakan Kode Cadangan**:
   - Ambil kode cadangan yang disimpan
   - Login dengan kode cadangan
   - Setup MFA di ponsel baru segera

2. **Hubungi Admin Segera**:
   - Email: helpdesk-simipelv2@kejaksaan.go.id
   - Telepon: (021) 123-4567 ext. 890
   - Jelaskan situasi darurat

3. **Verifikasi Identitas**:
   - Siapkan dokumen identitas (KTP, ID pegawai)
   - Berikan informasi akun (NIP, nama, satker)
   - Mungkin perlu konfirmasi dari atasan

### 🔴 Reset MFA Lengkap

**Kapan Diperlukan**:
- Semua kode cadangan habis
- Tidak bisa akses authenticator sama sekali
- Akun authenticator corrupt/hilang

**Proses Reset**:
1. **Hubungi Helpdesk**:
   - Email detail masalah ke helpdesk
   - Sertakan screenshot error jika ada
   - Berikan informasi lengkap akun

2. **Verifikasi Identitas**:
   - Admin akan minta verifikasi identitas
   - Mungkin perlu approval dari atasan
   - Proses bisa memakan waktu 1-2 hari kerja

3. **Setup Ulang**:
   - Setelah reset, login akan redirect ke setup MFA
   - Ikuti proses setup dari awal
   - Simpan kode cadangan baru dengan aman

### 🔴 Migrasi ke Ponsel Baru

**Langkah Aman Migrasi**:
1. **Sebelum Ganti Ponsel**:
   - Backup authenticator app jika mendukung
   - Catat kode cadangan yang tersisa
   - Screenshot QR code jika masih ada akses

2. **Setup di Ponsel Baru**:
   - Install aplikasi authenticator yang sama
   - Restore dari backup jika tersedia
   - Atau setup ulang dengan QR code

3. **Verifikasi dan Cleanup**:
   - Test login dengan ponsel baru
   - Hapus authenticator dari ponsel lama
   - Generate kode cadangan baru jika perlu

---

## Kontak Darurat

### Helpdesk SIMPEL
- **Email**: helpdesk-simipelv2@kejaksaan.go.id
- **Telepon**: (021) 123-4567 ext. 890
- **WhatsApp**: +62-812-3456-7890 (hanya darurat)
- **Jam Kerja**: Senin-Jumat, 08:00-16:00 WIB

### Informasi yang Perlu Disiapkan Saat Hubungi Helpdesk:
1. **Data Pribadi**:
   - NIP (Nomor Induk Pegawai)
   - Nama lengkap
   - Satker (Satuan Kerja)
   - Jabatan

2. **Detail Masalah**:
   - Deskripsi masalah yang detail
   - Kapan masalah mulai terjadi
   - Langkah yang sudah dicoba
   - Screenshot error jika ada

3. **Informasi Teknis**:
   - Browser yang digunakan
   - Sistem operasi (Windows/Mac/Linux)
   - Aplikasi authenticator yang digunakan
   - Versi aplikasi jika tahu

### Tingkat Prioritas Masalah:

**🔴 URGENT (Response < 2 jam)**:
- Tidak bisa login sama sekali
- Ponsel hilang/dicuri dengan akses sistem penting
- Suspek akun di-hack

**🟡 HIGH (Response < 4 jam)**:
- MFA tidak berfungsi tapi masih bisa akses dengan kode cadangan
- Masalah setup MFA untuk user baru

**🟢 NORMAL (Response < 24 jam)**:
- Pertanyaan umum tentang MFA
- Request regenerasi kode cadangan
- Masalah minor aplikasi authenticator

### Self-Service Resources:
- **Knowledge Base**: https://simipelv2.kejaksaan.go.id/help
- **Video Tutorial**: https://simipelv2.kejaksaan.go.id/video-mfa
- **FAQ**: https://simipelv2.kejaksaan.go.id/faq-mfa

---

**Tips Pencegahan**:
- Selalu backup kode cadangan di tempat aman
- Setup MFA di 2 device jika memungkinkan
- Update aplikasi authenticator secara berkala
- Jangan share kode MFA dengan siapa pun
- Laporkan masalah segera sebelum menjadi urgent

**Dokumen ini terakhir diperbarui**: [Tanggal Update]
**Versi**: 1.0
