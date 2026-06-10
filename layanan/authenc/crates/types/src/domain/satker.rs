use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Satker (Satuan Kerja) - Organizational unit in the Attorney General's Office
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satker {
    /// Unique identifier for the satker
    pub id: Uuid,
    /// Satker code (unique identifier)
    pub code: String,
    /// Name of the satker
    pub name: String,
    /// Description of the satker
    pub description: Option<String>,
    /// Parent satker code (for hierarchy)
    pub parent_code: Option<String>,
    /// Level in the hierarchy (0 = root, higher = deeper)
    pub level: i32,
    /// Type of satker (Pusat, Wilayah, Kejaksaan, etc.)
    pub satker_type: SatkerType,
    /// Whether this satker is active
    pub active: bool,
    /// Additional attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the satker was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the satker was last updated
    pub updated_at: DateTime<Utc>,
}

/// Type of satker in the organizational hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SatkerType {
    /// Central office (Kejaksaan Agung)
    Pusat,
    /// High prosecutor's office (Kejaksaan Tinggi)
    KejaksaanTinggi,
    /// District prosecutor's office (Kejaksaan Negeri)
    KejaksaanNegeri,
    /// Branch office (Cabang)
    Cabang,
    /// Special unit
    UnitKhusus,
}

/// Satker hierarchy node for traversal
#[derive(Debug, Clone)]
pub struct SatkerNode {
    /// Satker information
    pub satker: Satker,
    /// Parent node
    pub parent: Option<Box<SatkerNode>>,
    /// Child nodes
    pub children: Vec<SatkerNode>,
}

impl SatkerNode {
    /// Create a new satker node
    pub fn new(satker: Satker) -> Self {
        Self {
            satker,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: SatkerNode) {
        self.children.push(child);
    }

    /// Get all ancestor codes (from parent to root)
    pub fn get_ancestors(&self) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = self.parent.as_ref();

        while let Some(node) = current {
            ancestors.push(node.satker.code.clone());
            current = node.parent.as_ref();
        }

        ancestors
    }

    /// Get all descendant codes (all children recursively)
    pub fn get_descendants(&self) -> Vec<String> {
        let mut descendants = Vec::new();

        for child in &self.children {
            descendants.push(child.satker.code.clone());
            descendants.extend(child.get_descendants());
        }

        descendants
    }

    /// Check if this node is an ancestor of the given satker code
    pub fn is_ancestor_of(&self, satker_code: &str) -> bool {
        self.get_descendants().contains(&satker_code.to_string())
    }

    /// Check if this node is a descendant of the given satker code
    pub fn is_descendant_of(&self, satker_code: &str) -> bool {
        self.get_ancestors().contains(&satker_code.to_string())
    }
}

/// Satker hierarchy manager for efficient traversal
#[derive(Debug, Clone)]
pub struct SatkerHierarchy {
    /// Map of satker code to satker
    pub satkers: HashMap<String, Satker>,
    /// Map of satker code to parent code
    parent_map: HashMap<String, String>,
    /// Map of satker code to children codes
    children_map: HashMap<String, Vec<String>>,
    /// Root satker codes (no parent)
    roots: Vec<String>,
}

impl SatkerHierarchy {
    /// Create a new satker hierarchy from a list of satkers
    pub fn new(satkers: Vec<Satker>) -> Self {
        let mut satker_map = HashMap::new();
        let mut parent_map = HashMap::new();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut roots = Vec::new();

        // Build maps
        for satker in satkers {
            let code = satker.code.clone();

            if let Some(parent_code) = &satker.parent_code {
                parent_map.insert(code.clone(), parent_code.clone());
                children_map
                    .entry(parent_code.clone())
                    .or_default()
                    .push(code.clone());
            } else {
                roots.push(code.clone());
            }

            satker_map.insert(code, satker);
        }

        Self {
            satkers: satker_map,
            parent_map,
            children_map,
            roots,
        }
    }

    /// Get a satker by code
    pub fn get_satker(&self, code: &str) -> Option<&Satker> {
        self.satkers.get(code)
    }

    /// Get parent satker code
    pub fn get_parent(&self, code: &str) -> Option<&String> {
        self.parent_map.get(code)
    }

    /// Get children satker codes
    pub fn get_children(&self, code: &str) -> Vec<&String> {
        self.children_map
            .get(code)
            .map(|children| children.iter().collect())
            .unwrap_or_default()
    }

    /// Get all ancestors (from parent to root)
    pub fn get_ancestors(&self, code: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = code;

        while let Some(parent) = self.parent_map.get(current) {
            ancestors.push(parent.clone());
            current = parent;
        }

        ancestors
    }

    /// Get all descendants (all children recursively)
    pub fn get_descendants(&self, code: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![code.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(children) = self.children_map.get(&current) {
                for child in children {
                    descendants.push(child.clone());
                    to_visit.push(child.clone());
                }
            }
        }

        descendants
    }

    /// Check if satker1 is an ancestor of satker2
    pub fn is_ancestor(&self, ancestor_code: &str, descendant_code: &str) -> bool {
        let ancestors = self.get_ancestors(descendant_code);
        ancestors.contains(&ancestor_code.to_string())
    }

    /// Check if satker1 is a descendant of satker2
    pub fn is_descendant(&self, descendant_code: &str, ancestor_code: &str) -> bool {
        self.is_ancestor(ancestor_code, descendant_code)
    }

