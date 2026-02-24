use crate::error::MonsaktiError;
use bytes::{BufMut, BytesMut};
use serde_json::{Number, Value};
use std::error::Error;
use std::io::Write;
use tokio_postgres::Client;
use tokio_postgres::types::{IsNull, ToSql, Type};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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
    NumberText(&'a Number),
    JsonToText(&'a Value),
}

impl<'a> ToSql for SqlParam<'a> {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self {
            SqlParam::RefString(s) => s.to_sql(ty, out),
            SqlParam::OwnedString(s) => s.to_sql(ty, out),
            SqlParam::I64(n) => n.to_sql(ty, out),
            SqlParam::Uuid(u) => u.to_sql(ty, out),
            SqlParam::NullI64 => <Option<i64> as ToSql>::to_sql(&None, ty, out),
            SqlParam::NullString => <Option<String> as ToSql>::to_sql(&None, ty, out),
            SqlParam::JsonValue(v) => v.to_sql(ty, out),
            SqlParam::OwnedJsonString(s) => s.to_sql(ty, out),
            SqlParam::NumberText(n) => {
                let mut writer = out.writer();
                write!(writer, "{}", n)?;
                Ok(IsNull::No)
            }
            SqlParam::JsonToText(v) => {
                let writer = out.writer();
                serde_json::to_writer(writer, v)?;
                Ok(IsNull::No)
            }
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(
            *ty,
            Type::VARCHAR
                | Type::TEXT
                | Type::BPCHAR
                | Type::NAME
                | Type::UNKNOWN
                | Type::INT8
                | Type::UUID
                | Type::JSON
                | Type::JSONB
        )
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self {
            SqlParam::RefString(s) => s.to_sql_checked(ty, out),
            SqlParam::OwnedString(s) => s.to_sql_checked(ty, out),
            SqlParam::I64(n) => n.to_sql_checked(ty, out),
            SqlParam::Uuid(u) => u.to_sql_checked(ty, out),
            SqlParam::NullI64 => <Option<i64> as ToSql>::to_sql_checked(&None, ty, out),
            SqlParam::NullString => <Option<String> as ToSql>::to_sql_checked(&None, ty, out),
            SqlParam::JsonValue(v) => v.to_sql_checked(ty, out),
            SqlParam::OwnedJsonString(s) => s.to_sql_checked(ty, out),
            SqlParam::NumberText(_) => self.to_sql(ty, out),
            SqlParam::JsonToText(_) => self.to_sql(ty, out),
        }
    }
}

/// Helper untuk konversi JSON value ke PostgreSQL parameter - SIMPLIFIED for TEXT columns
pub fn json_to_sql_param<'a>(value: &'a Value, column_name: &str) -> SqlParam<'a> {
    match value {
        Value::String(s) => {
            // All _id columns are TEXT in our schema — do NOT auto-convert to UUID.
            // This avoids WrongType errors when the DB column is TEXT but the
            // value looks like a UUID string.
            SqlParam::RefString(s.as_str())
        }
        Value::Number(n) => {
            // For all numbers, convert to string
            // PostgreSQL TEXT columns accept strings, and NUMERIC can cast from string
            // This avoids complex type matching logic
            SqlParam::NumberText(n)
        }
        Value::Bool(b) => {
            // Convert bool to string for TEXT columns
            SqlParam::RefString(if *b { "true" } else { "false" })
        }
        Value::Null => {
            // For all columns, return NULL as Option<String>
            SqlParam::NullString
        }
        Value::Array(_arr) => {
            // For columns ending in _json or _data, pass as JSONB
            if column_name.ends_with("_json") || column_name.ends_with("_data") {
                return SqlParam::JsonValue(value);
            }
            // Otherwise convert to JSON string for TEXT columns
            SqlParam::JsonToText(value)
        }
        Value::Object(_) => {
            // For columns ending in _json or _data, pass as JSONB
            if column_name.ends_with("_json") || column_name.ends_with("_data") {
                return SqlParam::JsonValue(value);
            }
            // Otherwise convert to JSON string for TEXT columns
            SqlParam::JsonToText(value)
        }
    }
}

