use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;

impl PemakaianBmnService {
    /// Satker yang aset-nya sedang ditanyakan, setelah dipastikan boleh
    /// dilihat pemanggil.
    ///
    /// `diminta` = `None` berarti "satker saya sendiri". Di luar scope
    /// jawabannya **404, bukan 403**: 403 akan mengonfirmasi bahwa satker itu
    /// ada dan punya aset — oracle keberadaan yang sama yang ditutup #93.
    pub(crate) async fn resolve_target_satker(
        &self,
        scope: &crate::shared::satker_scope::SatkerScope,
        milik_sendiri: Option<&str>,
        diminta: Option<&str>,
    ) -> AppResult<String> {
        let target = diminta
            .or(milik_sendiri)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "Tidak ada identitas satker pada sesi Anda; sebutkan satker \
                     yang asetnya ingin diperiksa"
                        .to_string(),
                )
            })?
            .to_string();

        if !self.repository.satker_code_in_scope(scope, &target).await? {
            return Err(AppError::NotFound(format!(
                "Satker {target} tidak ditemukan"
            )));
        }
        Ok(target)
    }

    /// Check if a BMN is available for new permit (no active permit exists).
    ///
    /// Aset dikenali dari kode satker + kode barang + NUP; NUP saja tidak
    /// mengidentifikasi apa pun (lihat header `repository::lookup`).
    ///
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(
        &self,
        aset: AssetIdentityQuery<'_>,
    ) -> AppResult<BmnAvailabilityResponse> {
        let satker = self
            .resolve_target_satker(aset.scope, aset.milik_sendiri, aset.satker_diminta)
            .await?;
        self.repository
            .check_bmn_availability(aset.bmn_nup, aset.bmn_kode_barang, &satker)
            .await
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
        scope: &SatkerScope,
    ) -> AppResult<CekPegawaiResponse> {
        // The scope is a predicate inside the lookup, so an employee outside
        // the caller's satker is simply not there — no second check to forget,
        // and no answer that distinguishes "exists elsewhere" from "does not
        // exist" (the cross-tenant existence oracle #93 closed).
        let pegawai = self.repository.find_pegawai_by_nip(nip, scope).await?;
        let Some(pegawai) = pegawai else {
            return Err(AppError::BadRequest(
                "Pegawai tidak ditemukan / tidak berada di satker bersangkutan".into(),
            ));
        };

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
        aset: AssetIdentityQuery<'_>,
        tgl_mulai: chrono::NaiveDate,
        tgl_selesai: chrono::NaiveDate,
    ) -> AppResult<CekBmnResponse> {
        let bmn_nup = aset.bmn_nup;
        if tgl_selesai < tgl_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus setelah tanggal mulai".into(),
            ));
        }
        // NOTE on reach: `/pemakaian-bmn/cek-bmn`, the only caller of this
        // method, has no consumer yet — the pemakaian form uses
        // `/bank-aset/lookup` then `/availability`. It is corrected here rather
        // than left to be wired wrong later, and kept rather than deleted
        // because period-aware booking (Available / PemakaianBerurutan /
        // Overlap) is the capability the form still lacks.
        //
        // Resolve the satker FIRST: it is one third of the asset's identity, so
        // looking the asset up before knowing it can only be a guess. This used
        // to run the other way round and key the lookup on NUP alone, which
        // returned an arbitrary one of the ~44 rows that share a NUP nationally
        // — so the "BMN tidak ditemukan" guard passed on someone else's asset,
        // and the reference block shown next to the booking described it.
        let satker = self
            .resolve_target_satker(aset.scope, aset.milik_sendiri, aset.satker_diminta)
            .await?;

        // Lookup BMN info dari SIMAN cache via BankAsetRepository — reuse
        // helper yg sudah ada agar konsisten dgn `/bank-aset/lookup`.
        let bank_repo = crate::bank_aset::repository::BankAsetRepository::new(pool.clone());
        // Authoritative SIMAN enrichment read (BMN reference info) — the row
        // filter is the identity itself, so no additional AsetScope restriction
        // is needed: `satker` already came from the caller's own scope via
        // `resolve_target_satker`.
        let lookup = bank_repo
            .find_lookup_by_identity(
                crate::bank_aset::AsetIdentity {
                    nup: bmn_nup,
                    kode_barang: Some(aset.bmn_kode_barang),
                    satker_code: Some(&satker),
                },
                &crate::bank_aset::AsetScope::All,
            )
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
            return Err(AppError::BadRequest(format!(
                "BMN tidak ditemukan di SIMAN untuk satker {satker} (kode barang {}, NUP {bmn_nup})",
                aset.bmn_kode_barang
            )));
        }

        let status = self
            .repository
            .check_bmn_availability_for_period(
                bmn_nup,
                aset.bmn_kode_barang,
                &satker,
                tgl_mulai,
                tgl_selesai,
            )
            .await?;
        Ok(CekBmnResponse {
            bmn_nup: bmn_nup.to_string(),
            bmn_info,
            status,
        })
    }
}
