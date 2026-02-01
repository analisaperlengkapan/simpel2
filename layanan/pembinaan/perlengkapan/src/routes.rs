//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{
    Router,
    routing::get,
};

use crate::{AppState, handlers::*};

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))

        // Dashboard
        .route("/dashboard/stats", get(get_dashboard_stats))

        // Asset (Read-Only)
        .route("/assets", get(get_all_assets))
        .route("/assets/:id", get(get_asset_by_id))

        // Pengadaan
        .route("/pengadaan", get(get_all_pengadaan).post(create_pengadaan))
        .route("/pengadaan/:id", get(get_pengadaan_by_id))

        // Pengadaan Sub-Documents
        .route("/pengadaan/:id/hps", get(get_pengadaan_hps).post(create_pengadaan_hps))
        .route("/pengadaan/:id/skppbj", get(get_pengadaan_skppbj).post(create_pengadaan_skppbj))
        .route("/pengadaan/:id/spk", get(get_pengadaan_spk).post(create_pengadaan_spk))
        .route("/pengadaan/:id/ringkasan", get(get_pengadaan_ringkasan).post(create_pengadaan_ringkasan))
        .route("/pengadaan/:id/kontrak", get(get_pengadaan_kontrak).post(create_pengadaan_kontrak))
        .route("/pengadaan/:id/bast", get(get_pengadaan_bast).post(create_pengadaan_bast))
        .route("/pengadaan/:id/nodis", get(get_pengadaan_nodis).post(create_pengadaan_nodis))

        // Analisis Kebutuhan
        .route("/analisis", get(get_all_analisis).post(create_analisis))
        .route("/analisis/:id", get(get_analisis_by_id))

        // Pemakaian
        .route("/pemakaian", get(get_all_pemakaian).post(create_pemakaian))
        .route("/pemakaian/:id", get(get_pemakaian_by_id))

        // Hibah
        .route("/hibah", get(get_all_hibah).post(create_hibah))
        .route("/hibah/:id", get(get_hibah_by_id))

        // Mutasi
        .route("/mutasi", get(get_all_mutasi).post(create_mutasi))
        .route("/mutasi/:id", get(get_mutasi_by_id))

        // Penghapusan
        .route("/penghapusan", get(get_all_penghapusan).post(create_penghapusan))
        .route("/penghapusan/:id", get(get_penghapusan_by_id))

        // Pengalihan
        .route("/pengalihan", get(get_all_pengalihan).post(create_pengalihan))
        .route("/pengalihan/:id", get(get_pengalihan_by_id))

        // Pemeliharaan
        .route("/pemeliharaan", get(get_all_pemeliharaan).post(create_pemeliharaan))
        .route("/pemeliharaan/:id", get(get_pemeliharaan_by_id))

        .with_state(state)
}
