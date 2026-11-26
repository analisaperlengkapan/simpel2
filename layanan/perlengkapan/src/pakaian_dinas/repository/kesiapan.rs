//! Pre-flight readiness for the pakaian dinas workflow gates.
//!
//! Every level that advances a campaign (operator → wilayah → pusat) can move
//! while a lower level is still incomplete. The UI asks this module first and
//! shows a confirmation dialog; nothing here blocks — the person may proceed.
//! The counts live server-side because the client only ever holds one page of
//! the satker list, not the whole campaign.
//!
//! "Belum lengkap" is measured against the roster the campaign froze (V018)
//! when it has one, and against live MySIMKARI employees for legacy campaigns —
//! the same fallback `repository::roster` uses. A satker whose roster is empty
//! is complete by definition (0 filled out of 0 required).

use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::{KesiapanPakaianDinas, kesiapan_lengkap};
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs, scope_and};
use uuid::Uuid;

impl PakaianDinasRepository {
    /// Readiness of ONE satker's submission: how many of its employees still
    /// have no size saved. Scoped like every other satker read, so a caller
    /// outside the satker gets NotFound rather than a count of somebody else's
    /// roster.
    pub async fn hitung_kesiapan_satker(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<KesiapanPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params: Vec<BoxedParam> =
            vec![Box::new(pengajuan_id), Box::new(satker_code.to_string())];
        let scope_sql = scope_and(scope, "ps.satker_id", &mut params);
        let sql = format!(
            r#"
            SELECT
                CASE WHEN p.roster_snapshot_at IS NOT NULL
                     THEN (SELECT COUNT(*)
                           FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot snap
                           WHERE snap.pengajuan_id = ps.pengajuan_id
                             AND snap.satker_id = ps.satker_id)
                     ELSE (SELECT COUNT(*)
                           FROM integrasi.mysimkari_pegawai mp
                           JOIN integrasi.mysimkari_satker ms ON ms.api_id = mp.satker_id
                           WHERE ms.kode_satker = ps.satker_id)
                END AS total_pegawai,
                (SELECT COUNT(*)
                 FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
                 WHERE psp.pengajuan_satker_id = ps.id) AS terisi
            FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
            JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
            WHERE ps.pengajuan_id = $1 AND ps.satker_id = $2{scope_sql}
            "#
        );
        let row = client
            .query_opt(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| {
                AppError::NotFound("Satker tidak terdaftar pada pengajuan ini".to_string())
            })?;

        let total: i64 = row.get("total_pegawai");
        let terisi: i64 = row.get("terisi");
        let belum = (total - terisi).max(0);
        Ok(KesiapanPakaianDinas {
            tahap: "satker".to_string(),
            lengkap: belum == 0,
            total,
            pegawai_belum_diisi: belum,
            ..Default::default()
        })
    }

    /// Readiness of one campaign gate (`wilayah` / `pusat`), inside the caller's
    /// scope. `kecuali` excludes the row the caller is about to act on, so the
    /// dialog answers "are there OTHERS still behind?" and stays silent when the
    /// last satker is the one being moved.
    pub async fn hitung_kesiapan_level(
        &self,
        pengajuan_id: Uuid,
        kecuali: Option<Uuid>,
        tahap: &str,
        scope: &SatkerScope,
    ) -> AppResult<KesiapanPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(pengajuan_id), Box::new(kecuali)];
        let scope_sql = scope_and(scope, "ps.satker_id", &mut params);
        // Per-satker counts are materialised in the CTE first: PostgreSQL
        // aggregate FILTER clauses are no place for a correlated subquery, and
        // this also keeps the "filled vs required" comparison in one column.
        let sql = format!(
            r#"
            WITH satker_rows AS (
                SELECT
                    ps.id,
                    ps.aktivitas_id,
                    (SELECT COUNT(*)
                     FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
                     WHERE psp.pengajuan_satker_id = ps.id) AS terisi,
                    CASE WHEN p.roster_snapshot_at IS NOT NULL
                         THEN (SELECT COUNT(*)
                               FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot snap
                               WHERE snap.pengajuan_id = ps.pengajuan_id
                                 AND snap.satker_id = ps.satker_id)
                         ELSE (SELECT COUNT(*)
                               FROM integrasi.mysimkari_pegawai mp
                               JOIN integrasi.mysimkari_satker ms ON ms.api_id = mp.satker_id
                               WHERE ms.kode_satker = ps.satker_id)
                    END AS total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
                WHERE ps.pengajuan_id = $1
                  AND ($2::uuid IS NULL OR ps.id <> $2::uuid){scope_sql}
            )
            SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE aktivitas_id IN (1000, 1003)) AS belum_mengajukan,
                COUNT(*) FILTER (WHERE aktivitas_id IN (1001, 1007, 1012)) AS belum_diteruskan,
                COUNT(*) FILTER (WHERE aktivitas_id IN (1004, 1010)) AS belum_selesai,
                COUNT(*) FILTER (WHERE aktivitas_id = 1008) AS selesai,
                COUNT(*) FILTER (
                    WHERE aktivitas_id NOT IN (1000, 1003, 1008)
                      AND terisi < total_pegawai
                ) AS belum_lengkap
            FROM satker_rows
            "#
        );
        let row = client
            .query_one(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let total: i64 = row.get("total");
        let belum_mengajukan: i64 = row.get("belum_mengajukan");
        let belum_lengkap: i64 = row.get("belum_lengkap");
        let belum_diteruskan: i64 = row.get("belum_diteruskan");
        let belum_selesai: i64 = row.get("belum_selesai");
        let selesai: i64 = row.get("selesai");

        Ok(KesiapanPakaianDinas {
            tahap: tahap.to_string(),
            lengkap: kesiapan_lengkap(tahap, belum_mengajukan, belum_lengkap, belum_diteruskan),
            total,
            pegawai_belum_diisi: 0,
            belum_mengajukan,
            belum_lengkap,
            belum_diteruskan,
            belum_selesai,
            selesai,
        })
    }
}
