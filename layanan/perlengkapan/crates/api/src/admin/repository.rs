//! # Admin Repository
//!
//! SQL queries backing `/admin/audit` and `/admin/master` handlers.

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde_json::json;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::models::{AuditFilter, AuditLogEntry, MasterRecord, MasterSource};

// ═════════════════════════════════════════════════════════════════════════
// Audit log
// ═════════════════════════════════════════════════════════════════════════

pub async fn list_audit_logs(
    pool: &Pool,
    filter: &AuditFilter,
) -> AppResult<(Vec<AuditLogEntry>, i64)> {
    let client = pool.get().await?;

    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(25).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let mut where_sql = String::from("WHERE 1=1");
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

    if let Some(actor) = filter.actor.as_ref().filter(|s| !s.is_empty()) {
        // actor can be UUID or plain name match
        if let Ok(uuid) = Uuid::parse_str(actor) {
            params.push(Box::new(uuid));
            where_sql.push_str(&format!(" AND t.actor_user_id = ${}", params.len()));
        } else {
            params.push(Box::new(format!("%{}%", actor)));
            where_sql.push_str(&format!(" AND t.actor_nama ILIKE ${}", params.len()));
        }
    }

    if let Some(entity) = filter.entity_type.as_ref().filter(|s| !s.is_empty()) {
        params.push(Box::new(entity.clone()));
        where_sql.push_str(&format!(" AND i.entity_type = ${}", params.len()));
    }

    if let Some(action) = filter.action.as_ref().filter(|s| !s.is_empty()) {
        params.push(Box::new(action.clone()));
        where_sql.push_str(&format!(" AND t.action = ${}", params.len()));
    }

    if let Some(from) = filter.from.as_ref().filter(|s| !s.is_empty())
        && let Ok(dt) = DateTime::parse_from_rfc3339(from)
    {
        params.push(Box::new(dt.with_timezone(&Utc)));
        where_sql.push_str(&format!(" AND t.transitioned_at >= ${}", params.len()));
    }

    if let Some(to) = filter.to.as_ref().filter(|s| !s.is_empty())
        && let Ok(dt) = DateTime::parse_from_rfc3339(to)
    {
        params.push(Box::new(dt.with_timezone(&Utc)));
        where_sql.push_str(&format!(" AND t.transitioned_at <= ${}", params.len()));
    }

    if let Some(q) = filter.q.as_ref().filter(|s| !s.is_empty()) {
        params.push(Box::new(format!("%{}%", q)));
        let n = params.len();
        where_sql.push_str(&format!(
            " AND (t.komentar ILIKE ${n} OR t.action ILIKE ${n} OR t.actor_nama ILIKE ${n})"
        ));
    }

    let count_sql = format!(
        "SELECT COUNT(*) FROM perlengkapan.workflow_transitions t \
         JOIN perlengkapan.workflow_instances i ON i.id = t.workflow_instance_id \
         {where_sql}"
    );

    let params_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let total: i64 = client
        .query_one(count_sql.as_str(), &params_refs)
        .await?
        .get(0);

    let list_sql = format!(
        "SELECT t.id, t.transitioned_at, t.actor_user_id, t.actor_nama, t.actor_role, \
                t.action, i.entity_type, i.entity_id, t.komentar, \
                t.ip_address::TEXT AS ip_text, t.metadata \
         FROM perlengkapan.workflow_transitions t \
         JOIN perlengkapan.workflow_instances i ON i.id = t.workflow_instance_id \
         {where_sql} \
         ORDER BY t.transitioned_at DESC \
         LIMIT {per_page} OFFSET {offset}"
    );

    let rows = client.query(list_sql.as_str(), &params_refs).await?;

    let entries = rows
        .into_iter()
        .map(|row| {
            let ts: DateTime<Utc> = row.get("transitioned_at");
            let actor_id: Option<Uuid> = row.try_get("actor_user_id").ok();
            let entity_id: Option<Uuid> = row.try_get("entity_id").ok();
            AuditLogEntry {
                id: row.get::<_, Uuid>("id").to_string(),
                occurred_at: ts.to_rfc3339(),
                actor_id: actor_id.map(|u| u.to_string()),
                actor_name: row.try_get("actor_nama").ok(),
                actor_role: row.try_get("actor_role").ok(),
                action: row.get("action"),
                entity_type: row.get("entity_type"),
                entity_id: entity_id.map(|u| u.to_string()),
                summary: row.try_get("komentar").ok(),
                ip_address: row.try_get("ip_text").ok(),
                metadata: row.try_get("metadata").ok(),
            }
        })
        .collect();

    Ok((entries, total))
}

