//! Configuration types for Authenc services
//!
//! This module provides configuration structures for various Authenc components.

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// JWT token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Token issuer (e.g., "https://authenc.kejaksaan.go.id")
    pub issuer: String,

    /// Access token lifetime (default: 15 minutes)
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl_seconds: i64,

    /// Refresh token lifetime (default: 7 days)
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl_seconds: i64,

    /// Ed25519 signing key path (for loading from file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_key_path: Option<String>,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: "https://authenc.kejaksaan.go.id".to_string(),
            access_token_ttl_seconds: 900,     // 15 minutes
            refresh_token_ttl_seconds: 604800, // 7 days
            signing_key_path: None,
        }
    }
}

impl JwtConfig {
    /// Get access token TTL as Duration
    pub fn access_token_ttl(&self) -> Duration {
        Duration::seconds(self.access_token_ttl_seconds)
    }

    /// Get refresh token TTL as Duration
    pub fn refresh_token_ttl(&self) -> Duration {
        Duration::seconds(self.refresh_token_ttl_seconds)
    }
}

fn default_access_token_ttl() -> i64 {
    900 // 15 minutes
}

fn default_refresh_token_ttl() -> i64 {
    604800 // 7 days
}

/// SSO cookie configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoCookieConfig {
    /// Cookie name (default: "AUTHENC_SSO")
    #[serde(default = "default_cookie_name")]
    pub name: String,

    /// Cookie domain (e.g., ".kejaksaan.go.id")
    pub domain: Option<String>,

    /// Whether the cookie should be secure (HTTPS only)
    #[serde(default = "default_secure")]
    pub secure: bool,

    /// Whether the cookie should be HTTP only (not accessible via JavaScript)
    #[serde(default = "default_http_only")]
    pub http_only: bool,

    /// SameSite attribute ("Strict", "Lax", or "None")
    #[serde(default = "default_same_site")]
    pub same_site: String,

    /// Maximum age in seconds (default: 7 days)
    #[serde(default = "default_cookie_max_age")]
    pub max_age: i64,

    /// Cookie path (default: "/")
    #[serde(default = "default_cookie_path")]
    pub path: String,
}

impl Default for SsoCookieConfig {
    fn default() -> Self {
        Self {
            name: "AUTHENC_SSO".to_string(),
            domain: None,
            secure: true,
            http_only: true,
            same_site: "Lax".to_string(),
            max_age: 604800, // 7 days
            path: "/".to_string(),
        }
    }
}

impl SsoCookieConfig {
    /// Get max age as Duration
    pub fn max_age_duration(&self) -> Duration {
        Duration::seconds(self.max_age)
    }

    /// Create a production-ready SSO cookie configuration
    pub fn production(domain: String) -> Self {
        Self {
            name: "AUTHENC_SSO".to_string(),
            domain: Some(domain),
            secure: true,
            http_only: true,
            same_site: "Strict".to_string(),
            max_age: 604800, // 7 days
            path: "/".to_string(),
        }
    }

    /// Create a development SSO cookie configuration
    pub fn development() -> Self {
        Self {
            name: "AUTHENC_SSO_DEV".to_string(),
            domain: None,
            secure: false, // Allow HTTP in development
            http_only: true,
            same_site: "Lax".to_string(),
            max_age: 86400, // 1 day
            path: "/".to_string(),
        }
    }
}

fn default_cookie_name() -> String {
    "AUTHENC_SSO".to_string()
}

fn default_secure() -> bool {
    true
}

fn default_http_only() -> bool {
    true
}

fn default_same_site() -> String {
    "Lax".to_string()
}

fn default_cookie_max_age() -> i64 {
    604800 // 7 days
}

fn default_cookie_path() -> String {
    "/".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_jwt_config() {
        let config = JwtConfig::default();
        assert_eq!(config.issuer, "https://authenc.kejaksaan.go.id");
        assert_eq!(config.access_token_ttl_seconds, 900);
        assert_eq!(config.refresh_token_ttl_seconds, 604800);
    }

    #[test]
    fn test_jwt_config_durations() {
        let config = JwtConfig::default();
        assert_eq!(config.access_token_ttl().num_seconds(), 900);
        assert_eq!(config.refresh_token_ttl().num_seconds(), 604800);
    }

    #[test]
    fn test_jwt_config_serialization() {
        let config = JwtConfig {
            issuer: "https://test.example.com".to_string(),
            access_token_ttl_seconds: 600,
            refresh_token_ttl_seconds: 86400,
            signing_key_path: Some("/path/to/key".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: JwtConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.issuer, config.issuer);
        assert_eq!(
            deserialized.access_token_ttl_seconds,
            config.access_token_ttl_seconds
        );
        assert_eq!(
            deserialized.refresh_token_ttl_seconds,
            config.refresh_token_ttl_seconds
        );
        assert_eq!(deserialized.signing_key_path, config.signing_key_path);
    }

    #[test]
    fn test_default_sso_cookie_config() {
        let config = SsoCookieConfig::default();
        assert_eq!(config.name, "AUTHENC_SSO");
        assert_eq!(config.secure, true);
        assert_eq!(config.http_only, true);
        assert_eq!(config.same_site, "Lax");
        assert_eq!(config.max_age, 604800);
        assert_eq!(config.path, "/");
    }

    #[test]
    fn test_production_sso_cookie_config() {
        let config = SsoCookieConfig::production(".kejaksaan.go.id".to_string());
        assert_eq!(config.domain, Some(".kejaksaan.go.id".to_string()));
        assert_eq!(config.secure, true);
        assert_eq!(config.same_site, "Strict");
    }

    #[test]
    fn test_development_sso_cookie_config() {
        let config = SsoCookieConfig::development();
        assert_eq!(config.name, "AUTHENC_SSO_DEV");
        assert_eq!(config.secure, false);
        assert_eq!(config.max_age, 86400);
    }

    #[test]
    fn test_sso_cookie_max_age_duration() {
        let config = SsoCookieConfig::default();
        assert_eq!(config.max_age_duration().num_seconds(), 604800);
    }
}
