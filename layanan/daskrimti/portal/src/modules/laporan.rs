//! Reports module (from layanan/daskrimti/laporan)

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::handlers::reports;
use crate::state::AppState;

/// Create reports routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(reports::list_reports))
        .route("/", post(reports::generate_report))
        .route("/{id}", get(reports::get_report))
        .route("/{id}/download", get(reports::download_report))
}
