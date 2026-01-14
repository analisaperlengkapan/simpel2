use crate::error::MonsaktiError;
use serde_json::Value;
use std::fmt::Write;
use tokio_postgres::Client;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Helper untuk konversi JSON value ke PostgreSQL parameter - SIMPLIFIED for TEXT columns
pub fn json_to_sql_param(
    value: &Value,
    column_name: &str,
) -> Box<dyn tokio_postgres::types::ToSql + Sync + Send> {
    match value {
        Value::String(s) => {
            // Special handling for api_id column - convert to i64
            if column_name == "api_id" {
                if let Ok(num) = s.parse::<i64>() {
                    return Box::new(Some(num));
                }
                // If parse fails, return NULL - SKIP THIS COLUMN!
                return Box::new(None::<i64>);
            }

            // Try to parse as UUID for id/parent_id/satker_id columns only
            if (column_name == "id" || column_name.ends_with("_id"))
                && let Ok(uuid) = Uuid::parse_str(s) {
                    return Box::new(uuid);
                }

            // Everything else is TEXT - keep as string
            Box::new(s.clone())
        }
        Value::Number(n) => {
            // Special handling for api_id - store as i64
            if column_name == "api_id" {
                if let Some(num) = n.as_i64() {
                    return Box::new(Some(num));
                }
                // Return NULL for api_id if not i64
                return Box::new(None::<i64>);
            }

            // For all other numbers, convert to string
            // PostgreSQL TEXT columns accept strings, and NUMERIC can cast from string
            // This avoids complex type matching logic
            Box::new(n.to_string())
        }
        Value::Bool(b) => {
            // Convert bool to string for TEXT columns
            Box::new(if *b {
                "true".to_string()
            } else {
                "false".to_string()
            })
        }
        Value::Null => {
            // For api_id column, return NULL as Option<i64>
            if column_name == "api_id" {
                return Box::new(None::<i64>);
            }
            // For other columns, return NULL as Option<String>
            Box::new(None::<String>)
        }
        Value::Array(arr) => {
            // For JSONB columns, pass the value as serde_json::Value directly
            if column_name == "raw_data"
                || column_name.ends_with("_json")
                || column_name.ends_with("_data")
            {
                return Box::new(value.clone());
            }
            // Otherwise convert to JSON string for TEXT columns
            Box::new(serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string()))
        }
        Value::Object(_) => {
            // For JSONB columns, pass the value as serde_json::Value directly
            if column_name == "raw_data"
                || column_name.ends_with("_json")
                || column_name.ends_with("_data")
            {
                return Box::new(value.clone());
            }
            // Otherwise convert to JSON string for TEXT columns
            Box::new(serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()))
        }
    }
}

