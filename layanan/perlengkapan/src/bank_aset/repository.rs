use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use tracing::warn;

use super::models::*;
use super::scope::AsetScope;
use crate::shared::error::{AppError, AppResult, not_found};

/// Boxed bind parameter for the dynamic-SQL builders below.
type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Safety valve on `filter_options`, NOT a business rule — so it must sit far
/// above the natural cardinality of every column it guards, or it silently
/// becomes one.
///
/// It was 500, and `nama_satker` outgrew it. Measured on the staging SIMAN
/// snapshot (2026-08-26):
///
/// ```text
/// nama_satker   554 distinct     <- over the old cap
/// jenis_aset     15
/// ur_kondisi      4
/// ```
///
/// With `ORDER BY count DESC` the 54 satkers that fell off were the ones
/// holding the FEWEST assets — the smallest offices, which no national user
/// could then select in the Bank Aset filter at all. Nothing reported it: the
/// endpoint answered 200 with a list that merely stopped early.
///
/// 5 000 is chosen against the domain, not by taste: Kejaksaan has on the order
/// of 550 satkers nationally and that number moves by a handful per year, so
/// the cap now bounds a pathological query without ever bounding a real one.
const FILTER_OPTION_LIMIT: i64 = 5_000;

/// Borrow a boxed-param vec as the `&[&dyn ToSql]` slice tokio-postgres wants.
fn as_sql_params(params: &[BoxedParam]) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
    params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect()
}

// The SIMAN column mapping (which half of `integrasi.siman_aset` is actually
// written, and in what format) lives in one place so every consumer inherits
// the same answer instead of rediscovering the census: see
// `crate::shared::siman_columns`. It used to be duplicated here, which is how
// the BMN utilisation report in `pemakaian_bmn` kept reading the dead half long
// after the bank-aset surfaces were fixed.
use crate::shared::siman_columns::{
    ASSET_KATEGORI_SQL, ASSET_KONDISI_SQL, ASSET_NAMA_BARANG_SQL, ASSET_NUP_SQL,
    kode_barang_norm_sql, normalize_kode_barang,
};

#[derive(Clone)]
pub struct BankAsetRepository {
    pool: Pool,
}

/// Build the extra `AND` conditions that narrow a NUP match down to a single
/// asset, pushing each bind value onto `params` in the order the placeholders
/// reference it.
///
/// Split out of [`BankAsetRepository::find_lookup_by_identity`] so the SQL can
/// be asserted without a database: the failure mode this guards against —
/// comparing a dotted barang code against SIMAN's undotted one — produces an
/// empty result rather than an error, which no smoke test notices.
fn push_identity_conditions(aset: &AsetIdentity<'_>, params: &mut Vec<BoxedParam>) -> String {
    let mut extra = String::new();
    if let Some(kode_barang) = aset.kode_barang {
        // Both halves normalised through the same pair of helpers: SIMAN stores
        // undotted digits, callers hand us the dotted presentation form, and
        // comparing them raw is an equality that never holds.
        params.push(Box::new(normalize_kode_barang(kode_barang)));
        let i = params.len();
        extra.push_str(&format!(" AND {} = ${i}", kode_barang_norm_sql("")));
    }
    if let Some(satker_code) = aset.satker_code {
        // MySIMKARI `kode_satker` -> SIMAN `kdsatker_keu` through the
        // integrasi-owned map; the two coding schemes are different, and
        // matching on `nama_satker` (as this module once did) is the
        // string-equality trap that map exists to replace.
        params.push(Box::new(satker_code.to_string()));
        let i = params.len();
        extra.push_str(&format!(
            " AND kdsatker_keu IN (SELECT kdsatker_keu FROM integrasi.v_satker_code_map \
               WHERE kode_satker = ${i} AND kdsatker_keu IS NOT NULL)"
        ));
    }
    extra
}

