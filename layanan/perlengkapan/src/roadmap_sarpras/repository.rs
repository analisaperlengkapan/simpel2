//! Predictive Analytics repository layer
//!
//! Queries historical kebutuhan_bmn data and manages forecast snapshots.

use crate::shared::db::Database;
use crate::shared::error::AppError;
use lib_perlengkapan::models::{ForecastSnapshot, YearlyData};
use uuid::Uuid;

/// BE-local row mapping for the lib-perlengkapan forecast DTOs.
///
/// Kept out of `lib-perlengkapan` (F0-C) so that crate stays WASM-safe; the
/// orphan rule means a foreign type needs a local trait rather than an inherent
/// `from_row`. Call sites (`YearlyData::from_row`, `ForecastSnapshot::from_row`)
/// resolve through this trait, in scope within this module.
trait FromPgRow {
    fn from_row(row: &tokio_postgres::Row) -> Self;
}

impl FromPgRow for YearlyData {
    fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            tahun: row.get("tahun_anggaran"),
            total_kebutuhan: row.get::<_, i64>("total_kebutuhan"),
            total_existing: row.get::<_, i64>("total_existing"),
            total_gap: row.get::<_, i64>("total_gap"),
            jumlah_satker: row.get::<_, i64>("jumlah_satker"),
            estimasi_total_biaya: row
                .get::<_, Option<f64>>("estimasi_total_biaya")
                .unwrap_or(0.0),
        }
    }
}

impl FromPgRow for ForecastSnapshot {
    fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            method: row.get("method"),
            confidence_level: row.get("confidence_level"),
            satker_id: row.get("satker_id"),
            kode_barang: row.get("kode_barang"),
            predictions_json: row.get("predictions_json"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Clone)]
pub struct RoadmapRepository {
    db: Database,
}

impl RoadmapRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get aggregated historical kebutuhan_bmn data grouped by year.
    ///
    /// Optionally filter by satker_id and/or kode_barang.
    pub async fn get_historical_data(
        &self,
        satker_id: Option<Uuid>,
        kode_barang: Option<&str>,
    ) -> Result<Vec<YearlyData>, AppError> {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut idx = 1usize;

        if let Some(sid) = satker_id {
            conditions.push(format!("satker_id = ${}", idx));
            params.push(Box::new(sid));
            idx += 1;
        }
        if let Some(kb) = kode_barang {
            conditions.push(format!("kode_barang = ${}", idx));
            params.push(Box::new(kb.to_string()));
            // idx += 1; // no more params after this
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            r#"
            SELECT
                tahun_anggaran,
                SUM(jumlah_kebutuhan)::BIGINT     AS total_kebutuhan,
                SUM(jumlah_existing_baik)::BIGINT  AS total_existing,
                SUM(gap)::BIGINT                   AS total_gap,
                COUNT(DISTINCT satker_id)::BIGINT  AS jumlah_satker,
                SUM(COALESCE(estimasi_total, 0))   AS estimasi_total_biaya
            FROM perlengkapan.kebutuhan_bmn
            {}
            GROUP BY tahun_anggaran
            ORDER BY tahun_anggaran ASC
            "#,
            where_clause
        );

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = self.db.query(&query, &param_refs).await?;

        Ok(rows.iter().map(YearlyData::from_row).collect())
    }

    /// Save a forecast snapshot for later comparison.
    pub async fn save_forecast_snapshot(
        &self,
        method: &str,
        confidence_level: f64,
        satker_id: Option<Uuid>,
        kode_barang: Option<&str>,
        predictions_json: &str,
    ) -> Result<ForecastSnapshot, AppError> {
        let id = Uuid::new_v4();

        // Ensure the table exists (idempotent)
        self.ensure_snapshot_table().await?;

        let query = r#"
            INSERT INTO perlengkapan.roadmap_forecast_snapshots
                (id, method, confidence_level, satker_id, kode_barang, predictions_json, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING id, method, confidence_level, satker_id, kode_barang, predictions_json, created_at
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &id,
                    &method,
                    &confidence_level,
                    &satker_id,
                    &kode_barang,
                    &predictions_json,
                ],
            )
            .await?;

        Ok(ForecastSnapshot::from_row(&row))
    }

    /// Get previous forecast snapshots for comparison.
    pub async fn get_previous_snapshots(
        &self,
        satker_id: Option<Uuid>,
        kode_barang: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ForecastSnapshot>, AppError> {
        // Gracefully handle missing table
        self.ensure_snapshot_table().await?;

        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut idx = 1usize;

        if let Some(sid) = satker_id {
            conditions.push(format!("satker_id = ${}", idx));
            params.push(Box::new(sid));
            idx += 1;
        } else {
            conditions.push("satker_id IS NULL".to_string());
        }

        if let Some(kb) = kode_barang {
            conditions.push(format!("kode_barang = ${}", idx));
            params.push(Box::new(kb.to_string()));
            idx += 1;
        } else {
            conditions.push("kode_barang IS NULL".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            r#"
            SELECT id, method, confidence_level, satker_id, kode_barang, predictions_json, created_at
            FROM perlengkapan.roadmap_forecast_snapshots
            {}
            ORDER BY created_at DESC
            LIMIT ${}
            "#,
            where_clause, idx
        );

        params.push(Box::new(limit));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = self.db.query(&query, &param_refs).await?;

        Ok(rows.iter().map(ForecastSnapshot::from_row).collect())
    }

    /// Create the snapshot table if it doesn't exist yet.
    async fn ensure_snapshot_table(&self) -> Result<(), AppError> {
        let ddl = r#"
            CREATE TABLE IF NOT EXISTS perlengkapan.roadmap_forecast_snapshots (
                id UUID PRIMARY KEY,
                method TEXT NOT NULL,
                confidence_level DOUBLE PRECISION NOT NULL DEFAULT 0.95,
                satker_id UUID,
                kode_barang TEXT,
                predictions_json TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;
        let client = self.db.get_connection().await?;
        client
            .execute(ddl, &[])
            .await
            .map_err(|e| AppError::Database(format!("Failed to ensure snapshot table: {}", e)))?;
        Ok(())
    }
}
