// =====================================================
// SIMAN Database Integration Example
// Description: Contoh lengkap integrasi SIMAN API ke PostgreSQL
// =====================================================

use anyhow::{Context, Result};
use layanan_integrasi::{
    client::MonsaktiClient,
    config::Config,
    siman::{SimanAssetCategory, endpoints},
};
use tokio_postgres::Client;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Hasil sinkronisasi per kategori
#[derive(Debug)]
#[allow(dead_code)]
pub struct SyncResult {
    category: SimanAssetCategory,
    table_name: String,
    total_records: i32,
    success_records: i32,
    failed_records: i32,
    sync_id: Uuid,
    duration_secs: f64,
}

/// Main sync coordinator
pub struct SimanDatabaseSync {
    client: MonsaktiClient,
    db_client: Client,
    ba_key: String,
}

impl SimanDatabaseSync {
    /// Inisialisasi sync coordinator
    pub async fn new(config: Config, db_client: Client, ba_key: String) -> Result<Self> {
        let client = MonsaktiClient::new(config)
            .await
            .context("Failed to initialize MonsaktiClient")?;

        Ok(Self {
            client,
            db_client,
            ba_key,
        })
    }

    /// Sinkronisasi satu kategori aset
    pub async fn sync_category(&mut self, category: SimanAssetCategory) -> Result<SyncResult> {
        let table_name = category.table_name().to_string();
        info!(
            "Starting sync for category: {:?} ({})",
            category, table_name
        );

        // Start sync log
        let sync_id = self.start_sync_log(&table_name).await?;
        let start_time = std::time::Instant::now();

        let result = match self.fetch_and_insert_data(category, sync_id).await {
            Ok((total, success, failed)) => {
                let duration = start_time.elapsed().as_secs_f64();

                // Complete sync log
                self.complete_sync_log(sync_id, total, success, failed, None)
                    .await?;

                info!(
                    "Sync completed for {}: {} success, {} failed in {:.2}s",
                    table_name, success, failed, duration
                );

                SyncResult {
                    category,
                    table_name,
                    total_records: total,
                    success_records: success,
                    failed_records: failed,
                    sync_id,
                    duration_secs: duration,
                }
            }
            Err(e) => {
                let duration = start_time.elapsed().as_secs_f64();
                error!("Sync failed for {}: {}", table_name, e);

                // Mark sync as failed
                self.fail_sync_log(sync_id, &e.to_string()).await?;

                SyncResult {
                    category,
                    table_name,
                    total_records: 0,
                    success_records: 0,
                    failed_records: 1,
                    sync_id,
                    duration_secs: duration,
                }
            }
        };

        Ok(result)
    }