impl BankAsetRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        filter: ListFilter,
        scope: &AsetScope,
    ) -> AppResult<(Vec<BankAsetItem>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<BoxedParam> = Vec::new();

        // RBAC data-visibility predicate first (its `$n` index is resolved by
        // `push_condition`, so ordering vs the user filters below is irrelevant).
        if let Some(cond) = scope.push_condition(&mut params) {
            conditions.push(cond);
        }

        push_asset_filters(&filter.f, &mut params, &mut conditions);

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!(
            "SELECT COUNT(*) AS count FROM integrasi.siman_aset{}",
            where_clause
        );
        let count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();
        let total: i64 = client
            .query_one(&count_sql, &count_params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let offset = ((filter.page - 1) * filter.per_page).max(0);
        params.push(Box::new(filter.per_page as i64));
        let limit_idx = params.len();
        params.push(Box::new(offset as i64));
        let offset_idx = params.len();

        let order_clause = match filter.sort.as_deref() {
            Some("updated_at_asc") => "ORDER BY updated_at ASC",
            Some("nama_asc") => "ORDER BY ur_sskel ASC NULLS LAST",
            Some("nama_desc") => "ORDER BY ur_sskel DESC NULLS LAST",
            Some("nilai_asc") => {
                "ORDER BY (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END) ASC"
            }
            Some("nilai_desc") => {
                "ORDER BY (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END) DESC"
            }
            _ => "ORDER BY updated_at DESC",
        };

        let list_sql = format!(
            "SELECT id, {ASSET_KATEGORI_SQL} AS kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, {ASSET_NUP_SQL} AS nup,
             (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END) AS rph_aset,
             tgl_perlh, updated_at
             FROM integrasi.siman_aset
             {where_clause}
             {order_clause}
             LIMIT ${limit_idx} OFFSET ${offset_idx}",
            where_clause = where_clause,
            order_clause = order_clause,
            limit_idx = limit_idx,
            offset_idx = offset_idx,
        );

        let list_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&list_sql, &list_params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let items = rows.iter().map(row_to_item).collect();
        Ok((items, total))
    }

    /// Search the barang codification for a picker.
    ///
    /// # Why this is NOT satker-scoped
    ///
    /// Every other read in this module is scoped, because an asset belongs to a
    /// satker. A CODE does not. The caller here is the kebutuhan-BMN form,
    /// where a satker states what it NEEDS — very often a barang it does not
    /// own yet, which is the whole point of the request. Scoping the
    /// codification to what the satker already holds would make the form unable
    /// to express its most common need, and it would do so silently, as an
    /// empty search result.
    ///
    /// Nothing satker-specific is exposed by the absence of the scope: the rows
    /// are a national taxonomy plus two counts over the whole register.
    ///
    /// # Why it reads a snapshot
    ///
    /// The same search against `siman_aset` is a parallel seq scan over
    /// 624 533 rows measured at 1 285 ms — per keystroke, after a 300 ms
    /// debounce. `integrasi.mv_kodefikasi_barang` holds one row per
    /// (code, name) pair, 2 038 of them, and is refreshed on the sync cadence
    /// (migration 006).
    ///
    /// `q` matches the name case-insensitively anywhere, and the code by
    /// prefix. Infix on the code would let "100" pull in a third of the
    /// taxonomy through digits in the middle of unrelated codes; an operator
    /// typing digits is typing the start of a code.
    pub async fn kodefikasi(
        &self,
        q: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<KodefikasiBarang>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let needle = q.map(str::trim).filter(|s| !s.is_empty());
        // The code is stored undotted, so a query typed with dots has to be
        // reduced the same way before it is compared — normalising one half
        // only is the bug `kode_barang_norm_sql` exists to prevent.
        let kode_prefix = needle.map(|n| format!("{}%", normalize_kode_barang(n)));
        let nama_like = needle.map(|n| format!("%{}%", n));

        let (where_clause, params): (&str, Vec<BoxedParam>) = match (&nama_like, &kode_prefix) {
            (Some(nama), Some(kode)) => (
                "WHERE nama_barang ILIKE $1 OR kode_barang LIKE $2",
                vec![Box::new(nama.clone()), Box::new(kode.clone())],
            ),
            _ => ("", Vec::new()),
        };
        let mut params = params;
        params.push(Box::new(limit));
        let limit_idx = params.len();
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as _).collect();

        // Prevalence first: one name in a thousand is a near-homonym of
        // another, and the operator wants the mainstream item far more often.
        // `nama_barang` breaks the tie so the same query cannot answer in two
        // orders between calls.
        let rows = client
            .query(
                &format!(
                    "SELECT kode_barang, nama_barang, jumlah_aset, jumlah_satker
                       FROM integrasi.mv_kodefikasi_barang
                       {where_clause}
                      ORDER BY jumlah_aset DESC, nama_barang ASC
                      LIMIT ${limit_idx}"
                ),
                &refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| KodefikasiBarang {
                kode_barang: r.get("kode_barang"),
                nama_barang: r.get("nama_barang"),
                jumlah_aset: r.get("jumlah_aset"),
                jumlah_satker: r.get("jumlah_satker"),
            })
            .collect())
    }

    /// Find the slim lookup record for a NUP alone.
    ///
    /// Kept for the one surface that genuinely has nothing else to go on: the
    /// pemakaian-bmn form auto-fills `kode_barang` + `nama_barang` the moment
    /// the user types a NUP, before any barang has been chosen. Everywhere the
    /// caller already knows the satker and the barang code, use
    /// [`Self::find_lookup_by_identity`] instead — see its docs for why NUP
    /// alone picks the right asset only about a tenth of the time.
    pub async fn find_lookup_by_nup(
        &self,
        nup: &str,
        scope: &AsetScope,
    ) -> AppResult<Option<BankAsetLookup>> {
        self.find_lookup_by_identity(
            AsetIdentity {
                nup,
                kode_barang: None,
                satker_code: None,
            },
            scope,
        )
        .await
    }

    /// Find the slim lookup record for an asset identified the way the domain
    /// identifies one: **kode satker + kode barang + NUP**.
    ///
    /// `AsetIdentity` lets a caller supply the parts it actually holds; the
    /// query narrows by each part that is `Some`. Supplying only `nup`
    /// reproduces the old NUP-only behaviour.
    ///
    /// # Why the extra parts matter, measured
    ///
    /// NUP is a sequence number *within* one barang code at one satker, so on
    /// its own it names an enormous set: the 624 533-row staging snapshot holds
    /// only 14 142 distinct NUPs — **44 candidate rows per NUP on average, up
    /// to 44 017** — and 8 555 of those NUPs cover more than one barang code.
    ///
    /// Picking one of them arbitrarily therefore returns an asset whose
    /// `kd_brg` matches the caller's for an expected **10% of rows**. That is
    /// not a tail case, it is the common case, and it had a live consequence:
    /// `penghapusan_bmn::services::verify_asset_siman` compares the returned
    /// `kode_barang` against the usulan and sets `layak_lanjut` from the
    /// result, so a validator about to issue an SK saw "kode_barang SIMAN
    /// berbeda dari usulan" for roughly nine of every ten perfectly valid
    /// assets.
    ///
    /// `ORDER BY id` is not cosmetic either: `LIMIT 1` over those candidates
    /// with no ordering let the same request answer differently between calls.
    pub async fn find_lookup_by_identity(
        &self,
        aset: AsetIdentity<'_>,
        scope: &AsetScope,
    ) -> AppResult<Option<BankAsetLookup>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(aset.nup.to_string())];
        let extra = push_identity_conditions(&aset, &mut params);

        let scope_clause = match scope.push_condition(&mut params) {
            Some(cond) => format!(" AND {cond}"),
            None => String::new(),
        };
        let sql = format!(
            // Both the projection AND the predicate must use ASSET_NUP_SQL: with
            // `WHERE nup = $1` this endpoint 404'd for every real asset, because
            // the `nup` column is empty outside the e2e seed (see ASSET_NUP_SQL).
            // `nama_barang` reads ASSET_NAMA_BARANG_SQL (`ur_sskel`), not `nama`.
            // The two are different facts: `ur_sskel` is the standard name that
            // belongs to the barang code (2 038 codes, 2 038 distinct names —
            // a strict function of the code), while `nama` is whatever the SIMAN
            // operator typed for this one item. Measured on staging, `nama` is
            // blank in 138 607 rows and differs from the standard name in
            // 484 228 more — 99.7% of the table — and simply repeats `merk` in
            // 344 364. Auto-filling a pemakaian form's "nama barang" from it
            // put a brand string in a taxonomy field.
            "SELECT id, {ASSET_NUP_SQL} AS nup, kd_brg, {ASSET_NAMA_BARANG_SQL} AS nama_barang,
                    merk, tgl_perlh, ur_kondisi, nama_satker,
                    (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE NULL END) AS nilai_perolehan
             FROM integrasi.siman_aset
             WHERE {ASSET_NUP_SQL} = $1{extra}{scope_clause}
             ORDER BY id
             LIMIT 1"
        );
        let row = client
            .query_opt(&sql, &as_sql_params(&params))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| BankAsetLookup {
            id: r.get::<_, i64>("id").to_string(),
            nup: r
                .try_get::<_, Option<String>>("nup")
                .ok()
                .flatten()
                .unwrap_or_else(|| aset.nup.to_string()),
            kode_barang: r.try_get::<_, Option<String>>("kd_brg").ok().flatten(),
            nama_barang: r
                .try_get::<_, Option<String>>("nama_barang")
                .ok()
                .flatten()
                .filter(|s| !s.is_empty()),
            merk: r.try_get::<_, Option<String>>("merk").ok().flatten(),
            tahun_perolehan: r.try_get::<_, Option<String>>("tgl_perlh").ok().flatten(),
            kondisi: r.try_get::<_, Option<String>>("ur_kondisi").ok().flatten(),
            satker: r.try_get::<_, Option<String>>("nama_satker").ok().flatten(),
            nilai_perolehan: r
                .try_get::<_, Option<f64>>("nilai_perolehan")
                .ok()
                .flatten(),
        }))
    }

    /// Resolve `nilai_perolehan` for a SIMAN asset identified by NUP (+
    /// optional kode_barang untuk validasi konsistensi).
    ///
    /// Dipakai oleh penghapusan-bmn untuk mengisi field `nilai_perolehan`
    /// dari sumber otoritatif (SIMAN cache di `integrasi.siman_aset`)
    /// daripada percaya input operator.
    ///
    /// `kode_barang` masuk ke PREDIKAT, bukan disaring setelah baris terpilih.
    /// Hasil akhirnya sama untuk baris yang cocok, tapi penyaringan-setelah
    /// menyerah begitu `LIMIT 1` kebetulan memilih salah satu dari puluhan
    /// kandidat ber-NUP sama: nilai yang benar ada di tabel dan tetap tak
    /// terambil.
    pub async fn find_nilai_perolehan(
        &self,
        nup: &str,
        kode_barang: Option<&str>,
        scope: &AsetScope,
    ) -> AppResult<Option<f64>> {
        let lookup = self
            .find_lookup_by_identity(
                AsetIdentity {
                    nup,
                    kode_barang,
                    satker_code: None,
                },
                scope,
            )
            .await?;
        Ok(lookup.and_then(|l| l.nilai_perolehan))
    }

    pub async fn get(&self, id: i64, scope: &AsetScope) -> AppResult<BankAsetItem> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(id)];
        let scope_clause = match scope.push_condition(&mut params) {
            Some(cond) => format!(" AND {cond}"),
            None => String::new(),
        };
        let sql = format!(
            "SELECT id, {ASSET_KATEGORI_SQL} AS kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, {ASSET_NUP_SQL} AS nup,
             (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END) AS rph_aset,
             tgl_perlh, updated_at
             FROM integrasi.siman_aset WHERE id = $1{scope_clause}"
        );
        let row = client
            .query_opt(&sql, &as_sql_params(&params))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| row_to_item(&r))
            .ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    /// Summary of the caller's visible assets, narrowed by `filter`.
    ///
    /// The filter arrives on the SIGNATURE rather than being applied afterwards
    /// so that every breakdown below is built from the same `where_clause` — a
    /// summary whose totals answer one question and whose breakdown answers
    /// another is worse than no summary. One query is now the mechanism for
    /// that rather than a convention five queries had to keep agreeing on.
    ///
    /// # Why one pass
    ///
    /// This was five sequential queries over the same 624 533 rows, and the
    /// page showed blank headline numbers for as long as they took. Measured
    /// on the staging snapshot: a bare scan is 53 ms, but
    /// `rph_aset ~ '^[0-9]+…$'` plus the cast costs ~840 ms — and three of the
    /// five queries paid it. `GROUPING SETS` reads the table once, evaluates
    /// that regex once per row, and returns every breakdown from the same
    /// scan.
    ///
    /// | | five queries | one pass |
    /// |---|---|---|
    /// | run 1 | 4 246 ms | 1 469 ms |
    /// | run 2 | 4 313 ms | 1 628 ms |
    ///
    /// All four breakdowns and all four totals were verified equal to the
    /// queries they replace, by `EXCEPT` in both directions over the full
    /// 624 533 rows: 0 differing rows on every one.
    ///
    /// The remaining ~0.8 s is still that regex. It belongs in a generated
    /// numeric column on `integrasi.siman_aset`, which is a migration, not a
    /// query change.
    pub async fn dashboard(
        &self,
        filter: &AsetFilter,
        scope: &AsetScope,
    ) -> AppResult<BankAsetDashboard> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        // RBAC scope predicate FIRST, then the caller's filters — the order is
        // the guarantee. A filter appended to an already-scoped condition list
        // can only ever remove rows, so `?satker_kode=` belonging to another
        // region yields an empty summary rather than that region's summary.
        // Reversing these two lines is how a drill-down becomes a leak.
        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let mut conditions: Vec<String> = Vec::new();
        if let Some(cond) = scope.push_condition(&mut scope_params) {
            conditions.push(cond);
        }
        push_asset_filters(filter, &mut scope_params, &mut conditions);

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };
        let p = as_sql_params(&scope_params);

        // The CTE names each row's five dimensions and its value once; the
        // grouping sets then aggregate that projection. Postgres inlines a
        // non-recursive CTE referenced once, so this is one scan, not two.
        //
        // Two satker sets, not one, because the two numbers ask different
        // questions: the top-10 list is BY NAME (what a reader recognises),
        // while `total_satker` counts CODES. Measured on staging: 556 distinct
        // `kdsatker_keu` against 554 distinct `nama_satker`, so counting names
        // folds two satkers into one and under-reports the tile. A satker's
        // identity is its code (#43); the name is a label that can collide.
        //
        // `tahun` is NULL for exactly the rows the old per-year query excluded
        // in its WHERE, so dropping the NULL group below reproduces it — and
        // keeps those rows in every OTHER breakdown, where they belong.
        let rows = client
            .query(
                &format!(
                    r#"WITH aset AS (
                    SELECT
                        COALESCE(NULLIF({ASSET_KONDISI_SQL}, ''), 'TIDAK DIKETAHUI') AS kondisi,
                        {ASSET_KATEGORI_SQL} AS kategori,
                        COALESCE(nama_satker, 'TIDAK DIKETAHUI') AS satker_nama,
                        kdsatker_keu AS satker_kode,
                        CASE WHEN SUBSTRING(tgl_perlh FROM 1 FOR 4) ~ '^[0-9]{{4}}$'
                             THEN SUBSTRING(tgl_perlh FROM 1 FOR 4)::INT END AS tahun,
                        CASE WHEN rph_aset ~ '^[0-9]+(\.[0-9]+)?$'
                             THEN rph_aset::FLOAT8 ELSE 0 END AS nilai
                    FROM integrasi.siman_aset{where_clause}
                 )
                 SELECT
                    GROUPING(kondisi)     AS g_kondisi,
                    GROUPING(kategori)    AS g_kategori,
                    GROUPING(satker_nama) AS g_satker_nama,
                    GROUPING(satker_kode) AS g_satker_kode,
                    GROUPING(tahun)       AS g_tahun,
                    kondisi, kategori, satker_nama, satker_kode, tahun,
                    COUNT(*)::BIGINT           AS count,
                    COALESCE(SUM(nilai), 0)::FLOAT8 AS nilai
                 FROM aset
                 GROUP BY GROUPING SETS (
                    (), (kondisi), (kategori), (satker_nama), (satker_kode), (tahun)
                 )"#
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // `GROUPING(x)` is 0 when the row is grouped by `x` and 1 when it is
        // not, so a row belongs to exactly the set whose column reads 0.
        let in_set = |row: &tokio_postgres::Row, col: &str| -> bool { row.get::<_, i32>(col) == 0 };

        let mut total_aset: i64 = 0;
        let mut total_nilai: f64 = 0.0;
        let mut total_satker: i64 = 0;
        let mut total_kategori: i64 = 0;
        let mut kondisi_breakdown: Vec<KondisiStat> = Vec::new();
        let mut kategori_breakdown: Vec<KategoriStat> = Vec::new();
        let mut top_satker: Vec<SatkerStat> = Vec::new();
        let mut per_tahun: Vec<TahunStat> = Vec::new();

        for row in &rows {
            let count: i64 = row.get("count");
            if in_set(row, "g_kondisi") {
                kondisi_breakdown.push(KondisiStat {
                    kondisi: row.get("kondisi"),
                    count,
                });
            } else if in_set(row, "g_kategori") {
                total_kategori += 1;
                kategori_breakdown.push(KategoriStat {
                    kategori: row.get("kategori"),
                    count,
                    nilai: row.get("nilai"),
                });
            } else if in_set(row, "g_satker_nama") {
                top_satker.push(SatkerStat {
                    satker: row.get("satker_nama"),
                    count,
                    nilai: row.get("nilai"),
                });
            } else if in_set(row, "g_satker_kode") {
                // NULL is one group here but was not a value to
                // `COUNT(DISTINCT kdsatker_keu)`, which skips NULLs.
                if row.get::<_, Option<String>>("satker_kode").is_some() {
                    total_satker += 1;
                }
            } else if in_set(row, "g_tahun") {
                if let Some(tahun) = row.get::<_, Option<i32>>("tahun") {
                    per_tahun.push(TahunStat { tahun, count });
                }
            } else {
                total_aset = count;
                total_nilai = row.get("nilai");
            }
        }

        // Ordering moves here with the aggregation. The old queries left ties
        // to whatever order the executor produced; these tiebreakers make the
        // same counts come back in the same order every time.
        kondisi_breakdown.sort_by(|a, b| b.count.cmp(&a.count).then(a.kondisi.cmp(&b.kondisi)));
        kategori_breakdown.sort_by(|a, b| b.count.cmp(&a.count).then(a.kategori.cmp(&b.kategori)));
        top_satker.sort_by(|a, b| b.count.cmp(&a.count).then(a.satker.cmp(&b.satker)));
        top_satker.truncate(10);
        per_tahun.sort_by_key(|t| std::cmp::Reverse(t.tahun));
        per_tahun.truncate(20);

        Ok(BankAsetDashboard {
            total_aset,
            total_nilai_perolehan: total_nilai,
            total_satker,
            total_kategori,
            kondisi_breakdown,
            kategori_breakdown,
            top_satker,
            per_tahun,
        })
    }

    pub async fn sebaran(&self, scope: &AsetScope) -> AppResult<BankAsetSebaran> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let where_clause = scope
            .push_condition(&mut scope_params)
            .map(|c| format!(" WHERE {c}"))
            .unwrap_or_default();

        let rows = client
            .query(
                &format!(
                    "SELECT
                    kdsatker_keu AS kode_satker,
                    COALESCE(nama_satker, 'TIDAK DIKETAHUI') AS nama_satker,
                    COUNT(*)::BIGINT AS total_aset,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai_perolehan,
                    COUNT(*) FILTER (WHERE UPPER(COALESCE(ur_kondisi, '')) = 'BAIK')::BIGINT AS aset_baik,
                    COUNT(*) FILTER (WHERE UPPER(COALESCE(ur_kondisi, '')) LIKE 'RUSAK%')::BIGINT AS aset_rusak
                 FROM integrasi.siman_aset{where_clause}
                 GROUP BY kdsatker_keu, COALESCE(nama_satker, 'TIDAK DIKETAHUI')
                 ORDER BY total_aset DESC"
                ),
                &as_sql_params(&scope_params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let satker = rows
            .iter()
            .map(|r| SebaranSatker {
                kode_satker: r.try_get("kode_satker").ok(),
                nama_satker: r.get("nama_satker"),
                total_aset: r.get("total_aset"),
                nilai_perolehan: r.get("nilai_perolehan"),
                aset_baik: r.get("aset_baik"),
                aset_rusak: r.get("aset_rusak"),
            })
            .collect();

        Ok(BankAsetSebaran { satker })
    }

    pub async fn last_sync(&self, scope: &AsetScope) -> AppResult<LastSyncInfo> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let where_clause = scope
            .push_condition(&mut scope_params)
            .map(|c| format!(" WHERE {c}"))
            .unwrap_or_default();

        let row = client
            .query_one(
                &format!(
                    "SELECT MAX(updated_at) AS last_sync, COUNT(*)::BIGINT AS total
                 FROM integrasi.siman_aset{where_clause}"
                ),
                &as_sql_params(&scope_params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let last_sync: Option<DateTime<Utc>> = row.try_get("last_sync").ok();
        let total: i64 = row.get("total");

        Ok(LastSyncInfo {
            last_sync_at: last_sync,
            total_aset: total,
            source: "SIMAN".to_string(),
        })
    }

    /// Distinct values (with counts) for the filterable columns, so the FE can
    /// populate filter dropdowns DYNAMICALLY from the actual data instead of
    /// hard-coded lists. Columns are a fixed allow-list (no arbitrary-column SQL).
    /// Distinct filter values within what the caller can see, narrowed by
    /// `filter` so a selected region shrinks the satker list rather than
    /// leaving 554 entries for a reader who has already said which seven they
    /// mean.
    pub async fn filter_options(
        &self,
        filter: &AsetFilter,
        scope: &AsetScope,
    ) -> AppResult<BankAsetFilterOptions> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        // RBAC scope predicate (references `$1` when present): only offer filter
        // values that exist within the caller's visible asset set.
        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let mut conds: Vec<String> = Vec::new();
        if let Some(c) = scope.push_condition(&mut scope_params) {
            conds.push(c);
        }
        push_asset_filters(filter, &mut scope_params, &mut conds);
        let and_clause = if conds.is_empty() {
            String::new()
        } else {
            format!(" AND {}", conds.join(" AND "))
        };
        let scope_p = as_sql_params(&scope_params);

        // (label_expr, column) — column names are a hard-coded allow-list.
        async fn distinct(
            client: &deadpool_postgres::Object,
            col: &str,
            and_clause: &str,
            scope_p: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        ) -> AppResult<Vec<FilterOption>> {
            // `col` is from the fixed allow-list below, never user input.
            let sql = format!(
                "SELECT {col} AS value, COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset
                 WHERE {col} IS NOT NULL AND {col} <> ''{and_clause}
                 GROUP BY {col}
                 ORDER BY count DESC, value ASC
                 LIMIT {FILTER_OPTION_LIMIT}"
            );
            let rows = client
                .query(&sql, scope_p)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            // Hitting the cap means the dropdown is INCOMPLETE, and the old
            // limit hid that: it dropped the satkers with the fewest assets,
            // which are the smallest ones — the least likely to be missed and
            // the least able to complain. Say so instead of truncating in
            // silence.
            if rows.len() as i64 >= FILTER_OPTION_LIMIT {
                warn!(
                    "bank_aset filter options for `{col}` hit the {FILTER_OPTION_LIMIT} cap — \
                     the dropdown is truncated and some values cannot be selected"
                );
            }
            Ok(rows
                .iter()
                .map(|r| FilterOption {
                    value: r.get("value"),
                    count: r.get("count"),
                    label: None,
                })
                .collect())
        }

        // Code-keyed options carry a separate label, so they need their own
        // query rather than the `distinct` helper above.
        //
        // Both read the asset table through a SUBQUERY that carries the scope
        // predicate. That is not stylistic: the predicate names `kdsatker_keu`
        // unqualified, and the joins below introduce a second column of that
        // name — spliced into the outer query it would be ambiguous, and
        // Postgres would reject it at parse time.
        // `value` is the MySIMKARI `kode_satker`, not `kdsatker_keu`: it is the
        // identity both halves of the dashboard key on, so one selection
        // filters both. Satkers SIMAN knows but MySIMKARI does not (65 on
        // staging) have no such identity and are therefore not selectable —
        // they remain visible in the unfiltered totals.
        let satker_kode_sql = format!(
            "SELECT m.kode_satker AS value,
                    MIN(COALESCE(m.nama_satker, a.nama_satker)) AS label,
                    COUNT(*)::BIGINT AS count
               FROM (SELECT kdsatker_keu, nama_satker
                       FROM integrasi.siman_aset
                      WHERE kdsatker_keu IS NOT NULL AND kdsatker_keu <> ''{and_clause}) a
               JOIN integrasi.v_satker_code_map m ON m.kdsatker_keu = a.kdsatker_keu
              GROUP BY m.kode_satker
              ORDER BY count DESC, value ASC
              LIMIT {FILTER_OPTION_LIMIT}"
        );
        let satker_kode = client
            .query(&satker_kode_sql, &scope_p)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .iter()
            .map(|r| FilterOption {
                value: r.get("value"),
                count: r.get("count"),
                label: r.get::<_, Option<String>>("label"),
            })
            .collect();

        // Wilayah label comes from `v_satker_wilayah` — THE definition of the
        // tier (integrasi migration 005), the same one every MySIMKARI-side
        // scope climbs. Falling back to the bare code keeps SIMAN-only satkers
        // (65 on staging, whose kode_satker MySIMKARI does not know) visible
        // rather than dropping their region from the list.
        let wilayah_sql = format!(
            "SELECT a.wk AS value,
                    MIN(w.wilayah_nama) AS label,
                    COUNT(*)::BIGINT AS count
               FROM (SELECT substring(kdsatker_keu FROM 6 FOR 4) AS wk, kdsatker_keu
                       FROM integrasi.siman_aset
                      WHERE kdsatker_keu IS NOT NULL AND kdsatker_keu <> ''{and_clause}) a
               LEFT JOIN integrasi.v_satker_code_map m ON m.kdsatker_keu = a.kdsatker_keu
               LEFT JOIN integrasi.v_satker_wilayah  w ON w.kode_satker  = m.kode_satker
              WHERE a.wk IS NOT NULL AND a.wk <> ''
              GROUP BY a.wk
              ORDER BY count DESC, value ASC
              LIMIT {FILTER_OPTION_LIMIT}"
        );
        let wilayah = client
            .query(&wilayah_sql, &scope_p)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .iter()
            .map(|r| FilterOption {
                value: r.get("value"),
                count: r.get("count"),
                label: r.get::<_, Option<String>>("label"),
            })
            .collect();

        Ok(BankAsetFilterOptions {
            jenis: distinct(&client, "jenis_aset", &and_clause, &scope_p).await?,
            // Same expression as the list projection and the filter predicate:
            // a dropdown that offers values the predicate cannot match filters
            // to nothing.
            kategori: distinct(&client, ASSET_KATEGORI_SQL, &and_clause, &scope_p).await?,
            kondisi: distinct(&client, "ur_kondisi", &and_clause, &scope_p).await?,
            satker: distinct(&client, "nama_satker", &and_clause, &scope_p).await?,
            satker_kode,
            wilayah,
        })
    }
}

