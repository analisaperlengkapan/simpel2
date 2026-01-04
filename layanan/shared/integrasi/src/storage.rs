/// Storage strategy untuk menyimpan data hasil fetch
/// Mendukung multiple output destinations: File (JSON/CSV) dan Database
use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use serde_json::Value;

/// Strategy untuk menyimpan data
#[derive(Debug, Clone)]
#[derive(Default)]
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
