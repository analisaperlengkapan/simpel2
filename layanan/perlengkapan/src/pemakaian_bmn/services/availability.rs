use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};

impl PemakaianBmnService {
    /// Check if a BMN is available for new permit (no active permit exists)
    ///
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(
        &self,
        bmn_nup: &str,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<BmnAvailabilityResponse> {
        self.repository.check_bmn_availability(bmn_nup, scope).await
    }

    // ========================================================================
    // Fase 1.11: cek pegawai + cek BMN (with period)
    // ========================================================================

    /// Cek pegawai-in-satker (Fase 1.11). Lookup pegawai dari MySIMKARI
    /// cache, validate satker match, return info pegawai, pemakaian aktif,
    /// dan histori. Jika pegawai tidak ditemukan ATAU satker mismatch →
    /// `AppError::BadRequest` dgn pesan persis stakeholder.
    pub async fn cek_pegawai_in_satker(
        &self,
        nip: &str,
        satker_id: &str,
    ) -> AppResult<CekPegawaiResponse> {
        let pegawai = self.repository.find_pegawai_by_nip(nip).await?;
        let Some(pegawai) = pegawai else {
            return Err(AppError::BadRequest(
                "Pegawai tidak ditemukan / tidak berada di satker bersangkutan".into(),
            ));
        };
        // Match satker — case-insensitive utk toleransi format
        // (kode_satker biasanya string numerik tapi safe).
        let pegawai_satker = pegawai.satker_id.clone().unwrap_or_default();
        if pegawai_satker.is_empty() || !pegawai_satker.eq_ignore_ascii_case(satker_id) {
            return Err(AppError::BadRequest(
                "Pegawai tidak ditemukan / tidak berada di satker bersangkutan".into(),
            ));
        }

        let pemakaian_aktif = self.repository.list_pemakaian_aktif_by_pegawai(nip).await?;
        let histori_pemakaian = self
            .repository
            .list_pemakaian_histori_by_pegawai(nip, 50)
            .await?;
        Ok(CekPegawaiResponse {
            pegawai,
            pemakaian_aktif,
            histori_pemakaian,
        })
    }

    /// Cek ketersediaan BMN utk periode tertentu (Fase 1.11). Lookup BMN
    /// info dari `integrasi.siman_aset` (cache); jika tidak ada → pesan
    /// "BMN tidak ditemukan". Lalu cek izin aktif: bisa Available,
    /// PemakaianBerurutan (boleh — usulan setelah existing berakhir),
    /// atau Overlap (tolak).
    pub async fn cek_bmn_availability_for_period(
        &self,
        pool: &deadpool_postgres::Pool,
        bmn_nup: &str,
        tgl_mulai: chrono::NaiveDate,
        tgl_selesai: chrono::NaiveDate,
    ) -> AppResult<CekBmnResponse> {
        if tgl_selesai < tgl_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus setelah tanggal mulai".into(),
            ));
        }
        // Lookup BMN info dari SIMAN cache via BankAsetRepository — reuse
        // helper yg sudah ada agar konsisten dgn `/bank-aset/lookup`.
        let bank_repo = crate::bank_aset::repository::BankAsetRepository::new(pool.clone());
        // Authoritative SIMAN enrichment read (BMN reference info) — unscoped on
        // purpose; satker ownership of the BMN is enforced by the pemakaian
        // workflow, not this lookup.
        let lookup = bank_repo
            .find_lookup_by_nup(bmn_nup, &crate::bank_aset::AsetScope::All)
            .await?;
        let bmn_info = lookup.map(|l| BmnRefInfo {
            nup: l.nup.clone(),
            kode_barang: l.kode_barang,
            nama_barang: l.nama_barang,
            merk: l.merk,
            tahun_perolehan: l.tahun_perolehan,
            kondisi: l.kondisi,
        });
        if bmn_info.is_none() {
            return Err(AppError::BadRequest("BMN tidak ditemukan".into()));
        }

        let status = self
            .repository
            .check_bmn_availability_for_period(bmn_nup, tgl_mulai, tgl_selesai)
            .await?;
        Ok(CekBmnResponse {
            bmn_nup: bmn_nup.to_string(),
            bmn_info,
            status,
        })
    }
}
