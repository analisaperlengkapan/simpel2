/// Database operations for user consent management
use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::{ConsentGrantRequest, UserConsent},
};
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn grant_consent(
    db: &Database,
    user_id: Uuid,
    request: &ConsentGrantRequest,
) -> Result<UserConsent> {
    let consent_id = Uuid::new_v4();
    let granted_at = Utc::now();
    let expires_at = request
        .expires_in
        .map(|secs| granted_at + Duration::seconds(secs));
    let metadata = request
        .metadata
        .clone()
        .unwrap_or_else(|| serde_json::Value::Null);

    let query = r#"
        INSERT INTO user_consents (id, user_id, client_id, scopes, granted_at, expires_at, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (user_id, client_id)
        DO UPDATE SET
            scopes = EXCLUDED.scopes,
            granted_at = EXCLUDED.granted_at,
            expires_at = EXCLUDED.expires_at,
            metadata = EXCLUDED.metadata
        RETURNING id, user_id, client_id, scopes, granted_at, expires_at, metadata
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &consent_id,
                &user_id,
                &request.client_id,
                &request.scopes,
                &granted_at,
                &expires_at,
                &metadata,
            ],
        )
        .await?;

    Ok(UserConsent {
        id: row.get("id"),
        user_id: row.get("user_id"),
        client_id: row.get("client_id"),
        scopes: row.get("scopes"),
        granted_at: row.get("granted_at"),
        expires_at: row.get("expires_at"),
        metadata: row.get("metadata"),
    })
}

pub async fn revoke_consent(db: &Database, user_id: Uuid, client_id: &str) -> Result<()> {
    let query = "DELETE FROM user_consents WHERE user_id = $1 AND client_id = $2";

    let rows_affected = db.execute(query, &[&user_id, &client_id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::not_found(format!(
            "Consent for user {} and client {} not found",
            user_id, client_id
        )));
    }

    Ok(())
}

pub async fn revoke_consent_by_id(db: &Database, user_id: Uuid, consent_id: Uuid) -> Result<()> {
    let query = "DELETE FROM user_consents WHERE id = $1 AND user_id = $2";

    let rows_affected = db.execute(query, &[&consent_id, &user_id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::not_found(format!(
            "Consent with id {} for user {} not found",
            consent_id, user_id
        )));
    }

    Ok(())
}

pub async fn get_user_consents(db: &Database, user_id: Uuid) -> Result<Vec<UserConsent>> {
    let query = r#"
        SELECT id, user_id, client_id, scopes, granted_at, expires_at, metadata
        FROM user_consents
        WHERE user_id = $1
        AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY granted_at DESC
    "#;

    let rows = db.query(query, &[&user_id]).await?;

    rows.into_iter()
        .map(|row: tokio_postgres::Row| {
            Ok(UserConsent {
                id: row.get("id"),
                user_id: row.get("user_id"),
                client_id: row.get("client_id"),
                scopes: row.get("scopes"),
                granted_at: row.get("granted_at"),
                expires_at: row.get("expires_at"),
                metadata: row.get("metadata"),
            })
        })
        .collect()
}

pub async fn get_user_consent(
    db: &Database,
    user_id: Uuid,
    client_id: &str,
) -> Result<Option<UserConsent>> {
    let query = r#"
        SELECT id, user_id, client_id, scopes, granted_at, expires_at, metadata
        FROM user_consents
        WHERE user_id = $1 AND client_id = $2
        AND (expires_at IS NULL OR expires_at > NOW())
    "#;

    let row = db.query_opt(query, &[&user_id, &client_id]).await?;

    Ok(row.map(|row: tokio_postgres::Row| UserConsent {
        id: row.get("id"),
        user_id: row.get("user_id"),
        client_id: row.get("client_id"),
        scopes: row.get("scopes"),
        granted_at: row.get("granted_at"),
        expires_at: row.get("expires_at"),
        metadata: row.get("metadata"),
    }))
}

pub async fn has_consent(
    db: &Database,
    user_id: Uuid,
    client_id: &str,
    required_scopes: &[String],
) -> Result<bool> {
    let query = r#"
        SELECT scopes
        FROM user_consents
        WHERE user_id = $1 AND client_id = $2
        AND (expires_at IS NULL OR expires_at > NOW())
    "#;

    let row = db.query_opt(query, &[&user_id, &client_id]).await?;

    match row {
        Some(row) => {
            let granted_scopes: Vec<String> = row.get("scopes");
            Ok(required_scopes
                .iter()
                .all(|scope| granted_scopes.contains(scope)))
        }
        None => Ok(false),
    }
}

pub async fn cleanup_expired_consents(db: &Database) -> Result<i64> {
    let query = "DELETE FROM user_consents WHERE expires_at IS NOT NULL AND expires_at < NOW()";

    let rows_affected = db.execute(query, &[]).await?;
    Ok(rows_affected as i64)
}

pub async fn get_consent_stats(db: &Database, user_id: Uuid) -> Result<serde_json::Value> {
    let query = r#"
        SELECT
            COUNT(*) as total_consents,
            COUNT(*) FILTER (WHERE expires_at IS NULL) as permanent_consents,
            COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at > NOW()) as temporary_consents,
            COUNT(DISTINCT client_id) as unique_clients
        FROM user_consents
        WHERE user_id = $1
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&user_id]).await?;

    Ok(serde_json::json!({
        "total_consents": row.get::<_, i64>("total_consents"),
        "permanent_consents": row.get::<_, i64>("permanent_consents"),
        "temporary_consents": row.get::<_, i64>("temporary_consents"),
        "unique_clients": row.get::<_, i64>("unique_clients")
    }))
}
