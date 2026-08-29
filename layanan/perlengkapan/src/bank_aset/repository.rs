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
        if let Some(q) = &filter.search {
            // `no_aset` in this list is what makes the FE placeholder
            // "Cari nama/kode/NUP/merk..." truthful: NUP *is* `no_aset` (see
            // ASSET_NUP_SQL), so searching by NUP resolves here. Searching the
            // `nup` column instead would match nothing but the e2e seed rows.
            params.push(Box::new(format!("%{}%", q)));
            let idx = params.len();
            conditions.push(format!(
                "(ur_sskel ILIKE ${idx} OR nama ILIKE ${idx} OR kd_brg ILIKE ${idx} OR no_aset ILIKE ${idx} OR merk ILIKE ${idx})",
                idx = idx
            ));
        }

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

    pub async fn dashboard(&self, scope: &AsetScope) -> AppResult<BankAsetDashboard> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        // RBAC scope predicate (references `$1` when present); shared by every
        // aggregate below. `where_clause` for queries without a WHERE, `and_clause`
        // for the one that already has one.
        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let scope_cond = scope.push_condition(&mut scope_params);
        let where_clause = scope_cond
            .as_ref()
            .map(|c| format!(" WHERE {c}"))
            .unwrap_or_default();
        let and_clause = scope_cond
            .as_ref()
            .map(|c| format!(" AND {c}"))
            .unwrap_or_default();
        let p = as_sql_params(&scope_params);

        let totals = client
            .query_one(
                &format!(
                    "SELECT
                    COUNT(*)::BIGINT AS total_aset,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS total_nilai,
                    -- Counted on the CODE, not the name. Measured on staging:
                    -- 556 distinct `kdsatker_keu` but only 554 distinct
                    -- `nama_satker`, so two satkers were folded into one and the
                    -- tile under-reported. A satker's identity is its code
                    -- (#43); `nama_satker` is a label that can collide.
                    COUNT(DISTINCT kdsatker_keu)::BIGINT AS total_satker,
                    COUNT(DISTINCT {ASSET_KATEGORI_SQL})::BIGINT AS total_kategori
                 FROM integrasi.siman_aset{where_clause}"
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let total_aset: i64 = totals.get("total_aset");
        let total_nilai: f64 = totals.get("total_nilai");
        let total_satker: i64 = totals.get("total_satker");
        let total_kategori: i64 = totals.get("total_kategori");

        let kondisi_rows = client
            .query(
                &format!(
                    // Condition read through the shared expression rather than
                    // a local `COALESCE(ur_kondisi, …)`. This was the fifth
                    // spelling of "an asset's condition" in the crate: it
                    // ignored `kondisi` entirely and treated `''` as a real
                    // value. Harmless today (`kondisi` is populated in 0 of
                    // 624 533 staging rows) and precisely the kind of harmless
                    // that stops being harmless the day the ingest is fixed.
                    "SELECT COALESCE(NULLIF({ASSET_KONDISI_SQL}, ''), 'TIDAK DIKETAHUI') AS kondisi,
                        COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset{where_clause}
                 GROUP BY 1
                 ORDER BY count DESC"
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let kondisi_breakdown: Vec<KondisiStat> = kondisi_rows
            .iter()
            .map(|r| KondisiStat {
                kondisi: r.get("kondisi"),
                count: r.get("count"),
            })
            .collect();

        let kat_rows = client
            .query(
                &format!(
                    // The asset-type axis comes from SIMAN's `jenis_aset`, NOT
                    // from `kategori_aset` — see ASSET_KATEGORI_SQL.
                    "SELECT {ASSET_KATEGORI_SQL} AS kategori_aset,
                    COUNT(*)::BIGINT AS count,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai
                 FROM integrasi.siman_aset{where_clause}
                 GROUP BY {ASSET_KATEGORI_SQL}
                 ORDER BY count DESC"
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let kategori_breakdown: Vec<KategoriStat> = kat_rows
            .iter()
            .map(|r| KategoriStat {
                kategori: r.get("kategori_aset"),
                count: r.get("count"),
                nilai: r.get("nilai"),
            })
            .collect();

        let satker_rows = client
            .query(
                &format!(
                    "SELECT COALESCE(nama_satker, 'TIDAK DIKETAHUI') AS satker,
                    COUNT(*)::BIGINT AS count,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai
                 FROM integrasi.siman_aset{where_clause}
                 GROUP BY COALESCE(nama_satker, 'TIDAK DIKETAHUI')
                 ORDER BY count DESC
                 LIMIT 10"
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let top_satker: Vec<SatkerStat> = satker_rows
            .iter()
            .map(|r| SatkerStat {
                satker: r.get("satker"),
                count: r.get("count"),
                nilai: r.get("nilai"),
            })
            .collect();

        let tahun_rows = client
            .query(
                &format!(
                    "SELECT
                    NULLIF(SUBSTRING(tgl_perlh FROM 1 FOR 4), '')::INT AS tahun,
                    COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset
                 WHERE tgl_perlh IS NOT NULL AND SUBSTRING(tgl_perlh FROM 1 FOR 4) ~ '^[0-9]{{4}}$'{and_clause}
                 GROUP BY tahun
                 ORDER BY tahun DESC
                 LIMIT 20"
                ),
                &p,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let per_tahun: Vec<TahunStat> = tahun_rows
            .iter()
            .filter_map(|r| {
                let tahun: Option<i32> = r.try_get("tahun").ok();
                tahun.map(|t| TahunStat {
                    tahun: t,
                    count: r.get("count"),
                })
            })
            .collect();

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
    pub async fn filter_options(&self, scope: &AsetScope) -> AppResult<BankAsetFilterOptions> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        // RBAC scope predicate (references `$1` when present): only offer filter
        // values that exist within the caller's visible asset set.
        let mut scope_params: Vec<BoxedParam> = Vec::new();
        let and_clause = scope
            .push_condition(&mut scope_params)
            .map(|c| format!(" AND {c}"))
            .unwrap_or_default();
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
                })
                .collect())
        }

        Ok(BankAsetFilterOptions {
            jenis: distinct(&client, "jenis_aset", &and_clause, &scope_p).await?,
            // Same expression as the list projection and the filter predicate:
            // a dropdown that offers values the predicate cannot match filters
            // to nothing.
            kategori: distinct(&client, ASSET_KATEGORI_SQL, &and_clause, &scope_p).await?,
            kondisi: distinct(&client, "ur_kondisi", &and_clause, &scope_p).await?,
            satker: distinct(&client, "nama_satker", &and_clause, &scope_p).await?,
        })
    }
}

pub struct ListFilter {
    pub page: i32,
    pub per_page: i32,
    pub jenis: Option<String>,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
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
