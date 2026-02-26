/// Federated Identity Management operations (for identity brokering and linking)
use crate::Database;
use authenc_types::Result;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub async fn link_federated_identity(
    db: &Database,
    user_id: Uuid,
    realm_id: Uuid,
    identity_provider_alias: &str,
    federated_user_id: &str,
    federated_username: Option<&str>,
    token: Option<&str>,
    token_expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<&str>,
    federated_attributes: Option<&JsonValue>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO federated_identity_links (
            user_id, realm_id, identity_provider_alias, federated_user_id,
            federated_username, token, token_expires_at, refresh_token,
            federated_attributes, last_authenticated_at, authentication_count
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), 1)
        ON CONFLICT (user_id, identity_provider_alias)
        DO UPDATE SET
            federated_user_id = EXCLUDED.federated_user_id,
            federated_username = EXCLUDED.federated_username,
            token = EXCLUDED.token,
            token_expires_at = EXCLUDED.token_expires_at,
            refresh_token = EXCLUDED.refresh_token,
            federated_attributes = EXCLUDED.federated_attributes,
            last_authenticated_at = NOW(),
            authentication_count = federated_identity_links.authentication_count + 1,
            updated_at = NOW()
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &user_id,
                &realm_id,
                &identity_provider_alias,
                &federated_user_id,
                &federated_username,
                &token,
                &token_expires_at,
                &refresh_token,
                &federated_attributes,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn unlink_federated_identity(
    db: &Database,
    user_id: Uuid,
    identity_provider_alias: &str,
) -> Result<bool> {
    let query = r#"
        DELETE FROM federated_identity_links
        WHERE user_id = $1 AND identity_provider_alias = $2
    "#;

    let rows_affected = db
        .execute(query, &[&user_id, &identity_provider_alias])
        .await?;
    Ok(rows_affected > 0)
}

pub async fn get_user_federated_identities(db: &Database, user_id: Uuid) -> Result<Vec<JsonValue>> {
    let query = r#"
        SELECT
            id, identity_provider_alias, federated_user_id, federated_username,
            federated_attributes, linked_at, last_authenticated_at,
            authentication_count, token_expires_at
        FROM federated_identity_links
        WHERE user_id = $1
        ORDER BY last_authenticated_at DESC
    "#;

    let rows = db.query(query, &[&user_id]).await?;
    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "identity_provider_alias": row.get::<_, String>(1),
                "federated_user_id": row.get::<_, String>(2),
                "federated_username": row.get::<_, Option<String>>(3),
                "federated_attributes": row.get::<_, Option<JsonValue>>(4),
                "linked_at": row.get::<_, DateTime<Utc>>(5),
                "last_authenticated_at": row.get::<_, Option<DateTime<Utc>>>(6),
                "authentication_count": row.get::<_, i32>(7),
                "token_expires_at": row.get::<_, Option<DateTime<Utc>>>(8),
            })
        })
        .collect())
}

pub async fn find_user_by_federated_identity(
    db: &Database,
    identity_provider_alias: &str,
    federated_user_id: &str,
) -> Result<Option<Uuid>> {
    let query = r#"
        SELECT user_id FROM federated_identity_links
        WHERE identity_provider_alias = $1 AND federated_user_id = $2
    "#;

    match db
        .query_opt(query, &[&identity_provider_alias, &federated_user_id])
        .await?
    {
        Some(row) => Ok(Some(row.get(0))),
        None => Ok(None),
    }
}

pub async fn update_federated_tokens(
    db: &Database,
    user_id: Uuid,
    identity_provider_alias: &str,
    token: &str,
    token_expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<&str>,
) -> Result<bool> {
    let query = r#"
        UPDATE federated_identity_links
        SET token = $1, token_expires_at = $2, refresh_token = $3,
            last_authenticated_at = NOW(), authentication_count = authentication_count + 1,
            updated_at = NOW()
        WHERE user_id = $4 AND identity_provider_alias = $5
    "#;

    let rows_affected = db
        .execute(
            query,
            &[
                &token,
                &token_expires_at,
                &refresh_token,
                &user_id,
                &identity_provider_alias,
            ],
        )
        .await?;

    Ok(rows_affected > 0)
}

