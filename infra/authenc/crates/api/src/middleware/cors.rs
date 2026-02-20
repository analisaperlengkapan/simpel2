//! CORS middleware configuration for microfrontends

use tower_http::cors::{Any, CorsLayer};

/// CORS configuration for development environment
///
/// Allows all origins for local development.
pub fn development_cors() -> CorsLayer {
    CorsLayer::permissive()
}

/// CORS configuration for production environment
///
/// Restricts origins to known microfrontend domains.
/// Allows credentials (cookies, authorization headers).
pub fn production_cors(allowed_origins: Vec<String>) -> CorsLayer {
    use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
    use http::Method;
    use tower_http::cors::AllowOrigin;

    // Parse allowed origins
    let origins: Vec<http::HeaderValue> = allowed_origins
        .into_iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        // Allow specific origins (microfrontend domains)
        .allow_origin(AllowOrigin::list(origins))
        // Allow credentials (cookies, authorization headers)
        .allow_credentials(true)
        // Allow specific methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // Allow specific headers
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        // Expose specific headers to the browser
        .expose_headers([CONTENT_TYPE])
        // Cache preflight requests for 1 hour
        .max_age(std::time::Duration::from_secs(3600))
}

/// CORS configuration builder
pub struct CorsConfig {
    /// Allowed origins (microfrontend URLs)
    pub allowed_origins: Vec<String>,
    /// Whether to allow credentials
    pub allow_credentials: bool,
    /// Environment (development or production)
    pub environment: Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl CorsConfig {
    /// Create a new CORS configuration
    pub fn new(environment: Environment) -> Self {
        Self {
            allowed_origins: Vec::new(),
            allow_credentials: true,
            environment,
        }
    }

    /// Add an allowed origin
    pub fn add_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Add multiple allowed origins
    pub fn add_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins.extend(origins);
        self
    }

    /// Set whether to allow credentials
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Build the CORS layer
    pub fn build(self) -> CorsLayer {
        match self.environment {
            Environment::Development => development_cors(),
            Environment::Production => production_cors(self.allowed_origins),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::new(Environment::Production)
            // Default allowed origins for SIMPelv2 microfrontends
            .add_origin("https://portal.kejaksaan.go.id")
            .add_origin("https://perlengkapan.kejaksaan.go.id")
            .add_origin("https://intel.kejaksaan.go.id")
            .add_origin("https://pidsus.kejaksaan.go.id")
            .add_origin("https://pidum.kejaksaan.go.id")
            .add_origin("https://pidmil.kejaksaan.go.id")
            .add_origin("https://datun.kejaksaan.go.id")
            .add_origin("https://badiklat.kejaksaan.go.id")
            .add_origin("https://pengawasan.kejaksaan.go.id")
            .add_origin("https://pemulihan-aset.kejaksaan.go.id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfig::new(Environment::Production)
            .add_origin("https://example.com")
            .add_origin("https://test.com")
            .allow_credentials(true);

        assert_eq!(config.allowed_origins.len(), 2);
        assert!(config.allow_credentials);
        assert_eq!(config.environment, Environment::Production);
    }

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(config.allowed_origins.len() > 0);
        assert!(config.allow_credentials);
        assert_eq!(config.environment, Environment::Production);
    }

    #[test]
    fn test_development_cors() {
        let layer = development_cors();
        // Development CORS should be permissive
        // This is a smoke test to ensure it compiles
        drop(layer);
    }

    #[test]
    fn test_production_cors() {
        let origins = vec![
            "https://portal.kejaksaan.go.id".to_string(),
            "https://perlengkapan.kejaksaan.go.id".to_string(),
        ];
        let layer = production_cors(origins);
        // Production CORS should be restrictive
        // This is a smoke test to ensure it compiles
        drop(layer);
    }
}
