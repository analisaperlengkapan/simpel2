use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppResult, bad_request};
use crate::shared::satker_scope::SatkerScope;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

/// How a `jenis_pakaian_id` predicate has to be written, which depends on
/// whether the clothing-item tables are in scope for the query being built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum JenisPakaianPredicate {
    /// The query INNER JOINs `..._pakaian pp`, so the column is addressable
    /// directly and the predicate also narrows what is aggregated.
    Column,
    /// The query either does not join the item tables at all — the daftar
    /// COUNT joins only `psp`/`ps` — or LEFT JOINs them. A `pp.` predicate
    /// would be an unknown identifier in the first case, and in the second it
    /// would silently turn the LEFT JOIN into an inner one. Match on the
    /// pegawai row instead, which is valid in both.
    ExistsOnPegawai,
}

/// Resolved names for the id-shaped laporan filters, shown in the header of an
/// exported report so the reader can tell what it was narrowed by.
#[derive(Debug, Default, Clone)]
pub struct LaporanFilterLabels {
    pub satker: Option<String>,
    pub jenis_pakaian: Option<String>,
}

/// What [`push_laporan_filters`] bound, for callers that need to reference a
/// value a second time.
pub(crate) struct LaporanFilterSql {
    /// Next free placeholder index.
    pub next_idx: usize,
    /// Placeholder carrying `jenis_pakaian_id`, when the filter set it. The
    /// daftar queries reuse it in their `LEFT JOIN … pp` ON clause so the
    /// per-group size columns show the selected clothing type only.
    pub jenis_pakaian_idx: Option<usize>,
}

/// Appends the shared laporan filters to `sql` as bound parameters.
///
/// Every laporan query goes through here on purpose. When each call site kept
/// its own copy they drifted: `get_laporan_rekap_ukuran` never applied
/// `satker_id` while `get_laporan_rekap_long` — built from the same
/// `LaporanFilter` — did, so one filter produced a national table on screen and
/// a single-satker spreadsheet from the export button next to it.
pub(crate) fn push_laporan_filters(
    sql: &mut String,
    params: &mut Vec<Box<dyn ToSql + Sync + Send>>,
    filter: &LaporanFilter,
    mut idx: usize,
    jenis_pakaian: JenisPakaianPredicate,
    scope: &SatkerScope,
) -> LaporanFilterSql {
    // The caller's scope, ALWAYS, before any client-supplied narrowing. The
    // `satker_id` filter below is a convenience the client chooses; this is
    // not. Together they mean a client filter can only narrow what the scope
    // already allows, never widen it — the laporan surfaces used to return the
    // national table to a satker operator, measured on staging.
    //
    // `scope` is in this signature rather than at the six query sites for the
    // same reason `filter` is: this function exists because those sites had
    // already drifted apart once.
    if let Some(cond) = scope.push_condition("ps.satker_id", params) {
        sql.push_str(&format!(" AND {cond}"));
        idx = params.len() + 1;
    }
    if let Some(ref jk) = filter.jenis_kelamin {
        sql.push_str(&format!(" AND psp.jenis_kelamin = ${idx}"));
        params.push(Box::new(jk.clone()));
        idx += 1;
    }
    if let Some(ref satker_id) = filter.satker_id {
        sql.push_str(&format!(" AND ps.satker_id = ${idx}"));
        params.push(Box::new(satker_id.clone()));
        idx += 1;
    }
    if let Some(ref eselon) = filter.eselon {
        sql.push_str(&format!(" AND psp.eselon = ${idx}"));
        params.push(Box::new(eselon.clone()));
        idx += 1;
    }
    if let Some(ref jenis) = filter.jenis {
        sql.push_str(&format!(" AND psp.jenis = ${idx}"));
        params.push(Box::new(jenis.clone()));
        idx += 1;
    }

    let mut jenis_pakaian_idx = None;
    if let Some(jenis_pakaian_id) = filter.jenis_pakaian_id {
        match jenis_pakaian {
            JenisPakaianPredicate::Column => {
                sql.push_str(&format!(" AND pp.jenis_pakaian_id = ${idx}"));
            }
            JenisPakaianPredicate::ExistsOnPegawai => {
                sql.push_str(&format!(
                    " AND EXISTS (SELECT 1 \
                     FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pux \
                     JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian ppx \
                       ON pux.pakaian_id = ppx.id \
                     WHERE pux.pegawai_id = psp.id AND ppx.jenis_pakaian_id = ${idx})"
                ));
            }
        }
        params.push(Box::new(jenis_pakaian_id));
        jenis_pakaian_idx = Some(idx);
        idx += 1;
    }

    LaporanFilterSql {
        next_idx: idx,
        jenis_pakaian_idx,
    }
}

