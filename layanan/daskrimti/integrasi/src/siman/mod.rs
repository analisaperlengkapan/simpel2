pub mod endpoints;
/// Modul untuk integrasi dengan SIMAN API v2.0 (Kemenkeu)
/// SIMAN (Sistem Informasi Manajemen Aset Negara) adalah sistem yang dikelola
/// oleh Kementerian Keuangan untuk pengelolaan Barang Milik Negara (BMN).
/// API v2.0 menyediakan akses ke berbagai kategori aset melalui gateway API Kemenkeu
/// dengan autentikasi OAuth2 client credentials flow.
pub mod models;

pub use endpoints::{
    fetch_all_aset_paginated, get_aset_alat_besar, get_aset_alat_persenjataan,
    get_aset_angkutan_bermotor, get_aset_bangunan_air, get_aset_by_category,
    get_aset_gedung_bangunan, get_aset_instalasi_jaringan, get_aset_jalan_jembatan, get_aset_kdp,
    get_aset_khusus_tik, get_aset_non_tik, get_aset_rumah, get_aset_tak_berwujud, get_aset_tanah,
    get_aset_tetap_lainnya, get_aset_tetap_renovasi, get_row_count,
};
pub use models::{
    RowCountResponse, SimanAssetCategory, SimanDataRequest, SimanResponse, SimanTokenResponse,
};
