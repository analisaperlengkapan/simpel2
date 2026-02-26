//! Policy formatter module
//!
//! Provides formatting capabilities for policy files in TOML and JSON formats.

use super::toml_parser::TomlPolicyParser;
use super::{Policy, PolicyParseError, PolicyParser};

/// Output format for policy files
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Toml,
    Json,
    JsonPretty,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "toml" => Ok(OutputFormat::Toml),
            "json" => Ok(OutputFormat::Json),
            "json-pretty" | "pretty" => Ok(OutputFormat::JsonPretty),
            _ => Err(format!(
                "Invalid output format: {}. Valid formats: toml, json, json-pretty",
                s
            )),
        }
    }
}

/// Policy formatter
pub struct PolicyFormatter {
    toml_parser: TomlPolicyParser,
}

impl PolicyFormatter {
    pub fn new() -> Self {
        Self {
            toml_parser: TomlPolicyParser::new(),
        }
    }

    /// Format policy to specified output format
    pub fn format_policy(
        &self,
        policy: &Policy,
        format: OutputFormat,
    ) -> Result<String, PolicyParseError> {
        match format {
            OutputFormat::Toml => self.toml_parser.format(policy),
            OutputFormat::Json => serde_json::to_string(policy)
                .map_err(|e| PolicyParseError::SerializationError(e.to_string())),
            OutputFormat::JsonPretty => serde_json::to_string_pretty(policy)
                .map_err(|e| PolicyParseError::SerializationError(e.to_string())),
        }
    }

    /// Format policy file (read, parse, format, write)
    pub fn format_file(
        &self,
        input_path: &std::path::Path,
        output_format: OutputFormat,
        check_only: bool,
    ) -> Result<FormatResult, PolicyParseError> {
        // Read file
        let content = std::fs::read_to_string(input_path)?;

        // Parse policy
        let policy = self.toml_parser.parse(&content)?;

        // Format policy
        let formatted = self.format_policy(&policy, output_format)?;

        if check_only {
            // Check if formatting is needed
            let needs_formatting = content.trim() != formatted.trim();
            Ok(FormatResult {
                path: input_path.to_path_buf(),
                needs_formatting,
                formatted_content: None,
            })
        } else {
            // Write formatted content back
            std::fs::write(input_path, &formatted)?;
            Ok(FormatResult {
                path: input_path.to_path_buf(),
                needs_formatting: false,
                formatted_content: Some(formatted),
            })
        }
    }

    /// Convert between formats
    #[allow(dead_code)]
    pub fn convert(
        &self,
        input_content: &str,
        input_format: OutputFormat,
        output_format: OutputFormat,
    ) -> Result<String, PolicyParseError> {
        // Parse from input format
        let policy = match input_format {
            OutputFormat::Toml => self.toml_parser.parse(input_content)?,
            OutputFormat::Json | OutputFormat::JsonPretty => serde_json::from_str(input_content)
                .map_err(|e| PolicyParseError::SerializationError(e.to_string()))?,
        };

        // Format to output format
        self.format_policy(&policy, output_format)
    }
}

impl Default for PolicyFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of formatting operation
#[derive(Debug)]
#[allow(dead_code)]
pub struct FormatResult {
    pub path: std::path::PathBuf,
    pub needs_formatting: bool,
    pub formatted_content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_parser::{Capability, Effect, PolicyRule};

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("toml".parse::<OutputFormat>().unwrap(), OutputFormat::Toml);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "json-pretty".parse::<OutputFormat>().unwrap(),
            OutputFormat::JsonPretty
        );
        assert_eq!(
            "pretty".parse::<OutputFormat>().unwrap(),
            OutputFormat::JsonPretty
        );
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_format_policy_toml() {
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

        let formatter = PolicyFormatter::new();
        let result = formatter.format_policy(&policy, OutputFormat::Toml);
        assert!(result.is_ok());

        let toml_str = result.unwrap();
        assert!(toml_str.contains("name = \"test\""));
        assert!(toml_str.contains("namespace = \"default\""));
    }

    #[test]
    fn test_format_policy_json() {
        let policy = Policy {
            name: "test".to_string(),
            description: None,
            namespace: "default".to_string(),
            rules: vec![PolicyRule {
                effect: Effect::Allow,
                path: "secret/*".to_string(),
                capabilities: vec![Capability::Read],
                condition: None,
                mfa: None,
            }],
        };

        let formatter = PolicyFormatter::new();
        let result = formatter.format_policy(&policy, OutputFormat::Json);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_format_policy_json_pretty() {
        let policy = Policy {
            name: "test".to_string(),
            description: None,
            namespace: "default".to_string(),
            rules: vec![PolicyRule {
                effect: Effect::Allow,
                path: "secret/*".to_string(),
                capabilities: vec![Capability::Read],
                condition: None,
                mfa: None,
            }],
        };

        let formatter = PolicyFormatter::new();
        let result = formatter.format_policy(&policy, OutputFormat::JsonPretty);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("\"name\": \"test\""));
        assert!(json_str.contains('\n')); // Pretty format has newlines
    }

    #[test]
    fn test_convert_toml_to_json() {
        let toml_content = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["read"]
"#;

        let formatter = PolicyFormatter::new();
        let result = formatter.convert(toml_content, OutputFormat::Toml, OutputFormat::Json);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("\"name\":\"test\""));
    }
}
