use super::PemakaianBmnRepository;
use crate::bank_aset::scope::AsetScope;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use crate::shared::satker_scope::SatkerScope;
// Which half of `integrasi.siman_aset` is actually written by the SIMAN ingest,
// and in what format. Shared so this report and the bank-aset surfaces cannot
// disagree — they did for four releases, and this file was the last holdout.
use crate::shared::siman_columns::{ASSET_KONDISI_BAIK_PREDICATE, jenis_bmn_sql};

type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Build the shared `WHERE` fragment for the permit-derived monitoring reads.
///
/// The caller's [`SatkerScope`] goes in FIRST and unconditionally: it is derived
/// from the JWT, not from the request, so no combination of query parameters can
/// remove it. Everything the client supplies is ANDed on top and can therefore
/// only narrow the result — asking for another satker's `satker_code` yields
/// zero rows rather than that satker's data.
fn permit_where(
    scope: &SatkerScope,
    satker_code: Option<&str>,
    jenis_bmn: Option<&str>,
    status: Option<&str>,
    params: &mut Vec<BoxedParam>,
) -> String {
    let mut clauses = Vec::new();

    match status {
        // `SEMUA` is the explicit opt-out: show finished permits too.
        Some(s) if s.eq_ignore_ascii_case("semua") => {}
        Some(s) => {
            params.push(Box::new(s.to_ascii_uppercase()));
            clauses.push(format!("status = ${}", params.len()));
        }
        None => clauses.push("status = 'ACTIVE'".to_string()),
    }

    if let Some(cond) = scope.push_condition("satker_code", params) {
        clauses.push(cond);
    }
    if let Some(code) = satker_code {
        params.push(Box::new(code.to_string()));
        clauses.push(format!("satker_code = ${}", params.len()));
    }
    if let Some(jenis) = jenis_bmn {
        params.push(Box::new(jenis.to_string()));
        clauses.push(format!("jenis_bmn = ${}", params.len()));
    }

    clauses.join(" AND ")
}

fn param_refs(params: &[BoxedParam]) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
    params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect()
}

