//! Namespace Hierarchy Management for SIMKARI
//!
//! Implements hierarchical namespace structure for Kejaksaan RI:
//! - Pusat (Central) - Top level
//! - Wilayah (Regional) - Provincial level
//! - Satker (Work Unit) - Individual units
//!
//! This enables proper multi-tenancy with hierarchical access control
//! based on organizational structure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::CoreError;

/// Namespace hierarchy manager for SIMKARI organizational structure
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `NamespaceHierarchy`.
pub struct NamespaceHierarchy {
    /// Root namespace (Pusat/Central)
    pub root: Namespace,

    /// Map of wilayah (regional) namespaces by ID
    pub wilayah_map: HashMap<String, Namespace>,

    /// Map of satker (work unit) namespaces by ID
    pub satker_map: HashMap<String, Namespace>,

    /// All namespaces indexed by ID for quick lookup
    pub all_namespaces: HashMap<String, Namespace>,
}

/// Individual namespace representing an organizational unit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Mewakili pub `Namespace`.
pub struct Namespace {
    /// Unique namespace identifier (e.g., "pusat", "wilayah-sumut", "satker-kja001")
    pub id: String,

    /// Full hierarchical path (e.g., "pusat/wilayah-sumut/satker-kja001")
    pub path: String,

    /// Parent namespace ID (None for root)
    pub parent: Option<String>,

    /// Display name (e.g., "Kejaksaan Negeri Medan")
    pub name: String,

    /// Namespace type (Pusat, Wilayah, Satker)
    pub namespace_type: NamespaceType,

    /// Associated policies for this namespace
    pub policies: Vec<String>,

    /// Resource quotas for this namespace
    pub quotas: NamespaceQuotas,

    /// Metadata for additional information
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Created by user ID
    pub created_by: String,

    /// Active status
    pub is_active: bool,
}

/// Type of namespace in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Mewakili pub `NamespaceType`.
pub enum NamespaceType {
    /// Central/National level (Kejaksaan Agung)
    Pusat,

    /// Regional level (Kejaksaan Tinggi)
    Wilayah,

    /// Work unit level (Kejaksaan Negeri, etc.)
    Satker,
}

/// Resource quotas for namespace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Mewakili pub `NamespaceQuotas`.
pub struct NamespaceQuotas {
    /// Maximum number of secrets
    pub max_secrets: Option<u64>,

    /// Maximum storage size in bytes
    pub max_storage_bytes: Option<u64>,

    /// Maximum number of leases
    pub max_leases: Option<u64>,

    /// Maximum number of policies
    pub max_policies: Option<u64>,

    /// Current usage statistics
    pub current_usage: QuotaUsage,
}

/// Current quota usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
/// Mewakili pub `QuotaUsage`.
pub struct QuotaUsage {
    /// Current number of secrets
    pub secrets_count: u64,

    /// Current storage usage in bytes
    pub storage_bytes: u64,

    /// Current number of leases
    pub leases_count: u64,

    /// Current number of policies
    pub policies_count: u64,
}

impl NamespaceHierarchy {
    /// Create a new namespace hierarchy with root (Pusat)
    pub fn new(root_name: String, created_by: String) -> Self {
        let root = Namespace::new_root(root_name, created_by);
        let root_id = root.id.clone();

        let mut all_namespaces = HashMap::new();
        all_namespaces.insert(root_id, root.clone());

        Self {
            root,
            wilayah_map: HashMap::new(),
            satker_map: HashMap::new(),
            all_namespaces,
        }
    }

    /// Add a wilayah (regional) namespace
    pub fn add_wilayah(
        &mut self,
        id: String,
        name: String,
        created_by: String,
    ) -> Result<Namespace, CoreError> {
        // Validate ID format
        if !id.starts_with("wilayah-") {
            return Err(CoreError::validation(
                "Wilayah ID must start with 'wilayah-'",
            ));
        }

        // Check if already exists
        if self.all_namespaces.contains_key(&id) {
            return Err(CoreError::already_exists(format!("namespace: {}", id)));
        }

        let namespace = Namespace::new(
            id.clone(),
            format!("pusat/{}", id),
            Some(self.root.id.clone()),
            name,
            NamespaceType::Wilayah,
            created_by,
        );

        self.wilayah_map.insert(id.clone(), namespace.clone());
        self.all_namespaces.insert(id.clone(), namespace.clone());

        Ok(namespace)
    }

