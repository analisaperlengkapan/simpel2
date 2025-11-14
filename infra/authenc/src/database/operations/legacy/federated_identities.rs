/// Database operations for federated identity management
use crate::{

    database::Database,
    error::Result,
    models::user::{CreateFederatedIdentityRequest, FederatedIdentity},
};
use uuid::Uuid;

pub async fn create_federated_identity(
    db: &Database,
    request: &CreateFederatedIdentityRequest,
) -> Result<FederatedIdentity> {
    let external_attributes_json = request
        .external_attributes
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    let query = r#"
        INSERT INTO federated_identities (
            user_id, identity_provider_id, external_id, external_username,
            external_email, external_attributes
        )
        VALUES ($1, $2, $3, $4, $5, $6::jsonb)
        RETURNING id, user_id, identity_provider_id, external_id, external_username,
                  external_email, external_attributes, last_login_at, created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &request.user_id,
                &request.identity_provider_id,
                &request.external_id,
                &request.external_username,
                &request.external_email,
                &external_attributes_json,
            ],
        )
        .await?;

    Ok(FederatedIdentity {
        id: row.get(0),
        user_id: row.get(1),
        identity_provider_id: row.get(2),
        external_id: row.get(3),
        external_username: row.get(4),
        external_email: row.get(5),
        external_attributes: row
            .get::<_, Option<String>>(6)
            .and_then(|s: String| serde_json::from_str(&s).ok()),
        last_login_at: row.get(7),
        created_at: row.get(8),
        updated_at: row.get(9),
    })
}

pub async fn get_federated_identity_by_external_id(
    db: &Database,
    identity_provider_id: Uuid,
    external_id: &str,
) -> Result<Option<FederatedIdentity>> {
    let query = r#"
        SELECT id, user_id, identity_provider_id, external_id, external_username,
               external_email, external_attributes, last_login_at, created_at, updated_at
        FROM federated_identities
        WHERE identity_provider_id = $1 AND external_id = $2
    "#;

    let rows = db
        .query(query, &[&identity_provider_id, &external_id])
        .await?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r: tokio_postgres::Row| FederatedIdentity {
            id: r.get(0),
            user_id: r.get(1),
            identity_provider_id: r.get(2),
            external_id: r.get(3),
            external_username: r.get(4),
            external_email: r.get(5),
            external_attributes: r
                .get::<_, Option<String>>(6)
                .and_then(|s: String| serde_json::from_str(&s).ok()),
            last_login_at: r.get(7),
            created_at: r.get(8),
            updated_at: r.get(9),
        }))
}

pub async fn get_federated_identities_by_user(
    db: &Database,
    user_id: Uuid,
) -> Result<Vec<FederatedIdentity>> {
    let query = r#"
        SELECT id, user_id, identity_provider_id, external_id, external_username,
               external_email, external_attributes, last_login_at, created_at, updated_at
        FROM federated_identities
        WHERE user_id = $1
        ORDER BY created_at
    "#;

    let rows: Vec<tokio_postgres::Row> = db.query(query, &[&user_id]).await?;
    let mut identities = Vec::new();

    for row in rows {
        identities.push(FederatedIdentity {
            id: row.get(0),
            user_id: row.get(1),
            identity_provider_id: row.get(2),
            external_id: row.get(3),
            external_username: row.get(4),
            external_email: row.get(5),
            external_attributes: row
                .get::<_, Option<String>>(6)
                .and_then(|s: String| serde_json::from_str(&s).ok()),
            last_login_at: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        });
    }

    Ok(identities)
}

pub async fn update_last_login(db: &Database, federated_identity_id: Uuid) -> Result<()> {
    let query = r#"
        UPDATE federated_identities
        SET last_login_at = NOW(), updated_at = NOW()
        WHERE id = $1
    "#;

    db.execute(query, &[&federated_identity_id]).await?;
    Ok(())
}

pub async fn delete_federated_identity(
    db: &Database,
    federated_identity_id: Uuid,
) -> Result<()> {
    let query = r#"
        DELETE FROM federated_identities
        WHERE id = $1
    "#;

    db.execute(query, &[&federated_identity_id]).await?;
    Ok(())
}