/// Bulk insert data ke PostgreSQL dengan batch processing
pub async fn bulk_insert_postgres(
    db: &Client,
    table_name: &str,
    data: &[Value],
) -> Result<usize, MonsaktiError> {
    info!(
        "🔧 [BULK INSERT] Starting bulk_insert_postgres for table: {}, records: {}",
        table_name,
        data.len()
    );

    if data.is_empty() {
        info!("⚠️  [BULK INSERT] Data is empty, returning 0");
        return Ok(0);
    }

    // Validasi nama tabel untuk mencegah SQL injection
    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        error!("❌ [BULK INSERT] Invalid table name: {}", table_name);
        return Err(MonsaktiError::ConfigError(format!(
            "Nama tabel tidak valid: {}",
            table_name
        )));
    }

    let first_obj = data[0].as_object().ok_or_else(|| {
        error!("❌ [BULK INSERT] First data item is not an object");
        MonsaktiError::ApiError("Format data tidak valid".to_string())
    })?;

    info!(
        "📋 [BULK INSERT] First object keys: {:?}",
        first_obj.keys().collect::<Vec<_>>()
    );

    // Konversi key ke lowercase untuk konsistensi dengan PostgreSQL
    // Check if ANY object in the data has a non-null 'id' field
    let has_valid_id = data.iter().any(|item| {
        if let Some(obj) = item.as_object()
            && let Some(id_val) = obj.get("id").or_else(|| obj.get("ID")) {
                return !id_val.is_null();
            }
        false
    });

    // Determine if this is a SIMAN table (has different schema)
    let is_siman_table = table_name.starts_with("siman_");

    // Rename 'id' ke 'api_id' jika ada (untuk menyimpan ID dari API eksternal)
    // KECUALI untuk tabel SIMAN yang tidak punya kolom api_id
    let columns: Vec<String> = first_obj
        .keys()
        .filter_map(|k| {
            let lower = k.to_lowercase();

            // Skip internal SIMAN API fields that aren't in database schema
            if is_siman_table && (lower == "id" || lower == "tgl_tarik") {
                return None;
            }

            if lower == "id" {
                // Only include api_id if at least one object has non-null id
                if has_valid_id {
                    Some("api_id".to_string()) // Rename id -> api_id
                } else {
                    None // Skip if all ids are null
                }
            } else {
                Some(lower)
            }
        })
        .collect();

    if columns.is_empty() {
        error!("❌ [BULK INSERT] No columns to insert");
        return Err(MonsaktiError::ApiError(
            "Tidak ada kolom untuk diinsert".to_string(),
        ));
    }

    info!("📝 [BULK INSERT] Columns to insert: {:?}", columns);

    let mut count = 0;
    let mut failed = 0;

    // Batch insert untuk performa lebih baik (100 rows per batch)
    info!(
        "🔄 [BULK INSERT] Processing {} records in chunks of 100...",
        data.len()
    );
    for (chunk_idx, chunk) in data.chunks(100).enumerate() {
        info!(
            "📦 [BULK INSERT] Processing chunk {}, {} records",
            chunk_idx + 1,
            chunk.len()
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            Vec::with_capacity(chunk.len() * columns.len());
        let mut param_index = 1;

        // Optimize query construction using a single String buffer
        // Estimate capacity: "INSERT INTO table (cols) VALUES " + (chunk_len * (cols * 4 chars + 2))
        let mut query = String::with_capacity(256 + chunk.len() * columns.len() * 5);

        write!(
            query,
            "INSERT INTO {} ({}) VALUES ",
            table_name,
            columns.join(", ")
        )
        .unwrap();

        let mut has_valid_rows = false;
        let mut row_count = 0;

        for item in chunk {
            if let Some(obj) = item.as_object() {
                if row_count > 0 {
                    query.push_str(", ");
                }
                query.push('(');

                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    // Use write! to append placeholder directly without String allocation
                    write!(query, "${}", param_index).unwrap();
                    param_index += 1;

                    // Handle mapping: api_id in DB comes from id in JSON
                    let json_key = if col == "api_id" { "id" } else { col.as_str() };

                    let value = obj
                        .get(json_key)
                        .or_else(|| obj.get(&json_key.to_uppercase()))
                        .unwrap_or(&Value::Null);
                    params.push(json_to_sql_param(value, col));
                }
                query.push(')');

                has_valid_rows = true;
                row_count += 1;
            }
        }

        if !has_valid_rows {
            continue;
        }

        query.push_str(" ON CONFLICT DO NOTHING");

        // Convert to references for execute
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        match db.execute(&query, &param_refs[..]).await {
            Ok(rows) => {
                if rows > 0 {
                    count += rows as usize;
                    debug!(
                        "✅ [BULK INSERT] Batch {} inserted {} row(s)",
                        chunk_idx + 1,
                        rows
                    );
                } else {
                    debug!(
                        "⚠️  [BULK INSERT] Batch {} inserted 0 rows (possibly duplicates)",
                        chunk_idx + 1
                    );
                }
            }
            Err(e) => {
                // Log more details for debugging
                error!(
                    "❌ [BULK INSERT] Batch insert error for table {}: {:?} | Columns: {:?}",
                    table_name, e, columns
                );
                // Only log query if debug enabled to avoid log spam
                debug!("❌ [BULK INSERT] Failed query: {}", query);

                failed += chunk.len();
            }
        }
    }

    if failed > 0 {
        warn!(
            "⚠️  [BULK INSERT] {} baris gagal diinsert ke tabel {}",
            failed, table_name
        );
    }

    info!(
        "✅ [BULK INSERT] Completed: {} rows inserted, {} failed for table {}",
        count, failed, table_name
    );

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

    info!(
        "💾 [DB] save_to_database called: module={}, endpoint={}, table={}",
        module, endpoint, table_name
    );

    if let Some(array) = data.as_array() {
        info!("📦 [DB] Data is array with {} elements", array.len());

        if array.is_empty() {
            warn!(
                "⚠️  [DB] Tidak ada data untuk diinsert ke tabel {}",
                table_name
            );
            return Ok(0);
        }

        info!(
            "🔄 [DB] Calling bulk_insert_postgres for {} records...",
            array.len()
        );
        let insert_count = bulk_insert_postgres(db, &table_name, array).await?;
        info!(
            "✅ [DB] {} baris berhasil diinsert ke tabel {}",
            insert_count, table_name
        );
        Ok(insert_count)
    } else {
        warn!(
            "❌ [DB] Data bukan array, tidak dapat diinsert ke tabel {}",
            table_name
        );
        Ok(0)
    }
}
