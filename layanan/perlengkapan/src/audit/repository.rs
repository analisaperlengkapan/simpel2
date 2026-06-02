//! # Cross-module audit trail — repository
//!
//! Reads `perlengkapan.audit_log` (the canonical `AuditSink` table, V028 +
//! retention V037). All user-supplied filters are bound as `$N` parameters —
//! never string-interpolated — so the endpoint is injection-safe. The WHERE
//! builder is split into a pure [`build_filter`] helper that is unit-tested
//! without a database.

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::shared::error::AppResult;

use super::models::{AuditTrailEntry, AuditTrailQuery};

/// A single bound parameter. Kept as a typed enum (rather than an opaque
/// `Box<dyn ToSql>`) so [`build_filter`] stays pure and inspectable in tests.
#[derive(Debug, Clone, PartialEq)]
pub enum AuditParam {
    Uuid(Uuid),
    Text(String),
    Ts(DateTime<Utc>),
}

impl AuditParam {
    fn as_to_sql(&self) -> &(dyn ToSql + Sync) {
        match self {
            AuditParam::Uuid(v) => v,
            AuditParam::Text(v) => v,
            AuditParam::Ts(v) => v,
        }
    }
}

/// Build the parameterized `WHERE` clause for an audit-trail query.
///
/// Returns the SQL fragment (`WHERE 1=1 AND …`) plus the ordered list of
/// bound parameters. Pure & DB-free so it can be unit-tested: the contract is
/// that *no raw user input ever lands in the returned string* — only `$N`
/// placeholders.
pub fn build_filter(q: &AuditTrailQuery) -> (String, Vec<AuditParam>) {
    let mut sql = String::from("WHERE 1=1");
    let mut params: Vec<AuditParam> = Vec::new();

    // entity → resource_type (DoD: GET /audit?entity=...)
    if let Some(entity) = q.entity.as_ref().filter(|s| !s.is_empty()) {
        params.push(AuditParam::Text(entity.clone()));
        sql.push_str(&format!(" AND resource_type = ${}", params.len()));
    }

    if let Some(rid) = q.resource_id.as_ref().filter(|s| !s.is_empty()) {
        params.push(AuditParam::Text(rid.clone()));
        sql.push_str(&format!(" AND resource_id = ${}", params.len()));
    }

    if let Some(module) = q.module.as_ref().filter(|s| !s.is_empty()) {
        params.push(AuditParam::Text(module.clone()));
        sql.push_str(&format!(" AND module = ${}", params.len()));
    }

    if let Some(action) = q.action.as_ref().filter(|s| !s.is_empty()) {
        params.push(AuditParam::Text(action.clone()));
        sql.push_str(&format!(" AND action = ${}", params.len()));
    }

    if let Some(actor) = q.actor.as_ref().filter(|s| !s.is_empty()) {
        if let Ok(uuid) = Uuid::parse_str(actor) {
            params.push(AuditParam::Uuid(uuid));
            sql.push_str(&format!(" AND actor_user_id = ${}", params.len()));
        } else {
            params.push(AuditParam::Text(format!("%{}%", actor)));
            sql.push_str(&format!(" AND actor_username ILIKE ${}", params.len()));
        }
    }

    if let Some(from) = q.from.as_ref().filter(|s| !s.is_empty())
        && let Ok(dt) = DateTime::parse_from_rfc3339(from)
    {
        params.push(AuditParam::Ts(dt.with_timezone(&Utc)));
        sql.push_str(&format!(" AND occurred_at >= ${}", params.len()));
    }

    if let Some(to) = q.to.as_ref().filter(|s| !s.is_empty())
        && let Ok(dt) = DateTime::parse_from_rfc3339(to)
    {
        params.push(AuditParam::Ts(dt.with_timezone(&Utc)));
        sql.push_str(&format!(" AND occurred_at <= ${}", params.len()));
    }

    if let Some(text) = q.q.as_ref().filter(|s| !s.is_empty()) {
        params.push(AuditParam::Text(format!("%{}%", text)));
        let n = params.len();
        sql.push_str(&format!(
            " AND (message ILIKE ${n} OR action_name ILIKE ${n} OR actor_username ILIKE ${n})"
        ));
    }

    (sql, params)
}

/// List audit-log rows matching `query`, newest first, paginated. Returns the
/// page of entries plus the unpaginated total for the same filter.
pub async fn list_audit_trail(
    pool: &Pool,
    query: &AuditTrailQuery,
) -> AppResult<(Vec<AuditTrailEntry>, i64)> {
    let client = pool.get().await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let (where_sql, params) = build_filter(query);
    let param_refs: Vec<&(dyn ToSql + Sync)> = params.iter().map(AuditParam::as_to_sql).collect();

    let count_sql = format!("SELECT COUNT(*) FROM perlengkapan.audit_log {where_sql}");
    let total: i64 = client
        .query_one(count_sql.as_str(), &param_refs)
        .await?
        .get(0);

    let list_sql = format!(
        "SELECT id, occurred_at, actor_user_id, actor_username, actor_ip, \
                action, action_name, resource_type, resource_id, module, \
                success, message, metadata, retention_until \
         FROM perlengkapan.audit_log {where_sql} \
         ORDER BY occurred_at DESC \
         LIMIT {per_page} OFFSET {offset}"
    );

    let rows = client.query(list_sql.as_str(), &param_refs).await?;
    let entries = rows.into_iter().map(row_to_entry).collect();

    Ok((entries, total))
}

