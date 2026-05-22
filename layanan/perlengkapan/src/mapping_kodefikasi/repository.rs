//! # Mapping Kodefikasi Repository
//!
//! Simplified read-only database operations for mapping kodefikasi.
//! No proposal/verification CRUD — only queries for standard/non-standard codes.

use crate::mapping_kodefikasi::models::*;
use crate::shared::error::AppError;
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Repository for mapping kodefikasi read-only operations
pub struct MappingRepository {
    pool: Pool,
}

impl MappingRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// List non-standard codes from SIMAN data with pagination and optional filters
    pub async fn list_non_standard_codes(
        &self,
        search: Option<&str>,
        satker_id: Option<Uuid>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<NonStandardCode>, i64), AppError> {
        let offset = ((page - 1) * per_page) as i64;

        let mut conditions = vec![String::from("mb.id IS NULL")];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(search_term) = search {
            conditions.push(format!(
                "(sa.kode_barang ILIKE ${} OR sa.nama_barang ILIKE ${})",
                param_idx,
                param_idx + 1
            ));
            let like_term = format!("%{}%", search_term);
            params.push(Box::new(like_term.clone()));
            params.push(Box::new(like_term));
            param_idx += 2;
        }

        if let Some(sid) = satker_id {
            conditions.push(format!("sa.satker_id = ${}", param_idx));
            params.push(Box::new(sid));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");

        let count_query = format!(
            r#"
            SELECT COUNT(DISTINCT (sa.kode_barang, sa.nama_barang, sa.satker_id))
            FROM integrasi.siman_aset_tanah sa
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            WHERE {}
            "#,
            where_clause
        );

        let client = self.pool.get().await?;
        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client.query_one(&count_query, &params_refs).await?;
        let total: i64 = count_row.get(0);

        let data_query = format!(
            r#"
            SELECT DISTINCT
                sa.kode_barang as kode_lama,
                sa.nama_barang as nama_lama,
                sa.satker_id,
                COUNT(*) as jumlah_aset
            FROM integrasi.siman_aset_tanah sa
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            WHERE {}
            GROUP BY sa.kode_barang, sa.nama_barang, sa.satker_id
            ORDER BY jumlah_aset DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        params.push(Box::new(per_page as i64));
        params.push(Box::new(offset));
        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&data_query, &params_refs).await?;

        let codes = rows
            .into_iter()
            .map(|row| NonStandardCode {
                kode_lama: row.get("kode_lama"),
                nama_lama: row.get("nama_lama"),
                satker_id: row.get("satker_id"),
                jumlah_aset: row.get("jumlah_aset"),
                suggested_mapping: None,
            })
            .collect();

        Ok((codes, total))
    }

    /// List standard BMN codes from master table
    pub async fn list_standard_codes(
        &self,
        search: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<StandardBmnCode>, i64), AppError> {
        let offset = ((page - 1) * per_page) as i64;
        let client = self.pool.get().await?;

        if let Some(search_term) = search {
            let like_term = format!("%{}%", search_term);

            let count_row = client
                .query_one(
                    r#"
                    SELECT COUNT(*) FROM perlengkapan.ms_barang
                    WHERE kode ILIKE $1 OR nama ILIKE $2
                    "#,
                    &[&like_term, &like_term],
                )
                .await?;
            let total: i64 = count_row.get(0);

            let rows = client
                .query(
                    r#"
                    SELECT
                        mb.id, mb.kode, mb.nama, mb.kategori,
                        (SELECT COUNT(*) FROM integrasi.siman_aset_tanah sa WHERE sa.kode_barang = mb.kode) as jumlah_aset
                    FROM perlengkapan.ms_barang mb
                    WHERE mb.kode ILIKE $1 OR mb.nama ILIKE $2
                    ORDER BY mb.kode
                    LIMIT $3 OFFSET $4
                    "#,
                    &[&like_term, &like_term, &(per_page as i64), &offset],
                )
                .await?;

            let codes = rows
                .into_iter()
                .map(|row| StandardBmnCode {
                    id: row.get("id"),
                    kode: row.get("kode"),
                    nama: row.get("nama"),
                    kategori: row.get("kategori"),
                    jumlah_aset: row.get("jumlah_aset"),
                })
                .collect();

            Ok((codes, total))
        } else {
            let count_row = client
                .query_one("SELECT COUNT(*) FROM perlengkapan.ms_barang", &[])
                .await?;
            let total: i64 = count_row.get(0);

            let rows = client
                .query(
                    r#"
                    SELECT
                        mb.id, mb.kode, mb.nama, mb.kategori,
                        (SELECT COUNT(*) FROM integrasi.siman_aset_tanah sa WHERE sa.kode_barang = mb.kode) as jumlah_aset
                    FROM perlengkapan.ms_barang mb
                    ORDER BY mb.kode
                    LIMIT $1 OFFSET $2
                    "#,
                    &[&(per_page as i64), &offset],
                )
                .await?;

            let codes = rows
                .into_iter()
                .map(|row| StandardBmnCode {
                    id: row.get("id"),
                    kode: row.get("kode"),
                    nama: row.get("nama"),
                    kategori: row.get("kategori"),
                    jumlah_aset: row.get("jumlah_aset"),
                })
                .collect();

            Ok((codes, total))
        }
    }

    /// Suggest mapping using fuzzy matching
    pub async fn suggest_mapping(
        &self,
        nama_lama: &str,
    ) -> Result<Vec<MappingSuggestion>, AppError> {
        let query = r#"
            SELECT
                id,
                kode,
                nama,
                similarity(nama, $1) as score
            FROM perlengkapan.ms_barang
            WHERE similarity(nama, $1) > 0.3
            ORDER BY score DESC
            LIMIT 5
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[&nama_lama]).await?;

        let suggestions = rows
            .into_iter()
            .map(|row| MappingSuggestion {
                barang_id: row.get("id"),
                kode_baru: row.get("kode"),
                nama_baru: row.get("nama"),
                similarity_score: row.get("score"),
            })
            .collect();

        Ok(suggestions)
    }

    /// Get mapping progress statistics
    pub async fn get_mapping_progress(&self) -> Result<MappingProgress, AppError> {
        let client = self.pool.get().await?;

        let stats_row = client
            .query_one(
                r#"
                WITH non_standard AS (
                    SELECT COUNT(DISTINCT sa.kode_barang) as total
                    FROM integrasi.siman_aset_tanah sa
                    LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
                    WHERE mb.id IS NULL
                ),
                standard AS (
                    SELECT COUNT(*) as total FROM perlengkapan.ms_barang
                ),
                mapped AS (
                    SELECT COUNT(*) as total
                    FROM perlengkapan.mapping_kodefikasi
                    WHERE status_mapping = 'VERIFIED'
                )
                SELECT
                    (SELECT total FROM non_standard) as total_non_standard,
                    (SELECT total FROM standard) as total_standard,
                    (SELECT total FROM mapped) as total_mapped
                "#,
                &[],
            )
            .await?;

        let total_non_standard: i64 = stats_row.get("total_non_standard");
        let total_standard: i64 = stats_row.get("total_standard");
        let total_mapped: i64 = stats_row.get("total_mapped");

        let mapping_percentage = if total_non_standard > 0 {
            (total_mapped as f64 / total_non_standard as f64) * 100.0
        } else {
            100.0
        };

        // Top non-standard codes
        let codes_rows = client
            .query(
                r#"
                SELECT DISTINCT
                    sa.kode_barang as kode_lama,
                    sa.nama_barang as nama_lama,
                    sa.satker_id,
                    COALESCE(s.nama, 'Unknown') as satker_nama,
                    COUNT(*) as jumlah_aset,
                    mk.status_mapping,
                    mb2.kode as kode_baru,
                    mb2.nama as nama_baru
                FROM integrasi.siman_aset_tanah sa
                LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
                LEFT JOIN perlengkapan.mapping_kodefikasi mk ON sa.kode_barang = mk.kode_barang_lama
                    AND sa.satker_id = mk.satker_id
                LEFT JOIN perlengkapan.ms_barang mb2 ON mk.kode_barang_baru_id = mb2.id
                LEFT JOIN authenc.satkers s ON sa.satker_id = s.id
                WHERE mb.id IS NULL
                GROUP BY sa.kode_barang, sa.nama_barang, sa.satker_id, s.nama,
                         mk.status_mapping, mb2.kode, mb2.nama
                ORDER BY jumlah_aset DESC
                LIMIT 100
                "#,
                &[],
            )
            .await?;

        let non_standard_codes = codes_rows
            .into_iter()
            .map(|row| NonStandardCodeWithStatus {
                kode_lama: row.get("kode_lama"),
                nama_lama: row.get("nama_lama"),
                satker_id: row.get("satker_id"),
                satker_nama: row.get("satker_nama"),
                jumlah_aset: row.get("jumlah_aset"),
                status_mapping: row.get("status_mapping"),
                kode_baru: row.get("kode_baru"),
                nama_baru: row.get("nama_baru"),
            })
            .collect();

        Ok(MappingProgress {
            total_non_standard,
            total_mapped,
            total_standard,
            mapping_percentage,
            non_standard_codes,
        })
    }

    /// Get mapping progress by satker
    pub async fn get_mapping_progress_by_satker(
        &self,
    ) -> Result<Vec<MappingProgressBySatker>, AppError> {
        let query = r#"
            SELECT
                s.id as satker_id,
                s.nama as satker_nama,
                COUNT(DISTINCT CASE WHEN mb.id IS NULL THEN sa.kode_barang END) as total_non_standard,
                COUNT(DISTINCT CASE WHEN mk.status_mapping = 'VERIFIED' THEN mk.id END) as total_mapped
            FROM authenc.satkers s
            LEFT JOIN integrasi.siman_aset_tanah sa ON s.id = sa.satker_id
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            LEFT JOIN perlengkapan.mapping_kodefikasi mk ON sa.kode_barang = mk.kode_barang_lama
                AND sa.satker_id = mk.satker_id
            GROUP BY s.id, s.nama
            HAVING COUNT(DISTINCT CASE WHEN mb.id IS NULL THEN sa.kode_barang END) > 0
            ORDER BY total_non_standard DESC
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let progress = rows
            .into_iter()
            .map(|row| {
                let total_non_standard: i64 = row.get("total_non_standard");
                let total_mapped: i64 = row.get("total_mapped");
                let mapping_percentage = if total_non_standard > 0 {
                    (total_mapped as f64 / total_non_standard as f64) * 100.0
                } else {
                    0.0
                };

                MappingProgressBySatker {
                    satker_id: row.get("satker_id"),
                    satker_nama: row.get("satker_nama"),
                    total_non_standard,
                    total_mapped,
                    mapping_percentage,
                }
            })
            .collect();

        Ok(progress)
    }
}
