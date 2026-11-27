//! Pre-flight readiness for the pakaian dinas workflow gates.
//!
//! A confirmation dialog asks this module before a move, and nothing here
//! blocks: the person may always proceed. What it reports follows the scope of
//! the action, and only that:
//!
//! * ONE satker (the operator submitting, or a validator moving a single row):
//!   [`PakaianDinasRepository::hitung_kesiapan_satker`], the employees of THAT
//!   satker who still owe a size. Nothing about other satker: moving this one
//!   does not depend on them.
//! * SEVERAL rows at once (the bulk bar of the wilayah or pusat validator):
//!   [`PakaianDinasRepository::hitung_kesiapan_level`], in two separate groups:
//!   how complete the rows being moved are (`fokus_*`), and which satker in scope
//!   are left behind (never submitted, or not yet forwarded by the wilayah).
//!
//! The counts live server-side because the client only ever holds one page of
//! the satker list, not the whole campaign.
//!
//! "Belum mengisi" means: a person the campaign still expects a size from, with
//! NO size row saved. Three consequences, all deliberate:
//!
//! * Saving an empty form is not "filled". The participation row exists, but it
//!   has no `..._ukuran` rows, so the person stays in the denominator.
//! * Employees the operator removed ("Keluarkan dari Pengajuan") are out of the
//!   denominator. They have no sizes by design, and the exclusion table (V020)
//!   records that so they are not reported as missing.
//! * The roster is matched by NIP, not counted as two independent totals. A size
//!   left behind by an employee who has since transferred cannot stand in for
//!   their replacement, and a legacy campaign's live roster is measured against
//!   the people who are there NOW.
//!
//! The roster source is the frozen snapshot (V018) when the campaign has one,
//! and live MySIMKARI employees for legacy campaigns — the same fallback
//! `repository::roster` uses. A satker with nobody in the roster needs nothing
//! from anybody, so it is left out of the warnings rather than reported as
//! "belum mengajukan" forever (it cannot be submitted: the button needs at least
//! one size).

use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::{KesiapanPakaianDinas, kesiapan_lengkap};
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs, scope_and};
use uuid::Uuid;

