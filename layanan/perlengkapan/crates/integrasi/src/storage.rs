/// Storage strategy untuk menyimpan data hasil fetch
/// Mendukung multiple output destinations: File (JSON/CSV) dan Database
use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio_postgres::Client as PgClient;
use tracing::{error, info};

/// Sync status storage for tracking sync operations
pub struct SyncStatusStorage {
    db_client: Option<PgClient>,
}

impl SyncStatusStorage {
    pub fn new(db_client: Option<PgClient>) -> Self {
        Self { db_client }
    }

    /// Save sync status to database
    pub async fn save_sync_status(
        &self,
        service: &str,
        sync_type: &str,
        success: bool,
        error_message: Option<String>,
    ) -> Result<(), MonsaktiError> {
        if let Some(db) = &self.db_client {
            let query = r#"
                INSERT INTO sync_status (
                    service_name,
                    sync_type,
                    success,
                    error_message,
                    synced_at
                )
                VALUES ($1, $2, $3, $4, NOW())
            "#;

            db.execute(
                query,
                &[&service, &sync_type, &success, &error_message],
            )
            .await?;

            info!(
                "Sync status saved: {} - {} (success: {})",
                service, sync_type, success
            );
        }

        Ok(())
    }

    /// Get last sync time for a service
    pub async fn get_last_sync_time(
        &self,
        service: &str,
        sync_type: &str,
    ) -> Result<Option<DateTime<Utc>>, MonsaktiError> {
        if let Some(db) = &self.db_client {
            let query = r#"
                SELECT synced_at
                FROM sync_status
                WHERE service_name = $1 AND sync_type = $2 AND success = true
                ORDER BY synced_at DESC
                LIMIT 1
            "#;

            let row = db.query_opt(query, &[&service, &sync_type]).await?;

            if let Some(row) = row {
                let synced_at: DateTime<Utc> = row.get(0);
                return Ok(Some(synced_at));
            }
        }

        Ok(None)
    }