// ═════════════════════════════════════════════════════════════════════════
// Master data catalog
// ═════════════════════════════════════════════════════════════════════════

/// Whitelisted master tables — key -> descriptor. Keeps raw SQL off the
/// request path and guarantees callers can't target arbitrary tables.
pub struct MasterTable {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub schema: &'static str,
    pub table: &'static str,
    pub id_column: &'static str,
    pub id_is_uuid: bool,
    pub name_column: &'static str,
    pub code_column: Option<&'static str>,
    pub description_column: Option<&'static str>,
    pub active_column: Option<&'static str>,
    pub updated_column: Option<&'static str>,
    pub writable: bool,
}

pub fn master_catalog() -> &'static [MasterTable] {
    &[
        MasterTable {
            key: "aktivitas_bmn",
            label: "Aktivitas BMN",
            description: "Status workflow untuk pengajuan BMN dan pakaian dinas",
            icon: "fas fa-stream",
            schema: "perlengkapan",
            table: "ms_aktivitas_bmn",
            id_column: "id",
            id_is_uuid: false,
            name_column: "nama",
            code_column: Some("kode"),
            description_column: Some("deskripsi"),
            active_column: Some("is_active"),
            updated_column: Some("updated_at"),
            writable: true,
        },
        MasterTable {
            key: "jenis_pakaian_dinas",
            label: "Jenis Pakaian Dinas",
            description: "Kategori pakaian dinas (PDH, PDL, Toga, dll)",
            icon: "fas fa-user-tie",
            schema: "public",
            table: "ms_jenis_pakaian_dinas",
            id_column: "id",
            id_is_uuid: true,
            name_column: "nama",
            code_column: None,
            description_column: Some("deskripsi"),
            active_column: Some("is_active"),
            updated_column: Some("updated_at"),
            writable: true,
        },
        MasterTable {
            key: "spesifikasi_pakaian_dinas",
            label: "Spesifikasi Pakaian",
            description: "Spesifikasi per jenis pakaian dengan kategori ukuran",
            icon: "fas fa-ruler",
            schema: "public",
            table: "ms_spesifikasi_pakaian_dinas",
            id_column: "id",
            id_is_uuid: true,
            name_column: "nama",
            code_column: None,
            description_column: Some("deskripsi"),
            active_column: Some("is_active"),
            updated_column: Some("updated_at"),
            writable: false,
        },
        MasterTable {
            key: "subspesifikasi_pakaian_dinas",
            label: "Sub-spesifikasi",
            description: "Varian dari setiap spesifikasi pakaian dinas",
            icon: "fas fa-layer-group",
            schema: "public",
            table: "ms_subspesifikasi_pakaian_dinas",
            id_column: "id",
            id_is_uuid: true,
            name_column: "nama",
            code_column: None,
            description_column: None,
            active_column: Some("is_active"),
            updated_column: Some("updated_at"),
            writable: false,
        },
    ]
}

pub fn find_master_table(key: &str) -> Option<&'static MasterTable> {
    master_catalog().iter().find(|m| m.key == key)
}

pub async fn list_master_sources(pool: &Pool) -> AppResult<Vec<MasterSource>> {
    let client = pool.get().await?;
    let mut out = Vec::with_capacity(master_catalog().len());

    for entry in master_catalog() {
        let count_sql = format!(
            "SELECT COUNT(*)::BIGINT, MAX({}) FROM {}.{}",
            entry.updated_column.unwrap_or("NULL"),
            entry.schema,
            entry.table
        );

        let row = client.query_one(count_sql.as_str(), &[]).await?;
        let count: i64 = row.get(0);
        let last_updated: Option<DateTime<Utc>> = row.try_get(1).ok().flatten();

        out.push(MasterSource {
            key: entry.key.to_string(),
            label: entry.label.to_string(),
            description: entry.description.to_string(),
            icon: Some(entry.icon.to_string()),
            record_count: count,
            updated_at: last_updated.map(|t| t.to_rfc3339()),
        });
    }

    Ok(out)
}

