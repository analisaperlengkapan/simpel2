# Panduan Pengguna SIMPEL - Operator Satker

## Daftar Isi

1. [Pengenalan](#pengenalan)
2. [Login dan Akses](#login-dan-akses)
3. [Kebutuhan BMN](#kebutuhan-bmn)
4. [Pakaian Dinas](#pakaian-dinas)
5. [Pemakaian BMN](#pemakaian-bmn)
6. [Penghapusan BMN](#penghapusan-bmn)
7. [FAQ](#faq)

---

## Pengenalan

SIMPEL (Sistem Informasi Manajemen Perlengkapan) adalah sistem untuk mengelola Barang Milik Negara (BMN) di lingkungan Kejaksaan RI.

**Peran Operator Satker:**
- Menginput kebutuhan BMN satker
- Mengajukan kebutuhan pakaian dinas pegawai
- Membuat izin pemakaian BMN
- Mengajukan penghapusan BMN

---

## Login dan Akses

### Cara Login

1. Buka browser dan akses https://simpel.kejaksaan.go.id
2. Klik tombol **"Login"**
3. Masukkan **NIP** dan **Password** Anda
4. Klik **"Masuk"**

### Lupa Password

1. Klik **"Lupa Password?"** di halaman login
2. Masukkan NIP Anda
3. Ikuti instruksi yang dikirim ke email Anda

### Dashboard

Setelah login, Anda akan melihat dashboard dengan:
- **Ringkasan**: Total pengajuan, status, dan notifikasi
- **Menu Utama**: Akses ke semua modul
- **Notifikasi**: Pemberitahuan terbaru

---

## Kebutuhan BMN

### Mengajukan Kebutuhan BMN

**Langkah-langkah:**

1. **Buka Menu Kebutuhan BMN**
   - Klik **"Kebutuhan BMN"** di menu utama
   - Pilih **"Pengajuan Aktif"**

2. **Pilih Periode**
   - Pilih periode pengajuan yang sedang aktif
   - Klik **"Buat Pengajuan"**

3. **Input Data Kebutuhan**
   - Pilih jenis BMN yang dibutuhkan
   - Masukkan jumlah kebutuhan
   - Isi justifikasi kebutuhan (wajib)
   - Upload dokumen pendukung (surat permohonan, dll)

4. **Simpan Draft**
   - Klik **"Simpan Draft"** untuk menyimpan sementara
   - Anda dapat melanjutkan nanti

5. **Submit Pengajuan**
   - Pastikan semua data sudah lengkap
   - Klik **"Submit ke Validator Wilayah"**
   - Pengajuan tidak dapat diubah setelah disubmit

**Tips:**
- Pastikan justifikasi jelas dan lengkap
- Upload dokumen pendukung yang relevan
- Cek ketersediaan BMN existing di SIMAN sebelum mengajukan

### Melihat Status Pengajuan

1. Buka **"Kebutuhan BMN"** → **"Daftar Pengajuan"**
2. Status pengajuan:
   - **DRAFT**: Masih dalam proses input
   - **SUBMITTED**: Sudah disubmit, menunggu review
   - **REVIEWED_WILAYAH**: Sedang direview Validator Wilayah
   - **REVIEWED_PUSAT**: Sedang direview Validator Pusat
   - **APPROVED**: Disetujui
   - **REJECTED**: Ditolak

### Revisi Pengajuan

Jika pengajuan dikembalikan untuk revisi:

1. Buka pengajuan yang dikembalikan
2. Lihat catatan revisi dari validator
3. Perbaiki data sesuai catatan
4. Klik **"Submit Ulang"**

---

## Pakaian Dinas

### Mengajukan Kebutuhan Pakaian Dinas

**Langkah-langkah:**

1. **Buka Menu Pakaian Dinas**
   - Klik **"Pakaian Dinas"** di menu utama
   - Pilih periode yang aktif

2. **Input Ukuran Pegawai**
   - Sistem akan menampilkan daftar pegawai dari MySIMKARI
   - Untuk setiap pegawai, input:
     - Ukuran baju (S, M, L, XL, XXL, XXXL)
     - Ukuran celana (28, 30, 32, 34, 36, 38, 40)
     - Ukuran sepatu (38, 39, 40, 41, 42, 43, 44, 45)
   - Untuk pegawai perempuan, centang **"Dengan Hijab"** jika diperlukan

3. **Simpan Data**
   - Klik **"Simpan"** setelah semua ukuran diinput
   - Data akan tersimpan otomatis

4. **Submit Pengajuan**
   - Setelah semua pegawai diinput, klik **"Submit"**
   - Pengajuan akan masuk ke alur persetujuan 3 tingkat:
     1. Kejari (Pimpinan Satker)
     2. Kejati (Validator Wilayah)
     3. Kejagung (Validator Pusat)

**Tips:**
- Pastikan ukuran akurat untuk menghindari revisi
- Cek data pegawai (eselon, pangkat, jabatan) sudah benar
- Untuk pegawai baru, pastikan sudah terdaftar di MySIMKARI

### Melihat Laporan

1. Buka **"Pakaian Dinas"** → **"Laporan"**
2. Pilih jenis laporan:
   - **Laporan Daftar**: Daftar pegawai dengan ukuran
   - **Laporan Rekap**: Rekapitulasi per ukuran
3. Pilih format: PDF atau Excel
4. Klik **"Download"**

---

## Pemakaian BMN

### Membuat Izin Pemakaian BMN

**Langkah-langkah:**

1. **Buka Menu Pemakaian BMN**
   - Klik **"Pemakaian BMN"** di menu utama
   - Klik **"Buat Izin Baru"**

2. **Pilih Pegawai**
   - Pilih pegawai dari dropdown (data dari MySIMKARI)
   - Sistem akan menampilkan foto dan data pegawai

3. **Pilih BMN**
   - Klik **"Pilih BMN"**
   - Sistem akan menampilkan BMN yang tersedia
   - Pilih satu atau lebih BMN
   - Sistem akan validasi ketersediaan (satu BMN = satu izin aktif)

4. **Tentukan Periode**
   - Masukkan tanggal mulai
   - Masukkan tanggal selesai

5. **Generate Dokumen**
   - Klik **"Generate Dokumen Konsep"**
   - Sistem akan membuat dokumen DOCX dengan:
     - Halaman 1: Data pegawai + foto
     - Halaman 2+: Tabel BMN
   - Download dokumen

6. **Upload Dokumen Bertanda Tangan**
   - Minta tanda tangan Pimpinan Satker
   - Upload dokumen PDF yang sudah ditandatangani
   - Klik **"Upload"**
   - Status izin menjadi **COMPLETED**

**Tips:**
- Cek ketersediaan BMN sebelum memilih
- Pastikan periode pemakaian sesuai kebutuhan
- Simpan dokumen yang sudah ditandatangani dengan baik

### Memperpanjang Izin

Jika izin akan berakhir:

1. Buka izin yang akan berakhir
2. Klik **"Perpanjang Izin"**
3. Tentukan periode baru
4. Ulangi proses generate dan upload dokumen

### Notifikasi Kadaluarsa

Sistem akan mengirim notifikasi:
- **H-30**: 30 hari sebelum berakhir
- **H-14**: 14 hari sebelum berakhir
- **H-7**: 7 hari sebelum berakhir

---

## Penghapusan BMN

### Mengajukan Penghapusan BMN

**Langkah-langkah:**

1. **Buka Menu Penghapusan BMN**
   - Klik **"Penghapusan BMN"** di menu utama
   - Klik **"Buat Pengajuan Baru"**

2. **Pilih BMN**
   - Pilih BMN yang akan dihapus dari daftar
   - Sistem akan validasi BMN tidak sedang digunakan

3. **Input Alasan**
   - Pilih alasan penghapusan:
     - Rusak berat tidak dapat diperbaiki
     - Hilang
     - Kadaluarsa
     - Lainnya
   - Isi deskripsi detail

4. **Pilih Metode Penghapusan**
   - Dijual
   - Dihibahkan
   - Dimusnahkan

5. **Upload Dokumen Pendukung**
   - Upload foto kondisi BMN
   - Upload berita acara (jika ada)
   - Upload dokumen pendukung lainnya

6. **Submit Pengajuan**
   - Klik **"Submit ke Validator Wilayah"**
   - Pengajuan akan masuk ke alur persetujuan

**Tips:**
- Pastikan BMN tidak sedang digunakan (tidak ada izin aktif)
- Upload foto kondisi BMN yang jelas
- Isi alasan dengan lengkap dan detail

### Melihat SK Penghapusan

Setelah disetujui Validator Pusat:

1. Buka pengajuan yang sudah disetujui
2. Klik **"Lihat SK Penghapusan"**
3. Download SK dalam format PDF
4. SK dapat digunakan untuk proses penghapusan di SIMAN

---

## FAQ

### Pertanyaan Umum

**Q: Bagaimana cara mengubah password?**
A: Klik profil Anda di pojok kanan atas → **"Ubah Password"**

**Q: Saya lupa password, bagaimana?**
A: Klik **"Lupa Password?"** di halaman login dan ikuti instruksi

**Q: Pengajuan saya ditolak, apa yang harus dilakukan?**
A: Baca catatan penolakan dari validator, perbaiki data, dan submit ulang

**Q: BMN yang saya butuhkan tidak ada di daftar, bagaimana?**
A: Hubungi Validator Pusat untuk menambahkan BMN ke daftar eligible

**Q: Izin pemakaian saya akan berakhir, bagaimana memperpanjang?**
A: Buka izin → Klik **"Perpanjang Izin"** → Tentukan periode baru

**Q: Dokumen yang saya upload salah, bisa diubah?**
A: Hubungi Admin untuk menghapus dokumen, lalu upload ulang

**Q: Notifikasi tidak masuk ke email, kenapa?**
A: Cek folder spam/junk email Anda. Jika masih tidak ada, hubungi Admin

### Kontak Bantuan

**Helpdesk SIMPEL:**
- **Email:** helpdesk@simpel.kejaksaan.go.id
- **Telepon:** (021) 1234-5678
- **Jam Kerja:** Senin-Jumat, 08:00-16:00 WIB

**Panduan Video:**
- Kunjungi https://simpel.kejaksaan.go.id/panduan untuk video tutorial

---

**Versi:** 1.0.0
**Terakhir Diperbarui:** Februari 2026
