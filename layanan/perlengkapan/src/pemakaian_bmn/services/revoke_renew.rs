use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
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
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let current = self.repository.get_by_id(id).await?;

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
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;

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

        // Check BMN availability
        let availability = self
            .repository
            .check_bmn_availability(&current.bmn_nup)
            .await?;
        if !availability.is_available && availability.active_permit_id != Some(id) {
            return Err(AppError::BadRequest(format!(
                "BMN {} sedang digunakan oleh {} hingga {}",
                current.bmn_nup,
                availability.active_permit_holder.unwrap_or_default(),
                availability
                    .active_permit_expires
                    .map(|d| d.to_string())
                    .unwrap_or_default()
            )));
        }

        info!("Renewing permit {} by user {}", id, user_id);

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
            jenis_bmn: current.jenis_bmn.clone(),
            bmn_nup: current.bmn_nup.clone(),
            bmn_kode_barang: current.bmn_kode_barang.clone(),
            bmn_nama_barang: current.bmn_nama_barang.clone(),
            bmn_merk: current.bmn_merk.clone(),
            bmn_tahun_perolehan: current.bmn_tahun_perolehan,
            no_polisi: current.no_polisi.clone(),
            no_bpkb: current.no_bpkb.clone(),
            no_stnk: current.no_stnk.clone(),
            no_rangka: current.no_rangka.clone(),
            no_mesin: current.no_mesin.clone(),
            alamat: current.alamat.clone(),
            luas_tanah: current.luas_tanah,
            luas_bangunan: current.luas_bangunan,
            serial_number: current.serial_number.clone(),
            spesifikasi: current.spesifikasi.clone(),
            tanggal_mulai: request.tanggal_mulai,
            tanggal_selesai: request.tanggal_selesai,
            keperluan: request.keperluan,
            lokasi_pemakaian: current.lokasi_pemakaian.clone(),
            file_pendukung: current.file_pendukung.clone(),
            is_renewal: Some(true),
            previous_permit_id: Some(id),
            additional_bmn_items: vec![],
        };

        let new_permit = self
            .repository
            .create(create_request, user_id, user_nama, satker_code)
            .await?;

        Ok(new_permit)
    }
}
