//! Payload sanitization utilities for audit logging and data masking
//!
//! Provides utilities to mask or redact sensitive data (PII, passwords, tokens)
//! from JSON payloads before storage or logging.

use serde_json::{Map, Value, json};
use std::collections::HashSet;

/// Fields that should be completely redacted from logs
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

/// Fields that should be masked (partial visibility)
const MASKABLE_FIELDS: &[&str] = &[
    "email",
    "phone",
    "phone_number",
    "mobile",
    "address",
    "ip_address",
];

/// Configuration for the payload sanitizer
#[derive(Debug, Clone)]
pub struct SanitizerConfig {
    /// Additional field names to redact
    pub additional_sensitive_fields: HashSet<String>,
    /// Additional field names to mask
    pub additional_maskable_fields: HashSet<String>,
    /// Maximum byte size for the payload (truncates if exceeded)
    pub max_payload_size: usize,
    /// Whether to apply masking to PII fields
    pub mask_pii: bool,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            additional_sensitive_fields: HashSet::new(),
            additional_maskable_fields: HashSet::new(),
            max_payload_size: 10_000,
            mask_pii: true,
        }
    }
}

/// Sanitize a JSON Value by redacting sensitive fields and masking PII
pub fn sanitize_payload(payload: &Value, config: &SanitizerConfig) -> Value {
    match payload {
        Value::Object(map) => {
            let mut sanitized = Map::new();

            for (key, value) in map {
                let key_lower = key.to_lowercase();

                if is_sensitive_field(&key_lower, config) {
                    sanitized.insert(key.clone(), json!("[REDACTED]"));
                    continue;
                }

                if config.mask_pii && is_maskable_field(&key_lower, config) {
                    sanitized.insert(key.clone(), mask_value(value));
                    continue;
                }

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

fn is_maskable_field(field: &str, config: &SanitizerConfig) -> bool {
    MASKABLE_FIELDS.contains(&field) || config.additional_maskable_fields.contains(field)
}

fn mask_value(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            if s.len() <= 4 {
                json!("***")
            } else if s.contains('@') {
                mask_email(s)
            } else if s.len() > 10 {
                json!(format!("{}...{}", &s[..2], &s[s.len() - 2..]))
            } else {
                json!(format!("{}***", &s[..1]))
            }
        }
        _ => json!("[MASKED]"),
    }
}

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

/// Sanitize a string payload (parses as JSON if possible)
pub fn sanitize_string_payload(payload: &str, config: &SanitizerConfig) -> Result<String, String> {
    if payload.len() > config.max_payload_size {
        return Ok(format!(
            "[TRUNCATED: size {} exceeds limit {}]",
            payload.len(),
            config.max_payload_size
        ));
    }

    if let Ok(json_value) = serde_json::from_str::<Value>(payload) {
        let sanitized = sanitize_payload(&json_value, config);
        serde_json::to_string(&sanitized).map_err(|e| e.to_string())
    } else {
        Ok(payload.to_string())
    }
}
