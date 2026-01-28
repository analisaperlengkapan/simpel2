//! WebAuthn Database Operations
//!
//! Database operations for storing and retrieving WebAuthn credentials with attestation data.

use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::webauthn::WebauthnCredential;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Store a WebAuthn credential for a user
pub async fn store_credential(
    db: &Database,
    user_id: Uuid,
    credential: &WebauthnCredential,
) -> Result<()> {
    let client = db.get_connection().await?;

    let query = r#"
        INSERT INTO webauthn_credentials (
            id, user_id, credential_id, public_key, public_key_algorithm,
            signature_counter, attestation_object, authenticator_data,
            user_handle, credential_type, transports, aaguid,
            attestation_format, created_at, last_used_at, enabled
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (id) DO UPDATE SET
            signature_counter = EXCLUDED.signature_counter,
            last_used_at = EXCLUDED.last_used_at,
            enabled = EXCLUDED.enabled
    "#;

    client
        .execute(
            query,
            &[
                &credential.id,
                &user_id,
                &credential.credential_id,
                &credential.public_key,
                &credential.public_key_algorithm,
                &(credential.signature_counter as i32),
                &credential.attestation_object,
                &credential.authenticator_data,
                &credential.user_handle,
                &credential.credential_type,
                &credential.transports,
                &credential.aaguid,
                &credential.attestation_format,
                &credential.created_at,
                &credential.last_used_at,
                &credential.enabled,
            ],
        )
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to store WebAuthn credential: {}", e))
        })?;

    Ok(())
}

/// Get all WebAuthn credentials for a user
pub async fn get_user_credentials(db: &Database, user_id: Uuid) -> Result<Vec<WebauthnCredential>> {
    let client = db.get_connection().await?;

    let query = r#"
        SELECT
            id, user_id, credential_id, public_key, public_key_algorithm,
            signature_counter, attestation_object, authenticator_data,
            user_handle, credential_type, transports, aaguid,
            attestation_format, created_at, last_used_at, enabled
        FROM webauthn_credentials
        WHERE user_id = $1 AND enabled = true
        ORDER BY created_at DESC
    "#;

    let rows = client.query(query, &[&user_id]).await.map_err(|e| {
        AuthencError::internal(&format!("Failed to get WebAuthn credentials: {}", e))
    })?;

    let credentials = rows
        .iter()
        .map(|row| {
            Ok(WebauthnCredential {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                credential_id: row.try_get("credential_id")?,
                public_key: row.try_get("public_key")?,
                public_key_algorithm: row.try_get("public_key_algorithm")?,
                signature_counter: row.try_get::<_, i32>("signature_counter")? as u32,
                attestation_object: row.try_get("attestation_object")?,
                authenticator_data: row.try_get("authenticator_data")?,
                user_handle: row.try_get("user_handle")?,
                credential_type: row.try_get("credential_type")?,
                transports: row.try_get("transports")?,
                aaguid: row.try_get("aaguid")?,
                attestation_format: row.try_get("attestation_format")?,
                created_at: row.try_get("created_at")?,
                last_used_at: row.try_get("last_used_at")?,
                enabled: row.try_get("enabled")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(credentials)
}

/// Get a specific WebAuthn credential by credential ID
pub async fn get_credential_by_id(
    db: &Database,
    credential_id: &str,
) -> Result<Option<WebauthnCredential>> {
    let client = db.get_connection().await?;

    let query = r#"
        SELECT
            id, user_id, credential_id, public_key, public_key_algorithm,
            signature_counter, attestation_object, authenticator_data,
            user_handle, credential_type, transports, aaguid,
            attestation_format, created_at, last_used_at, enabled
        FROM webauthn_credentials
        WHERE credential_id = $1 AND enabled = true
    "#;

    let row_opt = client
        .query_opt(query, &[&hex::decode(credential_id).unwrap_or_default()])
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to get WebAuthn credential: {}", e))
        })?;

    if let Some(row) = row_opt {
        Ok(Some(WebauthnCredential {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            credential_id: row.try_get("credential_id")?,
            public_key: row.try_get("public_key")?,
            public_key_algorithm: row.try_get("public_key_algorithm")?,
            signature_counter: row.try_get::<_, i32>("signature_counter")? as u32,
            attestation_object: row.try_get("attestation_object")?,
            authenticator_data: row.try_get("authenticator_data")?,
            user_handle: row.try_get("user_handle")?,
            credential_type: row.try_get("credential_type")?,
            transports: row.try_get("transports")?,
            aaguid: row.try_get("aaguid")?,
            attestation_format: row.try_get("attestation_format")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
            enabled: row.try_get("enabled")?,
        }))
    } else {
        Ok(None)
    }
}

/// Update WebAuthn credential signature counter (for replay protection)
pub async fn update_signature_count(
    db: &Database,
    credential_id: &str,
    new_counter: i64,
) -> Result<()> {
    let client = db.get_connection().await?;

    let query = r#"
        UPDATE webauthn_credentials
        SET signature_counter = $1
        WHERE credential_id = $2 AND signature_counter < $1
    "#;

    let rows_affected = client
        .execute(
            query,
            &[
                &new_counter,
                &hex::decode(credential_id).unwrap_or_default(),
            ],
        )
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to update signature counter: {}", e))
        })?;

    if rows_affected == 0 {
        return Err(AuthencError::unauthorized(
            "Signature counter rollback detected - possible replay attack",
        ));
    }

    Ok(())
}

