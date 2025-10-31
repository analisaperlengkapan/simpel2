# Dokumen Persyaratan

## Pendahuluan

Dokumen ini menjelaskan persyaratan untuk migrasi sistem Perlengkapan (Manajemen Perlengkapan & Rantai Pasokan) berbasis Laravel yang ada ke arsitektur microfrontend/microservice berbasis Rust modern dari SIMPelv2. Modul Perlengkapan mengelola aset milik negara (Barang Milik Negara/BMN) untuk Kejaksaan Agung RI, termasuk pelacakan aset, perencanaan pengadaan, pemeliharaan, penghapusan, dan pelaporan.

Migrasi ini akan mengubah aplikasi monolitik PHP/Laravel menjadi sistem terdistribusi dengan frontend Leptos/WebAssembly dan backend microservice berbasis Axum Rust, sambil mempertahankan paritas fitur dan meningkatkan performa, keamanan, dan maintainability.

## Glosarium

- **Sistem Perlengkapan**: Modul manajemen perlengkapan dan rantai pasokan dalam SIMPelv2
- **BMN**: Barang Milik Negara (Aset Milik Negara)
- **Satker**: Satuan Kerja - unit organisasi dalam Kejaksaan RI
- **Modul Frontend**: Microfrontend berbasis Leptos WebAssembly di antarmuka/pembinaan/perlengkapan
- **Layanan Backend**: Microservice berbasis Axum Rust di layanan/pembinaan/perlengkapan
- **Kategori Aset**: Klasifikasi BMN termasuk Tanah, Angkutan, Gedung, TIK, Non-TIK, Senjata, dll.
- **Analisis Kebutuhan**: Analisis kebutuhan untuk perencanaan pengadaan aset
- **Pengadaan**: Proses pengadaan untuk memperoleh aset baru
- **Perawatan**: Manajemen pemeliharaan untuk aset yang ada
- **Penghapusan**: Proses penghapusan aset (pemindahtanganan, pemusnahan, atau alasan lain)
- **Hibah**: Manajemen hibah/donasi aset
- **Ijin**: Manajemen izin penggunaan aset oleh pegawai
- **Kode QR**: Kode Quick Response untuk identifikasi dan pelacakan aset
- **Layanan Authenc**: Microservice autentikasi dan otorisasi
- **Layanan Secreton**: Microservice manajemen rahasia
- **Portal**: Microfrontend gateway utama yang meng-host semua microfrontend lainnya

## Persyaratan

### Persyaratan 1: Fungsi Inti Manajemen Aset

**User Story:** Sebagai petugas Perlengkapan, saya ingin mengelola semua jenis aset milik negara dengan operasi CRUD yang komprehensif, sehingga saya dapat memelihara catatan aset yang akurat di semua kategori.

#### Kriteria Penerimaan

1. WHEN pengguna mengakses antarmuka manajemen aset, THE Modul Frontend SHALL menampilkan daftar terkategorisasi dari semua jenis aset (Tanah, Angkutan, Gedung, TIK, Non-TIK, Senjata, Alat Besar, Rumah, Jalan Jembatan, Bangunan Air, Jaringan, Konstruksi, Lain, Renovasi, Wujud)
2. WHEN pengguna memilih kategori aset, THE Modul Frontend SHALL mengambil dan menampilkan data aset terpaginasi dengan kemampuan filtering dan sorting
3. WHEN pengguna membuat atau memperbarui catatan aset, THE Layanan Backend SHALL memvalidasi semua field yang diperlukan sesuai standar klasifikasi BMN
4. WHEN catatan aset dimodifikasi, THE Layanan Backend SHALL mencatat perubahan dalam audit trail dengan identifikasi pengguna dan timestamp
5. WHERE data aset mencakup geolokasi, THE Modul Frontend SHALL menampilkan antarmuka peta interaktif untuk input dan visualisasi koordinat

### Persyaratan 2: Identifikasi dan Pelabelan Aset

**User Story:** Sebagai petugas Perlengkapan, saya ingin menghasilkan dan mencetak label kode QR untuk aset, sehingga saya dapat memungkinkan identifikasi dan pelacakan cepat aset fisik.

#### Kriteria Penerimaan

1. WHEN pengguna meminta label kode QR untuk aset, THE Layanan Backend SHALL menghasilkan kode QR unik yang berisi identifier aset dan metadata
2. WHEN pengguna memindai kode QR, THE Modul Frontend SHALL mengambil dan menampilkan informasi aset lengkap
3. WHEN pengguna meminta pencetakan label massal, THE Layanan Backend SHALL menghasilkan dokumen PDF yang berisi beberapa label kode QR dengan format yang tepat
4. THE Layanan Backend SHALL mendukung pembuatan label dalam mode individual dan batch dengan ukuran label yang dapat dikonfigurasi
5. WHEN kode QR dipindai melalui perangkat mobile, THE Modul Frontend SHALL menampilkan tampilan detail aset yang dioptimalkan untuk mobile

