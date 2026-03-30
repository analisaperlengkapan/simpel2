use crate::Database;
use authenc_types::domain::{
    CreateProtocolMapperRequest, ProtocolMapper, ProtocolMapperType, UpdateProtocolMapperRequest,
};
use authenc_types::{AuthencError, Result};
use chrono::Utc;
use uuid::Uuid;

/// Protocol mapper database operations module
pub mod protocol_mappers {
    use super::*;

    /// Create a new protocol mapper
    pub async fn create(
        db: &Database,
        realm_id: Uuid,
        request: CreateProtocolMapperRequest,
    ) -> Result<ProtocolMapper> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let config_json = serde_json::to_value(&request.config)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize config: {}", e)))?;

        let query = r#"
            INSERT INTO protocol_mappers (
                id, client_id, realm_id, name, protocol, mapper_type,
                config, enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
        "#;

        let row: tokio_postgres::Row = db
            .query_one(
                query,
                &[
                    &id,
                    &request.client_id,
                    &realm_id,
                    &request.name,
                    &request.protocol,
                    &request.mapper_type.to_string(),
                    &config_json,
                    &true, // enabled by default
                    &now,
                    &now,
                ],
            )
            .await?;

        ProtocolMapper::try_from(row)
    }

    /// Get protocol mapper by ID
    pub async fn get_by_id(db: &Database, id: Uuid) -> Result<Option<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE id = $1
        "#;