pub async fn list_master_records(
    pool: &Pool,
    table: &MasterTable,
    page: i32,
    per_page: i32,
    search: Option<&str>,
) -> AppResult<(Vec<MasterRecord>, i64)> {
    let client = pool.get().await?;
    let page = page.max(1);
    let per_page = per_page.clamp(1, 200);
    let offset = (page - 1) * per_page;

    let mut where_sql = String::from("WHERE 1=1");
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

    if let Some(q) = search.filter(|s| !s.is_empty()) {
        params.push(Box::new(format!("%{}%", q)));
        let n = params.len();
        let mut clauses = vec![format!("{} ILIKE ${}", table.name_column, n)];
        if let Some(code_col) = table.code_column {
            clauses.push(format!("{}::TEXT ILIKE ${}", code_col, n));
        }
        if let Some(desc_col) = table.description_column {
            clauses.push(format!("{} ILIKE ${}", desc_col, n));
        }
        where_sql.push_str(&format!(" AND ({})", clauses.join(" OR ")));
    }

    let params_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let count_sql = format!(
        "SELECT COUNT(*)::BIGINT FROM {}.{} {}",
        table.schema, table.table, where_sql
    );
    let total: i64 = client
        .query_one(count_sql.as_str(), &params_refs)
        .await?
        .get(0);

    let select_cols = build_select_columns(table);
    let order = table
        .updated_column
        .map(|c| format!("ORDER BY {} DESC NULLS LAST", c))
        .unwrap_or_else(|| format!("ORDER BY {} ASC", table.name_column));

    let list_sql = format!(
        "SELECT {select_cols} FROM {}.{} {where_sql} {order} LIMIT {per_page} OFFSET {offset}",
        table.schema, table.table
    );
    let rows = client.query(list_sql.as_str(), &params_refs).await?;

    let records = rows
        .into_iter()
        .map(|row| row_to_master_record(table, &row))
        .collect();

    Ok((records, total))
}

pub async fn create_master_record(
    pool: &Pool,
    table: &MasterTable,
    req: &super::models::MasterUpsertRequest,
) -> AppResult<MasterRecord> {
    if !table.writable {
        return Err(AppError::Authorization(format!(
            "Master data '{}' bersifat read-only di hub admin",
            table.key
        )));
    }
    let client = pool.get().await?;

    let mut cols: Vec<&str> = vec![table.name_column];
    let mut placeholders: Vec<String> = vec!["$1".to_string()];
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(req.name.clone())];

    if let Some(code_col) = table.code_column {
        cols.push(code_col);
        params.push(match req.code.as_deref() {
            Some(v) => coerce_code_param(code_col, v)?,
            None => Box::new(Option::<String>::None),
        });
        placeholders.push(format!("${}", params.len()));
    }
    if let Some(desc_col) = table.description_column {
        cols.push(desc_col);
        params.push(Box::new(req.description.clone()));
        placeholders.push(format!("${}", params.len()));
    }
    if let Some(active_col) = table.active_column {
        cols.push(active_col);
        params.push(Box::new(req.active));
        placeholders.push(format!("${}", params.len()));
    }

    let params_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let select_cols = build_select_columns(table);
    let sql = format!(
        "INSERT INTO {}.{} ({}) VALUES ({}) RETURNING {select_cols}",
        table.schema,
        table.table,
        cols.join(", "),
        placeholders.join(", ")
    );

    let row = client.query_one(sql.as_str(), &params_refs).await?;
    Ok(row_to_master_record(table, &row))
}

