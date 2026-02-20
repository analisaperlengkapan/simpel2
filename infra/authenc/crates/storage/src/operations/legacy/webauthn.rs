/// Database operations for WebAuthn credentials
use crate::{database::Database, error::Result, models::WebauthnCredential};
use chrono::Utc;
use uuid::Uuid;

pub async fn store_credential(
    db: &Database,
    user_id: Uuid,
    credential: &WebauthnCredential,
) -> Result<()> {
    let credential_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO webauthn_credentials (
            id, user_id, credential_id, public_key, public_key_algorithm,
            signature_counter, attestation_object, authenticator_data,
            user_handle, credential_type, transports, created_at, last_used_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    "#;

    db.execute(
        query,
        &[
            &credential_id,
            &user_id,
            &credential.credential_id,
            &credential.public_key,
            &credential.public_key_algorithm,
            &credential.signature_counter,
            &credential.attestation_object,
            &credential.authenticator_data,
            &credential.user_handle,
            &credential.credential_type,
            &credential.transports,
            &now,
            &credential.last_used_at,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_credential_by_id(
    db: &Database,
    credential_id: &str,
) -> Result<Option<WebauthnCredential>> {
    let query = r#"
        SELECT
            id, user_id, credential_id, public_key, public_key_algorithm,
            attestation_object, authenticator_data, user_handle,
            signature_counter, credential_type, transports,
            aaguid, attestation_format, created_at, last_used_at, enabled
        FROM webauthn_credentials
        WHERE credential_id = $1
    "#;

    let row = db.query(query, &[&credential_id]).await?;
    let rows = row;
    Ok(if rows.is_empty() {
        None
    } else {
        let r: &tokio_postgres::Row = &rows[0];
        Some(WebauthnCredential {
            id: r.get(0),
            user_id: r.get(1),
            credential_id: r.get(2),
            public_key: r.get(3),
            public_key_algorithm: r.get(4),
            signature_counter: r.get(5),
            attestation_object: r.get(6),
            authenticator_data: r.get(7),
            user_handle: r.get(8),
            credential_type: r.get(9),
            transports: r.get(10),
            aaguid: r.get(11),
            attestation_format: r.get(12),
            created_at: r.get(13),
            last_used_at: r.get(14),
            enabled: r.get(15),
        })
    })
}

pub async fn get_user_credentials(db: &Database, user_id: Uuid) -> Result<Vec<WebauthnCredential>> {
    let query = r#"
        SELECT
            id, user_id, credential_id, public_key, public_key_algorithm,
            attestation_object, authenticator_data, user_handle,
            signature_counter, credential_type, transports,
            aaguid, attestation_format, created_at, last_used_at, enabled
        FROM webauthn_credentials
        WHERE user_id = $1
        ORDER BY created_at DESC
    "#;

    let rows = db.query(query, &[&user_id]).await?;
    let credentials: Vec<WebauthnCredential> = rows
        .into_iter()
        .map(|row: tokio_postgres::Row| WebauthnCredential {
            id: row.get(0),
            user_id: row.get(1),
            credential_id: row.get(2),
            public_key: row.get(3),
            public_key_algorithm: row.get(4),
            signature_counter: row.get(5),
            attestation_object: row.get(6),
            authenticator_data: row.get(7),
            user_handle: row.get(8),
            credential_type: row.get(9),
            transports: row.get(10),
            aaguid: row.get(11),
            attestation_format: row.get(12),
            created_at: row.get(13),
            last_used_at: row.get(14),
            enabled: row.get(15),
        })
        .collect();
    Ok(credentials)
}

pub async fn update_signature_count(
    db: &Database,
    credential_id: &str,
    new_count: i64,
) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE webauthn_credentials
        SET signature_counter = $2, last_used_at = $3
        WHERE credential_id = $1
    "#;
    db.execute(query, &[&credential_id, &new_count, &now])
        .await?;
    Ok(())
}

pub async fn update_credential_usage(
    db: &Database,
    credential_id: &str,
    new_counter: i64,
    last_used: chrono::DateTime<Utc>,
) -> Result<()> {
    let query = r#"
        UPDATE webauthn_credentials
        SET signature_counter = $1,
            last_used_at = $2
        WHERE credential_id = $3 AND signature_counter < $1
    "#;

    let rows_affected = db
        .execute(query, &[&new_counter, &last_used, &credential_id])
        .await?;

    if rows_affected == 0 {
        use authenc_types::AuthencError;
        return Err(AuthencError::unauthorized(
            "Signature counter rollback detected - possible replay attack",
        ));
    }

    Ok(())
}

pub async fn delete_credential(db: &Database, credential_id: &str) -> Result<()> {
    let query = "DELETE FROM webauthn_credentials WHERE credential_id = $1";
    db.execute(query, &[&credential_id]).await?;
    Ok(())
}

pub async fn delete_user_credentials(db: &Database, user_id: Uuid) -> Result<()> {
    let query = "DELETE FROM webauthn_credentials WHERE user_id = $1";
    db.execute(query, &[&user_id]).await?;
    Ok(())
}
