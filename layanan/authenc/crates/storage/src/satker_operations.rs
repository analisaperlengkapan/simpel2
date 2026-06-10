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
use authenc_types::domain::Satker;
use authenc_types::domain::satker::SatkerType;

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
