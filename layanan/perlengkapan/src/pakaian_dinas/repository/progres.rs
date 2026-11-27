//! Monitoring queries: progress of a campaign per level, and the filled-in
//! figures of the satker rows on a page.
//!
//! They read the same roster and the same definition of "filled" as the readiness
//! warnings ([`super::kesiapan`]), so a number here never disagrees with a dialog:
//! a person the campaign still expects a size from, with at least one size saved;
//! people excluded by the operator owe nothing and are not counted.

use super::PakaianDinasRepository;
use super::kesiapan::{anggota_roster_cte, anggota_roster_cte_untuk};
use crate::pakaian_dinas::models::{
    BarisProgres, KelompokProgres, ProgresPakaianDinas, ProgresRingkasan, susun_progres,
    susun_ringkasan_kegiatan,
};
use crate::shared::error::{AppResult, bad_request};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs, scope_and};
use std::collections::HashMap;
use uuid::Uuid;

impl PakaianDinasRepository {
    /// Progress of a whole campaign inside the caller's scope: the summary, the
    /// table per wilayah and the satker furthest behind.
    ///
    /// The scope applies like everywhere else: a wilayah validator sees their own
    /// wilayah, pusat sees all, an operator sees their satker. So the same endpoint
    /// answers "how is my wilayah doing" and "how is the country doing".
    pub async fn hitung_progres(
        &self,
        pengajuan_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<ProgresPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(pengajuan_id)];
        let scope_sql = scope_and(scope, "ps.satker_id", &mut params);
        let sql = format!(
            r#"
            WITH {cte},
            per_satker AS (
                SELECT
                    ps.id,
                    ps.satker_id,
                    ps.aktivitas_id,
                    COUNT(a.nip) AS pegawai_total,
                    COUNT(a.nip) FILTER (WHERE t.satker_row_id IS NOT NULL) AS pegawai_terisi
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN anggota a ON a.satker_row_id = ps.id
                LEFT JOIN terisi t
                       ON t.satker_row_id = a.satker_row_id AND t.nip = a.nip
                WHERE ps.pengajuan_id = $1{scope_sql}
                GROUP BY ps.id, ps.satker_id, ps.aktivitas_id
            )
            SELECT p.id::text AS id, p.satker_id, s.nama_satker, w.wilayah_code, w.wilayah_nama,
                   p.aktivitas_id, p.pegawai_total, p.pegawai_terisi
            FROM per_satker p
            LEFT JOIN integrasi.mysimkari_satker s ON s.kode_satker = p.satker_id
            LEFT JOIN integrasi.v_satker_wilayah w ON w.kode_satker = p.satker_id
            "#,
            cte = anggota_roster_cte(&scope_sql),
        );
        let rows = client
            .query(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(susun_progres(
            rows.iter()
                .map(|r| BarisProgres {
                    id: r.get("id"),
                    satker_id: r.get("satker_id"),
                    satker_nama: r.get("nama_satker"),
                    wilayah_code: r.get("wilayah_code"),
                    wilayah_nama: r.get("wilayah_nama"),
                    aktivitas_id: r.get("aktivitas_id"),
                    pegawai_total: r.get("pegawai_total"),
                    pegawai_terisi: r.get("pegawai_terisi"),
                })
                .collect(),
        ))
    }

    /// Per participation row: how many employees the roster expects a size from and
    /// how many have given one. For the rows of one page of the satker list, so the
    /// card can say "12 dari 15" instead of a bare count.
    pub async fn hitung_isian_baris(
        &self,
        pengajuan_id: Uuid,
        baris_ids: &[Uuid],
    ) -> AppResult<HashMap<Uuid, (i64, i64)>> {
        if baris_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        // The page's rows are already inside the caller's scope (the list applied
        // it); this only narrows the roster CTE to them.
        let batas = " AND ps.id = ANY($2::uuid[])";
        let sql = format!(
            r#"
            WITH {cte}
            SELECT
                ps.id,
                COUNT(a.nip) AS pegawai_total,
                COUNT(a.nip) FILTER (WHERE t.satker_row_id IS NOT NULL) AS pegawai_terisi
            FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
            LEFT JOIN anggota a ON a.satker_row_id = ps.id
            LEFT JOIN terisi t ON t.satker_row_id = a.satker_row_id AND t.nip = a.nip
            WHERE ps.pengajuan_id = $1{batas}
            GROUP BY ps.id
            "#,
            cte = anggota_roster_cte(batas),
        );
        let rows = client
            .query(&sql, &[&pengajuan_id, &baris_ids])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| {
                (
                    r.get("id"),
                    (r.get("pegawai_total"), r.get("pegawai_terisi")),
                )
            })
            .collect())
    }

    /// The progress summary of each campaign on a page of the campaign list, inside the
    /// caller's scope. One query for the whole page: the satker rows are grouped by
    /// campaign and status in SQL, and the stages are assigned in Rust by the one
    /// definition ([`crate::pakaian_dinas::models::tahap_progres`]).
    ///
    /// A campaign with no rows in the caller's scope has no entry.
    pub async fn hitung_progres_kegiatan(
        &self,
        pengajuan_ids: &[Uuid],
        scope: &SatkerScope,
    ) -> AppResult<HashMap<Uuid, ProgresRingkasan>> {
        if pengajuan_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(pengajuan_ids.to_vec())];
        let scope_sql = scope_and(scope, "ps.satker_id", &mut params);
        let kunci = "ps.pengajuan_id = ANY($1::uuid[])";
        let sql = format!(
            r#"
            WITH {cte},
            per_satker AS (
                SELECT
                    ps.pengajuan_id,
                    ps.id,
                    ps.aktivitas_id,
                    COUNT(a.nip) AS pegawai_total,
                    COUNT(a.nip) FILTER (WHERE t.satker_row_id IS NOT NULL) AS pegawai_terisi
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN anggota a ON a.satker_row_id = ps.id
                LEFT JOIN terisi t
                       ON t.satker_row_id = a.satker_row_id AND t.nip = a.nip
                WHERE {kunci}{scope_sql}
                GROUP BY ps.pengajuan_id, ps.id, ps.aktivitas_id
            )
            SELECT pengajuan_id, aktivitas_id,
                   COUNT(*) AS satker,
                   COUNT(*) FILTER (WHERE pegawai_total = 0) AS tanpa_pegawai,
                   COALESCE(SUM(pegawai_total), 0)::bigint AS pegawai_total,
                   COALESCE(SUM(pegawai_terisi), 0)::bigint AS pegawai_terisi
            FROM per_satker
            GROUP BY pengajuan_id, aktivitas_id
            "#,
            cte = anggota_roster_cte_untuk(kunci, &scope_sql),
        );
        let rows = client
            .query(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(susun_ringkasan_kegiatan(
            rows.iter()
                .map(|r| KelompokProgres {
                    pengajuan_id: r.get("pengajuan_id"),
                    aktivitas_id: r.get("aktivitas_id"),
                    satker: r.get("satker"),
                    tanpa_pegawai: r.get("tanpa_pegawai"),
                    pegawai_total: r.get("pegawai_total"),
                    pegawai_terisi: r.get("pegawai_terisi"),
                })
                .collect(),
        ))
    }
}
