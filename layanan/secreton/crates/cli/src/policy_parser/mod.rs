//! Policy file parser module
//!
//! This module provides parsing and formatting capabilities for Secreton policy files.
//! It supports TOML format for policy definitions.

pub mod formatter;
pub mod toml_parser;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Policy parsing errors
#[derive(Debug, Error)]
pub enum PolicyParseError {
    #[error("TOML parse error at line {line}: {message}")]
    TomlError { line: usize, message: String },

    #[error("Invalid capability: {0}")]
    InvalidCapability(String),

    #[error("Invalid effect: {0}. Must be 'allow' or 'deny'")]
    #[allow(dead_code)]
    InvalidEffect(String),

    #[error("Invalid path pattern: {0}")]
    InvalidPath(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid condition: {0}")]
    InvalidCondition(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub field: String,
    pub message: String,
}

/// Policy definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub rules: Vec<PolicyRule>,
}

fn default_namespace() -> String {
    "default".to_string()
}

/// Policy rule definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyRule {
    pub effect: Effect,
    pub path: String,
    pub capabilities: Vec<Capability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa: Option<bool>,
}

/// Rule effect (allow or deny)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Allow,
    Deny,
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Allow => write!(f, "allow"),
            Effect::Deny => write!(f, "deny"),
        }
    }
}

/// Capability (operation type)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Create,
    Read,
    Update,
    Delete,
    List,
    Sudo,
    #[serde(rename = "*")]
    All,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Create => write!(f, "create"),
            Capability::Read => write!(f, "read"),
            Capability::Update => write!(f, "update"),
            Capability::Delete => write!(f, "delete"),
            Capability::List => write!(f, "list"),
            Capability::Sudo => write!(f, "sudo"),
            Capability::All => write!(f, "*"),
        }
    }
}

/// Condition for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_claims: Option<serde_json::Value>,
}

/// Time range condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeRange {
    pub start: String,
    pub end: String,
}

/// Trait for policy parsers
pub trait PolicyParser {
    /// Parse policy from string content
    fn parse(&self, content: &str) -> Result<Policy, PolicyParseError>;

    /// Format policy to string
    fn format(&self, policy: &Policy) -> Result<String, PolicyParseError>;

    /// Validate policy content without full parsing
    fn validate(&self, content: &str) -> Result<Vec<ValidationError>, PolicyParseError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_display() {
        assert_eq!(Effect::Allow.to_string(), "allow");
        assert_eq!(Effect::Deny.to_string(), "deny");
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Create.to_string(), "create");
        assert_eq!(Capability::Read.to_string(), "read");
        assert_eq!(Capability::All.to_string(), "*");
    }

    #[test]
    fn test_default_namespace() {
        let policy = Policy {
            name: "test".to_string(),
            description: None,
            namespace: default_namespace(),
            rules: vec![],
        };
        assert_eq!(policy.namespace, "default");
    }
}
