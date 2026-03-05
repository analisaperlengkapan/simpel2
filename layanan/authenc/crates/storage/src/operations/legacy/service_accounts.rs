/// Database operations for service accounts
use crate::Database;
use authenc_types::domain::service_account::ServiceAccount;
use authenc_types::{AuthencError, Result};
use chrono::{DateTime, Utc};
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn create_service_account(
    db: &Database,
    realm_id: Uuid,
    name: &str,
    description: Option<&str>,
    client_id: &str,
    client_secret_hash: &str,
    enabled: bool,
    roles: Vec<Uuid>,
) -> Result<ServiceAccount> {
    let service_account_id = Uuid::new_v4();
    let now = Utc::now();

    // Insert service account
    let query = r#"
        INSERT INTO service_accounts (
            id, name, description, client_id, client_secret_hash,
            realm_id, enabled, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, name, description, client_id, client_secret_hash,
                  realm_id, enabled, created_at, updated_at, last_used_at, attributes
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &service_account_id,
                &name,
                &description,
                &client_id,
                &client_secret_hash,
                &realm_id,
                &enabled,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create service account: {}", e);
            AuthencError::database(format!("Failed to create service account: {}", e))
        })?;

    let mut service_account = ServiceAccount {
        id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        client_id: row.get(3),
        client_secret_hash: row.get(4),
        realm_id: row.get(5),
        enabled: row.get(6),
        roles: Vec::new(),
        created_at: row.get(7),
        updated_at: row.get(8),
        last_used_at: row.get(9),
        attributes: row.get(10),
    };

    // Assign roles if provided
    if !roles.is_empty() {
        for role_id in &roles {
            assign_role(db, service_account_id, *role_id, None).await?;
        }
        service_account.roles = roles;
    }

    info!(
        "Created service account: {} ({})",
        service_account.name, service_account.id
    );

    Ok(service_account)
}

pub async fn get_service_account_by_id(
    db: &Database,
    service_account_id: Uuid,
) -> Result<Option<ServiceAccount>> {
    let query = r#"
        SELECT id, name, description, client_id, client_secret_hash,
               realm_id, enabled, created_at, updated_at, last_used_at, attributes
        FROM service_accounts
        WHERE id = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&service_account_id]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];

    // Get assigned roles
    let roles = get_service_account_roles(db, service_account_id).await?;

    Ok(Some(ServiceAccount {
        id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        client_id: row.get(3),
        client_secret_hash: row.get(4),
        realm_id: row.get(5),
        enabled: row.get(6),
        roles,
        created_at: row.get(7),
        updated_at: row.get(8),
        last_used_at: row.get(9),
        attributes: row.get(10),
    }))
}

pub async fn get_service_account_by_client_id(
    db: &Database,
    client_id: &str,
) -> Result<Option<ServiceAccount>> {
    let query = r#"
        SELECT id, name, description, client_id, client_secret_hash,
               realm_id, enabled, created_at, updated_at, last_used_at, attributes
        FROM service_accounts
        WHERE client_id = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_id]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    let service_account_id: Uuid = row.get(0);

    // Get assigned roles
    let roles = get_service_account_roles(db, service_account_id).await?;

    Ok(Some(ServiceAccount {
        id: service_account_id,
        name: row.get(1),
        description: row.get(2),
        client_id: row.get(3),
        client_secret_hash: row.get(4),
        realm_id: row.get(5),
        enabled: row.get(6),
        roles,
        created_at: row.get(7),
        updated_at: row.get(8),
        last_used_at: row.get(9),
        attributes: row.get(10),
    }))
}

