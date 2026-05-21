//! Satker management service for administrative operations
//!
//! Provides CRUD operations for the Satker organizational hierarchy and interfaces
//! directly with the authenc-storage backend.

use std::sync::Arc;
use uuid::Uuid;

use authenc_storage::Database;
use authenc_types::domain::satker::{Satker, SatkerType, SatkerHierarchy};
use authenc_types::error::AuthencError;
use authenc_types::Result;

/// Service for managing Satker organizational structures
pub struct SatkerManagementService {
    db: Database,
}

impl SatkerManagementService {
    /// Create a new Satker management service instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Retrieve a Satker by its unique code
    pub async fn get_satker(&self, code: &str) -> Result<Option<Satker>> {
        self.db.get_satker_by_code(code).await
    }

    /// List Satkers with optional filtering by parent code, level, or search term
    pub async fn list_satkers(
        &self,
        parent_code: Option<String>,
        level: Option<i32>,
        search: Option<String>,
    ) -> Result<Vec<Satker>> {
        // Fetch base list based on parent filtering
        let satkers = if let Some(parent) = &parent_code {
            self.db.get_satkers_by_parent(parent).await?
        } else {
            self.db.get_all_satkers().await?
        };

        // Filter by level if specified
        let mut filtered = if let Some(lvl) = level {
            satkers.into_iter().filter(|s| s.level == lvl).collect()
        } else {
            satkers
        };

        // Apply text search if specified
        if let Some(query) = search {
            let query_lower = query.to_lowercase();
            filtered = filtered
                .into_iter()
                .filter(|s| {
                    s.code.to_lowercase().contains(&query_lower)
                        || s.name.to_lowercase().contains(&query_lower)
                        || s.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .collect();
        }

        Ok(filtered)
    }

    /// Retrieve all root-level Satkers (those without parents)
    pub async fn get_root_satkers(&self) -> Result<Vec<Satker>> {
        self.db.get_root_satkers().await
    }

    /// Create a new Satker entry
    pub async fn create_satker(
        &self,
        code: String,
        name: String,
        description: Option<String>,
        parent_code: Option<String>,
        level: i32,
        satker_type: SatkerType,
        attributes: Option<serde_json::Value>,
    ) -> Result<Satker> {
        self.db
            .create_satker(code, name, description, parent_code, level, satker_type, attributes)
            .await
    }

    /// Update an existing Satker entry
    pub async fn update_satker(
        &self,
        code: &str,
        name: Option<String>,
        description: Option<String>,
        parent_code: Option<String>,
        level: Option<i32>,
        satker_type: Option<SatkerType>,
        attributes: Option<serde_json::Value>,
    ) -> Result<Satker> {
        self.db
            .update_satker(code, name, description, parent_code, level, satker_type, attributes)
            .await
    }

    /// Delete a Satker entry
    pub async fn delete_satker(&self, code: &str) -> Result<()> {
        self.db.deactivate_satker(code).await
    }

    /// Get fully populated hierarchy details for a specific Satker
    pub async fn get_satker_hierarchy_info(
        &self,
        code: &str,
    ) -> Result<super::SatkerHierarchyInfo> {
        let all = self.db.get_all_satkers().await?;
        let hierarchy = SatkerHierarchy::new(all);

        let satker = hierarchy.get_satker(code).ok_or_else(|| {
            AuthencError::not_found(format!("Satker dengan kode '{}' tidak ditemukan", code))
        })?;

        let parent = satker
            .parent_code
            .as_ref()
            .and_then(|p_code| hierarchy.get_satker(p_code).cloned());

        let children = hierarchy
            .get_children(code)
            .into_iter()
            .filter_map(|c_code| hierarchy.get_satker(c_code).cloned())
            .collect();

        let ancestors = hierarchy
            .get_ancestors(code)
            .into_iter()
            .filter_map(|a_code| hierarchy.get_satker(&a_code).cloned())
            .collect();

        let descendants = hierarchy
            .get_descendants(code)
            .into_iter()
            .filter_map(|d_code| hierarchy.get_satker(&d_code).cloned())
            .collect();

        Ok(super::SatkerHierarchyInfo {
            satker: satker.clone(),
            parent,
            children,
            ancestors,
            descendants,
            level: satker.level,
        })
    }
}