/// The SIMAN-asset predicate, shared by every surface that reads
/// `integrasi.siman_aset`.
///
/// It exists so the list and the dashboard cannot disagree about what a filter
/// MEANS. They used to: the list honoured jenis/kategori/kondisi/satker while
/// the dashboard honoured nothing at all, so "Peralatan Mesin Non TIK" narrowed
/// a table and left the summary above it describing the whole population.
///
/// Every field NARROWS. None of them can widen: [`push_asset_filters`] is only
/// ever called after the caller's [`AsetScope`] predicate is already in the
/// condition list, so the strongest thing a filter can do to an out-of-scope
/// satker is return nothing.
#[derive(Debug, Default, Clone)]
pub struct AsetFilter {
    pub jenis: Option<String>,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    /// Legacy: matches `nama_satker` exactly. Kept because the Bank Aset page's
    /// dropdown is populated with names; prefer [`Self::satker_kode`].
    pub satker: Option<String>,
    /// The satker's MySIMKARI `kode_satker` — the system's canonical satker
    /// identity (#43, #94), mapped to SIMAN's `kdsatker_keu` by the same
    /// cross-reference view [`AsetScope::Satker`] uses.
    ///
    /// NOT `kdsatker_keu` directly, even though that is what this table is
    /// keyed by. The perlengkapan half of the dashboard is keyed by
    /// `kode_satker`, so a drill-down carrying the SIMAN code would filter one
    /// half and not the other — and `satker` above is a NAME, which two
    /// satkers can share (554 distinct names for 556 distinct codes).
    pub satker_kode: Option<String>,
    /// SIMAN wilayah code, digits 6-9 of `kdsatker_keu` — the same expression
    /// [`AsetScope`]'s wilayah tier compares against, so a pusat user filtering
    /// to a region sees exactly what that region's validator sees.
    pub wilayah: Option<String>,
    /// Acquisition date bounds, inclusive, as `YYYY-MM-DD`.
    pub tgl_from: Option<String>,
    pub tgl_to: Option<String>,
    pub search: Option<String>,
}

