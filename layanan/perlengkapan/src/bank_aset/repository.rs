use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use super::models::*;
use crate::shared::error::{AppError, AppResult, not_found};

#[derive(Clone)]
pub struct BankAsetRepository {
    pool: Pool,
}

impl BankAsetRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, filter: ListFilter) -> AppResult<(Vec<BankAsetItem>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

        if let Some(jenis) = &filter.jenis {
            params.push(Box::new(jenis.clone()));
            conditions.push(format!("jenis_aset = ${}", params.len()));
        }
        if let Some(cat) = &filter.kategori {
            params.push(Box::new(cat.clone()));
            conditions.push(format!("kategori_aset = ${}", params.len()));
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
            "SELECT id, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, nup,
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

    /// Find the slim lookup record for a NUP. Used by the pemakaian-bmn
    /// form to auto-fill `kode_barang` + `nama_barang` (+ a few extras the
    /// UI may want to display) the moment the user types a NUP. Returns
    /// `None` when no asset with that NUP exists.
    pub async fn find_lookup_by_nup(&self, nup: &str) -> AppResult<Option<BankAsetLookup>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let row = client
            .query_opt(
                "SELECT id, nup, kd_brg, nama, merk, tgl_perlh, ur_kondisi, nama_satker,
                        (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE NULL END) AS nilai_perolehan
                 FROM integrasi.siman_aset
                 WHERE nup = $1
                 LIMIT 1",
                &[&nup],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| BankAsetLookup {
            id: r.get::<_, Uuid>("id"),
            nup: r
                .try_get::<_, Option<String>>("nup")
                .ok()
                .flatten()
                .unwrap_or_else(|| nup.to_string()),
            kode_barang: r.try_get::<_, Option<String>>("kd_brg").ok().flatten(),
            nama_barang: r.try_get::<_, Option<String>>("nama").ok().flatten(),
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
    pub async fn find_nilai_perolehan(
        &self,
        nup: &str,
        kode_barang: Option<&str>,
    ) -> AppResult<Option<f64>> {
        let lookup = self.find_lookup_by_nup(nup).await?;
        let Some(l) = lookup else {
            return Ok(None);
        };
        if let (Some(expected), Some(actual)) = (kode_barang, l.kode_barang.as_deref())
            && expected != actual
        {
            // NUP cocok tapi kode_barang berbeda → data tidak konsisten
            // → jangan kembalikan nilai (caller akan handle sebagai
            // "tidak ditemukan").
            return Ok(None);
        }
        Ok(l.nilai_perolehan)
    }

    pub async fn get(&self, id: Uuid) -> AppResult<BankAsetItem> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let row = client
            .query_opt(
                "SELECT id, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, kdsatker_keu, nup,
                 (CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END) AS rph_aset,
                 tgl_perlh, updated_at
                 FROM integrasi.siman_aset WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| row_to_item(&r))
            .ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    pub async fn dashboard(&self) -> AppResult<BankAsetDashboard> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let totals = client
            .query_one(
                "SELECT
                    COUNT(*)::BIGINT AS total_aset,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS total_nilai,
                    COUNT(DISTINCT nama_satker)::BIGINT AS total_satker,
                    COUNT(DISTINCT kategori_aset)::BIGINT AS total_kategori
                 FROM integrasi.siman_aset",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let total_aset: i64 = totals.get("total_aset");
        let total_nilai: f64 = totals.get("total_nilai");
        let total_satker: i64 = totals.get("total_satker");
        let total_kategori: i64 = totals.get("total_kategori");

        let kondisi_rows = client
            .query(
                "SELECT COALESCE(ur_kondisi, 'TIDAK DIKETAHUI') AS kondisi, COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset
                 GROUP BY COALESCE(ur_kondisi, 'TIDAK DIKETAHUI')
                 ORDER BY count DESC",
                &[],
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
                "SELECT kategori_aset,
                    COUNT(*)::BIGINT AS count,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai
                 FROM integrasi.siman_aset
                 GROUP BY kategori_aset
                 ORDER BY count DESC",
                &[],
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
                "SELECT COALESCE(nama_satker, 'TIDAK DIKETAHUI') AS satker,
                    COUNT(*)::BIGINT AS count,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai
                 FROM integrasi.siman_aset
                 GROUP BY COALESCE(nama_satker, 'TIDAK DIKETAHUI')
                 ORDER BY count DESC
                 LIMIT 10",
                &[],
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
                "SELECT
                    NULLIF(SUBSTRING(tgl_perlh FROM 1 FOR 4), '')::INT AS tahun,
                    COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset
                 WHERE tgl_perlh IS NOT NULL AND SUBSTRING(tgl_perlh FROM 1 FOR 4) ~ '^[0-9]{4}$'
                 GROUP BY tahun
                 ORDER BY tahun DESC
                 LIMIT 20",
                &[],
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

    pub async fn sebaran(&self) -> AppResult<BankAsetSebaran> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let rows = client
            .query(
                "SELECT
                    kdsatker_keu AS kode_satker,
                    COALESCE(nama_satker, 'TIDAK DIKETAHUI') AS nama_satker,
                    COUNT(*)::BIGINT AS total_aset,
                    COALESCE(SUM(CASE WHEN rph_aset ~ '^[0-9]+(\\.[0-9]+)?$' THEN rph_aset::FLOAT8 ELSE 0 END), 0)::FLOAT8 AS nilai_perolehan,
                    COUNT(*) FILTER (WHERE UPPER(COALESCE(ur_kondisi, '')) = 'BAIK')::BIGINT AS aset_baik,
                    COUNT(*) FILTER (WHERE UPPER(COALESCE(ur_kondisi, '')) LIKE 'RUSAK%')::BIGINT AS aset_rusak
                 FROM integrasi.siman_aset
                 GROUP BY kdsatker_keu, COALESCE(nama_satker, 'TIDAK DIKETAHUI')
                 ORDER BY total_aset DESC",
                &[],
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

    pub async fn last_sync(&self) -> AppResult<LastSyncInfo> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        let row = client
            .query_one(
                "SELECT MAX(updated_at) AS last_sync, COUNT(*)::BIGINT AS total
                 FROM integrasi.siman_aset",
                &[],
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
    pub async fn filter_options(&self) -> AppResult<BankAsetFilterOptions> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("DB conn: {}", e)))?;

        // (label_expr, column) — column names are a hard-coded allow-list.
        async fn distinct(
            client: &deadpool_postgres::Object,
            col: &str,
        ) -> AppResult<Vec<FilterOption>> {
            // `col` is from the fixed allow-list below, never user input.
            let sql = format!(
                "SELECT {col} AS value, COUNT(*)::BIGINT AS count
                 FROM integrasi.siman_aset
                 WHERE {col} IS NOT NULL AND {col} <> ''
                 GROUP BY {col}
                 ORDER BY count DESC, value ASC
                 LIMIT 500"
            );
            let rows = client
                .query(&sql, &[])
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            Ok(rows
                .iter()
                .map(|r| FilterOption {
                    value: r.get("value"),
                    count: r.get("count"),
                })
                .collect())
        }

        Ok(BankAsetFilterOptions {
            jenis: distinct(&client, "jenis_aset").await?,
            kategori: distinct(&client, "kategori_aset").await?,
            kondisi: distinct(&client, "ur_kondisi").await?,
            satker: distinct(&client, "nama_satker").await?,
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

fn row_to_item(row: &Row) -> BankAsetItem {
    BankAsetItem {
        id: row.get("id"),
        kategori_aset: row.get("kategori_aset"),
        no_aset: row.get("no_aset"),
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