### Persyaratan 3: Analisis Kebutuhan dan Perencanaan Pengadaan

**User Story:** Sebagai petugas perencanaan, saya ingin mengajukan dan melacak permintaan analisis kebutuhan untuk BMN dan pakaian dinas, sehingga perencanaan pengadaan dapat dikoordinasikan di seluruh unit organisasi.

#### Kriteria Penerimaan

1. WHEN pengguna Satker mengajukan permintaan analisis kebutuhan BMN, THE Layanan Backend SHALL membuat catatan pengajuan dengan pelacakan status
2. WHEN pengguna mengajukan kebutuhan pakaian dinas dengan data ukuran pegawai, THE Layanan Backend SHALL menyimpan pengukuran pegawai individual dan agregat kebutuhan
3. WHEN validator meninjau pengajuan analisis kebutuhan, THE Modul Frontend SHALL menampilkan semua detail pengajuan dengan kontrol alur kerja persetujuan
4. WHEN analisis kebutuhan disetujui atau ditolak, THE Layanan Backend SHALL memperbarui status pengajuan dan memberi notifikasi kepada pengaju
5. WHERE analisis kebutuhan memerlukan peringkat prioritas, THE Layanan Backend SHALL menghitung dan menyimpan skor prioritas berdasarkan kriteria yang ditentukan

### Persyaratan 4: Manajemen Penghapusan Aset

**User Story:** Sebagai koordinator penghapusan, saya ingin mengelola alur kerja penghapusan aset lengkap termasuk pengajuan, persetujuan, dan dokumentasi, sehingga penghapusan aset mematuhi peraturan pemerintah.

#### Kriteria Penerimaan

1. WHEN pengguna memulai permintaan penghapusan aset, THE Modul Frontend SHALL menyediakan formulir untuk berbagai jenis penghapusan (Pemindahtanganan, Pemusnahan, Sebab Lain)
2. WHEN permintaan penghapusan diajukan, THE Layanan Backend SHALL memvalidasi dokumentasi yang diperlukan termasuk foto dan file pendukung
3. WHEN permintaan penghapusan maju melalui tahap persetujuan, THE Layanan Backend SHALL melacak perubahan status dan memelihara riwayat persetujuan
4. WHEN penghapusan disetujui dengan SK (Surat Keputusan), THE Layanan Backend SHALL menyimpan dokumen keputusan dan memperbarui status aset
5. WHERE monitoring penghapusan diperlukan, THE Modul Frontend SHALL menampilkan status real-time dari semua permintaan penghapusan dengan filtering berdasarkan jenis dan status

### Persyaratan 5: Izin Aset dan Pelacakan Penggunaan

**User Story:** Sebagai manajer aset, saya ingin mengelola izin bagi pegawai untuk menggunakan aset tertentu, sehingga penggunaan aset diotorisasi dan dilacak dengan benar.

#### Kriteria Penerimaan

1. WHEN pengguna membuat izin penggunaan aset, THE Modul Frontend SHALL memungkinkan pemilihan beberapa pegawai dan beberapa aset
2. WHEN permintaan izin diajukan, THE Layanan Backend SHALL memvalidasi bahwa aset yang dipilih tersedia dan belum ditugaskan
3. WHEN izin disetujui, THE Layanan Backend SHALL menghasilkan dokumen izin dengan kode QR untuk verifikasi
4. WHEN izin kedaluwarsa atau dicabut, THE Layanan Backend SHALL memperbarui status ketersediaan aset
5. WHERE monitoring izin diperlukan, THE Modul Frontend SHALL menampilkan izin aktif dengan detail pegawai dan aset

### Persyaratan 6: Manajemen Pemeliharaan Aset

**User Story:** Sebagai koordinator pemeliharaan, saya ingin menjadwalkan dan melacak aktivitas pemeliharaan untuk aset, sehingga aset tetap dalam kondisi baik dan riwayat pemeliharaan terdokumentasi.

#### Kriteria Penerimaan

1. WHEN pengguna menjadwalkan pemeliharaan untuk aset, THE Layanan Backend SHALL membuat catatan pemeliharaan dengan tanggal terjadwal dan jenis pemeliharaan
2. WHEN pemeliharaan selesai, THE Modul Frontend SHALL memungkinkan entri detail pemeliharaan, biaya, dan dokumentasi pendukung
3. WHEN riwayat pemeliharaan diminta, THE Layanan Backend SHALL mengambil semua catatan pemeliharaan untuk aset yang ditentukan
4. THE Layanan Backend SHALL menghitung dan melacak total biaya pemeliharaan per aset dan per kategori
5. WHERE pemeliharaan preventif jatuh tempo, THE Layanan Backend SHALL menghasilkan notifikasi kepada personel yang bertanggung jawab