impl PakaianDinasRepository {
    pub async fn get_laporan_rekap_ukuran(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
        scope: &SatkerScope,
    ) -> AppResult<Vec<LaporanRekapUkuran>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut query = r#"
            SELECT
                pp.spesifikasi_nama as pakaian_nama,
                pp.spesifikasi_ukuran_group as ukuran_group,
                pu.ukuran,
                SUM(CASE WHEN psp.jenis_kelamin = 'L' THEN 1 ELSE 0 END) as jumlah_laki,
                SUM(CASE WHEN psp.jenis_kelamin = 'P' THEN 1 ELSE 0 END) as jumlah_perempuan,
                COUNT(*) as jumlah_total
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp ON pu.pegawai_id = psp.id
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id
            WHERE ps.pengajuan_id = $1
              AND ps.aktivitas_id = 1008
        "#
        .to_string();

        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        push_laporan_filters(
            &mut query,
            &mut params,
            filter,
            2,
            JenisPakaianPredicate::Column,
            scope,
        );

        query.push_str(" GROUP BY pp.spesifikasi_nama, pp.spesifikasi_ukuran_group, pu.ukuran ORDER BY pp.spesifikasi_nama, pu.ukuran");

        let param_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&query, &param_refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(LaporanRekapUkuran::from_row).collect())
    }

    pub async fn get_laporan_daftar_pegawai(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
        page: i32,
        per_page: i32,
        scope: &SatkerScope,
    ) -> AppResult<(Vec<LaporanDaftarPegawai>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        // Build dynamic filter with parameter binding
        let mut where_clause = "WHERE ps.pengajuan_id = $1 AND ps.aktivitas_id = 1008".to_string();
        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        let built = push_laporan_filters(
            &mut where_clause,
            &mut params,
            filter,
            2,
            JenisPakaianPredicate::ExistsOnPegawai,
            scope,
        );
        let idx = built.next_idx;
        // The COUNT below shares `where_clause` but joins neither `pu` nor `pp`,
        // which is why the clothing-type filter above is an EXISTS rather than a
        // `pp.` predicate. The data query additionally narrows what the per-group
        // MAX(CASE …) columns see, so a pengajuan carrying two clothing types
        // does not report one type's size under the other's column.
        let pakaian_join_filter = match built.jenis_pakaian_idx {
            Some(i) => format!(" AND pp.jenis_pakaian_id = ${i}"),
            None => String::new(),
        };

        let count_query = format!(
            r#"
            SELECT COUNT(*) as total
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            {}
            "#,
            where_clause
        );

        let count_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let count_row = client
            .query_one(&count_query, &count_refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        // LIMIT/OFFSET placeholders take the next two indices after the filter params.
        let limit_idx = idx;
        let offset_idx = idx + 1;
        let data_query = format!(
            r#"
            SELECT
                psp.nip, psp.nama, s.nama_satker as satker_nama, psp.jabatan, psp.pangkat,
                psp.jenis_kelamin, psp.gol_kd, psp.jenis, psp.eselon, psp.with_hijab,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'BAJU' THEN pu.ukuran END) as ukuran_baju,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'CELANA' THEN pu.ukuran END) as ukuran_celana,
                MAX(CASE WHEN pp.spesifikasi_ukuran_group = 'SEPATU' THEN pu.ukuran END) as ukuran_sepatu
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
            LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu ON psp.id = pu.pegawai_id
            LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id{}
            {}
            GROUP BY psp.id, psp.nip, psp.nama, s.nama_satker, psp.jabatan, psp.pangkat,
                     psp.jenis_kelamin, psp.gol_kd, psp.jenis, psp.eselon, psp.with_hijab
            ORDER BY s.nama_satker, psp.nama
            LIMIT ${} OFFSET ${}
            "#,
            pakaian_join_filter, where_clause, limit_idx, offset_idx
        );

        params.push(Box::new(per_page as i64));
        params.push(Box::new(offset as i64));
        let data_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&data_query, &data_refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<LaporanDaftarPegawai> =
            rows.iter().map(LaporanDaftarPegawai::from_row).collect();
        Ok((items, total))
    }

    /// Human-readable labels for the two id-shaped filters, for the exported
    /// document header. A report that was narrowed but does not say so is the
    /// same class of defect as a filter that is silently dropped, so each label
    /// falls back to the raw id rather than disappearing when the name cannot
    /// be resolved.
    pub async fn get_laporan_filter_labels(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
    ) -> AppResult<LaporanFilterLabels> {
        let mut labels = LaporanFilterLabels::default();
        if filter.satker_id.is_none() && filter.jenis_pakaian_id.is_none() {
            return Ok(labels);
        }

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        if let Some(ref satker_id) = filter.satker_id {
            let row = client
                .query_opt(
                    "SELECT nama_satker FROM integrasi.mysimkari_satker WHERE kode_satker = $1",
                    &[satker_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            labels.satker = Some(
                row.map(|r| r.get::<_, String>("nama_satker"))
                    .unwrap_or_else(|| satker_id.clone()),
            );
        }

        if let Some(jenis_pakaian_id) = filter.jenis_pakaian_id {
            // `jenis_pakaian_nama` is denormalised onto the pengajuan's own item
            // rows, so this stays inside perlengkapan and reflects the name as
            // it stood when the campaign was configured.
            let row = client
                .query_opt(
                    "SELECT jenis_pakaian_nama \
                     FROM perlengkapan.pengajuan_pakaian_dinas_pakaian \
                     WHERE pengajuan_id = $1 AND jenis_pakaian_id = $2 \
                     LIMIT 1",
                    &[&pengajuan_id, &jenis_pakaian_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            labels.jenis_pakaian = Some(
                row.map(|r| r.get::<_, String>("jenis_pakaian_nama"))
                    .unwrap_or_else(|| jenis_pakaian_id.to_string()),
            );
        }

        Ok(labels)
    }

    // ============ Rich report queries (long-format, for Excel/PDF export) ============

    /// Ordered, distinct clothing-item labels configured for a pengajuan.
    /// These become the dynamic columns of the daftar report.
    ///
    /// Takes the same filter as the rows so a clothing-type selection narrows
    /// the columns too; otherwise the export gains a column for every other
    /// type with nothing under it. The shared `push_laporan_filters` is not
    /// usable here — this query joins neither `psp` nor `ps`, so its
    /// pegawai-level predicates would be unknown identifiers.
    pub async fn get_laporan_daftar_columns(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
    ) -> AppResult<Vec<String>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut sql = "SELECT DISTINCT spesifikasi_nama \
                       FROM perlengkapan.pengajuan_pakaian_dinas_pakaian \
                       WHERE pengajuan_id = $1"
            .to_string();
        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        if let Some(jenis_pakaian_id) = filter.jenis_pakaian_id {
            sql.push_str(" AND jenis_pakaian_id = $2");
            params.push(Box::new(jenis_pakaian_id));
        }
        sql.push_str(" ORDER BY spesifikasi_nama");

        let refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&sql, &refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
    }

    /// Long-format daftar rows: one row per (pegawai × clothing item). Pegawai
    /// without any recorded sizes still appear once with NULL item/ukuran.
    pub async fn get_laporan_daftar_long(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
        scope: &SatkerScope,
    ) -> AppResult<Vec<DaftarLongRow>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut where_clause = "WHERE ps.pengajuan_id = $1 AND ps.aktivitas_id = 1008".to_string();
        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        let built = push_laporan_filters(
            &mut where_clause,
            &mut params,
            filter,
            2,
            JenisPakaianPredicate::ExistsOnPegawai,
            scope,
        );
        // Keeps the LEFT JOIN shape — a pegawai with no recorded size still
        // appears once with a NULL item — while restricting which item rows
        // are emitted when a clothing type is selected.
        let pakaian_join_filter = match built.jenis_pakaian_idx {
            Some(i) => format!(" AND pp.jenis_pakaian_id = ${i}"),
            None => String::new(),
        };

        let query = format!(
            r#"
            SELECT
                COALESCE(s.nama_satker, '-') AS satker_nama,
                psp.nip, psp.nama, psp.jabatan, psp.gol_kd, psp.jenis,
                psp.jenis_kelamin, psp.with_hijab,
                pp.spesifikasi_nama AS item_nama, pu.ukuran
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
            LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu ON psp.id = pu.pegawai_id
            LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id{}
            {}
            ORDER BY s.nama_satker, psp.nama, pp.spesifikasi_nama
            "#,
            pakaian_join_filter, where_clause
        );

        let refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&query, &refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| DaftarLongRow {
                satker_nama: r.get("satker_nama"),
                nip: r.get("nip"),
                nama: r.get("nama"),
                jabatan: r.try_get("jabatan").ok(),
                golongan: r.try_get("gol_kd").ok(),
                jenis: r.try_get("jenis").ok(),
                jenis_kelamin: r.get("jenis_kelamin"),
                with_hijab: r.try_get("with_hijab").unwrap_or(false),
                item_nama: r.try_get("item_nama").ok(),
                ukuran: r.try_get("ukuran").ok(),
            })
            .collect())
    }

    /// Long-format rekap rows: count per (pakaian × gender × satker × ukuran).
    pub async fn get_laporan_rekap_long(
        &self,
        pengajuan_id: Uuid,
        filter: &LaporanFilter,
        scope: &SatkerScope,
    ) -> AppResult<Vec<RekapLongRow>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut query = r#"
            SELECT
                pp.spesifikasi_nama AS pakaian_nama,
                pp.spesifikasi_ukuran_group AS ukuran_group,
                psp.jenis_kelamin AS gender,
                COALESCE(s.nama_satker, '-') AS satker_nama,
                pu.ukuran,
                COUNT(*) AS jumlah
            FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp ON pu.pegawai_id = psp.id
            JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
            LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
            JOIN perlengkapan.pengajuan_pakaian_dinas_pakaian pp ON pu.pakaian_id = pp.id
            WHERE ps.pengajuan_id = $1
              AND ps.aktivitas_id = 1008
        "#
        .to_string();

        let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(pengajuan_id)];
        push_laporan_filters(
            &mut query,
            &mut params,
            filter,
            2,
            JenisPakaianPredicate::Column,
            scope,
        );

        query.push_str(
            " GROUP BY pp.spesifikasi_nama, pp.spesifikasi_ukuran_group, \
              psp.jenis_kelamin, s.nama_satker, pu.ukuran \
              ORDER BY pp.spesifikasi_nama, psp.jenis_kelamin, s.nama_satker, pu.ukuran",
        );

        let refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|b| &**b as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&query, &refs[..])
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| RekapLongRow {
                pakaian_nama: r.get("pakaian_nama"),
                ukuran_group: r.get("ukuran_group"),
                gender: r.get("gender"),
                satker_nama: r.get("satker_nama"),
                ukuran: r.get("ukuran"),
                jumlah: r.get("jumlah"),
            })
            .collect())
    }

    // ============ Workflow Methods ============
}
