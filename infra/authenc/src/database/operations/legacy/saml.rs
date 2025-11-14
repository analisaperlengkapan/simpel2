/// Database operations for SAML
use crate::{

    database::Database,
    error::Result,
    models::{SamlServiceProvider, SamlSession},
};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_service_provider(
    db: &Database,
    sp: &SamlServiceProvider,
) -> Result<SamlServiceProvider> {
    let sp_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO saml_service_providers (
            id, entity_id, metadata_url, metadata_xml,
            signing_certificate, encryption_certificate,
            assertion_consumer_service_url, single_logout_service_url,
            name_id_format, realm_id, enabled, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING
            id, entity_id, metadata_url, metadata_xml,
            signing_certificate, encryption_certificate,
            assertion_consumer_service_url, single_logout_service_url,
            name_id_format, realm_id, enabled, created_at, updated_at, deleted_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &sp_id,
                &sp.entity_id,
                &sp.metadata_url,
                &sp.metadata_xml,
                &sp.signing_certificate,
                &sp.encryption_certificate,
                &sp.assertion_consumer_service_url,
                &sp.single_logout_service_url,
                &sp.name_id_format,
                &sp.realm_id,
                &sp.enabled,
                &now,
                &now,
            ],
        )
        .await?;

    // Convert row to SamlServiceProvider
    row.try_into()
}

pub async fn get_service_provider_by_entity_id(
    db: &Database,
    entity_id: &str,
) -> Result<Option<SamlServiceProvider>> {
    let query = r#"
        SELECT
            id, entity_id, metadata_url, metadata_xml,
            signing_certificate, encryption_certificate,
            assertion_consumer_service_url, single_logout_service_url,
            name_id_format, realm_id, enabled, created_at, updated_at, deleted_at
        FROM saml_service_providers
        WHERE entity_id = $1 AND deleted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&entity_id]).await?;
    // Convert row to SamlServiceProvider
    Ok(Some(row.try_into()?))
}

pub async fn create_session(db: &Database, session: &SamlSession) -> Result<()> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO saml_sessions (
            id, session_id, user_id, identity_provider_id,
            service_provider_id, name_id, name_id_format,
            session_index, authn_instant, expires_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

    db.execute(
        query,
        &[
            &session_id,
            &session.session_id,
            &session.user_id,
            &session.identity_provider_id,
            &session.service_provider_id,
            &session.name_id,
            &session.name_id_format,
            &session.session_index,
            &session.authn_instant,
            &session.expires_at,
            &now,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_session_by_id(db: &Database, session_id: &str) -> Result<Option<SamlSession>> {
    let query = r#"
        SELECT
            id, session_id, user_id, identity_provider_id,
            service_provider_id, name_id, name_id_format,
            session_index, authn_instant, expires_at, created_at
        FROM saml_sessions
        WHERE session_id = $1 AND expires_at > NOW()
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&session_id]).await?;
    // Convert row to SamlSession
    Ok(Some(row.try_into()?))
}

pub async fn cleanup_expired_sessions(db: &Database) -> Result<u64> {
    let query = "DELETE FROM saml_sessions WHERE expires_at < NOW()";
    db.execute(query, &[]).await
}
