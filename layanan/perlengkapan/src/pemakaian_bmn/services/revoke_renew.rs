use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

impl PemakaianBmnService {
    /// Revoke a permit
    ///
    /// Requirements: REQ-P009
    pub async fn revoke_permit(
        &self,
        id: Uuid,
        request: RevokePermitRequest,
        user_id: Uuid,
        user_nama: String,
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let current = self.repository.get_by_id(id, scope).await?;

        // Can only revoke ACTIVE permits
        if current.status != "ACTIVE" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status ACTIVE yang dapat dicabut".to_string(),
            ));
        }

        info!("Revoking permit {} by user {}", id, user_id);

        // Update status to REVOKED
        let permit = self
            .repository
            .update_status(id, "REVOKED", user_id, user_nama, Some(request.alasan))
            .await?;

        Ok(permit)
    }

    /// Renew a permit (create new permit based on existing one)
    ///
    /// Requirements: REQ-P008
    pub async fn renew_permit(
        &self,
        id: Uuid,
        request: RenewPermitRequest,
        user_id: Uuid,
        user_nama: String,
        // Renewing operator's MySIMKARI satker_code (from JWT claims, #66).
        satker_code: Option<String>,
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id, scope).await?;

        // Can renew ACTIVE or EXPIRED permits
        if !matches!(current.status.as_str(), "ACTIVE" | "EXPIRED") {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status ACTIVE atau EXPIRED yang dapat diperpanjang".to_string(),
            ));
        }

        // Validate date range
        if request.tanggal_selesai <= request.tanggal_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        // Cek tabrakan untuk perpanjangan. Asetnya diambil dari izin yang
        // sedang diperpanjang itu sendiri — kode satker + kode barang + NUP
        // milik baris itu — sehingga tak perlu ditebak dari sesi pemanggil.
        //
        // Versi lama ber-key `bmn_nup` saja lalu membandingkan
        // `active_permit_id != Some(id)` untuk mengecualikan diri sendiri. Dua
        // kesalahan sekaligus: kuncinya menyatukan 44.017 aset berbeda (lihat
        // header `repository::lookup`), dan pengecualian-diri lewat id yang
        // bisa teredaksi akan diam-diam menolak perpanjangan yang sah. Kini
        // pengecualiannya dilakukan di SQL.
        if let Some(bentrok) = self.repository.find_renewal_conflict(id).await? {
            return Err(AppError::BadRequest(format!(
                "BMN {} (kode barang {}) sedang digunakan oleh {} hingga {}",
                bentrok.bmn_nup, bentrok.bmn_kode_barang, bentrok.holder, bentrok.expires
            )));
        }

        info!("Renewing permit {} by user {}", id, user_id);

        // The renewal carries the WHOLE asset list forward. It used to send
        // `additional_bmn_items: vec![]`, so every asset but the first was
        // silently dropped at renewal — the permit came back one-asset-wide
        // and the rest were left with no permit at all.
        //
        // Dates are deliberately NOT copied per item: a renewal is a new
        // period for everything on the letter, and carrying the old per-item
        // dates would produce a "renewal" that expired before it was signed.
        // `None` here means each item takes the new permit's period.
        if current.bmn_items.is_empty() {
            return Err(AppError::BadRequest(
                "Izin ini tidak memuat satu pun BMN, jadi tidak ada yang bisa diperpanjang"
                    .to_string(),
            ));
        }
        let items = current
            .bmn_items
            .iter()
            .map(|it| CreateBmnItemRequest {
                jenis_bmn: it.jenis_bmn.clone(),
                bmn_nup: it.bmn_nup.clone(),
                bmn_kode_barang: it.bmn_kode_barang.clone(),
                bmn_nama_barang: it.bmn_nama_barang.clone(),
                bmn_merk: it.bmn_merk.clone(),
                bmn_tipe: it.bmn_tipe.clone(),
                bmn_tahun_perolehan: it.bmn_tahun_perolehan,
                bmn_kondisi: it.bmn_kondisi.clone(),
                no_polisi: it.no_polisi.clone(),
                no_bpkb: it.no_bpkb.clone(),
                no_stnk: it.no_stnk.clone(),
                no_rangka: it.no_rangka.clone(),
                no_mesin: it.no_mesin.clone(),
                alamat: it.alamat.clone(),
                luas_tanah: it.luas_tanah,
                luas_bangunan: it.luas_bangunan,
                serial_number: it.serial_number.clone(),
                spesifikasi: it.spesifikasi.clone(),
                lokasi_pemakaian: it.lokasi_pemakaian.clone(),
                tanggal_mulai: None,
                tanggal_selesai: None,
                detail_bmn: Some(it.detail_bmn.clone()),
                keterangan: it.keterangan.clone(),
            })
            .collect();

        // Create new permit as renewal
        let create_request = CreateIzinPemakaianRequest {
            pegawai_nip: current.pegawai_nip.clone(),
            pegawai_nama: current.pegawai_nama.clone(),
            pegawai_satker_id: current.pegawai_satker_id,
            pegawai_satker_nama: current.pegawai_satker_nama.clone(),
            pegawai_jabatan: current.pegawai_jabatan.clone(),
            pegawai_golongan: current.pegawai_golongan.clone(),
            pegawai_pangkat: current.pegawai_pangkat.clone(),
            pegawai_unit_kerja: current.pegawai_unit_kerja.clone(),
            foto_pegawai: current.foto_pegawai.clone(),
            tanggal_mulai: request.tanggal_mulai,
            tanggal_selesai: request.tanggal_selesai,
            keperluan: request.keperluan,
            file_pendukung: current.file_pendukung.clone(),
            is_renewal: Some(true),
            previous_permit_id: Some(id),
            items,
        };

        let new_permit = self
            .repository
            .create(create_request, user_id, user_nama, satker_code)
            .await?;

        Ok(new_permit)
    }
}