    /// Save MySIMKARI data to database
    pub async fn save_mysimkari_data(
        &self,
        data_type: &str,
        data: &Value,
    ) -> Result<u64, MonsaktiError> {
        if let Some(db) = &self.db_client {
            let _table_name = match data_type {
                "satker" => "mysimkari_satker",
                "pegawai" | "pegawai_mutations" => "mysimkari_pegawai",
                _ => {
                    error!("Unknown MySIMKARI data type: {}", data_type);
                    return Ok(0);
                }
            };

            // Parse data array
            let array = data
                .as_array()
                .ok_or_else(|| MonsaktiError::ApiError("Data is not an array".to_string()))?;

            if array.is_empty() {
                info!("No data to save for {}", data_type);
                return Ok(0);
            }

            let mut count = 0u64;

            for item in array {
                let result = match data_type {
                    "satker" => self.upsert_satker(db, item).await,
                    "pegawai" | "pegawai_mutations" => self.upsert_pegawai(db, item).await,
                    _ => continue,
                };

                match result {
                    Ok(_) => count += 1,
                    Err(e) => error!("Failed to upsert {} record: {:?}", data_type, e),
                }
            }

            info!("Saved {} {} records to database", count, data_type);
            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn upsert_satker(&self, db: &PgClient, data: &Value) -> Result<(), MonsaktiError> {
        let query = r#"
            INSERT INTO mysimkari_satker (
                id,
                kode_satker,
                nama_satker,
                wilayah,
                tipe_satker,
                raw_data,
                synced_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (kode_satker)
            DO UPDATE SET
                nama_satker = EXCLUDED.nama_satker,
                wilayah = EXCLUDED.wilayah,
                tipe_satker = EXCLUDED.tipe_satker,
                raw_data = EXCLUDED.raw_data,
                synced_at = NOW()
        "#;

        let id = data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let kode_satker = data
            .get("kode_satker")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let nama_satker = data
            .get("nama_satker")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let wilayah = data.get("wilayah").and_then(|v| v.as_str());
        let tipe_satker = data.get("tipe_satker").and_then(|v| v.as_str());

        db.execute(
            query,
            &[&id, &kode_satker, &nama_satker, &wilayah, &tipe_satker, &data],
        )
        .await?;

        Ok(())
    }

    async fn upsert_pegawai(&self, db: &PgClient, data: &Value) -> Result<(), MonsaktiError> {
        let query = r#"
            INSERT INTO mysimkari_pegawai (
                nip,
                nama,
                satker_id,
                satker_code,
                jabatan,
                golongan,
                status_pegawai,
                raw_data,
                synced_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            ON CONFLICT (nip)
            DO UPDATE SET
                nama = EXCLUDED.nama,
                satker_id = EXCLUDED.satker_id,
                satker_code = EXCLUDED.satker_code,
                jabatan = EXCLUDED.jabatan,
                golongan = EXCLUDED.golongan,
                status_pegawai = EXCLUDED.status_pegawai,
                raw_data = EXCLUDED.raw_data,
                synced_at = NOW()
        "#;

        let nip = data
            .get("nip")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let nama = data
            .get("nama")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let satker_id = data.get("satker_id").and_then(|v| v.as_str());
        let satker_code = data.get("satker_code").and_then(|v| v.as_str());
        let jabatan = data.get("jabatan").and_then(|v| v.as_str());
        let golongan = data.get("golongan").and_then(|v| v.as_str());
        let status_pegawai = data.get("status_pegawai").and_then(|v| v.as_str());

        db.execute(
            query,
            &[
                &nip,
                &nama,
                &satker_id,
                &satker_code,
                &jabatan,
                &golongan,
                &status_pegawai,
                &data,
            ],
        )
        .await?;

        Ok(())
    }
}

/// Strategy untuk menyimpan data
#[derive(Debug, Clone, Default)]
pub enum StorageStrategy {
    /// Simpan ke file JSON
    JsonFile { base_dir: String },
    /// Simpan ke file CSV
    CsvFile { base_dir: String },
    /// Simpan ke database PostgreSQL
    #[default]
    Database,
    /// Simpan ke file dan database
    Both { base_dir: String },
}

impl StorageStrategy {
    /// Simpan data sesuai strategy yang dipilih
    /// Returns the number of records successfully saved to database
    pub async fn save(
        &self,
        client: &MonsaktiClient,
        module: &str,
        endpoint: &str,
        data: &Value,
        context: &str, // e.g., "satker_123456" or "global"
    ) -> Result<usize, MonsaktiError> {
        let count = match self {
            StorageStrategy::JsonFile { base_dir } => {
                let filename = format!("{}/{}/{}.json", base_dir, context, endpoint);
                client.save_to_json(data, filename).await?;
                // File save doesn't have row count, return data array length
                data.as_array().map(|a| a.len()).unwrap_or(0)
            }
            StorageStrategy::CsvFile { base_dir } => {
                let filename = format!("{}/{}/{}.csv", base_dir, context, endpoint);
                client.save_to_csv(data, filename).await?;
                data.as_array().map(|a| a.len()).unwrap_or(0)
            }
            StorageStrategy::Database => client.save_to_database(module, endpoint, data).await?,
            StorageStrategy::Both { base_dir } => {
                // Save to file first
                let filename = format!("{}/{}/{}.json", base_dir, context, endpoint);
                client.save_to_json(data, filename).await?;
                // Then to database (get actual insert count)
                client.save_to_database(module, endpoint, data).await?
            }
        };
        Ok(count)
    }

    /// Simpan data dengan nama tabel eksplisit (untuk database)
    pub async fn save_with_table(
        &self,
        client: &MonsaktiClient,
        table_name: &str,
        data: &Value,
        context: &str,
    ) -> Result<(), MonsaktiError> {
        match self {
            StorageStrategy::JsonFile { base_dir } => {
                let filename = format!("{}/{}/{}.json", base_dir, context, table_name);
                client.save_to_json(data, filename).await?;
            }
            StorageStrategy::CsvFile { base_dir } => {
                let filename = format!("{}/{}/{}.csv", base_dir, context, table_name);
                client.save_to_csv(data, filename).await?;
            }
            StorageStrategy::Database => {
                client.save_to_postgres(table_name, data).await?;
            }
            StorageStrategy::Both { base_dir } => {
                let filename = format!("{}/{}/{}.json", base_dir, context, table_name);
                client.save_to_json(data, filename).await?;
                client.save_to_postgres(table_name, data).await?;
            }
        }
        Ok(())
    }
}

/// Helper untuk membuat storage strategy dari environment variable
pub fn storage_from_env() -> StorageStrategy {
    let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "database".to_string());
    let output_dir = std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "./data".to_string());

    match storage_type.to_lowercase().as_str() {
        "json" => StorageStrategy::JsonFile {
            base_dir: output_dir,
        },
        "csv" => StorageStrategy::CsvFile {
            base_dir: output_dir,
        },
        "both" => StorageStrategy::Both {
            base_dir: output_dir,
        },
        _ => StorageStrategy::Database,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_from_env_default() {
        let strategy = storage_from_env();
        matches!(strategy, StorageStrategy::Database);
    }
}
