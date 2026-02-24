/// OAuth2 Social Provider operations (for authentication via external providers)
use super::*;
use sha2::{Digest, Sha256};

pub async fn create_provider_config(
    db: &Database,
    realm_id: Uuid,
    provider_name: &str,
    alias: &str,
    display_name: Option<&str>,
    authorization_url: &str,
    token_url: &str,
    user_info_url: Option<&str>,
    client_id: &str,
    client_secret: &str,
    scopes: &str,
) -> Result<serde_json::Value> {
    let query = r#"
        INSERT INTO oauth2_provider_configs (
            realm_id, provider_name, alias, display_name,
            authorization_url, token_url, user_info_url,
            client_id, client_secret, scopes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, realm_id, provider_name, alias, enabled, created_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &realm_id,
                &provider_name,
                &alias,
                &display_name,
                &authorization_url,
                &token_url,
                &user_info_url,
                &client_id,
                &client_secret,
                &scopes,
            ],
        )
        .await?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "provider_name": row.get::<_, String>("provider_name"),
        "alias": row.get::<_, String>("alias"),
        "enabled": row.get::<_, bool>("enabled"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at")
    }))
}

pub async fn get_provider_config(
    db: &Database,
    config_id: Uuid,
) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, realm_id, provider_name, alias, display_name,
               authorization_url, token_url, user_info_url, jwks_url, issuer,
               client_id, client_secret, scopes, response_type, response_mode,
               pkce_enabled, pkce_method, trust_email, link_only, store_tokens,
               enabled, created_at, updated_at
        FROM oauth2_provider_configs
        WHERE id = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&config_id]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "provider_name": row.get::<_, String>("provider_name"),
        "alias": row.get::<_, String>("alias"),
        "display_name": row.get::<_, Option<String>>("display_name"),
        "authorization_url": row.get::<_, String>("authorization_url"),
        "token_url": row.get::<_, String>("token_url"),
        "user_info_url": row.get::<_, Option<String>>("user_info_url"),
        "jwks_url": row.get::<_, Option<String>>("jwks_url"),
        "issuer": row.get::<_, Option<String>>("issuer"),
        "client_id": row.get::<_, String>("client_id"),
        "client_secret": row.get::<_, String>("client_secret"),
        "scopes": row.get::<_, String>("scopes"),
        "response_type": row.get::<_, String>("response_type"),
        "response_mode": row.get::<_, String>("response_mode"),
        "pkce_enabled": row.get::<_, bool>("pkce_enabled"),
        "pkce_method": row.get::<_, String>("pkce_method"),
        "trust_email": row.get::<_, bool>("trust_email"),
        "link_only": row.get::<_, bool>("link_only"),
        "store_tokens": row.get::<_, bool>("store_tokens"),
        "enabled": row.get::<_, bool>("enabled"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    })))
}

pub async fn get_realm_provider_configs(
    db: &Database,
    realm_id: Uuid,
    enabled_only: bool,
) -> Result<Vec<serde_json::Value>> {
    let query = if enabled_only {
        r#"
            SELECT id, realm_id, provider_name, alias, display_name,
                   authorization_url, scopes, enabled
            FROM oauth2_provider_configs
            WHERE realm_id = $1 AND enabled = TRUE
            ORDER BY provider_name ASC
        "#
    } else {
        r#"
            SELECT id, realm_id, provider_name, alias, display_name,
                   authorization_url, scopes, enabled
            FROM oauth2_provider_configs
            WHERE realm_id = $1
            ORDER BY provider_name ASC
        "#
    };

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id]).await?;

    let mut configs = Vec::new();
    for row in rows {
        configs.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "provider_name": row.get::<_, String>("provider_name"),
            "alias": row.get::<_, String>("alias"),
            "display_name": row.get::<_, Option<String>>("display_name"),
            "authorization_url": row.get::<_, String>("authorization_url"),
            "scopes": row.get::<_, String>("scopes"),
            "enabled": row.get::<_, bool>("enabled")
        }));
    }

    Ok(configs)
}

