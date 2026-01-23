//! Dashboard module (from layanan/daskrimti/dasbor)

use axum::{Router, routing::get};
use std::sync::Arc;

use crate::handlers::dashboard;
use crate::state::AppState;

/// Create dashboard routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/overview", get(dashboard::get_overview))
        .route("/widgets", get(dashboard::get_widgets))
        .route("/widgets/{id}", get(dashboard::get_widget))
}