    /// Sinkronisasi semua kategori aset
    pub async fn sync_all_categories(&mut self) -> Result<Vec<SyncResult>> {
        let categories = vec![
            SimanAssetCategory::AlatBesar,
            SimanAssetCategory::AlatPersenjataan,
            SimanAssetCategory::AngkutanBermotor,
            SimanAssetCategory::TakBerwujud,
            SimanAssetCategory::BangunanAir,
            SimanAssetCategory::GedungBangunan,
            SimanAssetCategory::InstalasiJaringan,
            SimanAssetCategory::JalandanJembatan,
            SimanAssetCategory::KhususTIK,
            SimanAssetCategory::NonTIK,
            SimanAssetCategory::Rumah,
            SimanAssetCategory::Tanah,
            SimanAssetCategory::TetapLainnya,
            SimanAssetCategory::KDP,
            SimanAssetCategory::TetapRenovasi,
        ];

        let mut results = Vec::new();

        for category in categories {
            match self.sync_category(category).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to sync category {:?}: {}", category, e);
                    // Continue with next category
                }
            }
        }

        Ok(results)
    }

    /// Fetch data from API dan insert ke database
    async fn fetch_and_insert_data(
        &mut self,
        category: SimanAssetCategory,
        sync_id: Uuid,
    ) -> Result<(i32, i32, i32)> {
        // Get row count first
        let response = self.client.fetch_siman_row_count(category).await?;
        let row_count = response
            .data
            .and_then(|data| data.as_array().cloned())
            .and_then(|arr| arr.first().cloned())
            .and_then(|v| v.get("JUMLAH").cloned())
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i32;

        info!("Total rows for {:?}: {}", category, row_count);

        if row_count == 0 {
            return Ok((0, 0, 0));
        }

        // Fetch data dengan pagination otomatis
        let data = endpoints::fetch_all_aset_paginated(&mut self.client, category, 500)
            .await
            .context("Failed to fetch data from API")?;

        info!("Fetched {} records from API", data.len());

        // Insert data ke database
        let mut success_count = 0;
        let mut failed_count = 0;

        // Process in batches
        for chunk in data.chunks(100) {
            for record in chunk {
                match self.insert_record(category, record, sync_id).await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        warn!("Failed to insert record: {}", e);
                        failed_count += 1;
                    }
                }
            }
        }

        Ok((data.len() as i32, success_count, failed_count))
    }

    /// Insert single record ke database
    async fn insert_record(
        &self,
        category: SimanAssetCategory,
        record: &serde_json::Value,
        sync_id: Uuid,
    ) -> Result<()> {
        let table_name = format!("siman.{}", category.table_name());

        // Generic insert untuk semua tabel (simplified)
        // Dalam implementasi real, gunakan prepared statement spesifik per tabel
        let query = format!(
            r#"
            INSERT INTO {} (
                kd_jns_bmn, kd_brg, no_aset, tercatat,
                rph_aset, rph_susut, rph_mutasi,
                status_bmn_yn, bpybds_yn, flag_sap,
                jml_photo, kondisi,
                tgl_perolehan, tgl_pembukuan,
                nama_satker, kd_satker, kd_kantor,
                raw_data, sync_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
            ON CONFLICT (no_aset, kd_satker) DO UPDATE SET
                rph_aset = EXCLUDED.rph_aset,
                rph_susut = EXCLUDED.rph_susut,
                kondisi = EXCLUDED.kondisi,
                raw_data = EXCLUDED.raw_data,
                sync_id = EXCLUDED.sync_id,
                updated_at = CURRENT_TIMESTAMP
            "#,
            table_name
        );

        self.db_client
            .execute(
                &query,
                &[
                    &(record
                        .get("kd_jns_bmn")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32),
                    &record.get("kd_brg").and_then(|v| v.as_str()).unwrap_or(""),
                    &record.get("no_aset").and_then(|v| v.as_i64()).unwrap_or(0),
                    &record
                        .get("tercatat")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    &record.get("rph_aset").and_then(|v| v.as_i64()).unwrap_or(0),
                    &record
                        .get("rph_susut")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    &record
                        .get("rph_mutasi")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    &record.get("status_bmn_yn").and_then(|v| v.as_str()),
                    &record.get("bpybds_yn").and_then(|v| v.as_str()),
                    &record.get("flag_sap").and_then(|v| v.as_str()),
                    &(record
                        .get("jml_photo")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32),
                    &record.get("kondisi").and_then(|v| v.as_str()),
                    &record.get("tgl_perolehan").and_then(|v| v.as_str()),
                    &record.get("tgl_pembukuan").and_then(|v| v.as_str()),
                    &record.get("nama_satker").and_then(|v| v.as_str()),
                    &self.ba_key,
                    &record.get("kd_kantor").and_then(|v| v.as_str()),
                    &record,
                    &sync_id,
                ],
            )
            .await
            .context("Failed to insert record")?;

        Ok(())
    }

    /// Start sync log
    async fn start_sync_log(&self, table_name: &str) -> Result<Uuid> {
        let row = self
            .db_client
            .query_one(
                "SELECT siman.start_sync($1, $2)",
                &[&table_name, &self.ba_key],
            )
            .await
            .context("Failed to start sync log")?;

        let sync_id: Uuid = row.get(0);
        Ok(sync_id)
    }

    /// Complete sync log
    async fn complete_sync_log(
        &self,
        sync_id: Uuid,
        total: i32,
        success: i32,
        failed: i32,
        error_msg: Option<&str>,
    ) -> Result<()> {
        self.db_client
            .execute(
                "SELECT siman.complete_sync($1, $2, $3, $4, $5)",
                &[&sync_id, &total, &success, &failed, &error_msg],
            )
            .await
            .context("Failed to complete sync log")?;

        Ok(())
    }

    /// Mark sync as failed
    async fn fail_sync_log(&self, sync_id: Uuid, error_msg: &str) -> Result<()> {
        self.db_client
            .execute("SELECT siman.fail_sync($1, $2)", &[&sync_id, &error_msg])
            .await
            .context("Failed to mark sync as failed")?;

        Ok(())
    }

    /// Get summary statistics
    pub async fn get_summary_stats(&self) -> Result<Vec<SummaryStats>> {
        let rows = self
            .db_client
            .query(
                r#"
                SELECT kategori, jumlah_aset, total_nilai, last_updated
                FROM siman.v_siman_summary_per_satker
                WHERE kd_satker = $1
                ORDER BY total_nilai DESC
                "#,
                &[&self.ba_key],
            )
            .await?;

        let stats = rows
            .iter()
            .map(|row| SummaryStats {
                kategori: row.get("kategori"),
                jumlah_aset: row.get("jumlah_aset"),
                total_nilai: row.get("total_nilai"),
                last_updated: row.get("last_updated"),
            })
            .collect();

        Ok(stats)
    }
}

#[derive(Debug)]
pub struct SummaryStats {
    pub kategori: String,
    pub jumlah_aset: i64,
    pub total_nilai: i64,
    pub last_updated: chrono::NaiveDateTime,
}

// =====================================================
// Example Usage
// =====================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = Config::from_env()?;

    // Connect to database using tokio-postgres
    let database_url = std::env::var("DATABASE_URL")?;
    let (db_client, connection) = tokio_postgres::connect(&database_url, tokio_postgres::NoTls)
        .await
        .context("Failed to connect to database")?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    // Create sync coordinator
    let ba_key = std::env::var("SIMAN_BA_KEY")?;
    let mut sync = SimanDatabaseSync::new(config, db_client, ba_key).await?;

    // Example 1: Sync single category
    info!("=== Syncing Angkutan Bermotor ===");
    let result = sync
        .sync_category(SimanAssetCategory::AngkutanBermotor)
        .await?;
    info!("Result: {:?}", result);

    // Example 2: Sync all categories
    info!("=== Syncing All Categories ===");
    let results = sync.sync_all_categories().await?;
    for result in &results {
        info!(
            "{}: {} success / {} failed in {:.2}s",
            result.table_name, result.success_records, result.failed_records, result.duration_secs
        );
    }

    // Example 3: Get summary statistics
    info!("=== Summary Statistics ===");
    let stats = sync.get_summary_stats().await?;
    for stat in stats {
        info!(
            "{}: {} aset, total Rp {}",
            stat.kategori, stat.jumlah_aset, stat.total_nilai
        );
    }

    // Summary report
    let total_success: i32 = results.iter().map(|r| r.success_records).sum();
    let total_failed: i32 = results.iter().map(|r| r.failed_records).sum();
    let total_duration: f64 = results.iter().map(|r| r.duration_secs).sum();

    info!("=== Final Summary ===");
    info!("Total Success: {}", total_success);
    info!("Total Failed: {}", total_failed);
    info!("Total Duration: {:.2}s", total_duration);

    Ok(())
}