    /// Add a satker (work unit) namespace under a wilayah
    pub fn add_satker(
        &mut self,
        id: String,
        name: String,
        wilayah_id: String,
        created_by: String,
    ) -> Result<Namespace, CoreError> {
        // Validate ID format
        if !id.starts_with("satker-") {
            return Err(CoreError::validation("Satker ID must start with 'satker-'"));
        }

        // Check if already exists
        if self.all_namespaces.contains_key(&id) {
            return Err(CoreError::already_exists(format!("namespace: {}", id)));
        }

        // Validate parent wilayah exists
        let wilayah = self
            .wilayah_map
            .get(&wilayah_id)
            .ok_or_else(|| CoreError::not_found(format!("wilayah namespace: {}", wilayah_id)))?;

        let path = format!("{}/{}", wilayah.path, id);

        let namespace = Namespace::new(
            id.clone(),
            path,
            Some(wilayah_id),
            name,
            NamespaceType::Satker,
            created_by,
        );

        self.satker_map.insert(id.clone(), namespace.clone());
        self.all_namespaces.insert(id.clone(), namespace.clone());

        Ok(namespace)
    }

    /// Get namespace by ID
    pub fn get_namespace(&self, id: &str) -> Option<&Namespace> {
        self.all_namespaces.get(id)
    }

    /// Get mutable namespace by ID
    pub fn get_namespace_mut(&mut self, id: &str) -> Option<&mut Namespace> {
        self.all_namespaces.get_mut(id)
    }

    /// Validate namespace path format
    pub fn validate_path(&self, path: &str) -> Result<(), CoreError> {
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            return Err(CoreError::validation("Path cannot be empty"));
        }

        // Root path
        if parts.len() == 1 {
            if parts[0] != "pusat" {
                return Err(CoreError::validation("Root path must be 'pusat'"));
            }
            return Ok(());
        }

        // Wilayah path: pusat/wilayah-xxx
        if parts.len() == 2 {
            if parts[0] != "pusat" {
                return Err(CoreError::validation(
                    "Wilayah path must start with 'pusat'",
                ));
            }
            if !parts[1].starts_with("wilayah-") {
                return Err(CoreError::validation("Second level must be wilayah"));
            }
            return Ok(());
        }

        // Satker path: pusat/wilayah-xxx/satker-xxx
        if parts.len() == 3 {
            if parts[0] != "pusat" {
                return Err(CoreError::validation("Satker path must start with 'pusat'"));
            }
            if !parts[1].starts_with("wilayah-") {
                return Err(CoreError::validation("Second level must be wilayah"));
            }
            if !parts[2].starts_with("satker-") {
                return Err(CoreError::validation("Third level must be satker"));
            }
            return Ok(());
        }