/// Update credential usage timestamp and counter
pub async fn update_credential_usage(
    db: &Database,
    credential_id: &str,
    new_counter: i64,
    last_used: DateTime<Utc>,
) -> Result<()> {
    let client = db.get_connection().await?;

    let query = r#"
        UPDATE webauthn_credentials
        SET signature_counter = $1,
            last_used_at = $2
        WHERE credential_id = $3 AND signature_counter < $1
    "#;

    let rows_affected = client
        .execute(
            query,
            &[
                &new_counter,
                &last_used,
                &hex::decode(credential_id).unwrap_or_default(),
            ],
        )
        .await
        .map_err(|e| {
            AuthencError::internal(&format!("Failed to update credential usage: {}", e))
        })?;

    if rows_affected == 0 {
        return Err(AuthencError::unauthorized(
            "Signature counter rollback detected - possible replay attack",
        ));
    }

    Ok(())
}

/// Disable a WebAuthn credential
pub async fn disable_credential(db: &Database, credential_id: &str) -> Result<()> {
    let client = db.get_connection().await?;

    let query = r#"
        UPDATE webauthn_credentials
        SET enabled = false
        WHERE credential_id = $1
    "#;

    client
        .execute(query, &[&hex::decode(credential_id).unwrap_or_default()])
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to disable credential: {}", e)))?;

    Ok(())
}

/// Delete a WebAuthn credential
pub async fn delete_credential(db: &Database, credential_id: &str) -> Result<()> {
    let client = db.get_connection().await?;

    let query = r#"
        DELETE FROM webauthn_credentials
        WHERE credential_id = $1
    "#;

    client
        .execute(query, &[&hex::decode(credential_id).unwrap_or_default()])
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to delete credential: {}", e)))?;

    Ok(())
}

/// Get credential count for a user
pub async fn get_user_credential_count(db: &Database, user_id: Uuid) -> Result<i64> {
    let client = db.get_connection().await?;

    let query = r#"
        SELECT COUNT(*) as count
        FROM webauthn_credentials
        WHERE user_id = $1 AND enabled = true
    "#;

    let row = client
        .query_one(query, &[&user_id])
        .await
        .map_err(|e| AuthencError::internal(&format!("Failed to count credentials: {}", e)))?;

    Ok(row.get("count"))
}