pub async fn create_oauth2_state(
    db: &Database,
    state_token: &str,
    provider_config_id: Uuid,
    realm_id: Uuid,
    redirect_uri: &str,
    code_verifier: Option<&str>,
    code_challenge: Option<&str>,
    expires_in_seconds: i64,
) -> Result<Uuid> {
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in_seconds);

    let query = r#"
        INSERT INTO oauth2_states (
            state_token, provider_config_id, realm_id, redirect_uri,
            code_verifier, code_challenge, code_challenge_method, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
    "#;

    let code_challenge_method = if code_challenge.is_some() {
        Some("S256")
    } else {
        None
    };

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &state_token,
                &provider_config_id,
                &realm_id,
                &redirect_uri,
                &code_verifier,
                &code_challenge,
                &code_challenge_method,
                &expires_at,
            ],
        )
        .await?;

    Ok(row.get(0))
}

pub async fn validate_oauth2_state(
    db: &Database,
    state_token: &str,
) -> Result<Option<serde_json::Value>> {
    let query = r#"
        SELECT id, state_token, provider_config_id, realm_id, redirect_uri,
               code_verifier, expires_at, used
        FROM oauth2_states
        WHERE state_token = $1 AND expires_at > NOW() AND used = FALSE
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&state_token]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];

    // Mark as used
    let update_query = r#"
        UPDATE oauth2_states
        SET used = TRUE, used_at = NOW()
        WHERE id = $1
    "#;

    db.execute(update_query, &[&row.get::<_, Uuid>("id")])
        .await?;

    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "provider_config_id": row.get::<_, Uuid>("provider_config_id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "redirect_uri": row.get::<_, String>("redirect_uri"),
        "code_verifier": row.get::<_, Option<String>>("code_verifier"),
    })))
}

pub async fn record_token_exchange(
    db: &Database,
    provider_config_id: Uuid,
    user_id: Option<Uuid>,
    authorization_code: Option<&str>,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
    expires_in: Option<i32>,
    scope: Option<&str>,
    provider_user_id: Option<&str>,
    provider_email: Option<&str>,
    user_info_raw: Option<serde_json::Value>,
    success: bool,
    error_message: Option<&str>,
) -> Result<()> {
    let access_token_hash = access_token.map(|t| {
        let mut hasher = Sha256::new();
        hasher.update(t.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    let refresh_token_hash = refresh_token.map(|t| {
        let mut hasher = Sha256::new();
        hasher.update(t.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    let query = r#"
        INSERT INTO oauth2_token_exchanges (
            provider_config_id, user_id, authorization_code,
            access_token_hash, refresh_token_hash, expires_in, scope,
            provider_user_id, provider_email, user_info_raw,
            success, error_message
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    "#;

    db.execute(
        query,
        &[
            &provider_config_id,
            &user_id,
            &authorization_code,
            &access_token_hash,
            &refresh_token_hash,
            &expires_in,
            &scope,
            &provider_user_id,
            &provider_email,
            &user_info_raw,
            &success,
            &error_message,
        ],
    )
    .await?;

    Ok(())
}

pub async fn cleanup_expired_states(db: &Database) -> Result<i64> {
    let query = "DELETE FROM oauth2_states WHERE expires_at < NOW()";
    let count = db.execute(query, &[]).await?;
    Ok(count as i64)
}

pub async fn get_user_token_exchanges(
    db: &Database,
    user_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    let query = r#"
        SELECT ote.id, ote.provider_config_id, opc.provider_name,
               ote.provider_user_id, ote.provider_email, ote.scope,
               ote.success, ote.error_message, ote.created_at
        FROM oauth2_token_exchanges ote
        JOIN oauth2_provider_configs opc ON ote.provider_config_id = opc.id
        WHERE ote.user_id = $1
        ORDER BY ote.created_at DESC
        LIMIT $2
    "#;

    let limit_val = limit.unwrap_or(50);
    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id, &limit_val]).await?;

    let mut exchanges = Vec::new();
    for row in rows {
        exchanges.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "provider_config_id": row.get::<_, Uuid>("provider_config_id"),
            "provider_name": row.get::<_, String>("provider_name"),
            "provider_user_id": row.get::<_, Option<String>>("provider_user_id"),
            "provider_email": row.get::<_, Option<String>>("provider_email"),
            "scope": row.get::<_, Option<String>>("scope"),
            "success": row.get::<_, bool>("success"),
            "error_message": row.get::<_, Option<String>>("error_message"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at")
        }));
    }

    Ok(exchanges)
}