pub async fn update_master_record(
    pool: &Pool,
    table: &MasterTable,
    id: &str,
    req: &super::models::MasterUpsertRequest,
) -> AppResult<MasterRecord> {
    if !table.writable {
        return Err(AppError::Authorization(format!(
            "Master data '{}' bersifat read-only di hub admin",
            table.key
        )));
    }
    let client = pool.get().await?;

    let mut sets: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

    params.push(Box::new(req.name.clone()));
    sets.push(format!("{} = ${}", table.name_column, params.len()));

    if let Some(code_col) = table.code_column {
        params.push(match req.code.as_deref() {
            Some(v) => coerce_code_param(code_col, v)?,
            None => Box::new(Option::<String>::None),
        });
        sets.push(format!("{} = ${}", code_col, params.len()));
    }
    if let Some(desc_col) = table.description_column {
        params.push(Box::new(req.description.clone()));
        sets.push(format!("{} = ${}", desc_col, params.len()));
    }
    if let Some(active_col) = table.active_column {
        params.push(Box::new(req.active));
        sets.push(format!("{} = ${}", active_col, params.len()));
    }
    if let Some(updated_col) = table.updated_column {
        sets.push(format!("{} = NOW()", updated_col));
    }

    let id_param = coerce_id_param(table, id)?;
    params.push(id_param);
    let id_placeholder = params.len();

    let params_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let select_cols = build_select_columns(table);
    let sql = format!(
        "UPDATE {}.{} SET {} WHERE {} = ${id_placeholder} RETURNING {select_cols}",
        table.schema,
        table.table,
        sets.join(", "),
        table.id_column
    );

    let row = client
        .query_opt(sql.as_str(), &params_refs)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Record {} tidak ditemukan", id)))?;
    Ok(row_to_master_record(table, &row))
}

pub async fn delete_master_record(pool: &Pool, table: &MasterTable, id: &str) -> AppResult<()> {
    if !table.writable {
        return Err(AppError::Authorization(format!(
            "Master data '{}' bersifat read-only di hub admin",
            table.key
        )));
    }
    let client = pool.get().await?;
    let id_param = coerce_id_param(table, id)?;
    let param_ref: &(dyn ToSql + Sync) = id_param.as_ref();
    let sql = format!(
        "DELETE FROM {}.{} WHERE {} = $1",
        table.schema, table.table, table.id_column
    );
    let rows = client.execute(sql.as_str(), &[param_ref]).await?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Record {} tidak ditemukan", id)));
    }
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════
// Helpers
// ═════════════════════════════════════════════════════════════════════════

fn build_select_columns(table: &MasterTable) -> String {
    let mut cols = vec![
        format!("{}::TEXT AS id_text", table.id_column),
        format!("{} AS name_col", table.name_column),
    ];
    cols.push(match table.code_column {
        Some(c) => format!("{}::TEXT AS code_col", c),
        None => "NULL::TEXT AS code_col".to_string(),
    });
    cols.push(match table.description_column {
        Some(c) => format!("{} AS description_col", c),
        None => "NULL::TEXT AS description_col".to_string(),
    });
    cols.push(match table.active_column {
        Some(c) => format!("{} AS active_col", c),
        None => "TRUE AS active_col".to_string(),
    });
    cols.push(match table.updated_column {
        Some(c) => format!("{} AS updated_col", c),
        None => "NULL::TIMESTAMPTZ AS updated_col".to_string(),
    });
    cols.join(", ")
}

fn row_to_master_record(table: &MasterTable, row: &tokio_postgres::Row) -> MasterRecord {
    let updated_at: Option<DateTime<Utc>> = row.try_get("updated_col").ok().flatten();
    MasterRecord {
        id: row.get("id_text"),
        code: row.try_get("code_col").ok().flatten(),
        name: row.try_get("name_col").unwrap_or_default(),
        description: row.try_get("description_col").ok().flatten(),
        active: row.try_get("active_col").unwrap_or(true),
        extra: json!({"table": table.key}),
        updated_at: updated_at.map(|t| t.to_rfc3339()),
    }
}

fn coerce_id_param(table: &MasterTable, id: &str) -> AppResult<Box<dyn ToSql + Sync + Send>> {
    if table.id_is_uuid {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| AppError::BadRequest(format!("ID tidak valid: {}", id)))?;
        Ok(Box::new(uuid))
    } else {
        let n: i32 = id
            .parse()
            .map_err(|_| AppError::BadRequest(format!("ID harus integer: {}", id)))?;
        Ok(Box::new(n))
    }
}

fn coerce_code_param(code_col: &str, value: &str) -> AppResult<Box<dyn ToSql + Sync + Send>> {
    // ms_aktivitas_bmn.kode is INTEGER; everything else is text.
    if code_col == "kode" {
        let n: i32 = value
            .parse()
            .map_err(|_| AppError::BadRequest(format!("Kode harus integer: {}", value)))?;
        Ok(Box::new(n))
    } else {
        Ok(Box::new(value.to_string()))
    }
}
