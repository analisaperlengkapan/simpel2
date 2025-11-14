/// Database operations for authentication flows
use crate::database::Database;
use crate::error::{AuthencError, Result};
use chrono::Utc;
use log::error;
use uuid::Uuid;

pub async fn create_flow(db: &Database, flow: &serde_json::Value) -> Result<serde_json::Value> {
    let flow_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO authentication_flows (
            id, realm_id, alias, description, provider_id,
            top_level, built_in, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, realm_id, alias, description, provider_id,
                  top_level, built_in, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &flow_id,
                &flow
                    .get("realm_id")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &flow.get("alias").and_then(|v| v.as_str()).unwrap_or(""),
                &flow.get("description").and_then(|v| v.as_str()),
                &flow
                    .get("provider_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                &flow
                    .get("top_level")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                &flow
                    .get("built_in")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create authentication flow: {}", e);
            AuthencError::database("Failed to create authentication flow")
        })?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>(0).to_string(),
        "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
        "alias": row.get::<_, String>(2),
        "description": row.get::<_, Option<String>>(3),
        "provider_id": row.get::<_, String>(4),
        "top_level": row.get::<_, bool>(5),
        "built_in": row.get::<_, bool>(6),
        "created_at": row.get::<_, chrono::DateTime<Utc>>(7).to_rfc3339(),
        "updated_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
    }))
}

pub async fn get_flow(db: &Database, flow_id: Uuid) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, realm_id, alias, description, provider_id,
               top_level, built_in, created_at, updated_at
        FROM authentication_flows
        WHERE id = $1
    "#;

    let row_opt = db.query_opt(query, &[&flow_id]).await.map_err(|e| {
        error!("Failed to get authentication flow: {}", e);
        AuthencError::database("Failed to get authentication flow")
    })?;

    Ok(row_opt.map(|row| {
        serde_json::json!({
            "id": row.get::<_, Uuid>(0).to_string(),
            "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
            "alias": row.get::<_, String>(2),
            "description": row.get::<_, Option<String>>(3),
            "provider_id": row.get::<_, String>(4),
            "top_level": row.get::<_, bool>(5),
            "built_in": row.get::<_, bool>(6),
            "created_at": row.get::<_, chrono::DateTime<Utc>>(7).to_rfc3339(),
            "updated_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
        })
    }))
}

pub async fn list_flows(
    db: &Database,
    realm_id: Option<Uuid>,
) -> Result<Vec<serde_json::Value>> {
    let query = if realm_id.is_some() {
        r#"
            SELECT id, realm_id, alias, description, provider_id,
                   top_level, built_in, created_at, updated_at
            FROM authentication_flows
            WHERE realm_id = $1
            ORDER BY alias
        "#
    } else {
        r#"
            SELECT id, realm_id, alias, description, provider_id,
                   top_level, built_in, created_at, updated_at
            FROM authentication_flows
            ORDER BY alias
        "#
    };

    let rows: Vec<tokio_postgres::Row> = if let Some(realm_id) = realm_id {
        db.query(query, &[&realm_id]).await
    } else {
        db.query(query, &[]).await
    }
    .map_err(|e| {
        error!("Failed to list authentication flows: {}", e);
        AuthencError::database("Failed to list authentication flows")
    })?;

    Ok(rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0).to_string(),
                "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
                "alias": row.get::<_, String>(2),
                "description": row.get::<_, Option<String>>(3),
                "provider_id": row.get::<_, String>(4),
                "top_level": row.get::<_, bool>(5),
                "built_in": row.get::<_, bool>(6),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(7).to_rfc3339(),
                "updated_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
            })
        })
        .collect())
}

