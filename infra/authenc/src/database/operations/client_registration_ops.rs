/// Database operations for Dynamic Client Registration (RFC 7591/7592)
use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::{ClientRegistrationPolicy, ClientRegistrationToken, InitialAccessToken, OAuth2Client},
};
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Create a registration access token for a client
pub async fn create_registration_token(
    db: &Database,
    client_id: Uuid,
    token_hash: &str,
    realm_id: Option<Uuid>,
    expires_in_seconds: Option<i32>,
) -> Result<ClientRegistrationToken> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = expires_in_seconds.map(|secs| now + Duration::seconds(secs as i64));

    let query = r#"
        INSERT INTO client_registration_tokens (
            id, token_hash, client_id, realm_id, expires_at,
            revoked, created_at
        )
        VALUES ($1, $2, $3, $4, $5, false, $6)
        RETURNING id, token_hash, client_id, realm_id, expires_at,
                  revoked, revoked_at, created_at, last_used_at
    "#;

    db.query_one::<ClientRegistrationToken>(
        query,
        &[&id, &token_hash, &client_id, &realm_id, &expires_at, &now],
    )
    .await
}

/// Get registration token by hash
pub async fn get_registration_token_by_hash(
    db: &Database,
    token_hash: &str,
) -> Result<Option<ClientRegistrationToken>> {
    let query = r#"
        SELECT id, token_hash, client_id, realm_id, expires_at,
               revoked, revoked_at, created_at, last_used_at
        FROM client_registration_tokens
        WHERE token_hash = $1 AND revoked = false
    "#;

    match db.query_opt(query, &[&token_hash]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

/// Validate and consume registration token
pub async fn validate_registration_token(
    db: &Database,
    token_hash: &str,
    client_id: &str,
) -> Result<bool> {
    let query = r#"
        SELECT t.id, t.expires_at, c.client_id
        FROM client_registration_tokens t
        JOIN oauth2_clients c ON t.client_id = c.id
        WHERE t.token_hash = $1
          AND t.revoked = false
          AND c.client_id = $2
          AND c.deleted_at IS NULL
          AND (t.expires_at IS NULL OR t.expires_at > NOW())
    "#;

    match db.query_opt(query, &[&token_hash, &client_id]).await? {
        Some(_) => {
            // Update last_used_at
            let update_query = r#"
                UPDATE client_registration_tokens
                SET last_used_at = NOW()
                WHERE token_hash = $1
            "#;
            db.execute(update_query, &[&token_hash]).await?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Revoke registration token
pub async fn revoke_registration_token(db: &Database, token_hash: &str) -> Result<bool> {
    let query = r#"
        UPDATE client_registration_tokens
        SET revoked = true, revoked_at = NOW()
        WHERE token_hash = $1 AND revoked = false
    "#;

    let rows_affected = db.execute(query, &[&token_hash]).await?;
    Ok(rows_affected > 0)
}

/// Create an initial access token
pub async fn create_initial_access_token(
    db: &Database,
    token_hash: &str,
    realm_id: Option<Uuid>,
    count: i32,
    expires_in_seconds: Option<i32>,
    created_by: Option<Uuid>,
) -> Result<InitialAccessToken> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = expires_in_seconds.map(|secs| now + Duration::seconds(secs as i64));

    let query = r#"
        INSERT INTO initial_access_tokens (
            id, token_hash, realm_id, count, remaining_count,
            expires_at, revoked, created_at, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8)
        RETURNING id, token_hash, realm_id, count, remaining_count,
                  expires_at, revoked, revoked_at, created_at, created_by, last_used_at
    "#;

    db.query_one::<InitialAccessToken>(
        query,
        &[
            &id,
            &token_hash,
            &realm_id,
            &count,
            &count,
            &expires_at,
            &now,
            &created_by,
        ],
    )
    .await
}

/// Get initial access token by hash
pub async fn get_initial_access_token_by_hash(
    db: &Database,
    token_hash: &str,
) -> Result<Option<InitialAccessToken>> {
    let query = r#"
        SELECT id, token_hash, realm_id, count, remaining_count,
               expires_at, revoked, revoked_at, created_at, created_by, last_used_at
        FROM initial_access_tokens
        WHERE token_hash = $1
          AND revoked = false
          AND remaining_count > 0
          AND (expires_at IS NULL OR expires_at > NOW())
    "#;

    match db.query_opt(query, &[&token_hash]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

/// Consume one use of an initial access token
pub async fn consume_initial_access_token(db: &Database, token_hash: &str) -> Result<bool> {
    let query = r#"
        UPDATE initial_access_tokens
        SET remaining_count = remaining_count - 1,
            last_used_at = NOW()
        WHERE token_hash = $1
          AND revoked = false
          AND remaining_count > 0
          AND (expires_at IS NULL OR expires_at > NOW())
    "#;

    let rows_affected = db.execute(query, &[&token_hash]).await?;
    Ok(rows_affected > 0)
}

/// Revoke initial access token
pub async fn revoke_initial_access_token(db: &Database, token_hash: &str) -> Result<bool> {
    let query = r#"
        UPDATE initial_access_tokens
        SET revoked = true, revoked_at = NOW()
        WHERE token_hash = $1 AND revoked = false
    "#;

    let rows_affected = db.execute(query, &[&token_hash]).await?;
    Ok(rows_affected > 0)
}

/// List all initial access tokens for a realm
pub async fn list_initial_access_tokens(
    db: &Database,
    realm_id: Option<Uuid>,
) -> Result<Vec<InitialAccessToken>> {
    let query = if realm_id.is_some() {
        r#"
            SELECT id, token_hash, realm_id, count, remaining_count,
                   expires_at, revoked, revoked_at, created_at, created_by, last_used_at
            FROM initial_access_tokens
            WHERE realm_id = $1
            ORDER BY created_at DESC
        "#
    } else {
        r#"
            SELECT id, token_hash, realm_id, count, remaining_count,
                   expires_at, revoked, revoked_at, created_at, created_by, last_used_at
            FROM initial_access_tokens
            ORDER BY created_at DESC
        "#
    };

    let rows: Vec<InitialAccessToken> = if let Some(realm) = realm_id {
        db.query::<InitialAccessToken>(query, &[&realm]).await?
    } else {
        db.query::<InitialAccessToken>(query, &[]).await?
    };

    Ok(rows)
}

/// Get or create default client registration policy for a realm
pub async fn get_or_create_default_policy(
    db: &Database,
    realm_id: Uuid,
) -> Result<ClientRegistrationPolicy> {
    // Try to get existing policy
    let query = r#"
        SELECT id, realm_id, name, allow_dynamic_registration,
               require_initial_access_token, require_software_statement,
               allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
               max_redirect_uris, allowed_scopes, default_scopes,
               allowed_grant_types, allowed_response_types,
               require_https_redirect_uris, allow_localhost_redirect,
               client_secret_expires_in, registration_token_expires_in,
               enabled, created_at, updated_at
        FROM client_registration_policies
        WHERE realm_id = $1 AND name = 'default' AND enabled = true
    "#;

    if let Some(row) = db.query_opt(query, &[&realm_id]).await? {
        return row.try_into();
    }

    // Create default policy
    let id = Uuid::new_v4();
    let now = Utc::now();
    let insert_query = r#"
        INSERT INTO client_registration_policies (
            id, realm_id, name, allow_dynamic_registration,
            require_initial_access_token, require_software_statement,
            allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
            max_redirect_uris, allowed_scopes, default_scopes,
            allowed_grant_types, allowed_response_types,
            require_https_redirect_uris, allow_localhost_redirect,
            client_secret_expires_in, registration_token_expires_in,
            enabled, created_at, updated_at
        )
        VALUES (
            $1, $2, 'default', true, false, false,
            NULL, ARRAY['http://localhost*'], 10,
            ARRAY['openid', 'profile', 'email'],
            ARRAY['openid', 'profile'],
            ARRAY['authorization_code', 'refresh_token'],
            ARRAY['code'],
            true, true, NULL, 31536000,
            true, $3, $4
        )
        RETURNING id, realm_id, name, allow_dynamic_registration,
                  require_initial_access_token, require_software_statement,
                  allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
                  max_redirect_uris, allowed_scopes, default_scopes,
                  allowed_grant_types, allowed_response_types,
                  require_https_redirect_uris, allow_localhost_redirect,
                  client_secret_expires_in, registration_token_expires_in,
                  enabled, created_at, updated_at
    "#;

    db.query_one::<ClientRegistrationPolicy>(insert_query, &[&id, &realm_id, &now, &now])
        .await
}

/// Log client registration audit event
pub async fn log_registration_audit(
    db: &Database,
    event_type: &str,
    client_id: Option<Uuid>,
    client_identifier: Option<&str>,
    realm_id: Option<Uuid>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    initial_access_token_id: Option<Uuid>,
    registration_access_token_id: Option<Uuid>,
    success: bool,
    error_code: Option<&str>,
    error_description: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let ip_addr: Option<std::net::IpAddr> = ip_address.and_then(|ip| ip.parse().ok());

    let query = r#"
        INSERT INTO client_registration_audit_log (
            id, event_type, client_id, client_identifier, realm_id,
            ip_address, user_agent, initial_access_token_id,
            registration_access_token_id, success, error_code,
            error_description, metadata, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#;

    db.execute(
        query,
        &[
            &id,
            &event_type,
            &client_id,
            &client_identifier,
            &realm_id,
            &ip_addr,
            &user_agent,
            &initial_access_token_id,
            &registration_access_token_id,
            &success,
            &error_code,
            &error_description,
            &metadata,
            &now,
        ],
    )
    .await?;

    Ok(())
}

/// Enhanced client creation with full DCR metadata
pub async fn create_client_with_metadata(
    db: &Database,
    client: &OAuth2Client,
) -> Result<OAuth2Client> {
    let client_db_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO oauth2_clients (
            id, client_id, client_secret_hash, client_name, client_type,
            redirect_uris, scopes, grant_types, response_types,
            token_endpoint_auth_method, owner_id, realm_id, enabled,
            created_at, updated_at,
            logo_uri, client_uri, policy_uri, tos_uri,
            jwks_uri, jwks, sector_identifier_uri, subject_type,
            id_token_signed_response_alg, id_token_encrypted_response_alg,
            id_token_encrypted_response_enc, userinfo_signed_response_alg,
            userinfo_encrypted_response_alg, userinfo_encrypted_response_enc,
            request_object_signing_alg, request_object_encryption_alg,
            request_object_encryption_enc, token_endpoint_auth_signing_alg,
            default_max_age, require_auth_time, default_acr_values,
            initiate_login_uri, request_uris, application_type, contacts,
            client_id_issued_at, client_secret_expires_at,
            software_id, software_version, registration_access_token_hash
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
            $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
            $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41,
            $42, $43, $44, $45
        )
        RETURNING *
    "#;

    db.query_one::<OAuth2Client>(
        query,
        &[
            &client_db_id,
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
            &client.logo_uri,
            &client.client_uri,
            &client.policy_uri,
            &client.tos_uri,
            &client.jwks_uri,
            &client.jwks,
            &client.sector_identifier_uri,
            &client.subject_type,
            &client.id_token_signed_response_alg,
            &client.id_token_encrypted_response_alg,
            &client.id_token_encrypted_response_enc,
            &client.userinfo_signed_response_alg,
            &client.userinfo_encrypted_response_alg,
            &client.userinfo_encrypted_response_enc,
            &client.request_object_signing_alg,
            &client.request_object_encryption_alg,
            &client.request_object_encryption_enc,
            &client.token_endpoint_auth_signing_alg,
            &client.default_max_age,
            &client.require_auth_time,
            &client.default_acr_values,
            &client.initiate_login_uri,
            &client.request_uris,
            &client.application_type,
            &client.contacts,
            &client.client_id_issued_at,
            &client.client_secret_expires_at,
            &client.software_id,
            &client.software_version,
            &client.registration_access_token_hash,
        ],
    )
    .await
}

/// Update client with full metadata
pub async fn update_client_with_metadata(
    db: &Database,
    client_id: &str,
    client: &OAuth2Client,
) -> Result<OAuth2Client> {
    let now = Utc::now();

    let query = r#"
        UPDATE oauth2_clients
        SET client_name = $2, redirect_uris = $3, scopes = $4,
            grant_types = $5, response_types = $6,
            token_endpoint_auth_method = $7, enabled = $8, updated_at = $9,
            logo_uri = $10, client_uri = $11, policy_uri = $12, tos_uri = $13,
            jwks_uri = $14, jwks = $15, sector_identifier_uri = $16,
            subject_type = $17, id_token_signed_response_alg = $18,
            id_token_encrypted_response_alg = $19, id_token_encrypted_response_enc = $20,
            userinfo_signed_response_alg = $21, userinfo_encrypted_response_alg = $22,
            userinfo_encrypted_response_enc = $23, request_object_signing_alg = $24,
            request_object_encryption_alg = $25, request_object_encryption_enc = $26,
            token_endpoint_auth_signing_alg = $27, default_max_age = $28,
            require_auth_time = $29, default_acr_values = $30,
            initiate_login_uri = $31, request_uris = $32,
            application_type = $33, contacts = $34
        WHERE client_id = $1 AND deleted_at IS NULL
        RETURNING *
    "#;

    db.query_one::<OAuth2Client>(
        query,
        &[
            &client_id,
            &client.client_name,
            &client.redirect_uris,
            &client.scopes,
            &client.grant_types,
            &client.response_types,
            &client.token_endpoint_auth_method,
            &client.enabled,
            &now,
            &client.logo_uri,
            &client.client_uri,
            &client.policy_uri,
            &client.tos_uri,
            &client.jwks_uri,
            &client.jwks,
            &client.sector_identifier_uri,
            &client.subject_type,
            &client.id_token_signed_response_alg,
            &client.id_token_encrypted_response_alg,
            &client.id_token_encrypted_response_enc,
            &client.userinfo_signed_response_alg,
            &client.userinfo_encrypted_response_alg,
            &client.userinfo_encrypted_response_enc,
            &client.request_object_signing_alg,
            &client.request_object_encryption_alg,
            &client.request_object_encryption_enc,
            &client.token_endpoint_auth_signing_alg,
            &client.default_max_age,
            &client.require_auth_time,
            &client.default_acr_values,
            &client.initiate_login_uri,
            &client.request_uris,
            &client.application_type,
            &client.contacts,
        ],
    )
    .await
}

/// Get initial access token by ID
pub async fn get_initial_access_token_by_id(
    db: &Database,
    id: Uuid,
) -> Result<Option<InitialAccessToken>> {
    let query = r#"
        SELECT id, token_hash, realm_id, count, remaining_count,
               expires_at, revoked, revoked_at, created_at, created_by, last_used_at
        FROM initial_access_tokens
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&id]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

/// Create a software statement issuer
pub async fn create_software_statement_issuer(
    db: &Database,
    name: &str,
    issuer: &str,
    jwks_uri: Option<&str>,
    jwks: Option<serde_json::Value>,
    realm_id: Option<Uuid>,
) -> Result<crate::models::SoftwareStatementIssuer> {
    use crate::models::SoftwareStatementIssuer;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO software_statement_issuers (
            id, name, issuer, jwks_uri, jwks, realm_id,
            enabled, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)
        RETURNING id, name, issuer, jwks_uri, jwks, realm_id,
                  enabled, created_at, updated_at
    "#;

    db.query_one::<SoftwareStatementIssuer>(
        query,
        &[&id, &name, &issuer, &jwks_uri, &jwks, &realm_id, &now, &now],
    )
    .await
}

/// Get software statement issuer by issuer string
pub async fn get_software_statement_issuer_by_issuer(
    db: &Database,
    issuer: &str,
) -> Result<Option<crate::models::SoftwareStatementIssuer>> {
    let query = r#"
        SELECT id, name, issuer, jwks_uri, jwks, realm_id,
               enabled, created_at, updated_at
        FROM software_statement_issuers
        WHERE issuer = $1 AND enabled = true
    "#;

    match db.query_opt(query, &[&issuer]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

/// List software statement issuers for a realm
pub async fn list_software_statement_issuers(
    db: &Database,
    realm_id: Option<Uuid>,
) -> Result<Vec<crate::models::SoftwareStatementIssuer>> {
    let query = if realm_id.is_some() {
        r#"
            SELECT id, name, issuer, jwks_uri, jwks, realm_id,
                   enabled, created_at, updated_at
            FROM software_statement_issuers
            WHERE realm_id = $1
            ORDER BY created_at DESC
        "#
    } else {
        r#"
            SELECT id, name, issuer, jwks_uri, jwks, realm_id,
                   enabled, created_at, updated_at
            FROM software_statement_issuers
            ORDER BY created_at DESC
        "#
    };

    let rows: Vec<crate::models::SoftwareStatementIssuer> = if let Some(realm) = realm_id {
        db.query::<crate::models::SoftwareStatementIssuer>(query, &[&realm])
            .await?
    } else {
        db.query::<crate::models::SoftwareStatementIssuer>(query, &[])
            .await?
    };

    Ok(rows)
}

/// Update software statement issuer
pub async fn update_software_statement_issuer(
    db: &Database,
    id: Uuid,
    name: Option<&str>,
    jwks_uri: Option<&str>,
    jwks: Option<serde_json::Value>,
    enabled: Option<bool>,
) -> Result<crate::models::SoftwareStatementIssuer> {
    let now = Utc::now();

    // Build dynamic query based on what fields to update
    let mut updates = Vec::new();
    let mut param_num = 2; // $1 is the id

    if name.is_some() {
        updates.push(format!("name = ${}", param_num));
        param_num += 1;
    }
    if jwks_uri.is_some() {
        updates.push(format!("jwks_uri = ${}", param_num));
        param_num += 1;
    }
    if jwks.is_some() {
        updates.push(format!("jwks = ${}", param_num));
        param_num += 1;
    }
    if enabled.is_some() {
        updates.push(format!("enabled = ${}", param_num));
        param_num += 1;
    }
    updates.push(format!("updated_at = ${}", param_num));

    if updates.is_empty() {
        return Err(AuthencError::ValidationError {
            message: "No fields to update".to_string(),
        });
    }

    let query = format!(
        r#"
        UPDATE software_statement_issuers
        SET {}
        WHERE id = $1
        RETURNING id, name, issuer, jwks_uri, jwks, realm_id,
                  enabled, created_at, updated_at
    "#,
        updates.join(", ")
    );

    // Build parameters vector - convert Option<&str> to Option<String> for ToSql compatibility
    let name_owned = name.map(|s| s.to_string());
    let jwks_uri_owned = jwks_uri.map(|s| s.to_string());
    let enabled_val = enabled;

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&id];
    if let Some(ref n) = name_owned {
        params.push(n);
    }
    if let Some(ref j) = jwks_uri_owned {
        params.push(j);
    }
    if let Some(ref j) = jwks {
        params.push(j);
    }
    if let Some(ref e) = enabled_val {
        params.push(e);
    }
    params.push(&now);

    db.query_one::<crate::models::SoftwareStatementIssuer>(&query, &params[..])
        .await
}

/// Delete software statement issuer
pub async fn delete_software_statement_issuer(db: &Database, id: Uuid) -> Result<bool> {
    let query = r#"
        DELETE FROM software_statement_issuers
        WHERE id = $1
    "#;

    let rows_affected = db.execute(query, &[&id]).await?;
    Ok(rows_affected > 0)
}

/// Update client registration policy
pub async fn update_client_registration_policy(
    db: &Database,
    realm_id: Uuid,
    policy: &ClientRegistrationPolicy,
) -> Result<ClientRegistrationPolicy> {
    let now = Utc::now();

    let query = r#"
        UPDATE client_registration_policies
        SET name = $2,
            allow_dynamic_registration = $3,
            require_initial_access_token = $4,
            require_software_statement = $5,
            allowed_redirect_uri_patterns = $6,
            blocked_redirect_uri_patterns = $7,
            max_redirect_uris = $8,
            allowed_scopes = $9,
            default_scopes = $10,
            allowed_grant_types = $11,
            allowed_response_types = $12,
            require_https_redirect_uris = $13,
            allow_localhost_redirect = $14,
            client_secret_expires_in = $15,
            registration_token_expires_in = $16,
            enabled = $17,
            updated_at = $18
        WHERE realm_id = $1 AND id = $19
        RETURNING id, realm_id, name, allow_dynamic_registration,
                  require_initial_access_token, require_software_statement,
                  allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
                  max_redirect_uris, allowed_scopes, default_scopes,
                  allowed_grant_types, allowed_response_types,
                  require_https_redirect_uris, allow_localhost_redirect,
                  client_secret_expires_in, registration_token_expires_in,
                  enabled, created_at, updated_at
    "#;

    db.query_one::<ClientRegistrationPolicy>(
        query,
        &[
            &realm_id,
            &policy.name,
            &policy.allow_dynamic_registration,
            &policy.require_initial_access_token,
            &policy.require_software_statement,
            &policy.allowed_redirect_uri_patterns,
            &policy.blocked_redirect_uri_patterns,
            &policy.max_redirect_uris,
            &policy.allowed_scopes,
            &policy.default_scopes,
            &policy.allowed_grant_types,
            &policy.allowed_response_types,
            &policy.require_https_redirect_uris,
            &policy.allow_localhost_redirect,
            &policy.client_secret_expires_in,
            &policy.registration_token_expires_in,
            &policy.enabled,
            &now,
            &policy.id,
        ],
    )
    .await
}

/// Create a new client registration policy
pub async fn create_client_registration_policy(
    db: &Database,
    realm_id: Uuid,
    name: &str,
    policy: &ClientRegistrationPolicy,
) -> Result<ClientRegistrationPolicy> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO client_registration_policies (
            id, realm_id, name, allow_dynamic_registration,
            require_initial_access_token, require_software_statement,
            allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
            max_redirect_uris, allowed_scopes, default_scopes,
            allowed_grant_types, allowed_response_types,
            require_https_redirect_uris, allow_localhost_redirect,
            client_secret_expires_in, registration_token_expires_in,
            enabled, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
        )
        RETURNING id, realm_id, name, allow_dynamic_registration,
                  require_initial_access_token, require_software_statement,
                  allowed_redirect_uri_patterns, blocked_redirect_uri_patterns,
                  max_redirect_uris, allowed_scopes, default_scopes,
                  allowed_grant_types, allowed_response_types,
                  require_https_redirect_uris, allow_localhost_redirect,
                  client_secret_expires_in, registration_token_expires_in,
                  enabled, created_at, updated_at
    "#;

    db.query_one::<ClientRegistrationPolicy>(
        query,
        &[
            &id,
            &realm_id,
            &name,
            &policy.allow_dynamic_registration,
            &policy.require_initial_access_token,
            &policy.require_software_statement,
            &policy.allowed_redirect_uri_patterns,
            &policy.blocked_redirect_uri_patterns,
            &policy.max_redirect_uris,
            &policy.allowed_scopes,
            &policy.default_scopes,
            &policy.allowed_grant_types,
            &policy.allowed_response_types,
            &policy.require_https_redirect_uris,
            &policy.allow_localhost_redirect,
            &policy.client_secret_expires_in,
            &policy.registration_token_expires_in,
            &policy.enabled,
            &now,
            &now,
        ],
    )
    .await
}

/// Delete client registration policy
pub async fn delete_client_registration_policy(
    db: &Database,
    realm_id: Uuid,
    policy_id: Uuid,
) -> Result<bool> {
    let query = r#"
        DELETE FROM client_registration_policies
        WHERE id = $1 AND realm_id = $2
    "#;

    let rows_affected = db.execute(query, &[&policy_id, &realm_id]).await?;
    Ok(rows_affected > 0)
}
