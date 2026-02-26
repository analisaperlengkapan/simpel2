//! Config adapter to convert BootstrapConfig + ApplicationConfig to ApiConfig
//!
//! This is a simplified adapter that creates ApiConfig from the new secure config system.

use crate::config::*;
use std::time::Duration;

impl ApiConfig {
    /// Create ApiConfig from BootstrapConfig and ApplicationConfig (NEW SECURE SYSTEM)
    ///
    /// This is the new recommended way to load configuration.
    /// Bootstrap config contains infrastructure settings (storage, listeners, seal).
    /// Application config contains encrypted settings (auth, database, MFA).
    pub fn from_bootstrap_and_application(
        bootstrap: &secreton_core::config::BootstrapConfig,
        app: &secreton_core::config::ApplicationConfig,
    ) -> Result<Self, String> {
        // Start with default config
        let mut config = Self::default();

        // Override with bootstrap config
        config.http.bind_address = bootstrap
            .listener
            .http
            .address
            .parse()
            .map_err(|e| format!("Invalid HTTP address: {}", e))?;

        config.grpc.enabled = bootstrap.listener.grpc.enabled;
        config.grpc.bind_address = bootstrap
            .listener
            .grpc
            .address
            .parse()
            .map_err(|e| format!("Invalid gRPC ad {}", e))?;

        // Override with application config
        if let Some(ref jwt_secret) = app.auth.jwt_secret {
            config.auth.jwt.secret = jwt_secret.clone();
        }
        config.auth.jwt.expiration =
            Duration::from_secs(app.auth.jwt_expiration_hours as u64 * 3600);
        config.auth.jwt.issuer = app.auth.issuer.clone();
        config.auth.jwt.audience = app.auth.audience.clone();

        config.auth.mfa.enabled = app.mfa.enabled;

        config.cors.enabled = app.cors.enabled;
        config.cors.allowed_origins = app.cors.allowed_origins.clone();
        config.cors.allowed_methods = app.cors.allowed_methods.clone();
        config.cors.allowed_headers = app.cors.allowed_headers.clone();

        config.logging.level = app.logging.level.clone();
        config.logging.format = app.logging.format.clone();
        config.logging.json = app.logging.format == "json";

        // Storage config
        config.storage.backend = match &bootstrap.storage.backend {
            secreton_core::config::StorageBackend::Raft => "raft".to_string(),
            secreton_core::config::StorageBackend::File => "file".to_string(),
            secreton_core::config::StorageBackend::Postgres => "postgres".to_string(),
            secreton_core::config::StorageBackend::Memory => "memory".to_string(),
        };

        // Database config from application config
        if let Some(ref db_url) = app.database.url {
            if let Ok(url) = url::Url::parse(db_url) {
                if let Some(host) = url.host_str() {
                    config.database.host = host.to_string();
                }
                if let Some(port) = url.port() {
                    config.database.port = port;
                }
                config.database.username = url.username().to_string();
                if let Some(password) = url.password() {
                    config.database.password = password.to_string();
                }
                if let Some(db) = url.path().strip_prefix('/') {
                    if !db.is_empty() {
                        config.database.database = db.to_string();
                    }
                }
            }
        }

        Ok(config)
    }
}