pub struct ListFilter {
    pub page: i32,
    pub per_page: i32,
    pub sort: Option<String>,
    pub f: AsetFilter,
}

/// Append `filter` as extra `AND` conditions. Call AFTER the scope predicate.
///
/// `tgl_perlh` is compared as TEXT rather than cast to `date`. That is not
/// laziness: the column IS text, and every one of the 624 533 staging rows is
/// ISO `YYYY-MM-DD` (measured: zero rows fail `^\d{4}-\d{2}-\d{2}$`), for
/// which lexicographic order and calendar order are the same. A `::date` cast
/// would be equivalent today and would throw on the first malformed row a
/// future ingest writes — and with `panic = "abort"` that is the process, not
/// the request.
pub fn push_asset_filters(
    filter: &AsetFilter,
    params: &mut Vec<BoxedParam>,
    conditions: &mut Vec<String>,
) {
    if let Some(jenis) = &filter.jenis {
        params.push(Box::new(jenis.clone()));
        conditions.push(format!("jenis_aset = ${}", params.len()));
    }
    if let Some(cat) = &filter.kategori {
        params.push(Box::new(cat.clone()));
        conditions.push(format!("{ASSET_KATEGORI_SQL} = ${}", params.len()));
    }
    if let Some(kondisi) = &filter.kondisi {
        params.push(Box::new(kondisi.clone()));
        conditions.push(format!("ur_kondisi = ${}", params.len()));
    }
    if let Some(satker) = &filter.satker {
        params.push(Box::new(satker.clone()));
        conditions.push(format!("nama_satker = ${}", params.len()));
    }
    if let Some(kode) = &filter.satker_kode {
        params.push(Box::new(kode.clone()));
        conditions.push(format!(
            "kdsatker_keu IN (SELECT kdsatker_keu FROM integrasi.v_satker_code_map \
             WHERE kode_satker = ${} AND kdsatker_keu IS NOT NULL)",
            params.len()
        ));
    }
    if let Some(w) = &filter.wilayah {
        params.push(Box::new(w.clone()));
        conditions.push(format!(
            "substring(kdsatker_keu FROM 6 FOR 4) = ${}",
            params.len()
        ));
    }
    if let Some(from) = &filter.tgl_from {
        params.push(Box::new(from.clone()));
        conditions.push(format!("tgl_perlh >= ${}", params.len()));
    }
    if let Some(to) = &filter.tgl_to {
        params.push(Box::new(to.clone()));
        conditions.push(format!("tgl_perlh <= ${}", params.len()));
    }
    if let Some(q) = &filter.search {
        // `no_aset` in this list is what makes the FE placeholder
        // "Cari nama/kode/NUP/merk..." truthful: NUP *is* `no_aset` (see
        // ASSET_NUP_SQL), so searching by NUP resolves here. Searching the
        // `nup` column instead would match nothing but the e2e seed rows.
        params.push(Box::new(format!("%{}%", q)));
        let idx = params.len();
        conditions.push(format!(
            "(ur_sskel ILIKE ${idx} OR nama ILIKE ${idx} OR kd_brg ILIKE ${idx} \
             OR no_aset ILIKE ${idx} OR merk ILIKE ${idx})",
            idx = idx
        ));
    }
}