        Err(CoreError::validation(
            "Path depth exceeds maximum (3 levels)",
        ))
    }

    /// Get all ancestor namespaces for a given namespace
    pub fn get_ancestors(&self, namespace_id: &str) -> Vec<Namespace> {
        let mut ancestors = Vec::new();
        let mut current_id = namespace_id.to_string();

        while let Some(namespace) = self.all_namespaces.get(&current_id) {
            if let Some(parent_id) = &namespace.parent {
                if let Some(parent) = self.all_namespaces.get(parent_id) {
                    ancestors.push(parent.clone());
                    current_id = parent_id.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        ancestors
    }

    /// Get all descendant namespaces for a given namespace
    pub fn get_descendants(&self, namespace_id: &str) -> Vec<Namespace> {
        let mut descendants = Vec::new();

        for namespace in self.all_namespaces.values() {
            if let Some(parent_id) = &namespace.parent
                && parent_id == namespace_id
            {
                descendants.push(namespace.clone());
                // Recursively get descendants of this namespace
                descendants.extend(self.get_descendants(&namespace.id));
            }
        }

        descendants
    }

    /// Count all descendants recursively for a given namespace
    pub fn count_all_descendants(&self, namespace_id: &str) -> usize {
        self.get_descendants(namespace_id).len()
    }

    /// Delete a namespace (only if it has no children)
    pub fn delete_namespace(&mut self, namespace_id: &str) -> Result<(), CoreError> {
        // Cannot delete root
        if namespace_id == self.root.id {
            return Err(CoreError::invalid_operation("Cannot delete root namespace"));
        }

        // Check if namespace exists
        let namespace = self
            .all_namespaces
            .get(namespace_id)
            .ok_or_else(|| CoreError::not_found(format!("namespace: {}", namespace_id)))?;

        // Check for children
        let descendants = self.get_descendants(namespace_id);
        if !descendants.is_empty() {
            return Err(CoreError::invalid_operation(format!(
                "Cannot delete namespace with {} children",
                descendants.len()
            )));
        }

        // Remove from appropriate map
        match namespace.namespace_type {
            NamespaceType::Wilayah => {
                self.wilayah_map.remove(namespace_id);
            }
            NamespaceType::Satker => {
                self.satker_map.remove(namespace_id);
            }
            NamespaceType::Pusat => {
                return Err(CoreError::invalid_operation("Cannot delete root namespace"));
            }
        }

        // Remove from all namespaces
        self.all_namespaces.remove(namespace_id);

        Ok(())
    }

    /// List all namespaces
    pub fn list_all(&self) -> Vec<&Namespace> {
        self.all_namespaces.values().collect()
    }

    /// List namespaces by type
    pub fn list_by_type(&self, namespace_type: NamespaceType) -> Vec<&Namespace> {
        self.all_namespaces
            .values()
            .filter(|ns| ns.namespace_type == namespace_type)
            .collect()
    }
}

impl Namespace {
    /// Create a new namespace
    pub fn new(
        id: String,
        path: String,
        parent: Option<String>,
        name: String,
        namespace_type: NamespaceType,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            path,
            parent,
            name,
            namespace_type,
            policies: Vec::new(),
            quotas: NamespaceQuotas::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
            is_active: true,
        }
    }

    /// Create root namespace (Pusat)
    pub fn new_root(name: String, created_by: String) -> Self {
        Self::new(
            "pusat".to_string(),
            "pusat".to_string(),
            None,
            name,
            NamespaceType::Pusat,
            created_by,
        )
    }

    /// Add a policy to this namespace
    pub fn add_policy(&mut self, policy_name: String) {
        if !self.policies.contains(&policy_name) {
            self.policies.push(policy_name);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a policy from this namespace
    pub fn remove_policy(&mut self, policy_name: &str) -> bool {
        let initial_len = self.policies.len();
        self.policies.retain(|p| p != policy_name);
        let removed = self.policies.len() < initial_len;
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Update quotas
    pub fn update_quotas(&mut self, quotas: NamespaceQuotas) {
        self.quotas = quotas;
        self.updated_at = Utc::now();
    }

    /// Check if quota is exceeded
    pub fn is_quota_exceeded(&self) -> bool {
        if let Some(max_secrets) = self.quotas.max_secrets
            && self.quotas.current_usage.secrets_count >= max_secrets
        {
            return true;
        }

        if let Some(max_storage) = self.quotas.max_storage_bytes
            && self.quotas.current_usage.storage_bytes >= max_storage
        {
            return true;
        }

        if let Some(max_leases) = self.quotas.max_leases
            && self.quotas.current_usage.leases_count >= max_leases
        {
            return true;
        }

        if let Some(max_policies) = self.quotas.max_policies
            && self.quotas.current_usage.policies_count >= max_policies
        {
            return true;
        }

        false
    }

    /// Get quota usage percentage
    pub fn get_quota_usage_percentage(&self) -> f64 {
        let mut percentages = Vec::new();

        if let Some(max_secrets) = self.quotas.max_secrets
            && max_secrets > 0
        {
            percentages.push(
                (self.quotas.current_usage.secrets_count as f64 / max_secrets as f64) * 100.0,
            );
        }

        if let Some(max_storage) = self.quotas.max_storage_bytes
            && max_storage > 0
        {
            percentages.push(
                (self.quotas.current_usage.storage_bytes as f64 / max_storage as f64) * 100.0,
            );
        }

        if percentages.is_empty() {
            0.0
        } else {
            percentages.iter().sum::<f64>() / percentages.len() as f64
        }
    }
}

impl Default for NamespaceQuotas {
    fn default() -> Self {
        Self {
            max_secrets: Some(10000),
            max_storage_bytes: Some(10 * 1024 * 1024 * 1024), // 10 GB
            max_leases: Some(1000),
            max_policies: Some(100),
            current_usage: QuotaUsage::default(),
        }
    }
}

impl std::fmt::Display for NamespaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceType::Pusat => write!(f, "Pusat"),
            NamespaceType::Wilayah => write!(f, "Wilayah"),
            NamespaceType::Satker => write!(f, "Satker"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hierarchy() {
        let hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        assert_eq!(hierarchy.root.id, "pusat");
        assert_eq!(hierarchy.root.namespace_type, NamespaceType::Pusat);
        assert_eq!(hierarchy.all_namespaces.len(), 1);
    }

    #[test]
    fn test_add_wilayah() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        let wilayah = hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        assert_eq!(wilayah.id, "wilayah-sumut");
        assert_eq!(wilayah.path, "pusat/wilayah-sumut");
        assert_eq!(wilayah.parent, Some("pusat".to_string()));
        assert_eq!(hierarchy.wilayah_map.len(), 1);
        assert_eq!(hierarchy.all_namespaces.len(), 2);
    }

    #[test]
    fn test_add_satker() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        let satker = hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        assert_eq!(satker.id, "satker-kja001");
        assert_eq!(satker.path, "pusat/wilayah-sumut/satker-kja001");
        assert_eq!(satker.parent, Some("wilayah-sumut".to_string()));
        assert_eq!(hierarchy.satker_map.len(), 1);
        assert_eq!(hierarchy.all_namespaces.len(), 3);
    }

    #[test]
    fn test_validate_path() {
        let hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        assert!(hierarchy.validate_path("pusat").is_ok());
        assert!(hierarchy.validate_path("pusat/wilayah-sumut").is_ok());
        assert!(
            hierarchy
                .validate_path("pusat/wilayah-sumut/satker-kja001")
                .is_ok()
        );

        assert!(hierarchy.validate_path("invalid").is_err());
        assert!(hierarchy.validate_path("pusat/invalid").is_err());
        assert!(
            hierarchy
                .validate_path("pusat/wilayah-sumut/satker-kja001/extra")
                .is_err()
        );
    }

    #[test]
    fn test_get_ancestors() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        let ancestors = hierarchy.get_ancestors("satker-kja001");
        assert_eq!(ancestors.len(), 2); // wilayah and pusat
        assert_eq!(ancestors[0].id, "wilayah-sumut");
        assert_eq!(ancestors[1].id, "pusat");
    }

    #[test]
    fn test_get_descendants() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        let descendants = hierarchy.get_descendants("pusat");
        assert_eq!(descendants.len(), 2); // wilayah and satker

        let wilayah_descendants = hierarchy.get_descendants("wilayah-sumut");
        assert_eq!(wilayah_descendants.len(), 1); // only satker
    }

    #[test]
    fn test_delete_namespace() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        // Cannot delete wilayah with children
        assert!(hierarchy.delete_namespace("wilayah-sumut").is_err());

        // Can delete satker (no children)
        assert!(hierarchy.delete_namespace("satker-kja001").is_ok());
        assert_eq!(hierarchy.satker_map.len(), 0);

        // Now can delete wilayah
        assert!(hierarchy.delete_namespace("wilayah-sumut").is_ok());
        assert_eq!(hierarchy.wilayah_map.len(), 0);

        // Cannot delete root
        assert!(hierarchy.delete_namespace("pusat").is_err());
    }

    #[test]
    fn test_namespace_quotas() {
        let mut namespace =
            Namespace::new_root("Kejaksaan Agung RI".to_string(), "admin".to_string());

        // Default quotas should not be exceeded
        assert!(!namespace.is_quota_exceeded());

        // Set usage to exceed quota
        namespace.quotas.current_usage.secrets_count = 10001;
        assert!(namespace.is_quota_exceeded());

        // Test quota percentage
        namespace.quotas.current_usage.secrets_count = 5000;
        let percentage = namespace.get_quota_usage_percentage();
        assert!(percentage > 0.0 && percentage < 100.0);
    }
}
