//! Namespace Service
//!
//! Provides a thread-safe service wrapper for namespace hierarchy management.
//! This service is used by the API layer to manage namespaces with proper
//! synchronization and state management.

use std::sync::{Arc, RwLock};

use super::hierarchy::NamespaceHierarchy;

/// Thread-safe namespace service
#[derive(Debug, Clone)]
/// Mewakili pub `NamespaceService`.
pub struct NamespaceService {
    /// Namespace hierarchy protected by RwLock for concurrent access
    hierarchy: Arc<RwLock<NamespaceHierarchy>>,
}

impl NamespaceService {
    /// Create a new namespace service with default hierarchy
    pub fn new(root_name: String, created_by: String) -> Self {
        let hierarchy = NamespaceHierarchy::new(root_name, created_by);
        Self {
            hierarchy: Arc::new(RwLock::new(hierarchy)),
        }
    }

    /// Create namespace service from existing hierarchy
    pub fn from_hierarchy(hierarchy: NamespaceHierarchy) -> Self {
        Self {
            hierarchy: Arc::new(RwLock::new(hierarchy)),
        }
    }

    /// Get a read-only clone of the hierarchy
    pub fn hierarchy(&self) -> NamespaceHierarchy {
        self.hierarchy
            .read()
            .expect("Failed to acquire read lock on namespace hierarchy")
            .clone()
    }

    /// Update the hierarchy with a new version
    pub fn update_hierarchy(&self, new_hierarchy: NamespaceHierarchy) {
        let mut hierarchy = self
            .hierarchy
            .write()
            .expect("Failed to acquire write lock on namespace hierarchy");
        *hierarchy = new_hierarchy;
    }

    /// Execute a function with read access to the hierarchy
    pub fn with_hierarchy<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&NamespaceHierarchy) -> R,
    {
        let hierarchy = self
            .hierarchy
            .read()
            .expect("Failed to acquire read lock on namespace hierarchy");
        f(&hierarchy)
    }

    /// Execute a function with write access to the hierarchy
    pub fn with_hierarchy_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut NamespaceHierarchy) -> R,
    {
        let mut hierarchy = self
            .hierarchy
            .write()
            .expect("Failed to acquire write lock on namespace hierarchy");
        f(&mut hierarchy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_service_creation() {
        let service = NamespaceService::new("Kejaksaan Agung RI".to_string(), "admin".to_string());
        let hierarchy = service.hierarchy();
        assert_eq!(hierarchy.root.id, "pusat");
        assert_eq!(hierarchy.root.name, "Kejaksaan Agung RI");
    }

    #[test]
    fn test_namespace_service_update() {
        let service = NamespaceService::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        // Add a wilayah
        service.with_hierarchy_mut(|h| {
            h.add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .expect("Failed to add wilayah");
        });

        // Verify the wilayah was added
        let hierarchy = service.hierarchy();
        assert_eq!(hierarchy.wilayah_map.len(), 1);
        assert!(hierarchy.wilayah_map.contains_key("wilayah-sumut"));
    }

    #[test]
    fn test_namespace_service_concurrent_access() {
        use std::thread;

        let service = NamespaceService::new("Kejaksaan Agung RI".to_string(), "admin".to_string());
        let service_clone = service.clone();

        // Spawn a thread that reads the hierarchy
        let handle = thread::spawn(move || {
            let hierarchy = service_clone.hierarchy();
            hierarchy.root.id.clone()
        });

        // Read from the main thread
        let root_id = service.hierarchy().root.id.clone();

        // Wait for the spawned thread
        let thread_root_id = handle.join().expect("Thread panicked");

        assert_eq!(root_id, thread_root_id);
        assert_eq!(root_id, "pusat");
    }
}
