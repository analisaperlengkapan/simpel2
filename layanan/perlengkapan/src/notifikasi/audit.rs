//! Notifikasi-domain audit helpers — thin wrappers over the shared
//! `perlengkapan.audit_log` table written through
//! [`crate::shared::audit::PgAuditSink`].
//!
//! Callers that already hold an `Arc<dyn AuditSink>` from `AppState`
//! should prefer talking to it directly. These helpers exist so legacy
//! pool-based code paths (the older notifikasi service modules that
//! predate the shared sink) can write audit rows without threading the
//! port through their constructors.

use deadpool_postgres::Pool;
use serde_json::Value;
use uuid::Uuid;

use super::error::AppError;

/// Insert one row into `perlengkapan.audit_log`. Designed for legacy
/// callers that already plumb `Pool`; new code should call
/// `state.audit_sink.log(AuditEvent::…)` instead.
pub async fn insert_audit_log(
    pool: &Pool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action: &str,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Validation(format!("audit pool: {e}").into_boxed_str())
    })?;

    // Combine user_agent (a request-context hint) into metadata so the
    // shared schema keeps one JSONB column for free-form context.
    let mut metadata = details.clone();
    if let Some(obj) = metadata.as_object_mut() {
        if let Some(ua) = user_agent {
            obj.insert("user_agent".to_string(), Value::String(ua.to_string()));
        }
        if let Some(nid) = notification_id {
            obj.insert(
                "notification_id".to_string(),
                Value::String(nid.to_string()),
            );
        }
    }

    let row_id = Uuid::new_v4();
    client
        .execute(
            r#"
            INSERT INTO perlengkapan.audit_log
                (id, occurred_at, actor_user_id, actor_ip,
                 action, resource_type, resource_id, module,
                 success, metadata)
            VALUES
                ($1, NOW(), $2, $3, $4, 'notification', $5, 'notifikasi',
                 TRUE, $6)
            "#,
            &[
                &row_id,
                &user_id,
                &ip_address,
                &action,
                &notification_id.map(|n| n.to_string()),
                &metadata,
            ],
        )
        .await?;

    Ok(())
}

/// Read recent audit rows produced by the notifikasi module.
pub async fn query_audit_logs(
    pool: &Pool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let limit = limit.clamp(1, 500);
    let client = pool.get().await.map_err(|e| {
        AppError::Validation(format!("audit pool: {e}").into_boxed_str())
    })?;

    let rows = if let Some(nid) = notification_id {
        let nid_str = nid.to_string();
        client
            .query(
                "SELECT id, occurred_at, actor_user_id, actor_ip, action,
                        resource_id, success, metadata
                 FROM perlengkapan.audit_log
                 WHERE module = 'notifikasi'
                   AND resource_id = $1
                 ORDER BY occurred_at DESC
                 LIMIT $2",
                &[&nid_str, &limit],
            )
            .await?
    } else if let Some(uid) = user_id {
        client
            .query(
                "SELECT id, occurred_at, actor_user_id, actor_ip, action,
                        resource_id, success, metadata
                 FROM perlengkapan.audit_log
                 WHERE module = 'notifikasi'
                   AND actor_user_id = $1
                 ORDER BY occurred_at DESC
                 LIMIT $2",
                &[&uid, &limit],
            )
            .await?
    } else {
        client
            .query(
                "SELECT id, occurred_at, actor_user_id, actor_ip, action,
                        resource_id, success, metadata
                 FROM perlengkapan.audit_log
                 WHERE module = 'notifikasi'
                 ORDER BY occurred_at DESC
                 LIMIT $1",
                &[&limit],
            )
            .await?
    };

    Ok(rows
        .into_iter()
        .map(|r| {
            let id: Uuid = r.get("id");
            let occurred_at: chrono::DateTime<chrono::Utc> = r.get("occurred_at");
            let actor_user_id: Option<Uuid> = r.try_get("actor_user_id").ok().flatten();
            let actor_ip: Option<String> = r.try_get("actor_ip").ok().flatten();
            let action: String = r.get("action");
            let resource_id: Option<String> = r.try_get("resource_id").ok().flatten();
            let success: bool = r.get("success");
            let metadata: Option<Value> = r.try_get("metadata").ok().flatten();
            serde_json::json!({
                "id": id,
                "occurred_at": occurred_at,
                "actor_user_id": actor_user_id,
                "actor_ip": actor_ip,
                "action": action,
                "resource_id": resource_id,
                "success": success,
                "metadata": metadata,
            })
        })
        .collect())
}