impl PemakaianBmnRepository {
    /// Get active usage monitoring dashboard data
    ///
    /// Scoped: pusat/admin nasional, validator wilayah sebatas wilayahnya,
    /// operator/validator satker sebatas satkernya, caller tanpa identitas
    /// satker tidak melihat apa pun (fail-closed).
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: crate::pemakaian_bmn::models::MonitoringDashboardQuery,
        scope: &SatkerScope,
    ) -> AppResult<crate::pemakaian_bmn::models::ActiveUsageMonitoringDashboard> {
        let client = self.pool.client().await?;

        let mut params: Vec<BoxedParam> = Vec::new();
        let where_clause = permit_where(
            scope,
            query.satker_code.as_deref(),
            query.jenis_bmn.as_deref(),
            None,
            &mut params,
        );
        let param_refs = param_refs(&params);

        // Total active permits
        let total_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn WHERE {where_clause}"
        );

        let total_row = client
            .query_one(&total_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_active_permits: i64 = total_row.get("total");

        // Permits by jenis BMN
        let jenis_query = format!(
            r#"
            SELECT jenis_bmn, COUNT(*) as count
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {where_clause}
            GROUP BY jenis_bmn
            ORDER BY count DESC
            "#
        );

        let jenis_rows = client
            .query(&jenis_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_jenis_bmn = jenis_rows
            .into_iter()
            .map(|row| {
                let count: i64 = row.get("count");
                let percentage = if total_active_permits > 0 {
                    (count as f64 / total_active_permits as f64) * 100.0
                } else {
                    0.0
                };
                crate::pemakaian_bmn::models::PermitsByJenisBmn {
                    jenis_bmn: row.get("jenis_bmn"),
                    count,
                    percentage,
                }
            })
            .collect();

        // Permits by satker. Grouped on the authoritative MySIMKARI
        // `satker_code` (V003), not the legacy client-supplied
        // `pegawai_satker_id` UUID — grouping on the UUID split the same satker
        // across however many distinct UUIDs its rows happened to carry.
        let satker_query = format!(
            r#"
            SELECT satker_code, MIN(pegawai_satker_nama) AS satker_nama, COUNT(*) as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {where_clause}
            GROUP BY satker_code
            ORDER BY active_permits DESC
            LIMIT 10
            "#
        );

        let satker_rows = client
            .query(&satker_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_satker = satker_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::PermitsBySatker {
                satker_code: row.get("satker_code"),
                satker_nama: row.get("satker_nama"),
                active_permits: row.get("active_permits"),
            })
            .collect();

        // Expiring soon (next 30 days)
        let expiring_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, tanggal_selesai,
                   -- `date - date` is int4 in Postgres and this field is i64,
                   -- so an uncast difference makes `row.get` panic on the FIRST
                   -- row this query ever returns. Under `panic = "abort"` that
                   -- is not a 500, it is the process. Cast in SQL, matching how
                   -- every other aggregate in this crate declares its type.
                   (tanggal_selesai - CURRENT_DATE)::bigint as days_until_expiry
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {where_clause} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30
            ORDER BY tanggal_selesai ASC
            LIMIT 10
            "#
        );

        let expiring_rows = client
            .query(&expiring_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let expiring_soon = expiring_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::ExpiringPermitInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                tanggal_selesai: row.get("tanggal_selesai"),
                days_until_expiry: row.get("days_until_expiry"),
            })
            .collect();

        // Recent activations (last 7 days)
        let recent_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, approved_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {where_clause} AND approved_at >= CURRENT_DATE - 7
            ORDER BY approved_at DESC
            LIMIT 10
            "#
        );

        let recent_rows = client
            .query(&recent_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let recent_activations = recent_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::RecentActivationInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                activated_at: row.get("approved_at"),
            })
            .collect();

        Ok(
            crate::pemakaian_bmn::models::ActiveUsageMonitoringDashboard {
                total_active_permits,
                permits_by_jenis_bmn,
                permits_by_satker,
                expiring_soon,
                recent_activations,
            },
        )
    }

    /// Daftar "siapa memakai BMN apa", ter-scope per-role.
    ///
    /// Ini jawaban langsung atas permintaan stakeholder: satker mana, nama
    /// barangnya, NUP berapa, siapa pegawai yang memakai, dan berapa jangka
    /// waktu pemakaiannya — dengan batas pusat / wilayah / satker.
    pub async fn list_pemakaian_monitoring(
        &self,
        query: crate::pemakaian_bmn::models::PemakaianMonitoringQuery,
        scope: &SatkerScope,
    ) -> AppResult<crate::pemakaian_bmn::models::PemakaianBmnMonitoringPage> {
        let client = self.pool.client().await?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        let mut params: Vec<BoxedParam> = Vec::new();
        let mut where_clause = permit_where(
            scope,
            query.satker_code.as_deref(),
            query.jenis_bmn.as_deref(),
            query.status.as_deref(),
            &mut params,
        );

        if let Some(search) = query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            // Escape LIKE metacharacters so a search term cannot become a
            // pattern (`%` would otherwise match everything in scope).
            let escaped = search
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            params.push(Box::new(format!("%{escaped}%")));
            let i = params.len();
            where_clause.push_str(&format!(
                " AND (bmn_nama_barang ILIKE ${i} OR bmn_nup ILIKE ${i} \
                 OR bmn_kode_barang ILIKE ${i} OR pegawai_nama ILIKE ${i} \
                 OR pegawai_nip ILIKE ${i} OR COALESCE(nomor_izin, '') ILIKE ${i})"
            ));
        }

        let total: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE {where_clause}"
                ),
                &param_refs(&params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        params.push(Box::new(per_page));
        let limit_idx = params.len();
        params.push(Box::new(offset));
        let offset_idx = params.len();

        // `id` breaks the ORDER BY tie. Without it, rows sharing a
        // `tanggal_selesai` can swap between pages and a permit is silently
        // shown twice or not at all.
        let rows = client
            .query(
                &format!(
                    r#"
                    SELECT id, nomor_izin, satker_code, pegawai_satker_nama,
                           bmn_kode_barang, bmn_nama_barang, bmn_nup, bmn_merk, bmn_tipe,
                           jenis_bmn, pegawai_nip, pegawai_nama, pegawai_jabatan,
                           tanggal_mulai, tanggal_selesai, status,
                           (tanggal_selesai - tanggal_mulai)::bigint AS durasi_hari,
                           (tanggal_selesai - CURRENT_DATE)::bigint AS sisa_hari
                    FROM perlengkapan.izin_pemakaian_bmn
                    WHERE {where_clause}
                    ORDER BY tanggal_selesai ASC, id ASC
                    LIMIT ${limit_idx} OFFSET ${offset_idx}
                    "#
                ),
                &param_refs(&params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let data = rows
            .into_iter()
            .map(|row| {
                // merk and tipe are the SIMAN operator's own labels, kept apart
                // from `nama_barang` (which is fixed by the kode barang) so the
                // table never presents a free-text label as the official name.
                let merk: Option<String> = row.get("bmn_merk");
                let tipe: Option<String> = row.get("bmn_tipe");
                let merk_tipe = match (
                    merk.as_deref().map(str::trim).filter(|s| !s.is_empty()),
                    tipe.as_deref().map(str::trim).filter(|s| !s.is_empty()),
                ) {
                    (Some(m), Some(t)) => Some(format!("{m} {t}")),
                    (Some(m), None) => Some(m.to_string()),
                    (None, Some(t)) => Some(t.to_string()),
                    (None, None) => None,
                };
                crate::pemakaian_bmn::models::PemakaianBmnMonitoringRow {
                    id: row.get("id"),
                    nomor_izin: row.get("nomor_izin"),
                    satker_code: row.get("satker_code"),
                    satker_nama: row.get("pegawai_satker_nama"),
                    kode_barang: row.get("bmn_kode_barang"),
                    nama_barang: row.get("bmn_nama_barang"),
                    nup: row.get("bmn_nup"),
                    merk_tipe,
                    jenis_bmn: row.get("jenis_bmn"),
                    pegawai_nip: row.get("pegawai_nip"),
                    pegawai_nama: row.get("pegawai_nama"),
                    pegawai_jabatan: row.get("pegawai_jabatan"),
                    tanggal_mulai: row.get("tanggal_mulai"),
                    tanggal_selesai: row.get("tanggal_selesai"),
                    durasi_hari: row.get("durasi_hari"),
                    sisa_hari: row.get("sisa_hari"),
                    status: row.get("status"),
                }
            })
            .collect();

        Ok(crate::pemakaian_bmn::models::PemakaianBmnMonitoringPage {
            data,
            total,
            page,
            per_page,
            // Integer ceil-div. The f64 idiom used elsewhere in this
            // crate loses precision past 2^53 rows; this does not.
            total_pages: (total + per_page - 1) / per_page,
        })
    }

    /// Tiga kartu agregat headline dashboard monitoring (Fase 2.6):
    /// **sedang dipakai / tidak dipakai / akan expired**.
    ///
    /// Ketiganya ter-scope per-role. Kartu `tidak_dipakai` melintasi batas
    /// layanan — sisi izin dikunci lewat MySIMKARI `satker_code`, sisi SIMAN
    /// lewat `kdsatker_keu` — jadi ia memakai [`AsetScope`] yang memetakan
    /// keduanya via `integrasi.v_satker_code_map`. SIMAN best-effort: bila
    /// query SIMAN gagal, `tidak_dipakai = None` dan kartu lain tetap tersaji.
    pub async fn get_monitoring_summary(
        &self,
        query: crate::pemakaian_bmn::models::MonitoringDashboardQuery,
        scope: &SatkerScope,
        aset_scope: &AsetScope,
    ) -> AppResult<crate::pemakaian_bmn::models::MonitoringSummaryCards> {
        let client = self.pool.client().await?;

        let mut params: Vec<BoxedParam> = Vec::new();
        let where_clause = permit_where(
            scope,
            query.satker_code.as_deref(),
            query.jenis_bmn.as_deref(),
            None,
            &mut params,
        );
        let refs = param_refs(&params);

        // Kartu 1: sedang dipakai (izin ACTIVE).
        let sedang_dipakai: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE {where_clause}"
                ),
                &refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 2: akan expired dalam 30 hari.
        let akan_expired_30d: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn \
                     WHERE {where_clause} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30"
                ),
                &refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 3: tidak dipakai. Filter `jenis_bmn` tidak punya padanan tepat
        // di sisi SIMAN (`jenis_bmn` izin adalah 4 kelas turunan, bukan kolom
        // SIMAN), jadi saat difilter kartu ini disembunyikan alih-alih
        // menyajikan angka yang tak sebanding.
        let tidak_dipakai = if query.jenis_bmn.is_none() {
            match Self::count_idle_bmn(&client, scope, aset_scope, query.satker_code.as_deref())
                .await
            {
                Ok(n) => Some(n),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "SIMAN tidak tersedia utk kartu 'tidak dipakai' — disajikan null"
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(crate::pemakaian_bmn::models::MonitoringSummaryCards {
            sedang_dipakai,
            akan_expired_30d,
            tidak_dipakai,
        })
    }

    /// Jumlah BMN **kelas ber-izin** (kondisi Baik di SIMAN) yang tidak sedang
    /// dipakai, dalam scope pemanggil. Feeds the "BMN Tidak Dipakai" card.
    /// Dipisah agar kegagalan SIMAN dapat ditangani best-effort oleh pemanggil.
    ///
    /// Dua perbaikan sebelumnya, dan yang kedua BUKAN sekadar bug:
    ///
    /// 1. Kolom mati. `WHERE kondisi = 'BAIK'` cocok 0 dari 624 533 baris —
    ///    kolom `kondisi` tak pernah diisi ingest, dan `ur_kondisi` yang terisi
    ///    menulis "Baik", bukan "BAIK". `total` selalu 0, jadi
    ///    `(0 - utilized).max(0)` = 0: kartu ini menampilkan **0 untuk setiap
    ///    pengguna di setiap satker** sejak ada.
    ///
    /// 2. Penyebutnya salah secara KONSEP, bukan cuma salah kolom. Memperbaiki
    ///    (1) saja mengubah kartu dari 0 menjadi ~548 042 — yaitu seluruh BMN
    ///    kondisi baik se-Indonesia, termasuk 377 985 "Peralatan Mesin Non TIK"
    ///    (kursi, meja, lemari). Izin pemakaian tidak pernah diterbitkan untuk
    ///    kursi, jadi angka itu benar secara aritmatika tapi tak ada artinya.
    ///
    /// Yang ketiga diperbaiki di sini: angkanya **nasional untuk semua orang**.
    /// Kini kedua sisi ter-scope — sisi SIMAN lewat [`AsetScope`]
    /// (`kdsatker_keu` via `integrasi.v_satker_code_map`), sisi izin lewat
    /// [`SatkerScope`] (`satker_code`), sehingga selisihnya bermakna per-role.
    ///
    /// Pengurangannya tetap hampiran: sebuah izin bisa menunjuk aset yang tak
    /// lolos filter kelas/kondisi di sisi SIMAN, jadi `utilized` dapat melebihi
    /// bagiannya dari `total`. `max(0)` menjaga hasil tetap masuk akal.
    async fn count_idle_bmn(
        client: &deadpool_postgres::Object,
        scope: &SatkerScope,
        aset_scope: &AsetScope,
        satker_code: Option<&str>,
    ) -> Result<i64, tokio_postgres::Error> {
        let mut aset_params: Vec<BoxedParam> = Vec::new();
        let mut aset_clauses = vec![
            ASSET_KONDISI_BAIK_PREDICATE.to_string(),
            format!("{} <> 'LAINNYA'", jenis_bmn_sql("")),
        ];
        if let Some(cond) = aset_scope.push_condition(&mut aset_params) {
            aset_clauses.push(cond);
        }
        if let Some(code) = satker_code {
            aset_params.push(Box::new(code.to_string()));
            let i = aset_params.len();
            aset_clauses.push(format!(
                "kdsatker_keu IN (SELECT kdsatker_keu FROM integrasi.v_satker_code_map \
                 WHERE kode_satker = ${i} AND kdsatker_keu IS NOT NULL)"
            ));
        }
        let total: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM integrasi.siman_aset WHERE {}",
                    aset_clauses.join(" AND ")
                ),
                &param_refs(&aset_params),
            )
            .await?
            .get("c");

        // Counted on (kode barang, NUP) rather than NUP alone: NUP is only
        // unique within a satker AND a kode barang, so a bare `DISTINCT nup`
        // collapsed genuinely different assets into one.
        let mut izin_params: Vec<BoxedParam> = Vec::new();
        let izin_where = permit_where(scope, satker_code, None, None, &mut izin_params);
        let utilized: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(DISTINCT (bmn_kode_barang, bmn_nup)) AS c \
                     FROM perlengkapan.izin_pemakaian_bmn WHERE {izin_where}"
                ),
                &param_refs(&izin_params),
            )
            .await?
            .get("c");

        Ok((total - utilized).max(0))
    }
}
