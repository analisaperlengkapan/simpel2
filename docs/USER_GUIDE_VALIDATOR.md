# Panduan Pengguna SIMPEL - Validator (Wilayah & Pusat)

## Daftar Isi

1. [Pengenalan](#pengenalan)
2. [Login dan Dashboard](#login-dan-dashboard)
3. [Review Kebutuhan BMN](#review-kebutuhan-bmn)
4. [Review Pakaian Dinas](#review-pakaian-dinas)
5. [Monitoring Pemakaian BMN](#monitoring-pemakaian-bmn)
6. [Review Penghapusan BMN](#review-penghapusan-bmn)
7. [Laporan dan Analisis](#laporan-dan-analisis)

---

## Pengenalan

**Peran Validator Wilayah:**

- Review dan forward pengajuan dari satker di wilayahnya
- Monitoring pemakaian BMN di wilayah
- Memberikan rekomendasi ke Validator Pusat

**Peran Validator Pusat:**

- Inisiasi periode kebutuhan BMN
- Analisis kelayakan dengan data SIMAN dan MySIMKARI
- Approval final untuk semua pengajuan
- Generate SK Penghapusan BMN
- Monitoring nasional

---

## Login dan Dashboard

### Login

1. Akses https://simpel.kejaksaan.go.id
2. Login dengan NIP dan password
3. Dashboard akan menampilkan:
   - Pengajuan menunggu review
   - Statistik wilayah/nasional
   - Notifikasi penting

### Dashboard Validator Wilayah

Menampilkan:

- Total pengajuan di wilayah
- Pengajuan menunggu review
- Pengajuan yang sudah diforward
- Statistik per satker

### Dashboard Validator Pusat

Menampilkan:

- Statistik nasional
- Gap analysis BMN
- Workflow metrics
- Asset utilization

---

## Review Kebutuhan BMN

### Validator Wilayah

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Kebutuhan BMN"** → **"Review Wilayah"**
   - Lihat pengajuan dengan status **SUBMITTED**

2. **Review Detail Pengajuan**
   - Klik pengajuan untuk melihat detail
   - Cek kelengkapan data:
     - Justifikasi kebutuhan
     - Dokumen pendukung
     - Jumlah dan jenis BMN

3. **Ambil Keputusan**

   **Opsi 1: Forward ke Pusat**
   - Jika data lengkap dan layak
   - Klik **"Forward ke Validator Pusat"**
   - Tambahkan catatan rekomendasi (opsional)

   **Opsi 2: Return ke Satker**
   - Jika data tidak lengkap atau tidak layak
   - Klik **"Return ke Satker"**
   - **Wajib** isi catatan revisi yang jelas
   - Satker akan menerima notifikasi

**Tips:**

- Cek konsistensi data dengan kebutuhan satker
- Verifikasi dokumen pendukung
- Berikan catatan yang konstruktif jika return

### Validator Pusat

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Kebutuhan BMN"** → **"Review Pusat"**
   - Lihat pengajuan dengan status **REVIEWED_WILAYAH**

2. **Analisis Kelayakan**
   - Sistem akan menampilkan:
     - **Data SIMAN**: BMN existing di satker
     - **Data MySIMKARI**: Jumlah pegawai (eselon, golongan, jaksa/TU)
     - **Gap Analysis**: Selisih kebutuhan vs existing
     - **Prioritization Score**: Skor prioritas otomatis

3. **Review dengan Data Terintegrasi**
   - Bandingkan kebutuhan dengan existing BMN
   - Cek rasio BMN per pegawai
   - Evaluasi justifikasi dengan data objektif

4. **Ambil Keputusan**

   **Opsi 1: Approve**
   - Jika layak berdasarkan analisis
   - Tentukan jumlah yang disetujui (bisa berbeda dari yang diajukan)
   - Klik **"Approve"**
   - Tambahkan catatan (opsional)

   **Opsi 2: Reject**
   - Jika tidak layak
   - Klik **"Reject"**
   - **Wajib** isi alasan penolakan yang jelas

5. **Generate Laporan Analisis**
   - Klik **"Generate Laporan"**
   - Pilih format: PDF, DOCX, atau XLSX
   - Laporan berisi:
     - Gap analysis
     - Rekomendasi
     - Data pendukung dari SIMAN dan MySIMKARI

**Tips:**

- Gunakan data SIMAN dan MySIMKARI untuk keputusan objektif
- Pertimbangkan prioritas berdasarkan skor sistem
- Dokumentasikan alasan keputusan dengan baik

### Inisiasi Periode (Validator Pusat)

**Langkah Membuat Periode Baru:**

1. **Buka Menu Period Management**
   - Menu **"Kebutuhan BMN"** → **"Period Management"**
   - Klik **"Buat Periode Baru"**

2. **Input Data Periode**
   - Nama periode: "Kebutuhan BMN 2024"
   - Tahun: 2024
   - Tanggal mulai: 01/01/2024
   - Tanggal selesai: 31/12/2024
   - Deadline submission: 31/03/2024

3. **Pilih Eligible BMN**
   - Pilih jenis BMN yang boleh diajukan
   - Contoh: Kendaraan, Laptop, Furniture
   - Klik **"Simpan"**

4. **Pilih Eligible Satker**
   - Pilih satker yang boleh mengajukan
   - Opsi:
     - **Semua satker**
     - **Sebagian satker** (pilih manual)
   - Klik **"Simpan"**

5. **Aktivasi Periode**
   - Review konfigurasi
   - Klik **"Aktivasi"**
   - Satker akan menerima notifikasi

---

## Review Pakaian Dinas

### Validator Wilayah (Kejati)

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Pakaian Dinas"** → **"Review Wilayah"**

2. **Review Data Pegawai**
   - Cek kelengkapan data ukuran
   - Verifikasi data pegawai (eselon, pangkat, jabatan)
   - Halaman menampilkan **tanggal pembekuan daftar pegawai** untuk periode ini
     dalam WIB ("Data pegawai dikunci per pengajuan ini: per <tanggal> WIB"),
     tanggal yang sama yang tercetak di header ekspor PDF/Excel. Daftar orangnya
     adalah yang dibekukan saat validator pusat membuat periode, sehingga angka
     yang Anda review sama dengan yang dilihat operator dan wilayah.

3. **Ambil Keputusan**
   - **Approve**: Forward ke Kejagung
   - **Revisi**: Return ke Kejari dengan catatan
   - Sebelum diteruskan, muncul konfirmasi **"Teruskan ke Validator Pusat?"**
     yang menyebut jumlah satker di wilayah Anda yang **belum mengisi lengkap**
     atau **belum mengajukan** ke wilayah. Penerusan **tidak diblokir**: Anda
     tetap boleh meneruskan, dan satker itu masih bisa dilengkapi bila
     pengajuannya dikembalikan. Bila semua sudah lengkap, konfirmasi tampil
     tanpa peringatan.
   - Untuk beberapa satker sekaligus, konfirmasi yang sama muncul sekali.
     Baris yang tahapnya sudah berpindah di antara Anda membuka dialog dan
     menekan konfirmasi **ditolak sendiri** dengan alasannya; sisanya tetap
     diproses.

### Validator Pusat (Kejagung)

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Pakaian Dinas"** → **"Review Pusat"**

2. **Review Agregat**
   - Lihat rekapitulasi per wilayah
   - Cek total kebutuhan per ukuran
   - Verifikasi budget availability

3. **Approve Final**
   - Klik **"Approve"**
   - Sebelum kegiatan diselesaikan, muncul konfirmasi **"Selesaikan Kegiatan
     Pakaian Dinas?"** yang menyebut jumlah satker yang belum mengisi
     lengkap/belum mengajukan ke wilayah, dan jumlah satker yang belum
     diteruskan ke pusat. Penyelesaian **tidak diblokir**, tetapi satker yang
     belum masuk tidak ikut tercatat pada kegiatan ini.
   - Yang diselesaikan adalah **satker yang Anda pilih**, bukan seluruh
     kegiatan: satker lain masih dapat menyusul (diisi, diajukan, dan
     diteruskan) selama periode kegiatan masih berjalan, lalu diselesaikan pada
     aksi berikutnya. Selesaikan hanya setelah satker yang Anda butuhkan pada
     periode ini sudah masuk.
   - Data akan masuk ke master pegawai_pakaian_dinas

4. **Generate Laporan**
   - **Laporan Daftar**: Per pegawai
   - **Laporan Rekap**: Per ukuran
   - Format: PDF atau Excel

### Memantau Progres Pengisian Pakaian Dinas

Di bagian atas **Daftar Satker** setiap kegiatan ada panel **Progres Kegiatan**
yang menjawab "mana yang sudah dan mana yang belum", dari ringkas ke rinci:

1. **Kegiatan.** Jumlah satker per tahap (belum mengajukan, di wilayah, di
   pusat, selesai) dan jumlah pegawai yang sudah mengisi ukuran dibanding yang
   seharusnya mengisi, lengkap dengan persentasenya. Batang di bawahnya
   menunjukkan komposisi tahap dengan warna yang sama dengan kartu di atasnya.
2. **Wilayah.** Tabel per Kejati, yang **paling sedikit mengisi di urutan
   teratas**. Delapan baris pertama tampil; tombol **Tampilkan semua** membuka
   sisanya. Satker yang tidak berada di bawah Kejati mana pun (Kejaksaan Agung
   dan unit pusat) dikelompokkan sendiri.
3. **Satker.** Daftar satker yang **paling tertinggal** (belum selesai, urut
   dari yang paling sedikit mengisi). Setiap barisnya punya tombol **Lihat
   pegawai**.
4. **Pegawai.** Tombol itu membuka halaman ukuran pegawai satker tersebut,
   tempat terlihat siapa yang sudah dan belum mengisi. Kartu satker di bawah
   panel juga menyebut "12 dari 15 pegawai sudah mengisi".

Ringkasan yang sama (batang tahap, "30 dari 156 satker selesai" dan jumlah
pegawai yang sudah mengisi) juga muncul di **setiap kartu kegiatan** pada
Daftar Pengajuan dan di bagian **Progres Pakaian Dinas** pada Dasbor, sehingga
kegiatan yang tertinggal terlihat tanpa membuka satu per satu. Kartu di Dasbor
membuka daftar satker kegiatan itu.

Yang tampil mengikuti cakupan Anda: Validator Wilayah melihat wilayahnya
sendiri, Validator Pusat melihat seluruh kegiatan. Pegawai yang **dikeluarkan**
operator tidak dihitung (mereka tidak berutang ukuran), dan formulir yang
disimpan kosong tidak dianggap sudah mengisi, sama dengan hitungan pada
konfirmasi sebelum meneruskan.

---

## Monitoring Pemakaian BMN

### Dashboard Monitoring

**Akses:**

- Menu **"Pemakaian BMN"** → **"Monitoring Dashboard"**

**Fitur:**

1. **Overview Tab**
   - Total izin aktif
   - Izin yang akan berakhir (30 hari)
   - Breakdown per jenis BMN
   - Breakdown per satker

2. **Utilization Report Tab**
   - Total BMN vs BMN terpakai
   - Tingkat utilisasi (%)
   - BMN paling sering digunakan
   - BMN yang jarang digunakan

**Filter:**

- Per satker (Validator Wilayah)
- Per wilayah (Validator Pusat)
- Per jenis BMN

**Export:**

- Klik **"Export"** untuk download laporan
- Format: Excel atau PDF

### Tindak Lanjut

**Jika utilisasi rendah:**

- Identifikasi BMN yang jarang digunakan
- Koordinasi dengan satker untuk optimalisasi
- Pertimbangkan redistribusi BMN

**Jika izin akan berakhir:**

- Sistem akan kirim reminder otomatis ke pegawai
- Monitor perpanjangan izin
- Cek ketersediaan BMN untuk pengguna baru

---

## Review Penghapusan BMN

### Validator Wilayah

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Penghapusan BMN"** → **"Review Wilayah"**

2. **Review Pengajuan**
   - Cek alasan penghapusan
   - Verifikasi dokumen pendukung (foto, berita acara)
   - Pastikan BMN tidak sedang digunakan

3. **Ambil Keputusan**
   - **Forward ke Pusat**: Jika layak
   - **Return ke Satker**: Jika tidak lengkap

### Validator Pusat

**Langkah Review:**

1. **Buka Daftar Pengajuan**
   - Menu **"Penghapusan BMN"** → **"Review Pusat"**

2. **Review dengan Data SIMAN**
   - Sistem akan menampilkan detail BMN dari SIMAN
   - Cek nilai perolehan
   - Cek kondisi terakhir
   - Cek riwayat pemakaian

3. **Approve dan Generate SK**
   - Klik **"Approve"**
   - Klik **"Generate SK Penghapusan"**
   - Sistem akan membuat SK dengan format resmi
   - SK number: SK/2024/001 (auto-generated)

4. **Upload SK Bertanda Tangan**
   - Download SK draft (DOCX)
   - Minta tanda tangan pejabat berwenang
   - Upload SK PDF yang sudah ditandatangani
   - Klik **"Upload"**

5. **Distribusi SK**
   - Satker dapat melihat dan download SK
   - SK dapat digunakan untuk proses di SIMAN

---

## Laporan dan Analisis

### Laporan Kebutuhan BMN

**Jenis Laporan:**

1. **Laporan Gap Analysis**
   - Gap per satker
   - Gap per jenis BMN
   - Prioritas kebutuhan

2. **Laporan Pemenuhan**
   - Realisasi vs rencana
   - Trend pemenuhan per tahun

3. **Laporan Roadmap Sarpras**
   - Rencana 5 tahun
   - Proyeksi kebutuhan

**Cara Generate:**

- Menu **"Laporan"** → Pilih jenis laporan
- Tentukan filter (tahun, wilayah, satker)
- Pilih format (PDF, Excel)
- Klik **"Generate"**

### Laporan Pemakaian BMN

**Jenis Laporan:**

1. **Laporan Utilisasi**
   - Tingkat utilisasi per jenis BMN
   - BMN dengan utilisasi tinggi/rendah

2. **Laporan Riwayat Pemakaian**
   - Per BMN: Siapa saja yang pernah menggunakan
   - Per Pegawai: BMN apa saja yang pernah digunakan

3. **Laporan Izin Aktif**
   - Daftar izin yang sedang berjalan
   - Izin yang akan berakhir

### Laporan Penghapusan BMN

**Jenis Laporan:**

1. **Laporan SK Penghapusan**
   - Daftar SK yang sudah diterbitkan
   - Total nilai BMN yang dihapus

2. **Laporan Metode Penghapusan**
   - Breakdown per metode (dijual, dihibahkan, dimusnahkan)

---

## Tips dan Best Practices

### Untuk Validator Wilayah

1. **Review Cepat tapi Teliti**
   - Prioritaskan pengajuan urgent
   - Gunakan checklist review

2. **Komunikasi dengan Satker**
   - Berikan feedback yang konstruktif
   - Koordinasi jika ada pertanyaan

3. **Monitoring Berkala**
   - Cek dashboard setiap hari
   - Follow up pengajuan yang pending

### Untuk Validator Pusat

1. **Gunakan Data untuk Keputusan**
   - Manfaatkan integrasi SIMAN dan MySIMKARI
   - Analisis gap secara objektif

2. **Konsistensi Keputusan**
   - Gunakan kriteria yang sama untuk semua satker
   - Dokumentasikan alasan keputusan

3. **Perencanaan Strategis**
   - Gunakan roadmap sarpras untuk perencanaan jangka panjang
   - Koordinasi dengan unit terkait

---

## Kontak dan Dukungan

**Helpdesk SIMPEL:**

- **Email:** helpdesk@simpel.kejaksaan.go.id
- **Telepon:** (021) 1234-5678
- **Jam Kerja:** Senin-Jumat, 08:00-16:00 WIB

**Panduan Video:**

- https://simpel.kejaksaan.go.id/panduan

---

**Versi:** 1.0.0
**Terakhir Diperbarui:** Februari 2026