/// Query PostgreSQL for unique constraint columns on a table (excluding PRIMARY KEY).
/// Returns the column names that form unique constraints, used for ON CONFLICT UPSERT.
async fn get_unique_columns(db: &Client, table_name: &str) -> Vec<String> {
    // Use pg_class + pg_namespace join instead of $1::regclass to avoid
    // tokio-postgres parameter type inference issues with regclass OID
    let query = r#"
        SELECT a.attname
        FROM pg_constraint c
        JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY(c.conkey)
        JOIN pg_class cl ON cl.oid = c.conrelid
        JOIN pg_namespace ns ON ns.oid = cl.relnamespace
        WHERE c.contype = 'u'
          AND cl.relname = $1
          AND ns.nspname = $2
        ORDER BY a.attnum
    "#;
    // Try integrasi schema first, then public
    for schema in &["integrasi", "public"] {
        if let Ok(rows) = db.query(query, &[&table_name, schema]).await {
            let cols: Vec<String> = rows.iter().map(|r| r.get::<_, String>(0)).collect();
            if !cols.is_empty() {
                return cols;
            }
        }
    }
    Vec::new()
}

/// Auto-create table if it doesn't exist, inferring column types from data
async fn ensure_table_exists(
    db: &Client,
    table_name: &str,
    columns: &[String],
) -> Result<(), MonsaktiError> {
    // Check if table exists via pg_tables (search_path aware)
    let exists = db
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_tables WHERE tablename = $1)",
            &[&table_name],
        )
        .await
        .map(|row| row.get::<_, bool>(0))
        .unwrap_or(false);

    if exists {
        // Table exists — check for missing columns and add them
        let existing_cols_rows = db
            .query(
                "SELECT column_name FROM information_schema.columns WHERE table_name = $1",
                &[&table_name],
            )
            .await
            .unwrap_or_default();

        let existing_cols: std::collections::HashSet<String> = existing_cols_rows
            .iter()
            .map(|r| r.get::<_, String>(0))
            .collect();

        let mut added = 0u32;
        for col in columns {
            let col_lower = col.to_lowercase();
            if !existing_cols.contains(&col_lower) {
                let col_type = if col_lower == "api_id" {
                    "BIGINT"
                } else {
                    "TEXT"
                };
                let alter_sql = format!(
                    "ALTER TABLE {} ADD COLUMN IF NOT EXISTS {} {}",
                    table_name, col_lower, col_type
                );
                match db.execute(&alter_sql, &[]).await {
                    Ok(_) => {
                        info!(
                            "➕ [SCHEMA] Added missing column '{}' ({}) to table '{}'",
                            col_lower, col_type, table_name
                        );
                        added += 1;
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  [SCHEMA] Failed to add column '{}' to '{}': {}",
                            col_lower, table_name, e
                        );
                    }
                }
            }
        }
        if added > 0 {
            info!(
                "✅ [SCHEMA] Added {} missing column(s) to existing table '{}'",
                added, table_name
            );
        }
        return Ok(());
    }

    info!(
        "📋 [AUTO-CREATE] Table '{}' does not exist, creating...",
        table_name
    );

    // Build column definitions: all TEXT except special ones
    let col_defs: Vec<String> = std::iter::once("id BIGSERIAL PRIMARY KEY".to_string())
        .chain(columns.iter().map(|col| {
            // All columns are TEXT for simplicity and compatibility
            format!("{} TEXT", col)
        }))
        .chain(std::iter::once(
            "synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP".to_string(),
        ))
        .chain(std::iter::once(
            "created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP".to_string(),
        ))
        .collect();

    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name,
        col_defs.join(", ")
    );

    db.execute(&create_sql, &[]).await.map_err(|e| {
        error!(
            "❌ [AUTO-CREATE] Failed to create table '{}': {}",
            table_name, e
        );
        MonsaktiError::DatabaseError(e)
    })?;

    info!(
        "✅ [AUTO-CREATE] Table '{}' created with {} columns",
        table_name,
        columns.len()
    );

    Ok(())
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
            && let Some(id_val) = obj.get("id").or_else(|| obj.get("ID"))
        {
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

    // Auto-create table if it doesn't exist
    ensure_table_exists(db, table_name, &columns).await?;

    let mut count = 0;
    let mut failed = 0;

    // Batch insert untuk performa lebih baik (100 rows per batch)
    info!(
        "🔄 [BULK INSERT] Processing {} records in chunks of 100...",
        data.len()
    );

    // Prepare query once (optimization: hoisted out of loop)
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

    // Auto-detect unique constraint columns to build proper UPSERT query
    let unique_cols = get_unique_columns(db, table_name).await;
    let query = if !unique_cols.is_empty() {
        // Build ON CONFLICT (unique_col) DO UPDATE SET ... for all non-key columns
        let update_cols: Vec<String> = columns
            .iter()
            .filter(|c| !unique_cols.contains(c))
            .map(|c| format!("{} = EXCLUDED.{}", c, c))
            .collect();
        let update_clause = if update_cols.is_empty() {
            "DO NOTHING".to_string()
        } else {
            // Also update synced_at on conflict
            format!(
                "DO UPDATE SET {}, synced_at = CURRENT_TIMESTAMP",
                update_cols.join(", ")
            )
        };
        info!(
            "🔑 [BULK INSERT] Using UPSERT on unique column(s): {:?}",
            unique_cols
        );
        format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) {}",
            table_name,
            columns.join(", "),
            placeholders.join(", "),
            unique_cols.join(", "),
            update_clause
        )
    } else {
        format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT DO NOTHING",
            table_name,
            columns.join(", "),
            placeholders.join(", ")
        )
    };

    // Reuse vector allocation for params to reduce memory churn
    let mut params: Vec<SqlParam> = Vec::with_capacity(columns.len());

    for (chunk_idx, chunk) in data.chunks(100).enumerate() {
        info!(
            "📦 [BULK INSERT] Processing chunk {}, {} records",
            chunk_idx + 1,
            chunk.len()
        );
        for item in chunk {
            if let Some(obj) = item.as_object() {
                // Konversi nilai JSON ke parameter PostgreSQL
                params.clear();
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
                // Note: We cannot easily reuse param_refs vector due to borrow checker constraints
                // (referencing 'params' which is mutable across iterations).
                // However, Vec<&dyn ToSql> allocation is cheap compared to the data itself.
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

        // 2. api_id as string (treated as TEXT now, not i64)
        let v = json!("12345");
        let p = json_to_sql_param(&v, "api_id");
        if let SqlParam::RefString(s) = p {
            assert_eq!(s, "12345");
        } else {
            panic!("Expected RefString for api_id string");
        }

        // 3. api_id as number (converted to text representation)
        let v = json!(67890);
        let p = json_to_sql_param(&v, "api_id");
        if let SqlParam::NumberText(n) = p {
            assert_eq!(n.as_i64(), Some(67890));
        } else {
            panic!("Expected NumberText for api_id number");
        }

        // 4. non-numeric string for api_id (stays as text)
        let v = json!("not-a-number");
        let p = json_to_sql_param(&v, "api_id");
        if let SqlParam::RefString(s) = p {
            assert_eq!(s, "not-a-number");
        } else {
            panic!("Expected RefString for non-numeric api_id");
        }

        // 5. UUID-like string stays as TEXT (no auto-detection)
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let v = json!(uuid_str);
        let p = json_to_sql_param(&v, "id");
        if let SqlParam::RefString(s) = p {
            assert_eq!(s, uuid_str);
        } else {
            panic!("Expected RefString for UUID-like string");
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
            SqlParam::NullString => {}
            _ => panic!("Expected NullString"),
        }

        // 8. JSONB (column ending in _data)
        let v = json!({"foo": "bar"});
        let p = json_to_sql_param(&v, "extra_data");
        if let SqlParam::JsonValue(val) = p {
            assert_eq!(val, &v);
        } else {
            panic!("Expected JsonValue");
        }

        // 9. Array as String (for TEXT column)
        let v = json!([1, 2, 3]);
        let p = json_to_sql_param(&v, "tags"); // Not ending in _data or _json
        // Changed to JsonToText
        if let SqlParam::JsonToText(val) = p {
            assert_eq!(val, &v);
        } else {
            panic!("Expected JsonToText");
        }

        // 10. Number as String (for TEXT column)
        let v = json!(123.45);
        let p = json_to_sql_param(&v, "amount");
        if let SqlParam::NumberText(n) = p {
            assert_eq!(n.as_f64(), Some(123.45));
        } else {
            panic!("Expected NumberText");
        }
    }
}
