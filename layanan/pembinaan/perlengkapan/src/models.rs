//! # Data Models for Perlengkapan Service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

// Re-export shared models
pub use lib_perlengkapan::models::{
    AnalisisKebutuhan, ApiResponse, Asset, CategoryStat, CreateAnalisisRequest, CreateHibahRequest,
    CreateMutasiRequest, CreatePemakaianRequest, CreatePemeliharaanRequest,
    CreatePengadaanBastRequest, CreatePengadaanHpsRequest, CreatePengadaanKontrakRequest,
    CreatePengadaanNodisRequest, CreatePengadaanRequest, CreatePengadaanRingkasanRequest,
    CreatePengadaanSkppbjRequest, CreatePengadaanSpkRequest, CreatePengalihanRequest,
    CreatePenghapusanRequest, DashboardStats, Hibah, Mutasi, PaginatedResponse, Pemakaian,
    Pemeliharaan, Pengadaan, PengadaanBast, PengadaanHps, PengadaanKontrak, PengadaanNodis,
    PengadaanRingkasan, PengadaanSkppbj, PengadaanSpk, Pengalihan, Penghapusan,
};

// ============ Database Mapping Helpers ============

pub fn map_row_to_asset(row: &Row) -> Asset {
    Asset {
        id: row.get("id"),
        kategori_aset: row.get("kategori_aset"),
        no_aset: row.get("no_aset"),
        nama_aset: row
            .try_get("ur_sskel")
            .ok()
            .or_else(|| row.try_get("nama").ok()),
        kode_barang: row.try_get("kd_brg").ok(),
        merk: row.try_get("merk").ok(),
        tipe: row.try_get("tipe").ok(),
        kondisi: row.try_get("ur_kondisi").ok(),
        lokasi: row.try_get("alamat").ok(),
        satker: row.try_get("nama_satker").ok(),
        nilai_perolehan: row.try_get("rph_aset").ok(),
        tgl_perolehan: row.try_get("tgl_perlh").ok(),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
    }
}

