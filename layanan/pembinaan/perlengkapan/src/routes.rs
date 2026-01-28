//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{Router, routing::get};

use crate::{handlers::*, AppState};

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        // Health check (no auth required)
        .route("/health", get(health_check))
        // Dashboard routes
        .route("/dashboard/stats", get(get_dashboard_stats))
        // Asset routes (Read-Only)
        .route("/assets", get(get_all_assets))
        .route("/assets/:id", get(get_asset_by_id))
        // Pengadaan routes
        .route("/pengadaan", get(get_all_pengadaan).post(create_pengadaan))
        .route("/pengadaan/:id", get(get_pengadaan_by_id))
        // Analisis Kebutuhan routes
        .route("/analisis", get(get_all_analisis).post(create_analisis))
        // Add the service as state
        .with_state(state)
}
