//! Configuration module (from layanan/daskrimti/konfigurasi)

use axum::{routing::{get, put}, Router};
use std::sync::Arc;

use crate::handlers::configuration;
use crate::state::AppState;

/// Create configuration routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(configuration::list_configs))
        .route("/{id}", get(configuration::get_config))
        .route("/{id}", put(configuration::update_config))
}
