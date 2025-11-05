use crate::error::MonsaktiError;
use serde_json::Value;
use tokio_postgres::Client;
use tracing::{info, warn};

/// Helper untuk konversi JSON value ke PostgreSQL parameter
pub fn json_to_sql_param(value: &Value) -> Box<dyn tokio_postgres::types::ToSql + Sync + Send> {
    match value {
        Value::String(s) => Box::new(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else if let Some(f) = n.as_f64() {
                Box::new(f)
            } else {
                Box::new(n.to_string())
            }
        }
        Value::Bool(b) => Box::new(*b),
        Value::Null => Box::new(Option::<String>::None),
        _ => Box::new(value.to_string()),
    }
}

/// Bulk insert data ke PostgreSQL dengan batch processing
pub async fn bulk_insert_postgres(
    db: &Client,
    table_name: &str,
    data: &[Value],
) -> Result<usize, MonsaktiError> {
    if data.is_empty() {
        return Ok(0);
    }

    // Validasi nama tabel untuk mencegah SQL injection
    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(MonsaktiError::ConfigError(format!(
            "Nama tabel tidak valid: {}",
            table_name
        )));
    }

    let first_obj = data[0]
        .as_object()
        .ok_or_else(|| MonsaktiError::ApiError("Format data tidak valid".to_string()))?;

    // Konversi key ke lowercase untuk konsistensi dengan PostgreSQL
    let columns: Vec<String> = first_obj.keys().map(|k| k.to_lowercase()).collect();

    if columns.is_empty() {
        return Err(MonsaktiError::ApiError(
            "Tidak ada kolom untuk diinsert".to_string(),
        ));
    }

    let mut count = 0;
    let mut failed = 0;

    // Batch insert untuk performa lebih baik (100 rows per batch)
    for chunk in data.chunks(100) {
        for item in chunk {
            if let Some(obj) = item.as_object() {
                // Build query dengan placeholders
                let placeholders: Vec<String> =
                    (1..=columns.len()).map(|i| format!("${}", i)).collect();
                let query = format!(
                    "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT DO NOTHING",
                    table_name,
                    columns.join(", "),
                    placeholders.join(", ")
                );

                // Konversi nilai JSON ke parameter PostgreSQL
                let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
                    Vec::new();
                for col in &columns {
                    let value = obj
                        .get(col)
                        .or_else(|| obj.get(&col.to_uppercase()))
                        .unwrap_or(&Value::Null);
                    params.push(json_to_sql_param(value));
                }

                // Convert to references for execute
                let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
                    .iter()
                    .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
                    .collect();

                match db.execute(&query, &param_refs[..]).await {
                    Ok(rows) => {
                        if rows > 0 {
                            count += rows as usize;
                        }
                    }
                    Err(e) => {
                        warn!("Gagal insert row ke {}: {}", table_name, e);
                        failed += 1;
                    }
                }
            }
        }
    }

    if failed > 0 {
        warn!("{} baris gagal diinsert ke tabel {}", failed, table_name);
    }

    Ok(count)
}

/// Simpan data ke database dengan mapping tabel otomatis
pub async fn save_to_database(
    db: &Client,
    module: &str,
    endpoint: &str,
    data: &Value,
) -> Result<usize, MonsaktiError> {
    // Mapping endpoint ke nama tabel
    let table_name = format!("{}_{}", module.to_lowercase(), endpoint.to_lowercase());

    if let Some(array) = data.as_array() {
        if array.is_empty() {
            warn!("Tidak ada data untuk diinsert ke tabel {}", table_name);
            return Ok(0);
        }

        let insert_count = bulk_insert_postgres(db, &table_name, array).await?;
        info!(
            "{} baris berhasil diinsert ke tabel {}",
            insert_count, table_name
        );
        Ok(insert_count)
    } else {
        warn!(
            "Data bukan array, tidak dapat diinsert ke tabel {}",
            table_name
        );
        Ok(0)
    }
}
