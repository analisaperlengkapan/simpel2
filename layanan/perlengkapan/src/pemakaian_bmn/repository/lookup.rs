use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// Lookup pegawai dari cache MySIMKARI by NIP.
    pub async fn find_pegawai_by_nip(&self, nip: &str) -> AppResult<Option<PegawaiInfo>> {
        let client = self.pool.client().await?;
        let row = client
            .query_opt(
                r#"
                SELECT nip, nama, jabatan, golpang AS pangkat, satker_id, nama_satker, foto
                FROM integrasi.mysimkari_pegawai
                WHERE nip = $1
                "#,
                &[&nip],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.map(|r| PegawaiInfo {
            nip: r.get("nip"),
            nama: r.try_get("nama").ok().flatten(),
            jabatan: r.try_get("jabatan").ok().flatten(),
            pangkat: r.try_get("pangkat").ok().flatten(),
            satker_id: r.try_get("satker_id").ok().flatten(),
            nama_satker: r.try_get("nama_satker").ok().flatten(),
            foto: r.try_get("foto").ok().flatten(),
        }))
    }

    /// Pemakaian BMN yg saat ini aktif utk pegawai.
    pub async fn list_pemakaian_aktif_by_pegawai(
        &self,
        nip: &str,
    ) -> AppResult<Vec<PemakaianAktifEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1 AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&nip],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianAktifEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Histori pemakaian BMN pegawai (status non-aktif: Expired, Revoked,
    /// Rejected, Cancelled). Limit utk avoid blow-up.
    pub async fn list_pemakaian_histori_by_pegawai(
        &self,
        nip: &str,
        limit: i64,
    ) -> AppResult<Vec<PemakaianHistoriEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang, status,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1
                  AND status IN ('EXPIRED', 'REVOKED', 'REJECTED', 'CANCELLED')
                ORDER BY tanggal_selesai DESC
                LIMIT $2
                "#,
                &[&nip, &limit],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianHistoriEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                status: r.get("status"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Cek ketersediaan BMN utk periode tertentu (Fase 1.11). Mengembalikan
    /// `Available`, `PemakaianBerurutan` (existing berakhir sebelum
    /// usulan mulai → boleh), atau `Overlap` (tolak).
    pub async fn check_bmn_availability_for_period(
        &self,
        bmn_nup: &str,
        tgl_mulai: chrono::NaiveDate,
        tgl_selesai: chrono::NaiveDate,
    ) -> AppResult<BmnCheckStatus> {
        let client = self.pool.client().await?;
        // Cari izin ACTIVE utk NUP ini, urutkan tanggal_selesai DESC agar
        // izin paling baru di atas. Kita evaluasi overlap thd usulan
        // periode operator.
        let rows = client
            .query(
                r#"
                SELECT pegawai_nama, tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE bmn_nup = $1 AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&bmn_nup],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Iterasi: jika ada existing yg overlap dgn (tgl_mulai..tgl_selesai)
        // → Overlap. Jika semua existing berakhir sebelum tgl_mulai dan ada
        // ≥1 existing → PemakaianBerurutan. Jika kosong → Available.
        let mut latest_existing: Option<(String, chrono::NaiveDate)> = None;
        for r in &rows {
            let ex_holder: String = r.get("pegawai_nama");
            let ex_start: chrono::NaiveDate = r.get("tanggal_mulai");
            let ex_end: chrono::NaiveDate = r.get("tanggal_selesai");

            // Overlap = NOT (ex_end < tgl_mulai OR ex_start > tgl_selesai)
            let overlap = !(ex_end < tgl_mulai || ex_start > tgl_selesai);
            if overlap {
                return Ok(BmnCheckStatus::Overlap {
                    existing_holder: ex_holder,
                    existing_sampai_tgl: ex_end,
                });
            }
            if latest_existing
                .as_ref()
                .map(|(_, prev_end)| ex_end > *prev_end)
                .unwrap_or(true)
            {
                latest_existing = Some((ex_holder, ex_end));
            }
        }

        match latest_existing {
            Some((holder, end_tgl)) => Ok(BmnCheckStatus::PemakaianBerurutan {
                existing_holder: holder,
                existing_sampai_tgl: end_tgl,
            }),
            None => Ok(BmnCheckStatus::Available),
        }
    }

    /// Check if BMN is available (no active permit) — legacy, kept for
    /// existing callers. Fase 1.11 callers harus pakai
    /// `check_bmn_availability_for_period`.
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(
        &self,
        bmn_nup: &str,
    ) -> AppResult<BmnAvailabilityResponse> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT id, nomor_izin, pegawai_nama, tanggal_selesai
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1 AND status = 'ACTIVE'
            LIMIT 1
        "#;

        let row_opt = client
            .query_opt(query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = row_opt {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: false,
                active_permit_id: Some(row.get("id")),
                active_permit_holder: Some(row.get("pegawai_nama")),
                active_permit_expires: Some(row.get("tanggal_selesai")),
            })
        } else {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: true,
                active_permit_id: None,
                active_permit_holder: None,
                active_permit_expires: None,
            })
        }
    }
}