        match db.query_opt(query, &[&id]).await? {
            Some(row) => Ok(Some(ProtocolMapper::try_from(row)?)),
            None => Ok(None),
        }
    }

    /// Get protocol mapper by name within a realm
    pub async fn get_by_name(
        db: &Database,
        realm_id: Uuid,
        name: &str,
    ) -> Result<Option<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE realm_id = $1 AND name = $2
        "#;

        match db.query_opt(query, &[&realm_id, &name]).await? {
            Some(row) => Ok(Some(ProtocolMapper::try_from(row)?)),
            None => Ok(None),
        }
    }

    /// List all protocol mappers in a realm
    pub async fn list_by_realm(db: &Database, realm_id: Uuid) -> Result<Vec<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE realm_id = $1
            ORDER BY name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id]).await?;
        rows.into_iter()
            .map(ProtocolMapper::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// List protocol mappers by client ID
    pub async fn list_by_client(db: &Database, client_id: Uuid) -> Result<Vec<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE client_id = $1 AND enabled = TRUE
            ORDER BY name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_id]).await?;
        rows.into_iter()
            .map(ProtocolMapper::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// List protocol mappers by client scope ID
    pub async fn list_by_client_scope(
        db: &Database,
        client_scope_id: Uuid,
    ) -> Result<Vec<ProtocolMapper>> {
        let query = r#"
            SELECT pm.* FROM protocol_mappers pm
            JOIN client_scope_mappings csm ON csm.protocol_mapper_id = pm.id
            WHERE csm.scope_id = $1 AND pm.enabled = TRUE
            ORDER BY pm.name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&client_scope_id]).await?;
        rows.into_iter()
            .map(ProtocolMapper::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// List protocol mappers by protocol
    pub async fn list_by_protocol(
        db: &Database,
        realm_id: Uuid,
        protocol: &str,
    ) -> Result<Vec<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE realm_id = $1 AND protocol = $2 AND enabled = TRUE
            ORDER BY name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db.query(query, &[&realm_id, &protocol]).await?;
        rows.into_iter()
            .map(ProtocolMapper::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// List protocol mappers by type
    pub async fn list_by_type(
        db: &Database,
        realm_id: Uuid,
        mapper_type: ProtocolMapperType,
    ) -> Result<Vec<ProtocolMapper>> {
        let query = r#"
            SELECT * FROM protocol_mappers
            WHERE realm_id = $1 AND mapper_type = $2 AND enabled = TRUE
            ORDER BY name ASC
        "#;

        let rows: Vec<tokio_postgres::Row> = db
            .query(query, &[&realm_id, &mapper_type.to_string()])
            .await?;
        rows.into_iter()
            .map(ProtocolMapper::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Update protocol mapper
    pub async fn update(
        db: &Database,
        id: Uuid,
        request: UpdateProtocolMapperRequest,
    ) -> Result<ProtocolMapper> {
        let now = Utc::now();

        // Check if there are any fields to update
        if request.name.is_none() && request.config.is_none() && request.enabled.is_none() {
            return Err(AuthencError::validation("No fields to update"));
        }

        // We can't easily use dynamic building with ToSql parameter lists containing a mix of
        // owned values (like our serialized JSON) and references without boxing or trait objects
        // that complicate lifetimes.
        //
        // Instead, we will construct a single query with COALESCE for optional fields.
        let query = r#"
            UPDATE protocol_mappers
            SET
                name = COALESCE($1, name),
                config = COALESCE($2, config),
                enabled = COALESCE($3, enabled),
                updated_at = $4
            WHERE id = $5
            RETURNING *
        "#;

        // Serialize config if present
        let config_json = match &request.config {
            Some(config) => Some(serde_json::to_value(config).map_err(|e| {
                AuthencError::internal(format!("Failed to serialize config: {}", e))
            })?),
            None => None,
        };

        let row: tokio_postgres::Row = db
            .query_one(
                query,
                &[&request.name, &config_json, &request.enabled, &now, &id],
            )
            .await?;

        ProtocolMapper::try_from(row)
    }

    /// Delete protocol mapper
    pub async fn delete(db: &Database, id: Uuid) -> Result<()> {
        let query = r#"
            DELETE FROM protocol_mappers
            WHERE id = $1
        "#;

        db.execute(query, &[&id]).await?;
        Ok(())
    }

    /// Get effective protocol mappers for a client (includes client-level and scope-level mappers)
    pub async fn get_effective_mappers_for_client(
        db: &Database,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<Vec<ProtocolMapper>> {
        // Get client-level mappers
        let mut mappers = list_by_client(db, client_id).await?;

        // Get scope-level mappers
        if !scopes.is_empty() {
            let query = r#"
                SELECT DISTINCT pm.*
                FROM protocol_mappers pm
                JOIN client_scope_mappings csm ON csm.protocol_mapper_id = pm.id
                JOIN client_scopes cs ON cs.id = csm.scope_id
                WHERE cs.name = ANY($1) AND pm.enabled = TRUE
                ORDER BY pm.name ASC
            "#;

            let rows: Vec<tokio_postgres::Row> = db.query(query, &[&scopes]).await?;
            let scope_mappers: Vec<ProtocolMapper> = rows
                .into_iter()
                .map(ProtocolMapper::try_from)
                .collect::<Result<Vec<_>>>()?;

            mappers.extend(scope_mappers);
        }

        // Deduplicate by mapper ID
        mappers.sort_by_key(|m| m.id);
        mappers.dedup_by_key(|m| m.id);

        Ok(mappers)
    }

    /// Initialize standard protocol mappers for a realm
    pub async fn initialize_standard_mappers(
        db: &Database,
        realm_id: Uuid,
    ) -> Result<Vec<ProtocolMapper>> {
        use authenc_types::domain::standard_mappers::*;

        let standard_mappers = vec![
            username_mapper(realm_id),
            email_mapper(realm_id),
            full_name_mapper(realm_id),
            realm_roles_mapper(realm_id),
            groups_mapper(realm_id),
        ];

        let mut created_mappers = Vec::new();

        for mapper in standard_mappers {
            // Check if already exists
            if get_by_name(db, realm_id, &mapper.name).await?.is_some() {
                continue;
            }

            let config_json = serde_json::to_value(&mapper.config).map_err(|e| {
                AuthencError::internal(format!("Failed to serialize config: {}", e))
            })?;

            let query = r#"
                INSERT INTO protocol_mappers (
                    id, client_id, realm_id, name, protocol, mapper_type,
                    config, enabled, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            "#;

            let row: tokio_postgres::Row = db
                .query_one(
                    query,
                    &[
                        &mapper.id,
                        &mapper.client_id,
                        &realm_id,
                        &mapper.name,
                        &mapper.protocol,
                        &mapper.mapper_type.to_string(),
                        &config_json,
                        &mapper.enabled,
                        &mapper.created_at,
                        &mapper.updated_at,
                    ],
                )
                .await?;

            created_mappers.push(ProtocolMapper::try_from(row)?);
        }

        Ok(created_mappers)
    }
}

// Re-export for convenience
pub use protocol_mappers::*;
