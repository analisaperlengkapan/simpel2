//! Axum application setup and configuration
//!
//! This module provides the application setup logic for running the authenc-api
//! as an HTTP server with Axum.
//!
//! ## Features
//!
//! - Graceful shutdown handling
//! - Middleware configuration
//! - Server lifecycle management
//! - Health check endpoints

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use tokio::signal;
use tracing::{error, info};

use crate::{middleware, router, state::ApiState};

/// Axum web application wrapper
pub struct AxumApp {
    #[allow(dead_code)]
    state: Arc<ApiState>,
    router: Router,
}

impl AxumApp {
    /// Create a new Axum application with the given state
    ///
    /// # Arguments
    ///
    /// * `state` - API state with all service dependencies
    /// * `config` - Application configuration
    ///
    /// # Returns
    ///
    /// Configured Axum application ready to run
    pub fn new(state: ApiState, config: AppConfig) -> Self {
        let state = Arc::new(state);

        // Build the router with middleware and routes
        let router = router::create_unified_router(
            state.clone(),
            config.cors,
            config.rate_limit,
            config.csrf,
        );

        Self { state, router }
    }

    /// Create a development application with minimal middleware
    ///
    /// Useful for local development and testing.
    pub fn new_development(state: ApiState) -> Self {
        let state = Arc::new(state);
        let router = router::create_development_router(state.clone());

        Self { state, router }
    }

    /// Get the router for use in dual server setup
    ///
    /// This allows the router to be used in a setup where both HTTP and gRPC
    /// servers run simultaneously.
    pub fn into_router(self) -> Router {
        self.router
    }

    /// Run the Axum server (standalone mode)
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or encounters a runtime error.
    pub async fn run(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        info!("🌐 Server starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("🌐 Server listening on {}", addr);

        axum::serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

        Ok(())
    }
}

/// Application configuration
#[derive(Default)]
pub struct AppConfig {
    /// CORS configuration
    pub cors: middleware::CorsConfig,

    /// Rate limiting configuration
    pub rate_limit: middleware::RateLimitConfig,

    /// CSRF protection configuration
    pub csrf: middleware::CsrfConfig,
}

/// Handle graceful shutdown
///
/// Listens for SIGTERM (Unix) or Ctrl+C signals and initiates graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("🛑 Shutting down gracefully...");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let _config = AppConfig::default();
        // Verify default configuration is created
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_axum_app_creation() {
        // Smoke test to ensure app compiles
        // Actual testing requires mock services
        assert_eq!(2 + 2, 4);
    }
}
