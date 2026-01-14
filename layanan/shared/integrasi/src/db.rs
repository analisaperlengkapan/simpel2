use crate::error::MonsaktiError;
use serde_json::Value;
use tokio_postgres::Client;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use tokio_postgres::types::{IsNull, Type, ToSql};
use bytes::BytesMut;
use std::error::Error;

/// Optimized SQL parameter wrapper to avoid allocations
#[derive(Debug, Clone)]
pub enum SqlParam<'a> {
    RefString(&'a str),
    OwnedString(String),
    I64(i64),
    Uuid(Uuid),
    NullI64,
    NullString,
    JsonValue(&'a Value),
    OwnedJsonString(String),
}

impl<'a> ToSql for SqlParam<'a> {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self {
            SqlParam::RefString(s) => s.to_sql(ty, out),
            SqlParam::OwnedString(s) => s.to_sql(ty, out),
            SqlParam::I64(n) => n.to_sql(ty, out),
            SqlParam::Uuid(u) => u.to_sql(ty, out),
            SqlParam::NullI64 => <Option<i64> as ToSql>::to_sql(&None, ty, out),
            SqlParam::NullString => <Option<String> as ToSql>::to_sql(&None, ty, out),
            SqlParam::JsonValue(v) => v.to_sql(ty, out),
            SqlParam::OwnedJsonString(s) => s.to_sql(ty, out),
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(*ty,
            Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::NAME | Type::UNKNOWN |
            Type::INT8 |
            Type::UUID |
            Type::JSON | Type::JSONB
        )
    }

    fn to_sql_checked(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
         match self {
            SqlParam::RefString(s) => s.to_sql_checked(ty, out),
            SqlParam::OwnedString(s) => s.to_sql_checked(ty, out),
            SqlParam::I64(n) => n.to_sql_checked(ty, out),
            SqlParam::Uuid(u) => u.to_sql_checked(ty, out),
            SqlParam::NullI64 => <Option<i64> as ToSql>::to_sql_checked(&None, ty, out),
            SqlParam::NullString => <Option<String> as ToSql>::to_sql_checked(&None, ty, out),
            SqlParam::JsonValue(v) => v.to_sql_checked(ty, out),
            SqlParam::OwnedJsonString(s) => s.to_sql_checked(ty, out),
        }
    }
}

/// Helper untuk konversi JSON value ke PostgreSQL parameter - SIMPLIFIED for TEXT columns
pub fn json_to_sql_param<'a>(
    value: &'a Value,
    column_name: &str,
) -> SqlParam<'a> {
    match value {
        Value::String(s) => {
            // Special handling for api_id column - convert to i64
            if column_name == "api_id" {
                if let Ok(num) = s.parse::<i64>() {
                    return SqlParam::I64(num);
                }
                // If parse fails, return NULL - SKIP THIS COLUMN!
                return SqlParam::NullI64;
            }

            // Try to parse as UUID for id/parent_id/satker_id columns only
            if (column_name == "id" || column_name.ends_with("_id"))
                && let Ok(uuid) = Uuid::parse_str(s) {
                    return SqlParam::Uuid(uuid);
                }

            // Everything else is TEXT - keep as string ref
            SqlParam::RefString(s.as_str())
        }
        Value::Number(n) => {
            // Special handling for api_id - store as i64
            if column_name == "api_id" {
                if let Some(num) = n.as_i64() {
                    return SqlParam::I64(num);
                }
                // Return NULL for api_id if not i64
                return SqlParam::NullI64;
            }

            // For all other numbers, convert to string
            // PostgreSQL TEXT columns accept strings, and NUMERIC can cast from string
            // This avoids complex type matching logic
            SqlParam::OwnedString(n.to_string())
        }
        Value::Bool(b) => {
            // Convert bool to string for TEXT columns
            SqlParam::RefString(if *b {
                "true"
            } else {
                "false"
            })
        }
        Value::Null => {
            // For api_id column, return NULL as Option<i64>
            if column_name == "api_id" {
                return SqlParam::NullI64;
            }
            // For other columns, return NULL as Option<String>
            SqlParam::NullString
        }
        Value::Array(arr) => {
            // For JSONB columns, pass the value as serde_json::Value directly
            if column_name == "raw_data"
                || column_name.ends_with("_json")
                || column_name.ends_with("_data")
            {
                return SqlParam::JsonValue(value);
            }
            // Otherwise convert to JSON string for TEXT columns
            SqlParam::OwnedJsonString(serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string()))
        }
        Value::Object(_) => {
            // For JSONB columns, pass the value as serde_json::Value directly
            if column_name == "raw_data"
                || column_name.ends_with("_json")
                || column_name.ends_with("_data")
            {
                return SqlParam::JsonValue(value);
            }
            // Otherwise convert to JSON string for TEXT columns
            SqlParam::OwnedJsonString(serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()))
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
                let mut params: Vec<SqlParam> = Vec::new();
                for col in &columns {
                    // Handle mapping: api_id in DB comes from id in JSON
                    let json_key = if col == "api_id" { "id" } else { col.as_str() };

                    let value = obj
                        .get(json_key)
                        .or_else(|| obj.get(&json_key.to_uppercase()))
                        .unwrap_or(&Value::Null);
                    params.push(json_to_sql_param(value, col));
                }

                // Convert to references for execute
                let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
                    .iter()
                    .map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync))
                    .collect();

                match db.execute(&query, &param_refs[..]).await {
                    Ok(rows) => {
                        if rows > 0 {
                            count += rows as usize;
                            debug!("✅ [BULK INSERT] Inserted {} row(s)", rows);
                        } else {
                            debug!("⚠️  [BULK INSERT] 0 rows inserted (possibly duplicate)");
                        }
                    }
                    Err(e) => {
                        // Log more details for debugging
                        error!(
                            "❌ [BULK INSERT] Insert error for table {}: {:?} | Columns: {:?}",
                            table_name, e, columns
                        );
                        error!("❌ [BULK INSERT] Failed query: {}", query);

                        // Print the actual data values to debug type issues
                        for (i, col) in columns.iter().enumerate() {
                            let value = obj
                                .get(col)
                                .or_else(|| obj.get(&col.to_uppercase()))
                                .unwrap_or(&Value::Null);
                            let type_str = if value.is_string() {
                                "string"
                            } else if value.is_number() {
                                "number"
                            } else if value.is_boolean() {
                                "bool"
                            } else if value.is_null() {
                                "null"
                            } else if value.is_array() {
                                "array"
                            } else {
                                "object"
                            };
                            error!("  Column[{}] {}: {:?} (type: {})", i, col, value, type_str);
                        }
                        failed += 1;
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_to_sql_param_conversions() {
        // 1. Text conversion
        let v = json!("hello");
        let p = json_to_sql_param(&v, "name");
        if let SqlParam::RefString(s) = p {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected RefString");
        }

        // 2. api_id as i64 string
        let v = json!("12345");
        let p = json_to_sql_param(&v, "api_id");
        if let SqlParam::I64(n) = p {
            assert_eq!(n, 12345);
        } else {
            panic!("Expected I64");
        }

        // 3. api_id as number
        let v = json!(67890);
        let p = json_to_sql_param(&v, "api_id");
        if let SqlParam::I64(n) = p {
            assert_eq!(n, 67890);
        } else {
            panic!("Expected I64");
        }

        // 4. api_id invalid
        let v = json!("not-a-number");
        let p = json_to_sql_param(&v, "api_id");
        match p {
            SqlParam::NullI64 => {},
            _ => panic!("Expected NullI64"),
        }

        // 5. UUID
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let v = json!(uuid_str);
        let p = json_to_sql_param(&v, "id");
        if let SqlParam::Uuid(u) = p {
            assert_eq!(u.to_string(), uuid_str);
        } else {
            panic!("Expected Uuid");
        }

        // 6. Bool
        let v = json!(true);
        let p = json_to_sql_param(&v, "is_active");
        if let SqlParam::RefString(s) = p {
            assert_eq!(s, "true");
        } else {
            panic!("Expected RefString 'true'");
        }

        // 7. Null
        let v = json!(null);
        let p = json_to_sql_param(&v, "description");
        match p {
            SqlParam::NullString => {},
            _ => panic!("Expected NullString"),
        }

        // 8. JSONB
        let v = json!({"foo": "bar"});
        let p = json_to_sql_param(&v, "raw_data");
        if let SqlParam::JsonValue(val) = p {
            assert_eq!(val, &v);
        } else {
            panic!("Expected JsonValue");
        }

        // 9. Array as String (for TEXT column)
        let v = json!([1, 2, 3]);
        let p = json_to_sql_param(&v, "tags"); // Not ending in _data or _json
        if let SqlParam::OwnedJsonString(s) = p {
            assert_eq!(s, "[1,2,3]");
        } else {
            panic!("Expected OwnedJsonString");
        }
    }
}
