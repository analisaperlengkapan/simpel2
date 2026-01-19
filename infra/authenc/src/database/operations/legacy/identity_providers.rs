/// Database operations for identity provider management
use crate::{database::Database, error::Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Mewakili struktur data `IdentityProviderData`.
#[derive(Debug, Clone)]
pub struct IdentityProviderData {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub realm_id: Uuid,
    pub config: Value,
    pub truststore_path: Option<String>,
    pub keystore_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_identity_provider(
    db: &Database,
    name: &str,
    display_name: &str,
    provider_type: &str,
    enabled: bool,
    realm_id: Uuid,
    config: Value,
    truststore_path: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<IdentityProviderData> {
    let config_json = serde_json::to_string(&config)?;

    let query = r#"
        INSERT INTO identity_providers (
            name, display_name, provider_type, enabled, realm_id,
            config, truststore_path, keystore_path
        )
        VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8)
        RETURNING id, name, display_name, provider_type, enabled, realm_id,
                  config, truststore_path, keystore_path, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &name,
                &display_name,
                &provider_type,
                &enabled,
                &realm_id,
                &config_json,
                &truststore_path,
                &keystore_path,
            ],
        )
        .await?;

    Ok(IdentityProviderData {
        id: row.get(0),
        name: row.get(1),
        display_name: row.get(2),
        provider_type: row.get(3),
        enabled: row.get(4),
        realm_id: row.get(5),
        config: {
            let json_str: String = row.get(6);
            serde_json::from_str(&json_str)?
        },
        truststore_path: row.get(7),
        keystore_path: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    })
}

pub async fn get_identity_provider_by_id(
    db: &Database,
    provider_id: Uuid,
) -> Result<Option<IdentityProviderData>> {
    let query = r#"
        SELECT id, name, display_name, provider_type, enabled, realm_id,
               config, truststore_path, keystore_path, created_at, updated_at
        FROM identity_providers
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&provider_id]).await?;
    if rows.is_empty() {
        return Ok(None);
    }

    let row: &tokio_postgres::Row = &rows[0];
    Ok(Some(IdentityProviderData {
        id: row.get(0),
        name: row.get(1),
        display_name: row.get(2),
        provider_type: row.get(3),
        enabled: row.get(4),
        realm_id: row.get(5),
        config: {
            let json_str: String = row.get(6);
            serde_json::from_str(&json_str)?
        },
        truststore_path: row.get(7),
        keystore_path: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    }))
}

pub async fn get_identity_providers_by_realm(
    db: &Database,
    realm_id: Uuid,
) -> Result<Vec<IdentityProviderData>> {
    let query = r#"
        SELECT id, name, display_name, provider_type, enabled, realm_id,
               config, truststore_path, keystore_path, created_at, updated_at
        FROM identity_providers
        WHERE realm_id = $1 AND deleted_at IS NULL
        ORDER BY display_name
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id]).await?;
    let mut providers = Vec::new();

    for row in rows {
        providers.push(IdentityProviderData {
            id: row.get(0),
            name: row.get(1),
            display_name: row.get(2),
            provider_type: row.get(3),
            enabled: row.get(4),
            realm_id: row.get(5),
            config: {
                let json_str: String = row.get(6);
                serde_json::from_str(&json_str)?
            },
            truststore_path: row.get(7),
            keystore_path: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        });
    }

    Ok(providers)
}

pub async fn update_identity_provider(
    db: &Database,
    provider_id: Uuid,
    name: Option<&str>,
    display_name: Option<&str>,
    provider_type: Option<&str>,
    enabled: Option<bool>,
    config: Option<Value>,
    truststore_path: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<IdentityProviderData> {
    let config_json = config.as_ref().map(serde_json::to_string).transpose()?;

    let query = r#"
        UPDATE identity_providers
        SET name = COALESCE($2, name),
            display_name = COALESCE($3, display_name),
            provider_type = COALESCE($4, provider_type),
            enabled = COALESCE($5, enabled),
            config = COALESCE($6::jsonb, config),
            truststore_path = COALESCE($7, truststore_path),
            keystore_path = COALESCE($8, keystore_path),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, name, display_name, provider_type, enabled, realm_id,
                  config, truststore_path, keystore_path, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &provider_id,
                &name,
                &display_name,
                &provider_type,
                &enabled,
                &config_json,
                &truststore_path,
                &keystore_path,
            ],
        )
        .await?;

    Ok(IdentityProviderData {
        id: row.get(0),
        name: row.get(1),
        display_name: row.get(2),
        provider_type: row.get(3),
        enabled: row.get(4),
        realm_id: row.get(5),
        config: {
            let json_str: String = row.get(6);
            serde_json::from_str(&json_str)?
        },
        truststore_path: row.get(7),
        keystore_path: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    })
}

pub async fn delete_identity_provider(db: &Database, provider_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE identity_providers
        SET deleted_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    db.execute(query, &[&provider_id]).await?;
    Ok(())
}

pub async fn identity_provider_exists_and_enabled(
    db: &Database,
    provider_id: Uuid,
) -> Result<bool> {
    let query = r#"
        SELECT EXISTS(
            SELECT 1 FROM identity_providers
            WHERE id = $1 AND enabled = true AND deleted_at IS NULL
        )
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&provider_id]).await?;
    Ok(row.get(0))
}
