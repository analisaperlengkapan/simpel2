//! Satker read-model view for administrative/read endpoints.
//!
//! Satker IDENTITY is owned by integrasi/MySIMKARI (SoT, #42); authenc does NOT
//! create/update/delete satker master data. This service is a **read-only view**
//! over an in-memory snapshot of the integrasi read-model (the same `Vec<Satker>`
//! used to build `SatkerAuthorizationService`), serving get/list/roots/hierarchy.
//! RBAC extensions (roles/permissions per satker) live elsewhere, keyed by code.

use authenc_types::Result;
use authenc_types::domain::satker::{Satker, SatkerHierarchy};
use authenc_types::error::AuthencError;

/// Read-only view over the satker read-model (identity SoT = integrasi).
pub struct SatkerManagementService {
    satkers: Vec<Satker>,
}

impl SatkerManagementService {
    /// Build the view from a read-model snapshot (from integrasi at startup).
    pub fn new(satkers: Vec<Satker>) -> Self {
        Self { satkers }
    }

    /// Retrieve a Satker by its unique code.
    pub async fn get_satker(&self, code: &str) -> Result<Option<Satker>> {
        Ok(self.satkers.iter().find(|s| s.code == code).cloned())
    }

    /// List Satkers with optional filtering by parent code, level, or search term.
    pub async fn list_satkers(
        &self,
        parent_code: Option<String>,
        level: Option<i32>,
        search: Option<String>,
    ) -> Result<Vec<Satker>> {
        let mut filtered: Vec<Satker> = self
            .satkers
            .iter()
            .filter(|s| match &parent_code {
                Some(p) => s.parent_code.as_deref() == Some(p.as_str()),
                None => true,
            })
            .filter(|s| level.map(|lvl| s.level == lvl).unwrap_or(true))
            .cloned()
            .collect();

        if let Some(query) = search {
            let q = query.to_lowercase();
            filtered.retain(|s| {
                s.code.to_lowercase().contains(&q)
                    || s.name.to_lowercase().contains(&q)
                    || s.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&q))
                        .unwrap_or(false)
            });
        }

        Ok(filtered)
    }

    /// Retrieve all root-level Satkers (those without parents).
    pub async fn get_root_satkers(&self) -> Result<Vec<Satker>> {
        Ok(self
            .satkers
            .iter()
            .filter(|s| s.parent_code.is_none())
            .cloned()
            .collect())
    }

    /// Get fully populated hierarchy details for a specific Satker.
    pub async fn get_satker_hierarchy_info(
        &self,
        code: &str,
    ) -> Result<super::SatkerHierarchyInfo> {
        let hierarchy = SatkerHierarchy::new(self.satkers.clone());

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
