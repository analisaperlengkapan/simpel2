//! Security classification levels and related types

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Security classification levels
///
/// Defines hierarchical security levels for data classification and access control.
/// Higher levels can access lower levels, but not vice versa.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    /// Public information - no security controls required
    Public = 0,

    /// Internal use - basic access controls
    #[default]
    Internal = 1,

    /// Confidential - restricted access
    Confidential = 2,

    /// Secret - highly restricted access
    Secret = 3,

    /// Top Secret - maximum security controls
    #[serde(rename = "topsecret")]
    TopSecret = 4,
}

impl SecurityLevel {
    /// Get security level name
    pub fn name(&self) -> &'static str {
        match self {
            SecurityLevel::Public => "Public",
            SecurityLevel::Internal => "Internal",
            SecurityLevel::Confidential => "Confidential",
            SecurityLevel::Secret => "Secret",
            SecurityLevel::TopSecret => "Top Secret",
        }
    }

    /// Get security level from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "public" => Some(SecurityLevel::Public),
            "internal" => Some(SecurityLevel::Internal),
            "confidential" => Some(SecurityLevel::Confidential),
            "secret" => Some(SecurityLevel::Secret),
            "topsecret" | "top_secret" | "top-secret" => Some(SecurityLevel::TopSecret),
            _ => None,
        }
    }

    /// Check if current level can access target level
    pub fn can_access(&self, target: SecurityLevel) -> bool {
        *self >= target
    }

    /// Get numeric value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Unknown security level: {}", s))
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_levels_ordering() {
        assert!(SecurityLevel::TopSecret > SecurityLevel::Secret);
        assert!(SecurityLevel::Secret > SecurityLevel::Confidential);
        assert!(SecurityLevel::Confidential > SecurityLevel::Internal);
        assert!(SecurityLevel::Internal > SecurityLevel::Public);
    }

    #[test]
    fn test_can_access() {
        assert!(SecurityLevel::TopSecret.can_access(SecurityLevel::Public));
        assert!(SecurityLevel::TopSecret.can_access(SecurityLevel::Secret));
        assert!(!SecurityLevel::Public.can_access(SecurityLevel::Secret));
        assert!(!SecurityLevel::Internal.can_access(SecurityLevel::Confidential));
    }

    #[test]
    fn test_security_level_from_string() {
        assert_eq!(SecurityLevel::parse("public"), Some(SecurityLevel::Public));
        assert_eq!(
            SecurityLevel::parse("CONFIDENTIAL"),
            Some(SecurityLevel::Confidential)
        );
        assert_eq!(
            SecurityLevel::parse("top-secret"),
            Some(SecurityLevel::TopSecret)
        );
        assert_eq!(SecurityLevel::parse("invalid"), None);

        assert_eq!("public".parse::<SecurityLevel>(), Ok(SecurityLevel::Public));
        assert!("invalid".parse::<SecurityLevel>().is_err());
    }

    #[test]
    fn test_as_u8() {
        assert_eq!(SecurityLevel::Public.as_u8(), 0);
        assert_eq!(SecurityLevel::Internal.as_u8(), 1);
        assert_eq!(SecurityLevel::TopSecret.as_u8(), 4);
    }
}