pub fn map_row_to_pengadaan(row: &Row) -> Pengadaan {
    Pengadaan {
        id: row.get("id"),
        judul: row.get("judul"),
        deskripsi: row.get("deskripsi"),
        jenis: row.get("jenis"),
        status: row.get("status"),
        anggaran: row.get("anggaran"),
        target_selesai: row.get("target_selesai"),
        pic_user_id: row.get("pic_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_pengadaan_hps(row: &Row) -> PengadaanHps {
    PengadaanHps {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_hps: row.get("no_hps"),
        tgl_hps: row.get("tgl_hps"),
        nip_penandatangan: row.get("nip_penandatangan"),
        nama_penandatangan: row.get("nama_penandatangan"),
        pangkat_penandatangan: row.get("pangkat_penandatangan"),
        barang: row.get("barang"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_skppbj(row: &Row) -> PengadaanSkppbj {
    PengadaanSkppbj {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        nama_penandatangan: row.get("nama_penandatangan"),
        nip_penandatangan: row.get("nip_penandatangan"),
        pangkat_penandatangan: row.get("pangkat_penandatangan"),
        jabatan_penandatangan: row.get("jabatan_penandatangan"),
        alamat: row.get("alamat"),
        tgl_skppbj: row.get("tgl_skppbj"),
        penyedia: row.get("penyedia"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_spk(row: &Row) -> PengadaanSpk {
    PengadaanSpk {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_spk: row.get("no_spk"),
        no_permintaan: row.get("no_permintaan"),
        tgl_permintaan: row.get("tgl_permintaan"),
        no_ba: row.get("no_ba"),
        tgl_ba: row.get("tgl_ba"),
        tgl_mulai: row.get("tgl_mulai"),
        tgl_spk: row.get("tgl_spk"),
        tgl_selesai: row.get("tgl_selesai"),
        nama_penyedia: row.get("nama_penyedia"),
        keterangan: row.get("keterangan"),
        instruksi: row.get("instruksi"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_ringkasan(row: &Row) -> PengadaanRingkasan {
    PengadaanRingkasan {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_dipa: row.get("no_dipa"),
        tgl_dipa: row.get("tgl_dipa"),
        cara_pembayaran: row.get("cara_pembayaran"),
        alamat_penyedia: row.get("alamat_penyedia"),
        nama_bank: row.get("nama_bank"),
        kantor_bank: row.get("kantor_bank"),
        no_rek: row.get("no_rek"),
        npwp: row.get("npwp"),
        sanksi: row.get("sanksi"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_kontrak(row: &Row) -> PengadaanKontrak {
    PengadaanKontrak {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_kontrak: row.get("no_kontrak"),
        tgl_kontrak: row.get("tgl_kontrak"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_bast(row: &Row) -> PengadaanBast {
    PengadaanBast {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_bast: row.get("no_bast"),
        tgl_bast: row.get("tgl_bast"),
        nama_pejabat: row.get("nama_pejabat"),
        nip_pejabat: row.get("nip_pejabat"),
        pangkat_pejabat: row.get("pangkat_pejabat"),
        jabatan_pejabat: row.get("jabatan_pejabat"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_pengadaan_nodis(row: &Row) -> PengadaanNodis {
    PengadaanNodis {
        id: row.get("id"),
        pengadaan_id: row.get("pengadaan_id"),
        no_nodis: row.get("no_nodis"),
        tgl_nodis: row.get("tgl_nodis"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn map_row_to_analisis(row: &Row) -> AnalisisKebutuhan {
    AnalisisKebutuhan {
        id: row.get("id"),
        judul: row.get("judul"),
        kategori: row.get("kategori"),
        deskripsi: row.get("deskripsi"),
        prioritas: row.get("prioritas"),
        status: row.get("status"),
        estimasi_biaya: row.get("estimasi_biaya"),
        justifikasi: row.get("justifikasi"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_pemakaian(row: &Row) -> Pemakaian {
    Pemakaian {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        piminjam_nama: row.get("piminjam_nama"),
        tanggal_mulai: row.get("tanggal_mulai"),
        tanggal_selesai: row.get("tanggal_selesai"),
        status: row.get("status"),
        keperluan: row.get("keperluan"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_hibah(row: &Row) -> Hibah {
    Hibah {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        pemberi: row.get("pemberi"),
        penerima: row.get("penerima"),
        tanggal_hibah: row.get("tanggal_hibah"),
        keterangan: row.get("keterangan"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_mutasi(row: &Row) -> Mutasi {
    Mutasi {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        asal_satker: row.get("asal_satker"),
        tujuan_satker: row.get("tujuan_satker"),
        penanggung_jawab: row.get("penanggung_jawab"),
        tanggal_mutasi: row.get("tanggal_mutasi"),
        status: row.get("status"),
        keterangan: row.get("keterangan"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_penghapusan(row: &Row) -> Penghapusan {
    Penghapusan {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        tanggal_penghapusan: row.get("tanggal_penghapusan"),
        alasan: row.get("alasan"),
        metode_penghapusan: row.get("metode_penghapusan"),
        status: row.get("status"),
        nilai_residu: row.get("nilai_residu"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_pengalihan(row: &Row) -> Pengalihan {
    Pengalihan {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        pihak_lama: row.get("pihak_lama"),
        pihak_baru: row.get("pihak_baru"),
        tanggal_pengalihan: row.get("tanggal_pengalihan"),
        dasar_pengalihan: row.get("dasar_pengalihan"),
        status: row.get("status"),
        keterangan: row.get("keterangan"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

pub fn map_row_to_pemeliharaan(row: &Row) -> Pemeliharaan {
    Pemeliharaan {
        id: row.get("id"),
        asset_id: row.get("asset_id"),
        jenis_pemeliharaan: row.get("jenis_pemeliharaan"),
        biaya: row.get("biaya"),
        tanggal_mulai: row.get("tanggal_mulai"),
        tanggal_selesai: row.get("tanggal_selesai"),
        pelaksana: row.get("pelaksana"),
        status: row.get("status"),
        keterangan: row.get("keterangan"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}
