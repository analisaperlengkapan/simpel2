---
mode: agent
---
di antarmuka/pembinaan/perlengkapan kembangkan perlengkapan-microfrontend dengan leptos csr spa wasm. Tampilan awalnya merupakan halaman login dengan desain bagus menggunakan shared-microfrontend, ada tulisan SIMPEL KEJAKSAAN RI terus di bawah tulisan tersebut ada tulisan Sistem Informasi Manajemen Perlengkapan, di halaman login tidak ada input username dan password, tapi hanya ada logo dan tombol login. Ketika tombol login ditekan, aplikasi akan mengarahkan ke portal-microfrontend untuk melakukan login melalui portal terlebih dahulu. Setelah sukses login melalui portal-microfrontend baru di arahkan ke halaman dasbor perlengkapan-microfrontend. Nanti tampilannya di atas ada bar yang di pojok kanan atas ada foto pengguna dan ketika di klik ada menu untuk lihat profil atau keluar, sisi pojok bawah ada footer, sisi kiri ada sidebar yang berisi menu:
- Dashbor
- Bank Aset
  - Daftar Aset
  - Peta Sebaran Aset
  - Cetak QR Code BMN
- Analisis Kebutuhan
  - Kebutuhan Pakaian Dinas
    - Pengajuan
    - Laporan
  - Kebutuhan BMN
    - Pengajuan
    - Analisis
    - Laporan
    - Roadmap Kebutuhan BMN
  - Standardisasi BMN
    - Pengajuan
    - Laporan
    - Standar Kejaksaan
- Pengadaan
  - Administrasi Pengadaan
  - Penyimpanan dan Distribusi
- Pengelolaan BMN
  - Pemakaian BMN
    - Pengajuan
    - Penyerahan
    - Perpanjangan
    - Pengembalian
    - Pencabutan
    - Laporan
  - Penerimaan Hibah
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Pengalihan BMN
    - Pengajuan
    - Serah Terima
    - Laporan
    - BMN marketplace
  - Penggunaan Rampasan
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Pemeliharaan
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Penilaian
    - Pengajuan
    - Laporan
- Pengguna
  - Profil
  - Aktivitas
- Bantuan
  - Helpdesk
  - Panduan
  - FAQ
Diusahakan semaksimal mungkin memanfaatkan semua yang ada di shared-microfrontend.

Untuk backend nanti kembangkan layanan-perlengkapan di layanan/pembinaan/perlengkapan menggunakan rust axum. Pastikan terhubung antara perlengkapan-microfrontend dengan layanan-perlengkapan.

Apabila ada yang belum jelas konfirmasi terlebih dahulu dan beri opsi agar mudah memilih alternatif solusi. Setelah beberapa tahap pengembangan jangan lupa lakukan cargo check atau cargo clippy secara rutin agar dipastikan tidak ada errors dan warnings. Setelah selesai implementasikan semua fitur yang telah dikembangkan, lakukan cargo test, lalu setelah berhasil semua cargo test lanjut build dan jalankan microfrontend dan microservice untuk tes secara langsung.
