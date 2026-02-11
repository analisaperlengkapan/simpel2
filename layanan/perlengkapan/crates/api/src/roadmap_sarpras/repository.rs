//! Roadmap Sarpras repository layer

use crate::database::Database;
use crate::errors::AppError;
use chrono::Utc;
use lib_perlengkapan::models::{RoadmapRealizationComparison, RoadmapSarpras};
use uuid::Uuid;

use super::models::{
    CreateRoadmapBatchRequest, ListRoadmapQuery, RoadmapComparisonQuery, RoadmapSummary,
    SyncRealizationRequest, UpdateRoadmapRealizationRequest,
};

#[derive(Clone)]
pub struct RoadmapRepository {
    db: Database,
}

impl RoadmapRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a single roadmap item
    pub async fn create(
        &self,
        satker_id: Uuid,
        periode_mulai: i32,
        periode_akhir: i32,
        kode_barang: &str,
        nama_barang: &str,
        tahun_rencana: i32,
        jumlah_kebutuhan: i32,
        estimasi_anggaran: Option<f64>,
        keterangan: Option<&str>,
        created_by: Uuid,
    ) -> Result<RoadmapSarpras, AppError> {
        let roadmap_id = Uuid::new_v4();

        let query = r#"
            INSERT INTO perlengkapan.roadmap_sarpras (
                id, satker_id, periode_mulai, periode_akhir, kode_barang, nama_barang,
                tahun_rencana, jumlah_kebutuhan, jumlah_terpenuhi, estimasi_anggaran,
                realisasi_anggaran, status_pemenuhan, keterangan, created_at, updated_at,
                created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 0, $9, NULL, 'PLANNED', $10, NOW(), NOW(), $11, $11
            )
            RETURNING *
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &roadmap_id,
                    &satker_id,
                    &periode_mulai,
                    &periode_akhir,
                    &kode_barang,
                    &nama_barang,
                    &tahun_rencana,
                    &jumlah_kebutuhan,
                    &estimasi_anggaran,
                    &keterangan,
                    &created_by,
                ],
            )
            .await?;

        Ok(RoadmapSarpras::from_row(&row))
    }

    /// Create multiple roadmap items in a batch
    pub async fn create_batch(
        &self,
        request: &CreateRoadmapBatchRequest,
        created_by: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut roadmap_ids = Vec::new();

        // Use transaction for batch insert
        let mut client = self.db.get_connection().await?;
        let tx = client.transaction().await?;

        let query = r#"
            INSERT INTO perlengkapan.roadmap_sarpras (
                id, satker_id, periode_mulai, periode_akhir, kode_barang, nama_barang,
                tahun_rencana, jumlah_kebutuhan, jumlah_terpenuhi, estimasi_anggaran,
                realisasi_anggaran, status_pemenuhan, keterangan, created_at, updated_at,
                created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 0, $9, NULL, 'PLANNED', $10, NOW(), NOW(), $11, $11
            )
        "#;

        for item in &request.items {
            let roadmap_id = Uuid::new_v4();

            tx.execute(
                query,
                &[
                    &roadmap_id,
                    &request.satker_id,
                    &request.periode_mulai,
                    &request.periode_akhir,
                    &item.kode_barang,
                    &item.nama_barang,
                    &item.tahun_rencana,
                    &item.jumlah_kebutuhan,
                    &item.estimasi_anggaran,
                    &item.keterangan,
                    &created_by,
                ],
            )
            .await?;

            roadmap_ids.push(roadmap_id);
        }

        tx.commit().await?;

        Ok(roadmap_ids)
    }

    /// Get roadmap by ID
    pub async fn get_by_id(&self, roadmap_id: Uuid) -> Result<Option<RoadmapSarpras>, AppError> {
        let query = r#"
            SELECT * FROM perlengkapan.roadmap_sarpras
            WHERE id = $1
        "#;

        match self.db.query_opt(query, &[&roadmap_id]).await? {
            Some(row) => Ok(Some(RoadmapSarpras::from_row(&row))),
            None => Ok(None),
        }
    }

    /// List roadmaps with filters
    pub async fn list(
        &self,
        query: &ListRoadmapQuery,
    ) -> Result<(Vec<RoadmapSarpras>, i64), AppError> {
        let mut conditions = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut param_count = 1;

        if let Some(satker_id) = &query.satker_id {
            conditions.push(format!("satker_id = ${}", param_count));
            params.push(satker_id);
            param_count += 1;
        }

        if let Some(periode_mulai) = &query.periode_mulai {
            conditions.push(format!("periode_mulai = ${}", param_count));
            params.push(periode_mulai);
            param_count += 1;
        }

        if let Some(periode_akhir) = &query.periode_akhir {
            conditions.push(format!("periode_akhir = ${}", param_count));
            params.push(periode_akhir);
            param_count += 1;
        }

        if let Some(tahun_rencana) = &query.tahun_rencana {
            conditions.push(format!("tahun_rencana = ${}", param_count));
            params.push(tahun_rencana);
            param_count += 1;
        }

        if let Some(kode_barang) = &query.kode_barang {
            conditions.push(format!("kode_barang = ${}", param_count));
            params.push(kode_barang);
            param_count += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.roadmap_sarpras {}",
            where_clause
        );

        let total: i64 = self
            .db
            .query_one(&count_query, &params)
            .await?
            .get(0);

        // Get paginated results
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let data_query = format!(
            "SELECT * FROM perlengkapan.roadmap_sarpras {} ORDER BY tahun_rencana, created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_count, param_count + 1
        );

        params.push(&limit);
        params.push(&offset);

        let rows = self.db.query(&data_query, &params).await?;

        let roadmaps = rows
            .iter()
            .map(|row| RoadmapSarpras::from_row(row))
            .collect();

        Ok((roadmaps, total))
    }

    /// Update roadmap realization
    pub async fn update_realization(
        &self,
        roadmap_id: Uuid,
        request: &UpdateRoadmapRealizationRequest,
        updated_by: Uuid,
    ) -> Result<RoadmapSarpras, AppError> {
        let query = r#"
            UPDATE perlengkapan.roadmap_sarpras
            SET jumlah_terpenuhi = $1,
                realisasi_anggaran = $2,
                status_pemenuhan = $3,
                updated_at = NOW(),
                updated_by = $4
            WHERE id = $5
            RETURNING *
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &request.jumlah_terpenuhi,
                    &request.realisasi_anggaran,
                    &request.status_pemenuhan,
                    &updated_by,
                    &roadmap_id,
                ],
            )
            .await?;

        Ok(RoadmapSarpras::from_row(&row))
    }

    /// Sync realization increment (from MonSAKTI)
    pub async fn sync_realization_increment(
        &self,
        request: &SyncRealizationRequest,
    ) -> Result<RoadmapSarpras, AppError> {
        let query = r#"
            UPDATE perlengkapan.roadmap_sarpras
            SET jumlah_terpenuhi = jumlah_terpenuhi + $1,
                realisasi_anggaran = COALESCE(realisasi_anggaran, 0) + COALESCE($2, 0),
                status_pemenuhan = CASE
                    WHEN (jumlah_terpenuhi + $1) >= jumlah_kebutuhan THEN 'COMPLETED'
                    WHEN (jumlah_terpenuhi + $1) > 0 THEN 'IN_PROGRESS'
                    ELSE 'PLANNED'
                END,
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &request.jumlah_terpenuhi_increment,
                    &request.realisasi_anggaran_increment,
                    &request.roadmap_id,
                ],
            )
            .await?;

        Ok(RoadmapSarpras::from_row(&row))
    }

    /// Get roadmap vs realization comparison
    pub async fn get_comparison(
        &self,
        query: &RoadmapComparisonQuery,
    ) -> Result<Vec<RoadmapRealizationComparison>, AppError> {
        let sql = r#"
            SELECT
                id,
                satker_id,
                kode_barang,
                nama_barang,
                tahun_rencana,
                jumlah_kebutuhan,
                jumlah_terpenuhi,
                estimasi_anggaran,
                realisasi_anggaran,
                status_pemenuhan
            FROM perlengkapan.roadmap_sarpras
            WHERE satker_id = $1
              AND periode_mulai = $2
              AND periode_akhir = $3
            ORDER BY tahun_rencana, kode_barang
        "#;

        let rows = self
            .db
            .query(
                sql,
                &[&query.satker_id, &query.periode_mulai, &query.periode_akhir],
            )
            .await?;

        let comparisons = rows
            .iter()
            .map(|row| RoadmapRealizationComparison::from_row(row))
            .collect();

        Ok(comparisons)
    }

    /// Get roadmap summary statistics
    pub async fn get_summary(
        &self,
        satker_id: Uuid,
        periode_mulai: i32,
        periode_akhir: i32,
    ) -> Result<RoadmapSummary, AppError> {
        let query = r#"
            SELECT
                COUNT(*) as total_items,
                SUM(jumlah_kebutuhan) as total_kebutuhan,
                SUM(jumlah_terpenuhi) as total_terpenuhi,
                AVG(CASE
                    WHEN jumlah_kebutuhan > 0
                    THEN (jumlah_terpenuhi::DECIMAL / jumlah_kebutuhan * 100)
                    ELSE 0
                END) as persentase_pemenuhan_rata_rata,
                SUM(COALESCE(estimasi_anggaran, 0)) as total_estimasi_anggaran,
                SUM(COALESCE(realisasi_anggaran, 0)) as total_realisasi_anggaran
            FROM perlengkapan.roadmap_sarpras
            WHERE satker_id = $1
              AND periode_mulai = $2
              AND periode_akhir = $3
        "#;

        let row = self
            .db
            .query_one(query, &[&satker_id, &periode_mulai, &periode_akhir])
            .await?;

        Ok(RoadmapSummary {
            total_items: row.get("total_items"),
            total_kebutuhan: row.get::<_, Option<i64>>("total_kebutuhan").unwrap_or(0),
            total_terpenuhi: row.get::<_, Option<i64>>("total_terpenuhi").unwrap_or(0),
            persentase_pemenuhan_rata_rata: row
                .get::<_, Option<f64>>("persentase_pemenuhan_rata_rata")
                .unwrap_or(0.0),
            total_estimasi_anggaran: row
                .get::<_, Option<f64>>("total_estimasi_anggaran")
                .unwrap_or(0.0),
            total_realisasi_anggaran: row
                .get::<_, Option<f64>>("total_realisasi_anggaran")
                .unwrap_or(0.0),
        })
    }

    /// Delete roadmap by ID
    pub async fn delete(&self, roadmap_id: Uuid) -> Result<bool, AppError> {
        let query = r#"
            DELETE FROM perlengkapan.roadmap_sarpras
            WHERE id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&roadmap_id]).await?;

        Ok(rows_affected > 0)
    }
}