### Persyaratan 7: Manajemen Hibah dan Donasi

**User Story:** Sebagai koordinator hibah, saya ingin mengelola hibah dan donasi aset dengan dokumentasi lengkap, sehingga transfer aset mematuhi persyaratan hukum.

#### Kriteria Penerimaan

1. WHEN pengguna memulai proses hibah/donasi, THE Modul Frontend SHALL menyediakan formulir untuk informasi donor/penerima dan detail aset
2. WHEN hibah diajukan, THE Layanan Backend SHALL memvalidasi dokumen hukum yang diperlukan dan tanda tangan persetujuan
3. WHEN hibah disetujui, THE Layanan Backend SHALL memperbarui catatan kepemilikan aset dan menghasilkan dokumentasi transfer
4. THE Layanan Backend SHALL memelihara audit trail lengkap dari semua transaksi hibah
5. WHERE monitoring hibah diperlukan, THE Modul Frontend SHALL menampilkan status hibah dengan pelacakan dokumen

### Persyaratan 8: Pelaporan dan Analitik

**User Story:** Sebagai petugas manajemen, saya ingin menghasilkan laporan komprehensif tentang aset, pengadaan, dan aktivitas penghapusan, sehingga saya dapat menyediakan data akurat untuk pengambilan keputusan dan kepatuhan.

#### Kriteria Penerimaan

1. WHEN pengguna meminta laporan aset, THE Layanan Backend SHALL menghasilkan laporan yang difilter berdasarkan kategori, lokasi, status, dan rentang tanggal
2. WHEN pengguna mengekspor data laporan, THE Layanan Backend SHALL menyediakan output dalam format Excel dan PDF dengan format yang tepat
3. WHEN dashboard diakses, THE Modul Frontend SHALL menampilkan statistik real-time termasuk jumlah aset, nilai, dan distribusi status
4. THE Layanan Backend SHALL menghitung metrik agregat termasuk total nilai aset, depresiasi, dan biaya pemeliharaan
5. WHERE laporan kustom diperlukan, THE Modul Frontend SHALL menyediakan antarmuka pembuat laporan dengan parameter yang dapat dikonfigurasi

### Persyaratan 9: Integrasi dengan Sistem Eksternal

**User Story:** Sebagai administrator sistem, saya ingin sistem Perlengkapan terintegrasi dengan sistem eksternal (MONSAKTI, SIMKARI), sehingga sinkronisasi data otomatis dan konsisten.

#### Kriteria Penerimaan

1. WHEN data aset disinkronkan dengan MONSAKTI, THE Layanan Backend SHALL memetakan catatan aset ke format transaksi MONSAKTI
2. WHEN data pegawai diperlukan, THE Layanan Backend SHALL mengambil informasi pegawai terkini dari API SIMKARI
3. WHEN kesalahan integrasi terjadi, THE Layanan Backend SHALL mencatat detail kesalahan dan mencoba ulang dengan exponential backoff
4. THE Layanan Backend SHALL memelihara log integrasi dengan timestamp, data request/response, dan kode status
5. WHERE konflik data ada, THE Layanan Backend SHALL menandai konflik untuk resolusi manual

### Persyaratan 10: Autentikasi dan Otorisasi

**User Story:** Sebagai pengguna sistem, saya ingin autentikasi aman dengan kontrol akses berbasis peran, sehingga hanya personel yang berwenang yang dapat mengakses dan memodifikasi data aset.

#### Kriteria Penerimaan

1. WHEN pengguna mengakses modul Perlengkapan, THE Modul Frontend SHALL melakukan autentikasi melalui Layanan Authenc menggunakan token JWT
2. WHEN pengguna melakukan tindakan, THE Layanan Backend SHALL memverifikasi izin pengguna berdasarkan penugasan peran
3. WHEN sesi pengguna kedaluwarsa, THE Modul Frontend SHALL mengarahkan ke halaman login Portal
4. THE Layanan Backend SHALL menerapkan kontrol akses berbasis peran untuk semua endpoint API dengan peran termasuk Pelaksana, Validator Satker, Validator Wilayah, Validator Pusat, Admin
5. WHERE operasi sensitif dilakukan, THE Layanan Backend SHALL memerlukan pemeriksaan otorisasi tambahan

### Persyaratan 11: Audit Trail dan Logging Aktivitas

**User Story:** Sebagai petugas kepatuhan, saya ingin audit trail komprehensif dari semua aktivitas sistem, sehingga saya dapat melacak perubahan dan memastikan akuntabilitas.

