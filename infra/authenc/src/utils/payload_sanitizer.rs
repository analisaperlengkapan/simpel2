//! Payload sanitization utilities for audit logging
//!
//! Provides utilities to sanitize request/response payloads by removing
//! personally identifiable information (PII) and sensitive data before
//! storing in audit logs.

use serde_json::{Map, Value, json};
use std::collections::HashSet;

const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "password_hash",
    "secret",
    "secret_key",
    "api_key",
    "access_token",
    "refresh_token",
    "token",
    "authorization",
    "auth_token",
    "bearer",
    "private_key",
    "client_secret",
    "mfa_secret",
    "totp_secret",
    "backup_codes",
    "recovery_codes",
    "credit_card",
    "card_number",
    "cvv",
    "ssn",
    "social_security",
];

/// Fields that should be masked (show partial data)
const MASKABLE_FIELDS: &[&str] = &[
    "email",
    "phone",
    "phone_number",
    "mobile",
    "address",
    "ip_address",
];

/// Payload sanitizer configuration
#[derive(Debug, Clone)]
/// Fields that should be completely removed from audit logs
pub struct SanitizerConfig {
    /// Additional sensitive fields to remove
    pub additional_sensitive_fields: HashSet<String>,
    /// Additional fields to mask
    pub additional_maskable_fields: HashSet<String>,
    /// Maximum payload size to include (bytes)
    pub max_payload_size: usize,
    /// Whether to mask PII fields
    pub mask_pii: bool,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            additional_sensitive_fields: HashSet::new(),
            additional_maskable_fields: HashSet::new(),
            max_payload_size: 10_000, // 10KB default
            mask_pii: true,
        }
    }
}

/// Sanitize a JSON payload for audit logging
pub fn sanitize_payload(payload: &Value, config: &SanitizerConfig) -> Value {
    match payload {
        Value::Object(map) => {
            let mut sanitized = Map::new();

            for (key, value) in map {
                let key_lower = key.to_lowercase();

                // Check if field should be removed
                if is_sensitive_field(&key_lower, config) {
                    sanitized.insert(key.clone(), json!("[REDACTED]"));
                    continue;
                }

                // Check if field should be masked
                if config.mask_pii && is_maskable_field(&key_lower, config) {
                    sanitized.insert(key.clone(), mask_value(value));
                    continue;
                }

                // Recursively sanitize nested objects
                sanitized.insert(key.clone(), sanitize_payload(value, config));
            }

            Value::Object(sanitized)
        }
        Value::Array(arr) => {
            let sanitized: Vec<Value> = arr.iter().map(|v| sanitize_payload(v, config)).collect();
            Value::Array(sanitized)
        }
        _ => payload.clone(),
    }
}

fn is_sensitive_field(field: &str, config: &SanitizerConfig) -> bool {
    SENSITIVE_FIELDS.contains(&field) || config.additional_sensitive_fields.contains(field)
}

/// Check if a field should be masked
fn is_maskable_field(field: &str, config: &SanitizerConfig) -> bool {
    MASKABLE_FIELDS.contains(&field) || config.additional_maskable_fields.contains(field)
}

/// Mask a value (show partial data)
fn mask_value(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            if s.len() <= 4 {
                json!("***")
            } else if s.contains('@') {
                // Email masking: show first char and domain
                mask_email(s)
            } else if s.len() > 10 {
                // Long strings: show first 2 and last 2 chars
                json!(format!("{}...{}", &s[..2], &s[s.len() - 2..]))
            } else {
                // Short strings: show first char
                json!(format!("{}***", &s[..1]))
            }
        }
        _ => json!("[MASKED]"),
    }
}

/// Mask an email address
fn mask_email(email: &str) -> Value {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() > 2 {
            json!(format!("{}***{}", &local[..1], domain))
        } else {
            json!(format!("***{}", domain))
        }
    } else {
        json!("***")
    }
}

