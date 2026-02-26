//! Replication mode definitions

use serde::{Deserialize, Serialize};

/// Replication mode determines the behavior of secondary nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationMode {
    /// Performance replication mode
    ///
    /// - Secondary nodes serve read-only requests
    /// - Write requests are forwarded to primary
    /// - Optimized for geographic distribution and load balancing
    /// - Lower consistency guarantees (eventual consistency)
    /// - Replication includes: secrets, policies, configuration
    /// - Does NOT replicate: tokens, leases, ephemeral state
    Performance,

    /// Disaster Recovery (DR) replication mode
    ///
    /// - Secondary nodes remain sealed until promoted
    /// - Full replication of ALL data including ephemeral state
    /// - Replication includes: secrets, policies, tokens, leases, audit logs
    /// - Can be promoted to primary in case of disaster
    /// - Higher consistency guarantees (strong consistency)
    /// - Higher replication overhead
    DisasterRecovery,
}

impl ReplicationMode {
    /// Check if this mode allows read operations on secondary
    pub fn allows_secondary_reads(&self) -> bool {
        match self {
            Self::Performance => true,
            Self::DisasterRecovery => false,
        }
    }

    /// Check if this mode replicates ephemeral state
    pub fn replicates_ephemeral_state(&self) -> bool {
        match self {
            Self::Performance => false,
            Self::DisasterRecovery => true,
        }
    }

    /// Get the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Performance => "Performance Replication",
            Self::DisasterRecovery => "Disaster Recovery Replication",
        }
    }
}

impl std::fmt::Display for ReplicationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Performance => write!(f, "performance"),
            Self::DisasterRecovery => write!(f, "disaster_recovery"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_mode_properties() {
        let mode = ReplicationMode::Performance;
        assert!(mode.allows_secondary_reads());
        assert!(!mode.replicates_ephemeral_state());
        assert_eq!(mode.display_name(), "Performance Replication");
        assert_eq!(mode.to_string(), "performance");
    }

    #[test]
    fn test_dr_mode_properties() {
        let mode = ReplicationMode::DisasterRecovery;
        assert!(!mode.allows_secondary_reads());
        assert!(mode.replicates_ephemeral_state());
        assert_eq!(mode.display_name(), "Disaster Recovery Replication");
        assert_eq!(mode.to_string(), "disaster_recovery");
    }

    #[test]
    fn test_serialization() {
        let mode = ReplicationMode::Performance;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""performance""#);

        let mode = ReplicationMode::DisasterRecovery;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""disaster_recovery""#);
    }

    #[test]
    fn test_deserialization() {
        let mode: ReplicationMode = serde_json::from_str(r#""performance""#).unwrap();
        assert_eq!(mode, ReplicationMode::Performance);

        let mode: ReplicationMode = serde_json::from_str(r#""disaster_recovery""#).unwrap();
        assert_eq!(mode, ReplicationMode::DisasterRecovery);
    }
}
