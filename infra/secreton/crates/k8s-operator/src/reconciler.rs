//! Reconciliation logic for SecretSync resources
//!
//! This module contains the core reconciliation logic that can be tested
//! independently of the Kubernetes controller runtime.

use crate::{Error, Result, SecretSync};
use std::collections::HashMap;

/// Reconciler for SecretSync resources
pub struct Reconciler {
    /// Secreton client configuration
    pub secreton_url: String,
}

impl Reconciler {
    /// Create a new reconciler
    pub fn new(secreton_url: String) -> Self {
        Self { secreton_url }
    }

    /// Validate a SecretSync spec
    pub fn validate_spec(&self, spec: &crate::crd::SecretSyncSpec) -> Result<()> {
        // Validate secret path
        if spec.secreton_path.is_empty() {
            return Err(Error::InvalidSecretPath(
                "Secret path cannot be empty".to_string(),
            ));
        }

        if !spec.secreton_path.starts_with('/') {
            return Err(Error::InvalidSecretPath(
                "Secret path must start with /".to_string(),
            ));
        }

        // Validate target secret name
        if spec.target_secret.is_empty() {
            return Err(Error::InvalidSecretPath(
                "Target secret name cannot be empty".to_string(),
            ));
        }

        // Validate refresh interval
        if let Some(interval) = spec.refresh_interval {
            if interval < 10 {
                return Err(Error::ReconciliationFailed(
                    "Refresh interval must be at least 10 seconds".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Transform secret data according to transformation rules
    pub fn transform_data(
        &self,
        data: HashMap<String, String>,
        transform: Option<&crate::crd::TransformConfig>,
    ) -> Result<HashMap<String, String>> {
        let Some(transform) = transform else {
            return Ok(data);
        };

        // Apply key mappings if specified
        if let Some(mappings) = &transform.mappings {
            let mut result = HashMap::new();
            for (source_key, target_key) in mappings {
                if let Some(value) = data.get(source_key) {
                    result.insert(target_key.clone(), value.clone());
                }
            }
            return Ok(result);
        }

        // TODO: Implement template-based transformation
        Ok(data)
    }

    /// Calculate the next reconciliation time
    pub fn next_reconciliation_interval(
        &self,
        spec: &crate::crd::SecretSyncSpec,
    ) -> std::time::Duration {
        let interval_secs = spec.refresh_interval.unwrap_or(300);
        std::time::Duration::from_secs(interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{SecretSyncSpec, TransformConfig};

    #[test]
    fn test_validate_spec_valid() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "/secret/data/myapp".to_string(),
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: Some(300),
            secreton_url: None,
            auth: None,
            transform: None,
        };

        assert!(reconciler.validate_spec(&spec).is_ok());
    }

    #[test]
    fn test_validate_spec_empty_path() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "".to_string(),
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: Some(300),
            secreton_url: None,
            auth: None,
            transform: None,
        };

        assert!(reconciler.validate_spec(&spec).is_err());
    }

    #[test]
    fn test_validate_spec_invalid_path() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "secret/data/myapp".to_string(), // Missing leading /
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: Some(300),
            secreton_url: None,
            auth: None,
            transform: None,
        };

        assert!(reconciler.validate_spec(&spec).is_err());
    }

    #[test]
    fn test_validate_spec_short_interval() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "/secret/data/myapp".to_string(),
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: Some(5), // Too short
            secreton_url: None,
            auth: None,
            transform: None,
        };

        assert!(reconciler.validate_spec(&spec).is_err());
    }

    #[test]
    fn test_transform_data_no_transform() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        data.insert("key2".to_string(), "value2".to_string());

        let result = reconciler.transform_data(data.clone(), None).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_transform_data_with_mappings() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let mut data = HashMap::new();
        data.insert("db_host".to_string(), "localhost".to_string());
        data.insert("db_port".to_string(), "5432".to_string());

        let mut mappings = HashMap::new();
        mappings.insert("db_host".to_string(), "DATABASE_HOST".to_string());
        mappings.insert("db_port".to_string(), "DATABASE_PORT".to_string());

        let transform = TransformConfig {
            mappings: Some(mappings),
            template: None,
        };

        let result = reconciler.transform_data(data, Some(&transform)).unwrap();

        assert_eq!(result.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(result.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert!(!result.contains_key("db_host"));
        assert!(!result.contains_key("db_port"));
    }

    #[test]
    fn test_next_reconciliation_interval_default() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "/secret/data/myapp".to_string(),
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: None,
            secreton_url: None,
            auth: None,
            transform: None,
        };

        let interval = reconciler.next_reconciliation_interval(&spec);
        assert_eq!(interval.as_secs(), 300);
    }

    #[test]
    fn test_next_reconciliation_interval_custom() {
        let reconciler = Reconciler::new("https://secreton.internal:8200".to_string());

        let spec = SecretSyncSpec {
            secreton_path: "/secret/data/myapp".to_string(),
            secreton_namespace: None,
            target_secret: "myapp-secret".to_string(),
            refresh_interval: Some(600),
            secreton_url: None,
            auth: None,
            transform: None,
        };

        let interval = reconciler.next_reconciliation_interval(&spec);
        assert_eq!(interval.as_secs(), 600);
    }
}
