/// Session management database operations
use super::*;
use sha2::{Digest, Sha256};

/// Parameters for creating a new user session.
pub struct NewUserSession<'a> {
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub expires_in: i64,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub authentication_method: Option<&'a str>,
    pub protocol: Option<&'a str>,
}

pub async fn create_user_session(
    db: &Database,
    params: NewUserSession<'_>,
) -> Result<serde_json::Value> {
    let NewUserSession {
        user_id,
        realm_id,
        client_id,
        token,
        refresh_token,
        expires_in,
        ip_address,
        user_agent,
        authentication_method,
        protocol,
    } = params;

    // Hash tokens for storage
    let token_hash = hash_token(token);
    let refresh_token_hash = refresh_token.map(hash_token);

    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
    let refresh_token_expires_at = refresh_token.map(|_| {
        chrono::Utc::now() + chrono::Duration::days(30) // 30 days for refresh tokens
    });

    let query = r#"
        INSERT INTO user_sessions (
            user_id, realm_id, client_id,
            token_hash, refresh_token_hash,
            expires_at, refresh_token_expires_at,
            ip_address, user_agent,
            authentication_method, protocol
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, user_id, realm_id, client_id, started_at, expires_at,
                  last_accessed, refresh_count, revoked, authentication_method, protocol,
                  created_at, updated_at
    "#;

    let ip_addr: Option<std::net::IpAddr> = ip_address.and_then(|ip| ip.parse().ok());

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &user_id,
                &realm_id,
                &client_id,
                &token_hash,
                &refresh_token_hash,
                &expires_at,
                &refresh_token_expires_at,
                &ip_addr,
                &user_agent,
                &authentication_method,
                &protocol,
            ],
        )
        .await?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "user_id": row.get::<_, Uuid>("user_id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "client_id": row.get::<_, Option<Uuid>>("client_id"),
        "started_at": row.get::<_, chrono::DateTime<chrono::Utc>>("started_at"),
        "expires_at": row.get::<_, chrono::DateTime<chrono::Utc>>("expires_at"),
        "last_accessed": row.get::<_, chrono::DateTime<chrono::Utc>>("last_accessed"),
        "refresh_count": row.get::<_, i32>("refresh_count"),
        "revoked": row.get::<_, bool>("revoked"),
        "authentication_method": row.get::<_, Option<String>>("authentication_method"),
        "protocol": row.get::<_, Option<String>>("protocol"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    }))
}

pub async fn get_session_by_token(db: &Database, token: &str) -> Result<Option<serde_json::Value>> {
    let token_hash = hash_token(token);

    let query = r#"
        SELECT id, user_id, realm_id, client_id,
               started_at, expires_at, last_accessed,
               idle_expires_at, refresh_count,
               refresh_token_expires_at, offline_token_expires_at,
               ip_address, user_agent, revoked, revoked_at, revoked_reason,
               authentication_method, protocol,
               created_at, updated_at
        FROM user_sessions
        WHERE token_hash = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&token_hash]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row: &tokio_postgres::Row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "user_id": row.get::<_, Uuid>("user_id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "client_id": row.get::<_, Option<Uuid>>("client_id"),
        "started_at": row.get::<_, chrono::DateTime<chrono::Utc>>("started_at"),
        "expires_at": row.get::<_, chrono::DateTime<chrono::Utc>>("expires_at"),
        "last_accessed": row.get::<_, chrono::DateTime<chrono::Utc>>("last_accessed"),
        "idle_expires_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("idle_expires_at"),
        "refresh_count": row.get::<_, i32>("refresh_count"),
        "refresh_token_expires_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("refresh_token_expires_at"),
        "offline_token_expires_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("offline_token_expires_at"),
        "ip_address": row.get::<_, Option<std::net::IpAddr>>("ip_address").map(|ip| ip.to_string()),
        "user_agent": row.get::<_, Option<String>>("user_agent"),
        "revoked": row.get::<_, bool>("revoked"),
        "revoked_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("revoked_at"),
        "revoked_reason": row.get::<_, Option<String>>("revoked_reason"),
        "authentication_method": row.get::<_, Option<String>>("authentication_method"),
        "protocol": row.get::<_, Option<String>>("protocol"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    })))
}