pub async fn update_flow(
    db: &Database,
    flow_id: Uuid,
    flow: &serde_json::Value,
) -> Result<serde_json::Value> {
    let now = Utc::now();

    let query = r#"
        UPDATE authentication_flows
        SET alias = $2, description = $3, provider_id = $4,
            top_level = $5, updated_at = $6
        WHERE id = $1
        RETURNING id, realm_id, alias, description, provider_id,
                  top_level, built_in, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &flow_id,
                &flow.get("alias").and_then(|v| v.as_str()).unwrap_or(""),
                &flow.get("description").and_then(|v| v.as_str()),
                &flow
                    .get("provider_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                &flow
                    .get("top_level")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to update authentication flow: {}", e);
            AuthencError::database("Failed to update authentication flow")
        })?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>(0).to_string(),
        "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
        "alias": row.get::<_, String>(2),
        "description": row.get::<_, Option<String>>(3),
        "provider_id": row.get::<_, String>(4),
        "top_level": row.get::<_, bool>(5),
        "built_in": row.get::<_, bool>(6),
        "created_at": row.get::<_, chrono::DateTime<Utc>>(7).to_rfc3339(),
        "updated_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
    }))
}

pub async fn delete_flow(db: &Database, flow_id: Uuid) -> Result<()> {
    let query = "DELETE FROM authentication_flows WHERE id = $1";

    db.execute(query, &[&flow_id]).await.map_err(|e| {
        error!("Failed to delete authentication flow: {}", e);
        AuthencError::database("Failed to delete authentication flow")
    })?;

    Ok(())
}

pub async fn create_execution(
    db: &Database,
    execution: &serde_json::Value,
) -> Result<serde_json::Value> {
    let execution_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO authentication_executions (
            id, flow_id, authenticator, authenticator_config,
            authenticator_flow, requirement, priority, parent_flow,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, flow_id, authenticator, authenticator_config,
                  authenticator_flow, requirement, priority, parent_flow,
                  created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &execution_id,
                &execution
                    .get("flow_id")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &execution.get("authenticator").and_then(|v| v.as_str()),
                &execution
                    .get("authenticator_config")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &execution
                    .get("authenticator_flow")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                &execution
                    .get("requirement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("DISABLED"),
                &(execution
                    .get("priority")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32),
                &execution
                    .get("parent_flow")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create authentication execution: {}", e);
            AuthencError::database("Failed to create authentication execution")
        })?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>(0).to_string(),
        "flow_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
        "authenticator": row.get::<_, Option<String>>(2),
        "authenticator_config": row.get::<_, Option<Uuid>>(3).map(|u| u.to_string()),
        "authenticator_flow": row.get::<_, bool>(4),
        "requirement": row.get::<_, String>(5),
        "priority": row.get::<_, i32>(6),
        "parent_flow": row.get::<_, Option<Uuid>>(7).map(|u| u.to_string()),
        "created_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
        "updated_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
    }))
}

pub async fn create_session(
    db: &Database,
    session: &serde_json::Value,
) -> Result<serde_json::Value> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::hours(1); // Default 1 hour expiry

    let query = r#"
        INSERT INTO authentication_sessions (
            id, realm_id, user_id, client_id, flow_id, auth_state,
            protocol, redirect_uri, execution_status, authentication_notes,
            client_notes, required_actions, started_at, expires_at,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id, realm_id, user_id, client_id, flow_id, auth_state,
                  protocol, redirect_uri, started_at, expires_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &session_id,
                &session
                    .get("realm_id")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &session
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &session.get("client_id").and_then(|v| v.as_str()),
                &session
                    .get("flow_id")
                    .and_then(|v| v.as_str())
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                &session
                    .get("auth_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("STARTED"),
                &session
                    .get("protocol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("openid-connect"),
                &session.get("redirect_uri").and_then(|v| v.as_str()),
                &session
                    .get("execution_status")
                    .unwrap_or(&serde_json::json!({})),
                &session
                    .get("authentication_notes")
                    .unwrap_or(&serde_json::json!({})),
                &session
                    .get("client_notes")
                    .unwrap_or(&serde_json::json!({})),
                &session
                    .get("required_actions")
                    .unwrap_or(&serde_json::json!([])),
                &now,
                &expires_at,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create authentication session: {}", e);
            AuthencError::database("Failed to create authentication session")
        })?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>(0).to_string(),
        "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
        "user_id": row.get::<_, Option<Uuid>>(2).map(|u| u.to_string()),
        "client_id": row.get::<_, Option<String>>(3),
        "flow_id": row.get::<_, Option<Uuid>>(4).map(|u| u.to_string()),
        "auth_state": row.get::<_, String>(5),
        "protocol": row.get::<_, String>(6),
        "redirect_uri": row.get::<_, Option<String>>(7),
        "started_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
        "expires_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
    }))
}