/// One row per (participation row, employee) the campaign still expects a size
/// from. Both roster sources keep the same shape so callers need not care which
/// one a campaign uses; `$1` is the campaign id and `{scope}` the caller's
/// satker scope (applied to both branches).
fn anggota_roster_cte(scope: &str) -> String {
    format!(
        r#"
        anggota AS (
            SELECT ps.id AS satker_row_id, ps.aktivitas_id, snap.nip
            FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
            JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
            JOIN perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot snap
              ON snap.pengajuan_id = ps.pengajuan_id
             AND snap.satker_id = ps.satker_id
            WHERE p.roster_snapshot_at IS NOT NULL
              AND ps.pengajuan_id = $1{scope}
              AND NOT EXISTS (
                  SELECT 1
                  FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan d
                  WHERE d.pengajuan_id = ps.pengajuan_id
                    AND d.satker_id = ps.satker_id
                    AND d.nip = snap.nip
              )
            UNION ALL
            SELECT ps.id, ps.aktivitas_id, mp.nip
            FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
            JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
            JOIN integrasi.mysimkari_satker ms ON ms.kode_satker = ps.satker_id
            JOIN integrasi.mysimkari_pegawai mp ON mp.satker_id = ms.api_id
            WHERE p.roster_snapshot_at IS NULL
              AND ps.pengajuan_id = $1{scope}
              AND NOT EXISTS (
                  SELECT 1
                  FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan d
                  WHERE d.pengajuan_id = ps.pengajuan_id
                    AND d.satker_id = ps.satker_id
                    AND d.nip = mp.nip
              )
        ),
        terisi AS (
            SELECT DISTINCT psp.pengajuan_satker_id AS satker_row_id, psp.nip
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps
              ON ps.id = psp.pengajuan_satker_id
            WHERE ps.pengajuan_id = $1
              AND EXISTS (
                  SELECT 1
                  FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
                  WHERE pu.pegawai_id = psp.id
              )
        )
        "#
    )
}

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
            WITH {cte}
            SELECT
                COUNT(a.nip) AS total_pegawai,
                COUNT(a.nip) FILTER (WHERE t.satker_row_id IS NULL) AS belum
            FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
            LEFT JOIN anggota a ON a.satker_row_id = ps.id
            LEFT JOIN terisi t
                   ON t.satker_row_id = a.satker_row_id AND t.nip = a.nip
            WHERE ps.pengajuan_id = $1 AND ps.satker_id = $2{scope_sql}
            GROUP BY ps.id
            "#,
            cte = anggota_roster_cte(&scope_sql),
        );
        let row = client
            .query_opt(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| {
                AppError::NotFound("Satker tidak terdaftar pada pengajuan ini".to_string())
            })?;

        let total: i64 = row.get("total_pegawai");
        let belum: i64 = row.get("belum");
        Ok(KesiapanPakaianDinas {
            tahap: "satker".to_string(),
            lengkap: kesiapan_lengkap("satker", belum, 0, 0, 0),
            total,
            pegawai_belum_diisi: belum,
            ..Default::default()
        })
    }

    /// Readiness of a BULK move at one campaign gate (`wilayah` / `pusat`), inside
    /// the caller's scope.
    ///
    /// `terpilih` are the rows about to be moved (the ticked rows of the bulk
    /// bar). They are reported separately (`fokus_*`: how complete they are) and
    /// left OUT of the "behind" counts, so the dialog answers two different
    /// questions: "is what I am moving complete?" and "who is still behind?",
    /// and never calls a row it is about to move "behind".
    pub async fn hitung_kesiapan_level(
        &self,
        pengajuan_id: Uuid,
        terpilih: &[Uuid],
        tahap: &str,
        scope: &SatkerScope,
    ) -> AppResult<KesiapanPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Empty list = nothing is being moved: every satker counts as "behind" and
        // the focus group is empty.
        let mut params: Vec<BoxedParam> = vec![Box::new(pengajuan_id), Box::new(terpilih.to_vec())];
        let scope_sql = scope_and(scope, "ps.satker_id", &mut params);
        // One pass: roster members and saved sizes are joined once and "belum" is
        // counted per satker row; the rows are then split into the ones being
        // moved and the rest.
        let sql = format!(
            r#"
            WITH {cte},
            per_satker AS (
                SELECT
                    ps.id,
                    ps.aktivitas_id,
                    (ps.id = ANY($2::uuid[])) AS fokus,
                    COUNT(a.nip) AS total_pegawai,
                    COUNT(a.nip) FILTER (WHERE t.satker_row_id IS NULL) AS belum
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN anggota a ON a.satker_row_id = ps.id
                LEFT JOIN terisi t
                       ON t.satker_row_id = a.satker_row_id AND t.nip = a.nip
                WHERE ps.pengajuan_id = $1{scope_sql}
                GROUP BY ps.id, ps.aktivitas_id
            )
            SELECT
                COUNT(*) FILTER (WHERE NOT fokus) AS total,
                COUNT(*) FILTER (
                    WHERE NOT fokus
                      AND aktivitas_id IN (1000, 1003)
                      AND total_pegawai > 0
                ) AS belum_mengajukan,
                COUNT(*) FILTER (
                    WHERE NOT fokus AND aktivitas_id IN (1001, 1007, 1012)
                ) AS belum_diteruskan,
                COUNT(*) FILTER (WHERE fokus) AS fokus_satker,
                COUNT(*) FILTER (WHERE fokus AND belum > 0) AS fokus_belum_lengkap,
                COALESCE(SUM(belum) FILTER (WHERE fokus), 0)::bigint AS fokus_pegawai_belum,
                COALESCE(SUM(total_pegawai) FILTER (WHERE fokus), 0)::bigint
                    AS fokus_pegawai_total
            FROM per_satker
            "#,
            cte = anggota_roster_cte(&scope_sql),
        );
        let row = client
            .query_one(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let total: i64 = row.get("total");
        let belum_mengajukan: i64 = row.get("belum_mengajukan");
        let belum_diteruskan: i64 = row.get("belum_diteruskan");
        let fokus_satker: i64 = row.get("fokus_satker");
        let fokus_belum_lengkap: i64 = row.get("fokus_belum_lengkap");
        let fokus_pegawai_belum: i64 = row.get("fokus_pegawai_belum");
        let fokus_pegawai_total: i64 = row.get("fokus_pegawai_total");

        Ok(KesiapanPakaianDinas {
            tahap: tahap.to_string(),
            lengkap: kesiapan_lengkap(
                tahap,
                0,
                fokus_belum_lengkap,
                belum_mengajukan,
                belum_diteruskan,
            ),
            total,
            pegawai_belum_diisi: 0,
            fokus_satker,
            fokus_belum_lengkap,
            fokus_pegawai_belum,
            fokus_pegawai_total,
            belum_mengajukan,
            belum_diteruskan,
        })
    }
}