pub async fn create_identity_provider_mapper(
    db: &Database,
    realm_id: Uuid,
    name: &str,
    identity_provider_alias: &str,
    mapper_type: &str,
    config: &JsonValue,
    sync_mode: &str,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO identity_provider_mappers (
            realm_id, name, identity_provider_alias, mapper_type, config, sync_mode
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &realm_id,
                &name,
                &identity_provider_alias,
                &mapper_type,
                &config,
                &sync_mode,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_identity_provider_mappers(
    db: &Database,
    realm_id: Uuid,
    identity_provider_alias: Option<&str>,
) -> Result<Vec<JsonValue>> {
    let query = if identity_provider_alias.is_some() {
        r#"
            SELECT id, name, identity_provider_alias, mapper_type, config, sync_mode, created_at
            FROM identity_provider_mappers
            WHERE realm_id = $1 AND identity_provider_alias = $2
            ORDER BY name
        "#
    } else {
        r#"
            SELECT id, name, identity_provider_alias, mapper_type, config, sync_mode, created_at
            FROM identity_provider_mappers
            WHERE realm_id = $1
            ORDER BY identity_provider_alias, name
        "#
    };

    let rows = if let Some(alias) = identity_provider_alias {
        db.query(query, &[&realm_id, &alias]).await?
    } else {
        db.query(query, &[&realm_id]).await?
    };

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "name": row.get::<_, String>(1),
                "identity_provider_alias": row.get::<_, String>(2),
                "mapper_type": row.get::<_, String>(3),
                "config": row.get::<_, JsonValue>(4),
                "sync_mode": row.get::<_, String>(5),
                "created_at": row.get::<_, DateTime<Utc>>(6),
            })
        })
        .collect())
}

pub async fn create_identity_broker_config(
    db: &Database,
    realm_id: Uuid,
    alias: &str,
    display_name: Option<&str>,
    provider_type: &str,
    first_broker_login_flow: Option<&str>,
    post_broker_login_flow: Option<&str>,
    trust_email: bool,
    store_token: bool,
    link_only: bool,
    config: &JsonValue,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO identity_broker_configs (
            realm_id, alias, display_name, provider_type,
            first_broker_login_flow, post_broker_login_flow,
            trust_email, store_token, link_only, config
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &realm_id,
                &alias,
                &display_name,
                &provider_type,
                &first_broker_login_flow,
                &post_broker_login_flow,
                &trust_email,
                &store_token,
                &link_only,
                &config,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_identity_broker_config(
    db: &Database,
    realm_id: Uuid,
    alias: &str,
) -> Result<Option<JsonValue>> {
    let query = r#"
        SELECT
            id, alias, display_name, enabled, provider_type,
            first_broker_login_flow, post_broker_login_flow,
            trust_email, store_token, add_read_token_role_on_create,
            link_only, config, created_at, updated_at
        FROM identity_broker_configs
        WHERE realm_id = $1 AND alias = $2
    "#;

    match db.query_opt(query, &[&realm_id, &alias]).await? {
        Some(row) => Ok(Some(serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "alias": row.get::<_, String>(1),
            "display_name": row.get::<_, Option<String>>(2),
            "enabled": row.get::<_, bool>(3),
            "provider_type": row.get::<_, String>(4),
            "first_broker_login_flow": row.get::<_, Option<String>>(5),
            "post_broker_login_flow": row.get::<_, Option<String>>(6),
            "trust_email": row.get::<_, bool>(7),
            "store_token": row.get::<_, bool>(8),
            "add_read_token_role_on_create": row.get::<_, bool>(9),
            "link_only": row.get::<_, bool>(10),
            "config": row.get::<_, JsonValue>(11),
            "created_at": row.get::<_, DateTime<Utc>>(12),
            "updated_at": row.get::<_, DateTime<Utc>>(13),
        }))),
        None => Ok(None),
    }
}

pub async fn get_realm_identity_broker_configs(
    db: &Database,
    realm_id: Uuid,
    enabled_only: bool,
) -> Result<Vec<JsonValue>> {
    let query = if enabled_only {
        r#"
            SELECT
                id, alias, display_name, enabled, provider_type,
                trust_email, store_token, link_only, created_at
            FROM identity_broker_configs
            WHERE realm_id = $1 AND enabled = TRUE
            ORDER BY alias
        "#
    } else {
        r#"
            SELECT
                id, alias, display_name, enabled, provider_type,
                trust_email, store_token, link_only, created_at
            FROM identity_broker_configs
            WHERE realm_id = $1
            ORDER BY alias
        "#
    };

    let rows = db.query(query, &[&realm_id]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "alias": row.get::<_, String>(1),
                "display_name": row.get::<_, Option<String>>(2),
                "enabled": row.get::<_, bool>(3),
                "provider_type": row.get::<_, String>(4),
                "trust_email": row.get::<_, bool>(5),
                "store_token": row.get::<_, bool>(6),
                "link_only": row.get::<_, bool>(7),
                "created_at": row.get::<_, DateTime<Utc>>(8),
            })
        })
        .collect())
}