fn row_to_entry(row: tokio_postgres::Row) -> AuditTrailEntry {
    let occurred_at: DateTime<Utc> = row.get("occurred_at");
    let actor_id: Option<Uuid> = row.try_get("actor_user_id").ok().flatten();
    let retention: Option<DateTime<Utc>> = row.try_get("retention_until").ok();
    AuditTrailEntry {
        id: row.get::<_, Uuid>("id").to_string(),
        occurred_at: occurred_at.to_rfc3339(),
        actor_id: actor_id.map(|u| u.to_string()),
        actor_name: row.try_get("actor_username").ok().flatten(),
        actor_ip: row.try_get("actor_ip").ok().flatten(),
        action: row.get("action"),
        action_name: row.try_get("action_name").ok().flatten(),
        resource_type: row.get("resource_type"),
        resource_id: row.try_get("resource_id").ok().flatten(),
        module: row.get("module"),
        success: row.try_get("success").unwrap_or(true),
        message: row.try_get("message").ok().flatten(),
        metadata: row.try_get("metadata").ok().flatten(),
        retention_until: retention.map(|t| t.to_rfc3339()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No filters → a bare guard clause with zero bound params.
    #[test]
    fn empty_filter_binds_nothing() {
        let (sql, params) = build_filter(&AuditTrailQuery::default());
        assert_eq!(sql, "WHERE 1=1");
        assert!(params.is_empty());
    }

    /// entity → resource_type, bound as $1 (never interpolated).
    #[test]
    fn entity_maps_to_resource_type_param() {
        let q = AuditTrailQuery {
            entity: Some("penghapusan_bmn".to_string()),
            ..Default::default()
        };
        let (sql, params) = build_filter(&q);
        assert!(sql.contains("resource_type = $1"));
        assert!(!sql.contains("penghapusan_bmn"));
        assert_eq!(params, vec![AuditParam::Text("penghapusan_bmn".to_string())]);
    }

    /// A UUID actor binds exactly; a non-UUID actor falls back to ILIKE.
    #[test]
    fn actor_uuid_vs_name() {
        let uuid = Uuid::new_v4();
        let (_, params) = build_filter(&AuditTrailQuery {
            actor: Some(uuid.to_string()),
            ..Default::default()
        });
        assert_eq!(params, vec![AuditParam::Uuid(uuid)]);

        let (sql, params) = build_filter(&AuditTrailQuery {
            actor: Some("budi".to_string()),
            ..Default::default()
        });
        assert!(sql.contains("actor_username ILIKE $1"));
        assert_eq!(params, vec![AuditParam::Text("%budi%".to_string())]);
    }

    /// A SQLi payload is bound, not interpolated: it appears only inside the
    /// parameter list, never in the SQL string.
    #[test]
    fn injection_payload_is_parameterized() {
        let payload = "x'; DROP TABLE perlengkapan.audit_log;--";
        let (sql, params) = build_filter(&AuditTrailQuery {
            entity: Some(payload.to_string()),
            resource_id: Some(payload.to_string()),
            module: Some(payload.to_string()),
            q: Some(payload.to_string()),
            ..Default::default()
        });
        assert!(!sql.contains("DROP TABLE"));
        assert!(sql.contains("resource_type = $1"));
        // every clause is a placeholder; payloads live only in params
        assert!(params.iter().any(|p| matches!(p, AuditParam::Text(t) if t.contains("DROP TABLE"))));
    }

    /// Placeholder numbering stays sequential as clauses accumulate.
    #[test]
    fn placeholders_are_sequential() {
        let q = AuditTrailQuery {
            entity: Some("kebutuhan_bmn".to_string()),
            module: Some("kebutuhan".to_string()),
            action: Some("approve".to_string()),
            ..Default::default()
        };
        let (sql, params) = build_filter(&q);
        assert!(sql.contains("resource_type = $1"));
        assert!(sql.contains("module = $2"));
        assert!(sql.contains("action = $3"));
        assert_eq!(params.len(), 3);
    }

    /// Malformed RFC3339 bounds are ignored rather than bound blindly.
    #[test]
    fn invalid_dates_are_skipped() {
        let (sql, params) = build_filter(&AuditTrailQuery {
            from: Some("not-a-date".to_string()),
            to: Some("2026-01-01T00:00:00Z".to_string()),
            ..Default::default()
        });
        assert!(!sql.contains("occurred_at >="));
        assert!(sql.contains("occurred_at <= $1"));
        assert_eq!(params.len(), 1);
    }
}
