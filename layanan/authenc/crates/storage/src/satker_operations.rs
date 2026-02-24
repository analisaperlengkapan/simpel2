//! Satker (Government Organizational Unit) Database Operations
//!
//! Database operations for managing Satker hierarchy (Kejaksaan RI organizational structure)
//!
//! **Status**: MIGRATED BUT DISABLED
//! **Reason**: Depends on models not yet migrated:
//! - `models::satker::{Satker, SatkerType}`
//!
//! **TODO**: Enable after Phase 3 (models migration)

#![allow(dead_code, unused_imports)]

use crate::database::Database;
use authenc_types::AuthencError;
// use crate::models::satker::{Satker, SatkerType}; // TODO: Migrate models
use chrono::Utc;
use uuid::Uuid;

/*
impl Database {
    /// Get a satker by code
    pub async fn get_satker_by_code(&self, code: &str) -> Result<Option<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE code = $1 AND active = true
        ";

        let row = client.query_opt(query, &[&code]).await?;

        Ok(row.map(|r| Satker {
            id: r.get("id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            parent_code: r.get("parent_code"),
            level: r.get("level"),
            satker_type: serde_json::from_value(r.get("satker_type"))
                .unwrap_or(SatkerType::KejaksaanNegeri),
            active: r.get("active"),
            attributes: r.get("attributes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Get all satkers
    pub async fn get_all_satkers(&self) -> Result<Vec<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE active = true
            ORDER BY level ASC, code ASC
        ";

        let rows = client.query(query, &[]).await?;

        Ok(rows
            .into_iter()
            .map(|r| Satker {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                description: r.get("description"),
                parent_code: r.get("parent_code"),
                level: r.get("level"),
                satker_type: serde_json::from_value(r.get("satker_type"))
                    .unwrap_or(SatkerType::KejaksaanNegeri),
                active: r.get("active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Get satkers by parent code
    pub async fn get_satkers_by_parent(
        &self,
        parent_code: &str,
    ) -> Result<Vec<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE parent_code = $1 AND active = true
            ORDER BY code ASC
        ";

        let rows = client.query(query, &[&parent_code]).await?;

        Ok(rows
            .into_iter()
            .map(|r| Satker {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                description: r.get("description"),
                parent_code: r.get("parent_code"),
                level: r.get("level"),
                satker_type: serde_json::from_value(r.get("satker_type"))
                    .unwrap_or(SatkerType::KejaksaanNegeri),
                active: r.get("active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Get satkers at a specific level
    pub async fn get_satkers_by_level(&self, level: i32) -> Result<Vec<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE level = $1 AND active = true
            ORDER BY code ASC
        ";

        let rows = client.query(query, &[&level]).await?;

        Ok(rows
            .into_iter()
            .map(|r| Satker {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                description: r.get("description"),
                parent_code: r.get("parent_code"),
                level: r.get("level"),
                satker_type: serde_json::from_value(r.get("satker_type"))
                    .unwrap_or(SatkerType::KejaksaanNegeri),
                active: r.get("active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Create a new satker
    pub async fn create_satker(
        &self,
        code: String,
        name: String,
        description: Option<String>,
        parent_code: Option<String>,
        level: i32,
        satker_type: SatkerType,
        attributes: Option<serde_json::Value>,
    ) -> Result<Satker, AuthencError> {
        let client = self.get_connection().await?;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let satker_type_json = serde_json::to_value(&satker_type)?;

        let query = "
            INSERT INTO satkers (id, code, name, description, parent_code, level, satker_type,
                                active, attributes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, code, name, description, parent_code, level, satker_type,
                      active, attributes, created_at, updated_at
        ";

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &code,
                    &name,
                    &description,
                    &parent_code,
                    &level,
                    &satker_type_json,
                    &true, // active
                    &attributes,
                    &now,
                    &now,
                ],
            )
            .await?;

        Ok(Satker {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            parent_code: row.get("parent_code"),
            level: row.get("level"),
            satker_type: serde_json::from_value(row.get("satker_type"))
                .unwrap_or(SatkerType::KejaksaanNegeri),
            active: row.get("active"),
            attributes: row.get("attributes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Update a satker
    pub async fn update_satker(
        &self,
        code: &str,
        name: Option<String>,
        description: Option<String>,
        parent_code: Option<String>,
        level: Option<i32>,
        satker_type: Option<SatkerType>,
        attributes: Option<serde_json::Value>,
    ) -> Result<Satker, AuthencError> {
        let client = self.get_connection().await?;

        // Get current satker
        let current = self
            .get_satker_by_code(code)
            .await?
            .ok_or_else(|| AuthencError::not_found(format!("Satker '{}' not found", code)))?;

        let updated_name = name.unwrap_or(current.name);
        let updated_description = description.or(current.description);
        let updated_parent_code = parent_code.or(current.parent_code);
        let updated_level = level.unwrap_or(current.level);
        let updated_satker_type = satker_type.unwrap_or(current.satker_type);
        let updated_attributes = attributes.or(current.attributes);
        let now = Utc::now();
        let satker_type_json = serde_json::to_value(&updated_satker_type)?;

        let query = "
            UPDATE satkers
            SET name = $1, description = $2, parent_code = $3, level = $4,
                satker_type = $5, attributes = $6, updated_at = $7
            WHERE code = $8
            RETURNING id, code, name, description, parent_code, level, satker_type,
                      active, attributes, created_at, updated_at
        ";

        let row = client
            .query_one(
                query,
                &[
                    &updated_name,
                    &updated_description,
                    &updated_parent_code,
                    &updated_level,
                    &satker_type_json,
                    &updated_attributes,
                    &now,
                    &code,
                ],
            )
            .await?;

        Ok(Satker {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            parent_code: row.get("parent_code"),
            level: row.get("level"),
            satker_type: serde_json::from_value(row.get("satker_type"))
                .unwrap_or(SatkerType::KejaksaanNegeri),
            active: row.get("active"),
            attributes: row.get("attributes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Deactivate a satker (soft delete)
    pub async fn deactivate_satker(&self, code: &str) -> Result<(), AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            UPDATE satkers
            SET active = false, updated_at = $1
            WHERE code = $2
        ";

        client.execute(query, &[&Utc::now(), &code]).await?;

        Ok(())
    }

    /// Reactivate a satker
    pub async fn reactivate_satker(&self, code: &str) -> Result<(), AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            UPDATE satkers
            SET active = true, updated_at = $1
            WHERE code = $2
        ";

        client.execute(query, &[&Utc::now(), &code]).await?;

        Ok(())
    }

    /// Get root satkers (no parent)
    pub async fn get_root_satkers(&self) -> Result<Vec<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE parent_code IS NULL AND active = true
            ORDER BY code ASC
        ";

        let rows = client.query(query, &[]).await?;

        Ok(rows
            .into_iter()
            .map(|r| Satker {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                description: r.get("description"),
                parent_code: r.get("parent_code"),
                level: r.get("level"),
                satker_type: serde_json::from_value(r.get("satker_type"))
                    .unwrap_or(SatkerType::KejaksaanNegeri),
                active: r.get("active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Search satkers by name or code
    pub async fn search_satkers(&self, query_str: &str) -> Result<Vec<Satker>, AuthencError> {
        let client = self.get_connection().await?;

        let search_pattern = format!("%{}%", query_str);

        let query = "
            SELECT id, code, name, description, parent_code, level, satker_type,
                   active, attributes, created_at, updated_at
            FROM satkers
            WHERE (code ILIKE $1 OR name ILIKE $1) AND active = true
            ORDER BY level ASC, code ASC
            LIMIT 100
        ";

        let rows = client.query(query, &[&search_pattern]).await?;

        Ok(rows
            .into_iter()
            .map(|r| Satker {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                description: r.get("description"),
                parent_code: r.get("parent_code"),
                level: r.get("level"),
                satker_type: serde_json::from_value(r.get("satker_type"))
                    .unwrap_or(SatkerType::KejaksaanNegeri),
                active: r.get("active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
}
*/
