/// Database operations for OAuth2
use crate::{
    database::Database,
    error::Result,
    models::{OAuth2AccessToken, OAuth2AuthorizationCode, OAuth2Client},
};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_client(db: &Database, client: &OAuth2Client) -> Result<OAuth2Client> {
    let client_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO oauth2_clients (
            id, client_id, client_secret_hash, client_name, client_type,
            redirect_uris, scopes, grant_types, response_types,
            token_endpoint_auth_method, owner_id, realm_id,
            enabled, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING
            id, client_id, client_secret_hash, client_name, client_type,
            redirect_uris, scopes, grant_types, response_types,
            token_endpoint_auth_method, owner_id, realm_id,
            enabled, created_at, updated_at, deleted_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &client_id,
                &client.client_id,
                &client.client_secret_hash,
                &client.client_name,
                &client.client_type,
                &client.redirect_uris,
                &client.scopes,
                &client.grant_types,
                &client.response_types,
                &client.token_endpoint_auth_method,
                &client.owner_id,
                &client.realm_id,
                &client.enabled,
                &now,
                &now,
            ],
        )
        .await?;

    // Convert row to OAuth2Client
    row.try_into()
}

pub async fn get_client_by_id(db: &Database, client_id: &str) -> Result<Option<OAuth2Client>> {
    let query = r#"
        SELECT
            id, client_id, client_secret_hash, client_name, client_type,
            redirect_uris, scopes, grant_types, response_types,
            token_endpoint_auth_method, owner_id, realm_id,
            enabled, created_at, updated_at, deleted_at
        FROM oauth2_clients
        WHERE client_id = $1 AND deleted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&client_id]).await?;
    // Convert row to OAuth2Client
    Ok(Some(row.try_into()?))
}

pub async fn store_authorization_code(db: &Database, code: &OAuth2AuthorizationCode) -> Result<()> {
    let code_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO oauth2_authorization_codes (
            id, code, client_id, user_id, redirect_uri, scopes,
            code_challenge, code_challenge_method, expires_at,
            used, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

    db.execute(
        query,
        &[
            &code_id,
            &code.code,
            &code.client_id,
            &code.user_id,
            &code.redirect_uri,
            &code.scopes,
            &code.code_challenge,
            &code.code_challenge_method,
            &code.expires_at,
            &false,
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_authorization_code(
    db: &Database,
    code: &str,
) -> Result<Option<OAuth2AuthorizationCode>> {
    let query = r#"
        SELECT
            id, code, client_id, user_id, redirect_uri, scopes,
            code_challenge, code_challenge_method, expires_at,
            used, created_at
        FROM oauth2_authorization_codes
        WHERE code = $1 AND used = false AND expires_at > NOW()
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&code]).await?;
    // Convert row to OAuth2AuthorizationCode
    Ok(Some(row.try_into()?))
}

pub async fn mark_code_used(db: &Database, code: &str) -> Result<()> {
    let query = "UPDATE oauth2_authorization_codes SET used = true WHERE code = $1";
    db.execute(query, &[&code]).await?;
    Ok(())
}

pub async fn store_access_token(db: &Database, token: &OAuth2AccessToken) -> Result<()> {
    let token_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO oauth2_access_tokens (
            id, token_hash, refresh_token_hash, client_id, user_id,
            scopes, expires_at, refresh_expires_at, revoked,
            created_at, last_used_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

    db.execute(
        query,
        &[
            &token_id,
            &token.token_hash,
            &token.refresh_token_hash,
            &token.client_id,
            &token.user_id,
            &token.scopes,
            &token.expires_at,
            &token.refresh_expires_at,
            &false,
            &now,
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_access_token(
    db: &Database,
    token_hash: &str,
) -> Result<Option<OAuth2AccessToken>> {
    let query = r#"
        SELECT
            id, token_hash, refresh_token_hash, client_id, user_id,
            scopes, expires_at, refresh_expires_at, revoked,
            revoked_at, created_at, last_used_at
        FROM oauth2_access_tokens
        WHERE token_hash = $1 AND revoked = false AND expires_at > NOW()
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&token_hash]).await?;
    // Convert row to OAuth2AccessToken
    Ok(Some(row.try_into()?))
}

pub async fn revoke_token(db: &Database, token_hash: &str) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE oauth2_access_tokens
        SET revoked = true, revoked_at = $2
        WHERE token_hash = $1
    "#;
    db.execute(query, &[&token_hash, &now]).await?;
    Ok(())
}

pub async fn get_all_clients(db: &Database) -> Result<Vec<OAuth2Client>> {
    let query = r#"
        SELECT
            id, client_id, client_secret_hash, client_name, client_type,
            redirect_uris, scopes, grant_types, response_types,
            token_endpoint_auth_method, owner_id, realm_id,
            enabled, created_at, updated_at, deleted_at
        FROM oauth2_clients
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[]).await?;
    let mut clients = Vec::new();

    for row in rows {
        clients.push(row.try_into()?);
    }

    Ok(clients)
}

pub async fn delete_client(db: &Database, client_id: &str) -> Result<bool> {
    let now = Utc::now();
    let query = r#"
        UPDATE oauth2_clients
        SET deleted_at = $2
        WHERE client_id = $1 AND deleted_at IS NULL
    "#;

    let rows_affected = db.execute(query, &[&client_id, &now]).await?;
    Ok(rows_affected > 0)
}

pub async fn revoke_user_tokens(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE oauth2_access_tokens
        SET revoked = true, revoked_at = $2
        WHERE user_id = $1 AND revoked = false
    "#;
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}