pub async fn get_user_sessions(db: &Database, user_id: Uuid) -> Result<Vec<serde_json::Value>> {
    let query = r#"
        SELECT id, user_id, realm_id, client_id,
               started_at, expires_at, last_accessed,
               refresh_count, ip_address, user_agent,
               authentication_method, protocol,
               created_at, updated_at
        FROM user_sessions
        WHERE user_id = $1 AND NOT revoked AND expires_at > NOW()
        ORDER BY last_accessed DESC
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id]).await?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "user_id": row.get::<_, Uuid>("user_id"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "client_id": row.get::<_, Option<Uuid>>("client_id"),
            "started_at": row.get::<_, chrono::DateTime<chrono::Utc>>("started_at"),
            "expires_at": row.get::<_, chrono::DateTime<chrono::Utc>>("expires_at"),
            "last_accessed": row.get::<_, chrono::DateTime<chrono::Utc>>("last_accessed"),
            "refresh_count": row.get::<_, i32>("refresh_count"),
            "ip_address": row.get::<_, Option<std::net::IpAddr>>("ip_address").map(|ip| ip.to_string()),
            "user_agent": row.get::<_, Option<String>>("user_agent"),
            "authentication_method": row.get::<_, Option<String>>("authentication_method"),
            "protocol": row.get::<_, Option<String>>("protocol"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
        }));
    }

    Ok(sessions)
}

pub async fn touch_session(db: &Database, session_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE user_sessions
        SET last_accessed = NOW(), updated_at = NOW()
        WHERE id = $1 AND NOT revoked
    "#;

    db.execute(query, &[&session_id]).await?;
    Ok(())
}

pub async fn rotate_refresh_token(
    db: &Database,
    session_id: Uuid,
    old_refresh_token: &str,
    new_refresh_token: &str,
    client_ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<bool> {
    let old_hash = hash_token(old_refresh_token);
    let new_hash = hash_token(new_refresh_token);

    // Verify old token matches
    let verify_query = r#"
        SELECT id FROM user_sessions
        WHERE id = $1 AND refresh_token_hash = $2 AND NOT revoked
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(verify_query, &[&session_id, &old_hash]).await?;
    if rows.is_empty() {
        return Ok(false);
    }

    // Update with new token
    let update_query = r#"
        UPDATE user_sessions
        SET refresh_token_hash = $1,
            refresh_count = refresh_count + 1,
            refresh_token_expires_at = NOW() + INTERVAL '30 days',
            updated_at = NOW()
        WHERE id = $2
    "#;

    db.execute(update_query, &[&new_hash, &session_id]).await?;

    // Log rotation
    let ip_addr: Option<std::net::IpAddr> = client_ip.and_then(|ip| ip.parse().ok());

    let log_query = r#"
        INSERT INTO refresh_token_history (
            user_session_id, old_token_hash, new_token_hash,
            client_ip, user_agent
        )
        VALUES ($1, $2, $3, $4, $5)
    "#;

    db.execute(
        log_query,
        &[&session_id, &old_hash, &new_hash, &ip_addr, &user_agent],
    )
    .await?;

    Ok(true)
}

pub async fn revoke_session(db: &Database, session_id: Uuid, reason: Option<&str>) -> Result<()> {
    let query = r#"
        UPDATE user_sessions
        SET revoked = TRUE,
            revoked_at = NOW(),
            revoked_reason = $2,
            updated_at = NOW()
        WHERE id = $1
    "#;

    db.execute(query, &[&session_id, &reason]).await?;
    Ok(())
}

pub async fn revoke_user_sessions(
    db: &Database,
    user_id: Uuid,
    reason: Option<&str>,
) -> Result<i64> {
    let query = r#"
        UPDATE user_sessions
        SET revoked = TRUE,
            revoked_at = NOW(),
            revoked_reason = $2,
            updated_at = NOW()
        WHERE user_id = $1 AND NOT revoked
    "#;

    let count = db.execute(query, &[&user_id, &reason]).await?;
    Ok(count as i64)
}

pub async fn cleanup_expired_sessions(db: &Database) -> Result<i64> {
    let query = r#"
        DELETE FROM user_sessions
        WHERE expires_at < NOW() OR (idle_expires_at IS NOT NULL AND idle_expires_at < NOW())
    "#;

    let count = db.execute(query, &[]).await?;
    Ok(count as i64)
}

/// Parameters for [`create_offline_token`].
pub struct NewOfflineToken<'a> {
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub token: &'a str,
    pub scope: Option<&'a str>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Option<serde_json::Value>,
}