pub async fn get_service_accounts_by_realm(
    db: &Database,
    realm_id: Uuid,
    first: Option<i64>,
    max: Option<i64>,
) -> Result<Vec<ServiceAccount>> {
    let query = r#"
        SELECT id, name, description, client_id, client_secret_hash,
               realm_id, enabled, created_at, updated_at, last_used_at, attributes
        FROM service_accounts
        WHERE realm_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let limit = max.unwrap_or(100).min(1000);
    let offset = first.unwrap_or(0);

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id, &limit, &offset]).await?;

    let mut service_accounts = Vec::new();

    for row in rows {
        let service_account_id: Uuid = row.get(0);
        let roles = get_service_account_roles(db, service_account_id).await?;

        service_accounts.push(ServiceAccount {
            id: service_account_id,
            name: row.get(1),
            description: row.get(2),
            client_id: row.get(3),
            client_secret_hash: row.get(4),
            realm_id: row.get(5),
            enabled: row.get(6),
            roles,
            created_at: row.get(7),
            updated_at: row.get(8),
            last_used_at: row.get(9),
            attributes: row.get(10),
        });
    }

    Ok(service_accounts)
}

pub async fn update_service_account(
    db: &Database,
    service_account_id: Uuid,
    name: Option<&str>,
    description: Option<Option<&str>>,
    enabled: Option<bool>,
) -> Result<ServiceAccount> {
    let now = Utc::now();

    // Build dynamic query based on what's being updated
    let has_name = name.is_some();
    let has_desc = description.is_some();
    let has_enabled = enabled.is_some();

    if !has_name && !has_desc && !has_enabled {
        // No updates, just return current service account
        return get_service_account_by_id(db, service_account_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Service account not found"));
    }

    let query = r#"
        UPDATE service_accounts
        SET
            name = COALESCE($2, name),
            description = CASE WHEN $3::boolean THEN $4 ELSE description END,
            enabled = COALESCE($5, enabled),
            updated_at = $6
        WHERE id = $1
        RETURNING id, name, description, client_id, client_secret_hash,
                  realm_id, enabled, created_at, updated_at, last_used_at, attributes
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &service_account_id,
                &name,
                &has_desc,
                &description.flatten(),
                &enabled,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Service account update failed: {}", e);
            AuthencError::database(format!("Failed to update service account: {}", e))
        })?;

    let roles = get_service_account_roles(db, service_account_id).await?;

    Ok(ServiceAccount {
        id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        client_id: row.get(3),
        client_secret_hash: row.get(4),
        realm_id: row.get(5),
        enabled: row.get(6),
        roles,
        created_at: row.get(7),
        updated_at: row.get(8),
        last_used_at: row.get(9),
        attributes: row.get(10),
    })
}

pub async fn update_service_account_secret(
    db: &Database,
    service_account_id: Uuid,
    new_secret_hash: &str,
) -> Result<()> {
    let now = Utc::now();

    let query = r#"
        UPDATE service_accounts
        SET client_secret_hash = $2, updated_at = $3
        WHERE id = $1
    "#;

    db.execute(query, &[&service_account_id, &new_secret_hash, &now])
        .await
        .map_err(|e| {
            error!("Failed to update service account secret: {}", e);
            AuthencError::database(format!("Failed to update secret: {}", e))
        })?;

    info!(
        "Regenerated client secret for service account: {}",
        service_account_id
    );

    Ok(())
}

pub async fn update_last_used(
    db: &Database,
    service_account_id: Uuid,
    last_used_at: DateTime<Utc>,
) -> Result<()> {
    let query = r#"
        UPDATE service_accounts
        SET last_used_at = $2
        WHERE id = $1
    "#;

    db.execute(query, &[&service_account_id, &last_used_at])
        .await?;

    Ok(())
}

pub async fn delete_service_account(db: &Database, service_account_id: Uuid) -> Result<()> {
    let query = "DELETE FROM service_accounts WHERE id = $1";

    let rows_affected = db
        .execute(query, &[&service_account_id])
        .await
        .map_err(|e| {
            error!("Service account deletion failed: {}", e);
            AuthencError::database(format!("Failed to delete service account: {}", e))
        })?;

    if rows_affected == 0 {
        return Err(AuthencError::not_found("Service account not found"));
    }

    info!("Deleted service account: {}", service_account_id);

    Ok(())
}