pub async fn log_federated_authentication(
    db: &Database,
    user_id: Option<Uuid>,
    realm_id: Uuid,
    identity_provider_alias: &str,
    federated_user_id: Option<&str>,
    success: bool,
    error_code: Option<&str>,
    error_message: Option<&str>,
    action: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    session_id: Option<Uuid>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO federated_auth_log (
            user_id, realm_id, identity_provider_alias, federated_user_id,
            success, error_code, error_message, action,
            ip_address, user_agent, session_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
    "#;

    let ip_parsed = ip_address.and_then(|ip| ip.parse::<std::net::IpAddr>().ok());

    let rows = db
        .query(
            query,
            &[
                &user_id,
                &realm_id,
                &identity_provider_alias,
                &federated_user_id,
                &success,
                &error_code,
                &error_message,
                &action,
                &ip_parsed,
                &user_agent,
                &session_id,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn create_account_linking_request(
    db: &Database,
    user_id: Uuid,
    realm_id: Uuid,
    identity_provider_alias: &str,
    federated_user_id: &str,
    federated_username: Option<&str>,
    federated_email: Option<&str>,
    federated_attributes: Option<&JsonValue>,
    confirmation_token: &str,
    expires_in_seconds: i64,
) -> Result<Uuid> {
    let expires_at = Utc::now() + Duration::seconds(expires_in_seconds);

    let query = r#"
        INSERT INTO account_linking_requests (
            user_id, realm_id, identity_provider_alias, federated_user_id,
            federated_username, federated_email, federated_attributes,
            confirmation_token, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &user_id,
                &realm_id,
                &identity_provider_alias,
                &federated_user_id,
                &federated_username,
                &federated_email,
                &federated_attributes,
                &confirmation_token,
                &expires_at,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn confirm_account_linking_request(
    db: &Database,
    confirmation_token: &str,
    resolved_by: &str,
) -> Result<Option<(Uuid, Uuid, String, String)>> {
    // First, get the request details
    let query = r#"
        SELECT id, user_id, realm_id, identity_provider_alias, federated_user_id, expires_at, status
        FROM account_linking_requests
        WHERE confirmation_token = $1
    "#;

    match db.query_opt(query, &[&confirmation_token]).await? {
        Some(row) => {
            let request_id: Uuid = row.get(0);
            let user_id: Uuid = row.get(1);
            let realm_id: Uuid = row.get(2);
            let identity_provider_alias: String = row.get(3);
            let federated_user_id: String = row.get(4);
            let expires_at: DateTime<Utc> = row.get(5);
            let status: String = row.get(6);

            // Check if expired or already resolved
            if status != "PENDING" {
                return Ok(None);
            }

            if Utc::now() > expires_at {
                // Update status to expired
                let update_query = r#"
                    UPDATE account_linking_requests
                    SET status = 'EXPIRED', resolved_at = NOW(), resolved_by = $1
                    WHERE id = $2
                "#;
                db.execute(update_query, &[&"SYSTEM", &request_id]).await?;
                return Ok(None);
            }

            // Update status to approved
            let update_query = r#"
                UPDATE account_linking_requests
                SET status = 'APPROVED', resolved_at = NOW(), resolved_by = $1
                WHERE id = $2
            "#;
            db.execute(update_query, &[&resolved_by, &request_id])
                .await?;

            Ok(Some((
                user_id,
                realm_id,
                identity_provider_alias,
                federated_user_id,
            )))
        }
        None => Ok(None),
    }
}

pub async fn reject_account_linking_request(
    db: &Database,
    confirmation_token: &str,
    resolved_by: &str,
) -> Result<bool> {
    let query = r#"
        UPDATE account_linking_requests
        SET status = 'REJECTED', resolved_at = NOW(), resolved_by = $1
        WHERE confirmation_token = $2 AND status = 'PENDING'
    "#;

    let rows_affected = db
        .execute(query, &[&resolved_by, &confirmation_token])
        .await?;
    Ok(rows_affected > 0)
}

pub async fn cleanup_expired_linking_requests(db: &Database) -> Result<u64> {
    let query = r#"
        UPDATE account_linking_requests
        SET status = 'EXPIRED', resolved_at = NOW(), resolved_by = 'SYSTEM'
        WHERE status = 'PENDING' AND expires_at < NOW()
    "#;

    db.execute(query, &[]).await
}