pub async fn create_offline_token(
    db: &Database,
    params: NewOfflineToken<'_>,
) -> Result<serde_json::Value> {
    let NewOfflineToken {
        user_id,
        realm_id,
        client_id,
        token,
        scope,
        expires_at,
        data,
    } = params;
    let token_hash = hash_token(token);

    let query = r#"
        INSERT INTO offline_tokens (
            user_id, realm_id, client_id,
            token_hash, scope, expires_at, data
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, realm_id, client_id,
                  created_at, expires_at, last_used_at,
                  scope, revoked, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &user_id,
                &realm_id,
                &client_id,
                &token_hash,
                &scope,
                &expires_at,
                &data,
            ],
        )
        .await?;

    Ok(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "user_id": row.get::<_, Uuid>("user_id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "client_id": row.get::<_, Uuid>("client_id"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "expires_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("expires_at"),
        "last_used_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("last_used_at"),
        "scope": row.get::<_, Option<String>>("scope"),
        "revoked": row.get::<_, bool>("revoked"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    }))
}

pub async fn get_offline_token(db: &Database, token: &str) -> Result<Option<serde_json::Value>> {
    let token_hash = hash_token(token);

    let query = r#"
        SELECT id, user_id, realm_id, client_id,
               created_at, expires_at, last_used_at,
               scope, data, revoked, revoked_at,
               created_at, updated_at
        FROM offline_tokens
        WHERE token_hash = $1
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&token_hash]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>("id"),
        "user_id": row.get::<_, Uuid>("user_id"),
        "realm_id": row.get::<_, Uuid>("realm_id"),
        "client_id": row.get::<_, Uuid>("client_id"),
        "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
        "expires_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("expires_at"),
        "last_used_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("last_used_at"),
        "scope": row.get::<_, Option<String>>("scope"),
        "data": row.get::<_, Option<serde_json::Value>>("data"),
        "revoked": row.get::<_, bool>("revoked"),
        "revoked_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("revoked_at"),
        "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
    })))
}

pub async fn touch_offline_token(db: &Database, token_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE offline_tokens
        SET last_used_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND NOT revoked
    "#;

    db.execute(query, &[&token_id]).await?;
    Ok(())
}

pub async fn revoke_offline_token(db: &Database, token_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE offline_tokens
        SET revoked = TRUE, revoked_at = NOW(), updated_at = NOW()
        WHERE id = $1
    "#;

    db.execute(query, &[&token_id]).await?;
    Ok(())
}

pub async fn update_device_session_activity(
    db: &Database,
    session_id: Uuid,
    risk_score: Option<f64>,
    risk_factors: Option<serde_json::Value>,
) -> Result<()> {
    let query = if risk_score.is_some() && risk_factors.is_some() {
        r#"
            UPDATE device_sessions
            SET last_activity = NOW(),
                risk_score = $2,
                risk_factors = $3,
                updated_at = NOW()
            WHERE id = $1 AND is_active
        "#
    } else {
        r#"
            UPDATE device_sessions
            SET last_activity = NOW(), updated_at = NOW()
            WHERE id = $1 AND is_active
        "#
    };

    if let (Some(score), Some(factors)) = (risk_score, risk_factors) {
        db.execute(query, &[&session_id, &score, &factors]).await?;
    } else {
        db.execute(query, &[&session_id]).await?;
    }

    Ok(())
}

pub async fn end_device_session(db: &Database, session_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE device_sessions
        SET is_active = FALSE,
            ended_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
    "#;

    db.execute(query, &[&session_id]).await?;
    Ok(())
}

pub async fn get_user_device_sessions(
    db: &Database,
    user_id: Uuid,
) -> Result<Vec<serde_json::Value>> {
    let query = r#"
        SELECT id, device_id, user_id, user_session_id,
               session_identifier, started_at, last_activity,
               ip_address, location, risk_score, risk_factors,
               is_active, created_at, updated_at
        FROM device_sessions
        WHERE user_id = $1 AND is_active
        ORDER BY last_activity DESC
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id]).await?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "device_id": row.get::<_, Uuid>("device_id"),
            "user_id": row.get::<_, Uuid>("user_id"),
            "user_session_id": row.get::<_, Option<Uuid>>("user_session_id"),
            "session_identifier": row.get::<_, String>("session_identifier"),
            "started_at": row.get::<_, chrono::DateTime<chrono::Utc>>("started_at"),
            "last_activity": row.get::<_, chrono::DateTime<chrono::Utc>>("last_activity"),
            "ip_address": row.get::<_, Option<std::net::IpAddr>>("ip_address").map(|ip| ip.to_string()),
            "location": row.get::<_, Option<serde_json::Value>>("location"),
            "risk_score": row.get::<_, f64>("risk_score"),
            "risk_factors": row.get::<_, Option<serde_json::Value>>("risk_factors"),
            "is_active": row.get::<_, bool>("is_active"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
        }));
    }

    Ok(sessions)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