pub async fn assign_role(
    db: &Database,
    service_account_id: Uuid,
    role_id: Uuid,
    granted_by: Option<Uuid>,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO service_account_roles (id, service_account_id, role_id, granted_at, granted_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (service_account_id, role_id) DO NOTHING
    "#;

    db.execute(
        query,
        &[&id, &service_account_id, &role_id, &now, &granted_by],
    )
    .await
    .map_err(|e| {
        error!("Failed to assign role to service account: {}", e);
        AuthencError::database(format!("Failed to assign role: {}", e))
    })?;

    Ok(())
}

pub async fn revoke_role(db: &Database, service_account_id: Uuid, role_id: Uuid) -> Result<()> {
    let query = "DELETE FROM service_account_roles WHERE service_account_id = $1 AND role_id = $2";

    let rows_affected = db
        .execute(query, &[&service_account_id, &role_id])
        .await
        .map_err(|e| {
            error!("Failed to revoke role from service account: {}", e);
            AuthencError::database(format!("Failed to revoke role: {}", e))
        })?;

    if rows_affected == 0 {
        warn!(
            "Attempted to revoke role {} from service account {} but assignment didn't exist",
            role_id, service_account_id
        );
    }

    Ok(())
}

pub async fn get_service_account_roles(
    db: &Database,
    service_account_id: Uuid,
) -> Result<Vec<Uuid>> {
    let query = r#"
        SELECT role_id
        FROM service_account_roles
        WHERE service_account_id = $1
        ORDER BY granted_at
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&service_account_id]).await?;

    Ok(rows.iter().map(|row| row.get(0)).collect())
}

pub async fn count_service_accounts(db: &Database, realm_id: Uuid) -> Result<i64> {
    let query = "SELECT COUNT(*)::bigint FROM service_accounts WHERE realm_id = $1";

    let row: tokio_postgres::Row = db.query_one(query, &[&realm_id]).await.map_err(|e| {
        error!("Failed to count service accounts: {}", e);
        AuthencError::database(format!("Failed to count service accounts: {}", e))
    })?;

    Ok(row.get(0))
}

pub async fn log_auth_attempt(
    db: &Database,
    service_account_id: Uuid,
    success: bool,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    error_message: Option<&str>,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let event_type = if success {
        "authenticated"
    } else {
        "auth_failed"
    };

    let query = r#"
        INSERT INTO service_account_audit_log (
            id, service_account_id, event_type, success,
            ip_address, user_agent, error_message, timestamp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#;

    let ip_parsed = ip_address.and_then(|ip| ip.parse::<std::net::IpAddr>().ok());

    db.execute(
        query,
        &[
            &id,
            &service_account_id,
            &event_type,
            &success,
            &ip_parsed,
            &user_agent,
            &error_message,
            &now,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to log service account auth attempt: {}", e);
        AuthencError::database(format!("Failed to log auth attempt: {}", e))
    })?;

    Ok(())
}

pub async fn get_audit_log(
    db: &Database,
    service_account_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    let query = r#"
        SELECT id, event_type, event_details, performed_by,
               ip_address, user_agent, success, error_message, timestamp
        FROM service_account_audit_log
        WHERE service_account_id = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
    "#;

    let limit_val = limit.unwrap_or(100).min(1000);
    let offset_val = offset.unwrap_or(0);

    let rows: Vec<tokio_postgres::Row> = db
        .query(query, &[&service_account_id, &limit_val, &offset_val])
        .await?;

    let mut logs = Vec::new();

    for row in rows {
        let ip: Option<std::net::IpAddr> = row.get(4);
        logs.push(serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "event_type": row.get::<_, String>(1),
            "event_details": row.get::<_, serde_json::Value>(2),
            "performed_by": row.get::<_, Option<Uuid>>(3),
            "ip_address": ip.map(|i| i.to_string()),
            "user_agent": row.get::<_, Option<String>>(5),
            "success": row.get::<_, bool>(6),
            "error_message": row.get::<_, Option<String>>(7),
            "timestamp": row.get::<_, DateTime<Utc>>(8),
        }));
    }

    Ok(logs)
}
