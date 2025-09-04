//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::*,
    middleware::auth_middleware,
    services::PerlengkapanService,
};

pub fn create_routes(service: PerlengkapanService) -> Router {
    Router::new()
        // Health check (no auth required)
        .route("/health", get(health_check))

        // Dashboard routes
        .route("/dashboard/stats", get(get_dashboard_stats))

        // Aset routes
        .route("/aset", get(get_all_aset).post(create_aset))
        .route("/aset/:id", get(get_aset_by_id).put(update_aset).delete(delete_aset))

        // Pengadaan routes
        .route("/pengadaan", get(get_all_pengadaan).post(create_pengadaan))
        .route("/pengadaan/:id", get(get_pengadaan_by_id))

        // Analisis Kebutuhan routes
        .route("/analisis", get(get_all_analisis).post(create_analisis))

        // Apply authentication middleware to all routes except health
        .layer(middleware::from_fn(auth_middleware))

        // Add the service as state
        .with_state(service)
}
