//! Pakaian dinas: referensi ukuran (grup, daftar ukuran) dan profil ukuran per spesifikasi (V019).
//!
//! Fitur baru mengikuti struktur feature-first (`antarmuka/AGENTS.md`): halaman dan klien HTTP-nya
//! tinggal di sini, bukan di `pages/` dan `api/` lama yang sedang dimigrasi.

pub mod api;
pub mod referensi_ukuran_page;

pub use referensi_ukuran_page::ReferensiUkuranPage;