pub async fn get_session(db: &Database, session_id: Uuid) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, realm_id, user_id, client_id, flow_id, auth_state,
               protocol, redirect_uri, execution_status, authentication_notes,
               client_notes, required_actions, started_at, completed_at,
               expires_at, success, error_message
        FROM authentication_sessions
        WHERE id = $1
    "#;

    let row_opt = db.query_opt(query, &[&session_id]).await.map_err(|e| {
        error!("Failed to get authentication session: {}", e);
        AuthencError::database("Failed to get authentication session")
    })?;

    Ok(row_opt.map(|row| {
        serde_json::json!({
            "id": row.get::<_, Uuid>(0).to_string(),
            "realm_id": row.get::<_, Option<Uuid>>(1).map(|u| u.to_string()),
            "user_id": row.get::<_, Option<Uuid>>(2).map(|u| u.to_string()),
            "client_id": row.get::<_, Option<String>>(3),
            "flow_id": row.get::<_, Option<Uuid>>(4).map(|u| u.to_string()),
            "auth_state": row.get::<_, String>(5),
            "protocol": row.get::<_, String>(6),
            "redirect_uri": row.get::<_, Option<String>>(7),
            "execution_status": row.get::<_, serde_json::Value>(8),
            "authentication_notes": row.get::<_, serde_json::Value>(9),
            "client_notes": row.get::<_, serde_json::Value>(10),
            "required_actions": row.get::<_, serde_json::Value>(11),
            "started_at": row.get::<_, chrono::DateTime<Utc>>(12).to_rfc3339(),
            "completed_at": row.get::<_, Option<chrono::DateTime<Utc>>>(13).map(|dt| dt.to_rfc3339()),
            "expires_at": row.get::<_, chrono::DateTime<Utc>>(14).to_rfc3339(),
            "success": row.get::<_, Option<bool>>(15),
            "error_message": row.get::<_, Option<String>>(16),
        })
    }))
}

pub async fn update_session(
    db: &Database,
    session_id: Uuid,
    session: &serde_json::Value,
) -> Result<()> {
    let now = Utc::now();

    let query = r#"
        UPDATE authentication_sessions
        SET auth_state = $2, execution_status = $3, authentication_notes = $4,
            client_notes = $5, required_actions = $6, updated_at = $7
        WHERE id = $1
    "#;

    db.execute(
        query,
        &[
            &session_id,
            &session
                .get("auth_state")
                .and_then(|v| v.as_str())
                .unwrap_or("IN_PROGRESS"),
            &session
                .get("execution_status")
                .unwrap_or(&serde_json::json!({})),
            &session
                .get("authentication_notes")
                .unwrap_or(&serde_json::json!({})),
            &session
                .get("client_notes")
                .unwrap_or(&serde_json::json!({})),
            &session
                .get("required_actions")
                .unwrap_or(&serde_json::json!([])),
            &now,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to update authentication session: {}", e);
        AuthencError::database("Failed to update authentication session")
    })?;

    Ok(())
}

pub async fn complete_session(
    db: &Database,
    session_id: Uuid,
    success: bool,
    error_message: Option<String>,
) -> Result<()> {
    let now = Utc::now();
    let auth_state = if success { "COMPLETED" } else { "FAILED" };

    let query = r#"
        UPDATE authentication_sessions
        SET auth_state = $2, completed_at = $3, success = $4,
            error_message = $5, updated_at = $6
        WHERE id = $1
    "#;

    db.execute(
        query,
        &[
            &session_id,
            &auth_state,
            &now,
            &success,
            &error_message,
            &now,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to complete authentication session: {}", e);
        AuthencError::database("Failed to complete authentication session")
    })?;

    Ok(())
}

pub async fn cleanup_expired_sessions(db: &Database) -> Result<i64> {
    let query = "DELETE FROM authentication_sessions WHERE expires_at < NOW()";

    let rows_affected = db.execute(query, &[]).await.map_err(|e| {
        error!("Failed to cleanup expired sessions: {}", e);
        AuthencError::database("Failed to cleanup expired sessions")
    })?;

    Ok(rows_affected as i64)
}
