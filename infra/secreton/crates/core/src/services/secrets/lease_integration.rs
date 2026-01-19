//! Lease Integration Helpers
//!
//! This module provides helper functions for integrating leases with secret engines
//! and ensuring proper audit logging of lease-related operations.

use std::collections::HashMap;

/// Lease information to include in secret responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Mewakili pub `LeaseInfo`.
pub struct LeaseInfo {
    /// Lease ID
    pub lease_id: String,

    /// TTL in seconds
    pub ttl: i64,

    /// Whether the lease is renewable
    pub renewable: bool,

    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// Number of times renewed
    pub renew_count: u32,
}

impl From<&crate::services::lease::EnhancedLease> for LeaseInfo {
    fn from(lease: &crate::services::lease::EnhancedLease) -> Self {
        let ttl = (lease.expired_at - chrono::Utc::now()).num_seconds();

        Self {
            lease_id: lease.id.clone(),
            ttl,
            renewable: lease.renewable,
            expires_at: lease.expired_at,
            renew_count: lease.renew_count,
        }
    }
}

/// Add lease information to audit log metadata
pub fn add_lease_to_audit_metadata(
    metadata: &mut HashMap<String, String>,
    lease: &crate::services::lease::EnhancedLease,
) {
    metadata.insert("lease_id".to_string(), lease.id.clone());
    metadata.insert(
        "lease_ttl".to_string(),
        (lease.expired_at - lease.issued_at)
            .num_seconds()
            .to_string(),
    );
    metadata.insert("lease_renewable".to_string(), lease.renewable.to_string());
    metadata.insert(
        "lease_expires_at".to_string(),
        lease.expired_at.to_rfc3339(),
    );
    metadata.insert("lease_resource".to_string(), lease.resource.clone());
    metadata.insert(
        "lease_resource_type".to_string(),
        lease.resource_type.clone(),
    );
}

/// Create audit metadata for lease creation
pub fn create_lease_audit_metadata(
    lease: &crate::services::lease::EnhancedLease,
    operation: &str,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("operation".to_string(), operation.to_string());
    add_lease_to_audit_metadata(&mut metadata, lease);
    metadata
}

/// Create audit metadata for lease renewal
pub fn create_lease_renewal_audit_metadata(
    lease: &crate::services::lease::EnhancedLease,
    increment: i64,
) -> HashMap<String, String> {
    let mut metadata = create_lease_audit_metadata(lease, "lease_renewal");
    metadata.insert("renewal_increment".to_string(), increment.to_string());
    metadata.insert("renew_count".to_string(), lease.renew_count.to_string());
    if let Some(last_renewed) = lease.last_renewed_at {
        metadata.insert("last_renewed_at".to_string(), last_renewed.to_rfc3339());
    }
    metadata
}

/// Create audit metadata for lease revocation
pub fn create_lease_revocation_audit_metadata(
    lease_id: &str,
    reason: &str,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("operation".to_string(), "lease_revocation".to_string());
    metadata.insert("lease_id".to_string(), lease_id.to_string());
    metadata.insert("revocation_reason".to_string(), reason.to_string());
    metadata
}

/// Create audit metadata for lease expiration
pub fn create_lease_expiration_audit_metadata(
    lease: &crate::services::lease::EnhancedLease,
) -> HashMap<String, String> {
    let mut metadata = create_lease_audit_metadata(lease, "lease_expiration");
    metadata.insert("expired_at".to_string(), lease.expired_at.to_rfc3339());
    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_lease() -> crate::services::lease::EnhancedLease {
        let now = Utc::now();
        crate::services::lease::EnhancedLease {
            id: "lease-123".to_string(),
            user: "user1".to_string(),
            resource: "/secret/data/test".to_string(),
            resource_type: "kv".to_string(),
            issued_at: now,
            expired_at: now + Duration::seconds(3600),
            status: "active".to_string(),
            namespace: "default".to_string(),
            parent_id: None,
            child_ids: Vec::new(),
            renewable: true,
            max_ttl: 86400,
            renew_count: 0,
            max_renewals: None,
            last_renewed_at: None,
            revoke_callback: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_lease_info_from_lease() {
        let lease = create_test_lease();
        let info = LeaseInfo::from(&lease);

        assert_eq!(info.lease_id, "lease-123");
        assert!(info.renewable);
        assert_eq!(info.renew_count, 0);
        assert!(info.ttl > 0 && info.ttl <= 3600);
    }

    #[test]
    fn test_add_lease_to_audit_metadata() {
        let lease = create_test_lease();
        let mut metadata = HashMap::new();

        add_lease_to_audit_metadata(&mut metadata, &lease);

        assert_eq!(metadata.get("lease_id"), Some(&"lease-123".to_string()));
        assert_eq!(metadata.get("lease_renewable"), Some(&"true".to_string()));
        assert_eq!(
            metadata.get("lease_resource"),
            Some(&"/secret/data/test".to_string())
        );
        assert_eq!(metadata.get("lease_resource_type"), Some(&"kv".to_string()));
    }

    #[test]
    fn test_create_lease_audit_metadata() {
        let lease = create_test_lease();
        let metadata = create_lease_audit_metadata(&lease, "lease_creation");

        assert_eq!(
            metadata.get("operation"),
            Some(&"lease_creation".to_string())
        );
        assert_eq!(metadata.get("lease_id"), Some(&"lease-123".to_string()));
    }

    #[test]
    fn test_create_lease_renewal_audit_metadata() {
        let mut lease = create_test_lease();
        lease.renew_count = 2;
        lease.last_renewed_at = Some(Utc::now());

        let metadata = create_lease_renewal_audit_metadata(&lease, 1800);

        assert_eq!(
            metadata.get("operation"),
            Some(&"lease_renewal".to_string())
        );
        assert_eq!(metadata.get("renewal_increment"), Some(&"1800".to_string()));
        assert_eq!(metadata.get("renew_count"), Some(&"2".to_string()));
        assert!(metadata.contains_key("last_renewed_at"));
    }

    #[test]
    fn test_create_lease_revocation_audit_metadata() {
        let metadata = create_lease_revocation_audit_metadata("lease-123", "manual_revocation");

        assert_eq!(
            metadata.get("operation"),
            Some(&"lease_revocation".to_string())
        );
        assert_eq!(metadata.get("lease_id"), Some(&"lease-123".to_string()));
        assert_eq!(
            metadata.get("revocation_reason"),
            Some(&"manual_revocation".to_string())
        );
    }

    #[test]
    fn test_create_lease_expiration_audit_metadata() {
        let lease = create_test_lease();
        let metadata = create_lease_expiration_audit_metadata(&lease);
        assert_eq!(
            metadata.get("operation"),
            Some(&"lease_expiration".to_string())
        );
        assert!(metadata.contains_key("expired_at"));
    }
}