// The release profile builds with panic=abort, so ANY decode panic here kills
// the whole process (observed: e2e run 28566784769 — `id` decoded as Uuid
// against the BIGSERIAL column aborted the service mid-request). Every column
// of integrasi.siman_aset except id/jenis_aset is nullable, so decode
// defensively: no bare row.get() on nullable columns.
fn row_to_item(row: &Row) -> BankAsetItem {
    BankAsetItem {
        id: row.get::<_, i64>("id").to_string(),
        kategori_aset: row
            .try_get::<_, Option<String>>("kategori_aset")
            .ok()
            .flatten()
            .unwrap_or_default(),
        no_aset: row
            .try_get::<_, Option<String>>("no_aset")
            .ok()
            .flatten()
            .unwrap_or_default(),
        nama_aset: row
            .try_get("ur_sskel")
            .ok()
            .or_else(|| row.try_get("nama").ok()),
        kode_barang: row.try_get("kd_brg").ok(),
        merk: row.try_get("merk").ok(),
        tipe: row.try_get("tipe").ok(),
        kondisi: row.try_get("ur_kondisi").ok(),
        lokasi: row.try_get("alamat").ok(),
        satker: row.try_get("nama_satker").ok(),
        kode_satker: row.try_get("kdsatker_keu").ok(),
        nup: row.try_get("nup").ok(),
        nilai_perolehan: row.try_get("rph_aset").ok(),
        tgl_perolehan: row.try_get("tgl_perlh").ok(),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bind values, rendered as the debug strings tokio-postgres would
    /// serialise — enough to assert ORDER and CONTENT without a connection.
    fn render(params: &[BoxedParam]) -> Vec<String> {
        params.iter().map(|p| format!("{p:?}")).collect()
    }

    #[test]
    fn nup_alone_adds_no_conditions() {
        let mut params: Vec<BoxedParam> = vec![Box::new("677".to_string())];
        let extra = push_identity_conditions(
            &AsetIdentity {
                nup: "677",
                kode_barang: None,
                satker_code: None,
            },
            &mut params,
        );
        assert_eq!(extra, "");
        assert_eq!(params.len(), 1, "no extra binds");
    }

    #[test]
    fn kode_barang_is_compared_dot_insensitively_on_both_sides() {
        let mut params: Vec<BoxedParam> = vec![Box::new("677".to_string())];
        let extra = push_identity_conditions(
            &AsetIdentity {
                nup: "677",
                // Dotted presentation form, as the penghapusan record stores it.
                kode_barang: Some("3.05.02.01.002"),
                satker_code: None,
            },
            &mut params,
        );
        // The COLUMN side strips dots...
        assert!(
            extra.contains("replace(") && extra.contains("'.', ''"),
            "column side must be normalised: {extra}"
        );
        // ...and so does the BOUND VALUE. Normalising only one half is the bug
        // this pair of assertions exists to catch: SIMAN holds `3050201002`,
        // so a raw comparison matches zero rows and reads as "not in SIMAN".
        assert_eq!(render(&params)[1], "\"3050201002\"");
        assert!(extra.contains("= $2"), "binds in order: {extra}");
    }

    #[test]
    fn satker_is_translated_through_the_integrasi_code_map() {
        let mut params: Vec<BoxedParam> = vec![Box::new("677".to_string())];
        let extra = push_identity_conditions(
            &AsetIdentity {
                nup: "677",
                kode_barang: None,
                satker_code: Some("0200010"),
            },
            &mut params,
        );
        // MySIMKARI kode_satker is NOT SIMAN's kdsatker_keu; a direct
        // comparison would silently match nothing.
        assert!(
            extra.contains("integrasi.v_satker_code_map"),
            "must go through the map: {extra}"
        );
        assert!(
            !extra.contains("nama_satker"),
            "never match on name: {extra}"
        );
        assert_eq!(render(&params)[1], "\"0200010\"");
    }

    #[test]
    fn placeholders_follow_the_order_values_are_pushed() {
        let mut params: Vec<BoxedParam> = vec![Box::new("677".to_string())];
        let extra = push_identity_conditions(
            &AsetIdentity {
                nup: "677",
                kode_barang: Some("3050201002"),
                satker_code: Some("0200010"),
            },
            &mut params,
        );
        assert_eq!(render(&params).len(), 3);
        assert_eq!(render(&params)[1], "\"3050201002\"");
        assert_eq!(render(&params)[2], "\"0200010\"");
        // $2 is the barang code, $3 the satker — swapping them would compare a
        // satker code against a barang column and return nothing, with no error.
        let kb = extra.find("= $2").expect("barang bound at $2");
        let sk = extra.find("kode_satker = $3").expect("satker bound at $3");
        assert!(kb < sk, "conditions appear in bind order: {extra}");
    }

    #[test]
    fn scope_binds_after_the_identity_binds() {
        // Regression guard for the numbering: AsetScope::push_condition appends
        // to the SAME vector, so it must be called last or its $n would point
        // at an identity value.
        let mut params: Vec<BoxedParam> = vec![Box::new("677".to_string())];
        let extra = push_identity_conditions(
            &AsetIdentity {
                nup: "677",
                kode_barang: Some("3050201002"),
                satker_code: None,
            },
            &mut params,
        );
        let scope = AsetScope::Satker("0200010".to_string());
        let cond = scope.push_condition(&mut params).expect("restricted");
        assert!(extra.contains("$2"), "identity took $2: {extra}");
        assert!(cond.contains("$3"), "scope took $3: {cond}");
    }
}