/// Sanitize a string payload (JSON or plain text)
/// Check if a field is sensitive and should be removed
pub fn sanitize_string_payload(payload: &str, config: &SanitizerConfig) -> Result<String, String> {
    // Check size limit
    if payload.len() > config.max_payload_size {
        return Ok(format!(
            "[TRUNCATED: payload size {} exceeds limit {}]",
            payload.len(),
            config.max_payload_size
        ));
    }

    // Try to parse as JSON
    if let Ok(json_value) = serde_json::from_str::<Value>(payload) {
        let sanitized = sanitize_payload(&json_value, config);
        serde_json::to_string(&sanitized).map_err(|e| e.to_string())
    } else {
        // Plain text - just truncate if needed
        Ok(payload.to_string())
    }
}

/// Create a sanitized summary of a payload for audit logs
pub fn create_payload_summary(payload: &Value, max_fields: usize) -> Value {
    match payload {
        Value::Object(map) => {
            let mut summary = Map::new();
            let field_count = map.len();

            // Include first N fields
            for (i, (key, value)) in map.iter().enumerate() {
                if i >= max_fields {
                    summary.insert(
                        "_truncated".to_string(),
                        json!(format!("... {} more fields", field_count - max_fields)),
                    );
                    break;
                }

                // For nested objects, just show type
                let summary_value = match value {
                    Value::Object(_) => json!("[Object]"),
                    Value::Array(arr) => json!(format!("[Array: {} items]", arr.len())),
                    Value::String(s) if s.len() > 50 => {
                        json!(format!("{}...", &s[..50]))
                    }
                    _ => value.clone(),
                };

                summary.insert(key.clone(), summary_value);
            }

            Value::Object(summary)
        }
        _ => payload.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_sensitive_fields() {
        let payload = json!({
            "username": "testuser",
            "password": "secret123",
            "email": "test@example.com"
        });

        let config = SanitizerConfig::default();
        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized["username"], json!("testuser"));
        assert_eq!(sanitized["password"], json!("[REDACTED]"));
        assert_eq!(sanitized["email"], json!("t***@example.com"));
    }

    #[test]
    fn test_sanitize_nested_objects() {
        let payload = json!({
            "user": {
                "name": "Test User",
                "credentials": {
                    "password": "secret",
                    "api_key": "key123"
                }
            }
        });

        let config = SanitizerConfig::default();
        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized["user"]["name"], json!("Test User"));
        assert_eq!(
            sanitized["user"]["credentials"]["password"],
            json!("[REDACTED]")
        );
        assert_eq!(
            sanitized["user"]["credentials"]["api_key"],
            json!("[REDACTED]")
        );
    }

    #[test]
    fn test_mask_email() {
        let email = "testuser@example.com";
        let masked = mask_email(email);
        assert_eq!(masked, json!("t***@example.com"));
    }

    #[test]
    fn test_sanitize_array() {
        let payload = json!([
            {"password": "secret1"},
            {"password": "secret2"}
        ]);

        let config = SanitizerConfig::default();
        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized[0]["password"], json!("[REDACTED]"));
        assert_eq!(sanitized[1]["password"], json!("[REDACTED]"));
    }

    #[test]
    fn test_payload_size_limit() {
        let large_payload = "x".repeat(20000);
        let config = SanitizerConfig::default();

        let result = sanitize_string_payload(&large_payload, &config);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("TRUNCATED"));
    }

    #[test]
    fn test_create_payload_summary() {
        let payload = json!({
            "field1": "value1",
            "field2": "value2",
            "field3": "value3",
            "field4": "value4",
            "field5": "value5"
        });

        let summary = create_payload_summary(&payload, 3);
        let obj = summary.as_object().unwrap();

        assert_eq!(obj.len(), 4); // 3 fields + truncated message
        assert!(obj.contains_key("_truncated"));
    }

    #[test]
    fn test_custom_sensitive_fields() {
        let payload = json!({
            "username": "testuser",
            "custom_secret": "sensitive_data"
        });

        let mut config = SanitizerConfig::default();
        config
            .additional_sensitive_fields
            .insert("custom_secret".to_string());

        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized["username"], json!("testuser"));
        assert_eq!(sanitized["custom_secret"], json!("[REDACTED]"));
    }
}
