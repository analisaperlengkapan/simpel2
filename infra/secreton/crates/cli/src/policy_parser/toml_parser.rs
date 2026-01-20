//! TOML policy parser implementation

use super::{
    Capability, Condition, Policy, PolicyParseError, PolicyParser, TimeRange, ValidationError,
};
#[cfg(test)]
use super::{Effect, PolicyRule};
use std::collections::HashSet;

/// TOML policy parser
pub struct TomlPolicyParser;

impl TomlPolicyParser {
    pub fn new() -> Self {
        Self
    }

    /// Validate path pattern
    pub fn validate_path(path: &str) -> Result<(), PolicyParseError> {
        if path.is_empty() {
            return Err(PolicyParseError::InvalidPath(
                "Path cannot be empty".to_string(),
            ));
        }

        // Check for valid path characters
        let valid_chars = path
            .chars()
            .all(|c| c.is_alphanumeric() || c == '/' || c == '*' || c == '-' || c == '_');

        if !valid_chars {
            return Err(PolicyParseError::InvalidPath(format!(
                "Path '{}' contains invalid characters",
                path
            )));
        }

        Ok(())
    }

    /// Validate capabilities list
    pub fn validate_capabilities(capabilities: &[Capability]) -> Result<(), PolicyParseError> {
        if capabilities.is_empty() {
            return Err(PolicyParseError::InvalidCapability(
                "At least one capability is required".to_string(),
            ));
        }

        // Check for duplicate capabilities
        let mut seen = HashSet::new();
        for cap in capabilities {
            let cap_str = cap.to_string();
            if !seen.insert(cap_str.clone()) {
                return Err(PolicyParseError::InvalidCapability(format!(
                    "Duplicate capability: {}",
                    cap_str
                )));
            }
        }

        // If "all" is present, it should be the only capability
        if capabilities.contains(&Capability::All) && capabilities.len() > 1 {
            return Err(PolicyParseError::InvalidCapability(
                "Capability '*' (all) cannot be combined with other capabilities".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate time range format
    fn validate_time_range(time_range: &TimeRange) -> Result<(), PolicyParseError> {
        // Basic validation - check if strings are not empty
        if time_range.start.is_empty() || time_range.end.is_empty() {
            return Err(PolicyParseError::InvalidCondition(
                "Time range start and end cannot be empty".to_string(),
            ));
        }

        // Try to parse as RFC3339 timestamps
        if chrono::DateTime::parse_from_rfc3339(&time_range.start).is_err() {
            return Err(PolicyParseError::InvalidCondition(format!(
                "Invalid start time format: {}. Expected RFC3339 format",
                time_range.start
            )));
        }

        if chrono::DateTime::parse_from_rfc3339(&time_range.end).is_err() {
            return Err(PolicyParseError::InvalidCondition(format!(
                "Invalid end time format: {}. Expected RFC3339 format",
                time_range.end
            )));
        }

        Ok(())
    }

    /// Validate IP address format
    fn validate_ip(ip: &str) -> Result<(), PolicyParseError> {
        // Check if it's a valid IP or CIDR notation
        if ip.contains('/') {
            // CIDR notation
            let parts: Vec<&str> = ip.split('/').collect();
            if parts.len() != 2 {
                return Err(PolicyParseError::InvalidCondition(format!(
                    "Invalid CIDR notation: {}",
                    ip
                )));
            }

            // Validate IP part
            if parts[0].parse::<std::net::IpAddr>().is_err() {
                return Err(PolicyParseError::InvalidCondition(format!(
                    "Invalid IP address in CIDR: {}",
                    parts[0]
                )));
            }

            // Validate prefix length
            if let Ok(prefix) = parts[1].parse::<u8>() {
                let max_prefix = if parts[0].contains(':') { 128 } else { 32 };
                if prefix > max_prefix {
                    return Err(PolicyParseError::InvalidCondition(format!(
                        "Invalid CIDR prefix length: {}",
                        prefix
                    )));
                }
            } else {
                return Err(PolicyParseError::InvalidCondition(format!(
                    "Invalid CIDR prefix: {}",
                    parts[1]
                )));
            }
        } else {
            // Plain IP address
            if ip.parse::<std::net::IpAddr>().is_err() {
                return Err(PolicyParseError::InvalidCondition(format!(
                    "Invalid IP address: {}",
                    ip
                )));
            }
        }

        Ok(())
    }

    /// Validate condition
    pub fn validate_condition(condition: &Condition) -> Result<(), PolicyParseError> {
        if let Some(time_range) = &condition.time_range {
            Self::validate_time_range(time_range)?;
        }

        if let Some(allowed_ips) = &condition.allowed_ips {
            if allowed_ips.is_empty() {
                return Err(PolicyParseError::InvalidCondition(
                    "allowed_ips cannot be empty if specified".to_string(),
                ));
            }

            for ip in allowed_ips {
                Self::validate_ip(ip)?;
            }
        }

        Ok(())
    }
}

impl Default for TomlPolicyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyParser for TomlPolicyParser {
    fn parse(&self, content: &str) -> Result<Policy, PolicyParseError> {
        // Parse TOML
        let policy: Policy = toml::from_str(content).map_err(|e| {
            // Try to extract line number from error message
            let error_msg = e.to_string();
            let line = error_msg
                .split("line ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            PolicyParseError::TomlError {
                line,
                message: error_msg,
            }
        })?;

        // Validate policy name
        if policy.name.is_empty() {
            return Err(PolicyParseError::MissingField("name".to_string()));
        }

        // Validate namespace
        if policy.namespace.is_empty() {
            return Err(PolicyParseError::MissingField("namespace".to_string()));
        }

        // Validate rules
        if policy.rules.is_empty() {
            return Err(PolicyParseError::MissingField(
                "rules (at least one rule is required)".to_string(),
            ));
        }

        // Validate each rule
        for rule in &policy.rules {
            Self::validate_path(&rule.path)?;
            Self::validate_capabilities(&rule.capabilities)?;

            if let Some(condition) = &rule.condition {
                Self::validate_condition(condition)?;
            }
        }

        Ok(policy)
    }

    fn format(&self, policy: &Policy) -> Result<String, PolicyParseError> {
        toml::to_string_pretty(policy)
            .map_err(|e| PolicyParseError::SerializationError(e.to_string()))
    }

    fn validate(&self, content: &str) -> Result<Vec<ValidationError>, PolicyParseError> {
        let mut errors = Vec::new();

        // Tryarse
        match self.parse(content) {
            Ok(_) => Ok(errors),
            Err(e) => {
                // Convert parse error to validation error
                match e {
                    PolicyParseError::TomlError { line, message } => {
                        errors.push(ValidationError {
                            line: Some(line),
                            field: "toml".to_string(),
                            message,
                        });
                        Ok(errors)
                    }
                    PolicyParseError::InvalidCapability(msg) => {
                        errors.push(ValidationError {
                            line: None,
                            field: "capabilities".to_string(),
                            message: msg,
                        });
                        Ok(errors)
                    }
                    PolicyParseError::InvalidEffect(msg) => {
                        errors.push(ValidationError {
                            line: None,
                            field: "effect".to_string(),
                            message: msg,
                        });
                        Ok(errors)
                    }
                    PolicyParseError::InvalidPath(msg) => {
                        errors.push(ValidationError {
                            line: None,
                            field: "path".to_string(),
                            message: msg,
                        });
                        Ok(errors)
                    }
                    PolicyParseError::MissingField(field) => {
                        errors.push(ValidationError {
                            line: None,
                            field: field.clone(),
                            message: format!("Missing required field: {}", field),
                        });
                        Ok(errors)
                    }
                    PolicyParseError::InvalidCondition(msg) => {
                        errors.push(ValidationError {
                            line: None,
                            field: "condition".to_string(),
                            message: msg,
                        });
                        Ok(errors)
                    }
                    _ => Err(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_policy() {
        let toml_content = r#"
name = "test-policy"
description = "A test policy"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/data/test/*"
capabilities = ["read", "list"]
"#;

        let parser = TomlPolicyParser::new();
        let result = parser.parse(toml_content);
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.name, "test-policy");
        assert_eq!(policy.namespace, "default");
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].path, "secret/data/test/*");
        assert_eq!(policy.rules[0].capabilities.len(), 2);
    }

    #[test]
    fn test_parse_policy_with_condition() {
        let toml_content = r#"
name = "conditional-policy"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/data/database/*"
capabilities = ["read"]
mfa = true

[rules.condition]
allowed_ips = ["192.168.1.0/24", "10.0.0.1"]
"#;

        let parser = TomlPolicyParser::new();
        let result = parser.parse(toml_content);
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.rules[0].mfa, Some(true));
        assert!(policy.rules[0].condition.is_some());
    }

    #[test]
    fn test_validate_invalid_path() {
        let result = TomlPolicyParser::validate_path("");
        assert!(result.is_err());

        let result = TomlPolicyParser::validate_path("invalid path with spaces");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_capabilities() {
        // Empty capabilities
        let result = TomlPolicyParser::validate_capabilities(&[]);
        assert!(result.is_err());

        // Valid capabilities
        let result = TomlPolicyParser::validate_capabilities(&[Capability::Read, Capability::List]);
        assert!(result.is_ok());

        // All with other capabilities
        let result = TomlPolicyParser::validate_capabilities(&[Capability::All, Capability::Read]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_ip() {
        // Valid IPv4
        assert!(TomlPolicyParser::validate_ip("192.168.1.1").is_ok());

        // Valid IPv6
        assert!(TomlPolicyParser::validate_ip("2001:db8::1").is_ok());

        // Valid CIDR
        assert!(TomlPolicyParser::validate_ip("192.168.1.0/24").is_ok());

        // Invalid IP
        assert!(TomlPolicyParser::validate_ip("999.999.999.999").is_err());

        // Invalid CIDR
        assert!(TomlPolicyParser::validate_ip("192.168.1.0/99").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let policy = Policy {
            name: "test".to_string(),
            description: Some("Test policy".to_string()),
            namespace: "default".to_string(),
            rules: vec![PolicyRule {
                effect: Effect::Allow,
                path: "secret/*".to_string(),
                capabilities: vec![Capability::Read],
                condition: None,
                mfa: None,
            }],
        };

        let parser = TomlPolicyParser::new();
        let formatted = parser.format(&policy).unwrap();
        let parsed = parser.parse(&formatted).unwrap();

        assert_eq!(policy, parsed);
    }
}
