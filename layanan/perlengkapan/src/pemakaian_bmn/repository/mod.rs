//! # Pemakaian BMN Repository
//!
//! Data access layer for BMN usage permits.
//! Requirements: REQ-P001, REQ-P002, REQ-P003, REQ-P008, REQ-P011, REQ-P012, REQ-P013

use deadpool_postgres::Pool;

use super::models::*;

/// Repository for pemakaian BMN data access
#[derive(Clone)]
pub struct PemakaianBmnRepository {
    pool: Pool,
}

impl PemakaianBmnRepository {
    /// Create a new repository instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Helper function to convert database row to IzinPemakaianBmn
    pub(crate) fn row_to_permit(&self, row: tokio_postgres::Row) -> IzinPemakaianBmn {
        let status: String = row.get("status");
        let (status_label, status_tone) =
            crate::pemakaian_bmn::models::PemakaianBmnStatus::describe(&status);
        IzinPemakaianBmn {
            id: row.get("id"),
            nomor_izin: row.get("nomor_izin"),
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            pegawai_satker_id: row.get("pegawai_satker_id"),
            pegawai_satker_nama: row.get("pegawai_satker_nama"),
            pegawai_jabatan: row.get("pegawai_jabatan"),
            pegawai_golongan: row.try_get("pegawai_golongan").ok().flatten(),
            pegawai_pangkat: row.try_get("pegawai_pangkat").ok().flatten(),
            pegawai_unit_kerja: row.try_get("pegawai_unit_kerja").ok().flatten(),
            foto_pegawai: row.try_get("foto_pegawai").ok().flatten(),
            jenis_bmn: row.get("jenis_bmn"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.get("bmn_merk"),
            bmn_tahun_perolehan: row.get("bmn_tahun_perolehan"),
            no_polisi: row.get("no_polisi"),
            no_bpkb: row.get("no_bpkb"),
            no_stnk: row.get("no_stnk"),
            no_rangka: row.get("no_rangka"),
            no_mesin: row.get("no_mesin"),
            alamat: row.get("alamat"),
            luas_tanah: row.get("luas_tanah"),
            luas_bangunan: row.get("luas_bangunan"),
            serial_number: row.get("serial_number"),
            spesifikasi: row.get("spesifikasi"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            keperluan: row.get("keperluan"),
            lokasi_pemakaian: row.get("lokasi_pemakaian"),
            is_renewal: row.get("is_renewal"),
            previous_permit_id: row.get("previous_permit_id"),
            file_pendukung: row.get("file_pendukung"),
            document_id: row.get("document_id"),
            document_url: row.get("document_url"),
            konsep_surat_url: row.try_get("konsep_surat_url").ok().flatten(),
            konsep_surat_generated_at: row.try_get("konsep_surat_generated_at").ok().flatten(),
            konsep_surat_pdf_url: row.try_get("konsep_surat_pdf_url").ok().flatten(),
            konsep_surat_pdf_generated_at: row
                .try_get("konsep_surat_pdf_generated_at")
                .ok()
                .flatten(),
            signed_pdf_url: row.try_get("signed_pdf_url").ok().flatten(),
            signed_pdf_uploaded_at: row.try_get("signed_pdf_uploaded_at").ok().flatten(),
            is_completed: row.try_get("is_completed").unwrap_or(false),
            status,
            status_label,
            status_tone,
            catatan_approval: row.get("catatan_approval"),
            catatan_revocation: row.get("catatan_revocation"),
            approved_by: row.get("approved_by"),
            approved_by_nama: row.get("approved_by_nama"),
            approved_at: row.get("approved_at"),
            // V035 (Fase 1.5): kolom baru — try_get + default agar tetap
            // kompatibel dgn schema legacy (sebelum V035 di-apply di env dev).
            validator_satker_id: row.try_get("validator_satker_id").ok().flatten(),
            validator_satker_nama: row.try_get("validator_satker_nama").ok().flatten(),
            tanggal_validasi_satker: row.try_get("tanggal_validasi_satker").ok().flatten(),
            catatan_validator_satker: row.try_get("catatan_validator_satker").ok().flatten(),
            approver_satker_id: row.try_get("approver_satker_id").ok().flatten(),
            approver_satker_nama: row.try_get("approver_satker_nama").ok().flatten(),
            tanggal_approval_satker: row.try_get("tanggal_approval_satker").ok().flatten(),
            catatan_approver_satker: row.try_get("catatan_approver_satker").ok().flatten(),
            version: row.try_get("version").unwrap_or(1),
            revoked_by: row.get("revoked_by"),
            revoked_by_nama: row.get("revoked_by_nama"),
            revoked_at: row.get("revoked_at"),
            created_by: row.get("created_by"),
            created_by_nama: row.get("created_by_nama"),
            updated_by: row.get("updated_by"),
            updated_by_nama: row.get("updated_by_nama"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            bmn_items: vec![],
        }
    }
}

mod crud;
mod history;
mod items_docs;
mod lifecycle;
mod listing;
mod lookup;
mod monitoring;
mod numbering;
mod workflow;