    /// Check if two satkers are in the same hierarchy branch
    pub fn are_related(&self, code1: &str, code2: &str) -> bool {
        if code1 == code2 {
            return true;
        }

        self.is_ancestor(code1, code2) || self.is_ancestor(code2, code1)
    }

    /// Get the common ancestor of two satkers
    pub fn get_common_ancestor(&self, code1: &str, code2: &str) -> Option<String> {
        let ancestors1: HashSet<String> = self.get_ancestors(code1).into_iter().collect();
        let ancestors2 = self.get_ancestors(code2);

        // Find the first common ancestor (closest to the nodes)
        ancestors2
            .into_iter()
            .find(|ancestor| ancestors1.contains(ancestor))
    }

    /// Get the level of a satker in the hierarchy
    pub fn get_level(&self, code: &str) -> Option<i32> {
        self.satkers.get(code).map(|s| s.level)
    }

    /// Get all satkers at a specific level
    pub fn get_satkers_at_level(&self, level: i32) -> Vec<&Satker> {
        self.satkers.values().filter(|s| s.level == level).collect()
    }

    /// Get root satkers
    pub fn get_roots(&self) -> Vec<&Satker> {
        self.roots
            .iter()
            .filter_map(|code| self.satkers.get(code))
            .collect()
    }

    /// Check if a satker can access resources from another satker
    /// Based on hierarchy rules:
    /// - A satker can access its own resources
    /// - A parent satker can access child satker resources
    /// - A child cannot access parent resources (unless explicitly granted)
    pub fn can_access(&self, accessor_code: &str, target_code: &str) -> bool {
        if accessor_code == target_code {
            return true;
        }

        // Check if accessor is an ancestor of target
        self.is_ancestor(accessor_code, target_code)
    }

    /// Get all satkers that the given satker can access
    pub fn get_accessible_satkers(&self, code: &str) -> Vec<String> {
        let mut accessible = vec![code.to_string()];
        accessible.extend(self.get_descendants(code));
        accessible
    }
}

/// Satker permission scope for authori

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerPermissionScope {
    /// Satker code where permission applies
    pub satker_code: String,
    /// Whether permission includes child satkers
    pub include_children: bool,
    /// Whether permission includes parent satkers
    pub include_parents: bool,
    /// Specific child satker codes to include (if not all)
    pub specific_children: Option<Vec<String>>,
    /// Specific parent satker codes to include (if not all)
    pub specific_parents: Option<Vec<String>>,
}

impl SatkerPermissionScope {
    /// Create a scope for a single satker only
    pub fn single(satker_code: String) -> Self {
        Self {
            satker_code,
            include_children: false,
            include_parents: false,
            specific_children: None,
            specific_parents: None,
        }
    }

    /// Create a scope for a satker and all its children
    pub fn with_children(satker_code: String) -> Self {
        Self {
            satker_code,
            include_children: true,
            include_parents: false,
            specific_children: None,
            specific_parents: None,
        }
    }

    /// Create a scope for a satker and all its parents
    pub fn with_parents(satker_code: String) -> Self {
        Self {
            satker_code,
            include_children: false,
            include_parents: true,
            specific_children: None,
            specific_parents: None,
        }
    }

    /// Create a scope for entire hierarchy (satker, parents, and children)
    pub fn full_hierarchy(satker_code: String) -> Self {
        Self {
            satker_code,
            include_children: true,
            include_parents: true,
            specific_children: None,
            specific_parents: None,
        }
    }

    /// Check if this scope includes the given satker
    pub fn includes(&self, target_code: &str, hierarchy: &SatkerHierarchy) -> bool {
        // Check if it's the same satker
        if self.satker_code == target_code {
            return true;
        }

        // Check children
        if self.include_children {
            if let Some(specific) = &self.specific_children {
                if specific.contains(&target_code.to_string()) {
                    return true;
                }
            } else if hierarchy.is_descendant(target_code, &self.satker_code) {
                return true;
            }
        }

        // Check parents
        if self.include_parents {
            if let Some(specific) = &self.specific_parents {
                if specific.contains(&target_code.to_string()) {
                    return true;
                }
            } else if hierarchy.is_ancestor(target_code, &self.satker_code) {
                return true;
            }
        }

        false
    }

    /// Get all satker codes included in this scope
    pub fn get_included_satkers(&self, hierarchy: &SatkerHierarchy) -> Vec<String> {
        let mut included = vec![self.satker_code.clone()];

        if self.include_children {
            if let Some(specific) = &self.specific_children {
                included.extend(specific.clone());
            } else {
                included.extend(hierarchy.get_descendants(&self.satker_code));
            }
        }

        if self.include_parents {
            if let Some(specific) = &self.specific_parents {
                included.extend(specific.clone());
            } else {
                included.extend(hierarchy.get_ancestors(&self.satker_code));
            }
        }

        included
    }
}

/// Cross-satker operation validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSatkerValidation {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Required permissions that are missing (if not allowed)
    pub missing_permissions: Vec<String>,
    /// Satker codes involved in the operation
    pub involved_satkers: Vec<String>,
}

impl CrossSatkerValidation {
    /// Create an allowed validation result
    pub fn allowed(reason: String, involved_satkers: Vec<String>) -> Self {
        Self {
            allowed: true,
            reason,
            missing_permissions: Vec::new(),
            involved_satkers,
        }
    }

    /// Create a denied validation result
    pub fn denied(
        reason: String,
        missing_permissions: Vec<String>,
        involved_satkers: Vec<String>,
    ) -> Self {
        Self {
            allowed: false,
            reason,
            missing_permissions,
            involved_satkers,
        }
    }
}