#### Kriteria Penerimaan

1. WHEN modifikasi data terjadi, THE Layanan Backend SHALL mencatat identifier pengguna, timestamp, jenis tindakan, dan data yang diubah
2. WHEN log audit di-query, THE Layanan Backend SHALL menyediakan filtering berdasarkan pengguna, rentang tanggal, jenis tindakan, dan entitas
3. THE Layanan Backend SHALL menyimpan log audit dalam tabel audit terpisah dengan catatan yang tidak dapat diubah
4. THE Layanan Backend SHALL menyertakan alamat IP dan informasi user agent dalam catatan audit
5. WHERE laporan audit dihasilkan, THE Modul Frontend SHALL menampilkan ringkasan aktivitas dengan kemampuan drill-down

### Persyaratan 12: Upload File dan Manajemen Dokumen

**User Story:** Sebagai pengguna, saya ingin mengunggah dan mengelola dokumen pendukung untuk aset dan transaksi, sehingga semua dokumentasi relevan tersimpan secara terpusat dan dapat diakses.

#### Kriteria Penerimaan

1. WHEN pengguna mengunggah file, THE Layanan Backend SHALL memvalidasi jenis file, ukuran, dan memindai malware
2. WHEN file disimpan, THE Layanan Backend SHALL menghasilkan identifier unik dan menyimpan metadata termasuk nama file, ukuran, dan timestamp upload
3. WHEN pengguna mengunduh file, THE Layanan Backend SHALL menyajikan file dengan header content-type yang sesuai
4. THE Layanan Backend SHALL mendukung beberapa upload file per transaksi dengan batas ukuran total maksimum
5. WHERE file tidak lagi diperlukan, THE Layanan Backend SHALL menyediakan fungsi soft-delete dengan kebijakan retensi

### Persyaratan 13: Pencarian dan Filtering

**User Story:** Sebagai pengguna, saya ingin kemampuan pencarian dan filtering yang kuat di semua data aset, sehingga saya dapat dengan cepat menemukan aset atau transaksi tertentu.

#### Kriteria Penerimaan

1. WHEN pengguna memasukkan istilah pencarian, THE Modul Frontend SHALL melakukan pencarian real-time di nama aset, kode, dan deskripsi
2. WHEN pengguna menerapkan filter, THE Layanan Backend SHALL mengembalikan hasil yang cocok dengan semua kriteria filter yang dipilih
3. THE Modul Frontend SHALL menyediakan opsi filter untuk kategori aset, status, lokasi, rentang tanggal, dan rentang nilai
4. THE Layanan Backend SHALL mendukung pencarian full-text dengan peringkat relevansi
5. WHERE hasil pencarian melebihi batas tampilan, THE Modul Frontend SHALL menyediakan paginasi dengan ukuran halaman yang dapat dikonfigurasi

### Persyaratan 14: Responsivitas Mobile dan Aksesibilitas

**User Story:** Sebagai pengguna mobile, saya ingin antarmuka Perlengkapan bekerja dengan mulus di perangkat mobile, sehingga saya dapat mengakses informasi aset di lapangan.

#### Kriteria Penerimaan

1. WHEN Modul Frontend diakses di perangkat mobile, THE Modul Frontend SHALL menampilkan layout responsif yang dioptimalkan untuk layar kecil
2. WHEN pengguna berinteraksi dengan formulir di mobile, THE Modul Frontend SHALL menyediakan kontrol input yang ramah sentuh
3. THE Modul Frontend SHALL mematuhi standar aksesibilitas WCAG 2.1 AA
4. THE Modul Frontend SHALL mendukung tampilan data offline dengan caching service worker
5. WHERE fitur khusus mobile tersedia, THE Modul Frontend SHALL mengaktifkan akses kamera untuk pemindaian kode QR

### Persyaratan 15: Performa dan Skalabilitas

**User Story:** Sebagai administrator sistem, saya ingin sistem Perlengkapan menangani penggunaan konkuren tinggi dengan waktu respons cepat, sehingga produktivitas pengguna tidak terdampak.

#### Kriteria Penerimaan

1. WHEN Modul Frontend dimuat, THE Modul Frontend SHALL mencapai First Contentful Paint dalam 1.5 detik
2. WHEN permintaan API dibuat, THE Layanan Backend SHALL merespons dalam 200 milidetik untuk 95% permintaan
3. THE Layanan Backend SHALL mendukung setidaknya 1000 pengguna konkuren tanpa degradasi performa
4. THE Layanan Backend SHALL mengimplementasikan connection pooling database dengan ukuran pool yang dapat dikonfigurasi
5. WHERE dataset besar diambil, THE Layanan Backend SHALL mengimplementasikan paginasi berbasis cursor untuk meminimalkan penggunaan memori
