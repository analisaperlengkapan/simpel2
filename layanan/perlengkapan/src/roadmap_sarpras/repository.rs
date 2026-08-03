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

        // Reads `perlengkapan.roadmap_sarpras` — the roadmap table this module
        // is named after and already exports from. It previously read
        // `perlengkapan.kebutuhan_bmn`, which exists in no environment, so every
        // /forecast endpoint 500'd for as long as they have been mounted.
        //
        // The mistake was only ever the table name: roadmap_sarpras carries the
        // exact quantities this aggregate wants, including a real
        // `estimasi_anggaran`, so no part of the forecast has to be invented.
        // `gap` is computed rather than stored, floored at zero so an
        // over-fulfilled row cannot subtract from another year's shortfall.
        //
        // Types are deliberate: `SUM(integer)` yields BIGINT and
        // `SUM(double precision)` yields DOUBLE PRECISION, so the i64/f64 reads
        // in `from_row` decode. A NUMERIC here would panic instead (#116).
        let query = format!(
            r#"
            SELECT
                tahun_rencana                            AS tahun_anggaran,
                SUM(jumlah_kebutuhan)::BIGINT            AS total_kebutuhan,
                SUM(COALESCE(jumlah_terpenuhi, 0))::BIGINT AS total_existing,
                SUM(GREATEST(jumlah_kebutuhan - COALESCE(jumlah_terpenuhi, 0), 0))::BIGINT
                                                         AS total_gap,
                COUNT(DISTINCT satker_id)::BIGINT        AS jumlah_satker,
                SUM(COALESCE(estimasi_anggaran, 0))      AS estimasi_total_biaya
            FROM perlengkapan.roadmap_sarpras
            {}
            GROUP BY tahun_rencana
            ORDER BY tahun_rencana ASC
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

    // NOTE: there is deliberately no `ensure_snapshot_table` here. It ran a
    // `CREATE TABLE IF NOT EXISTS` before every snapshot read and write, which
    // made the schema a side effect of serving a request: invisible to anyone
    // reading `migrations/`, unreviewable, and a DDL round-trip per call. The
    // table is created by `V008__roadmap_forecast_snapshots.sql` instead, the
    // same move V007 made for the boot-time indexes.
}
