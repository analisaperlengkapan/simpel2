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
        // Pemakaian routes
        .route("/pemakaian", get(get_all_pemakaian).post(create_pemakaian))
        // Hibah routes
        .route("/hibah", get(get_all_hibah).post(create_hibah))
        // Mutasi routes
        .route("/mutasi", get(get_all_mutasi).post(create_mutasi))
        // Penghapusan routes
        .route("/penghapusan", get(get_all_penghapusan).post(create_penghapusan))
        // Pengalihan routes
        .route("/pengalihan", get(get_all_pengalihan).post(create_pengalihan))
        // Pemeliharaan routes
        .route("/pemeliharaan", get(get_all_pemeliharaan).post(create_pemeliharaan))
        // Add the service as state
        .with_state(state)
}
