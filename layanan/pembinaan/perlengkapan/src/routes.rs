//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{AppState, handlers::*, kebutuhan_bmn, pakaian_dinas};

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
        .route(
            "/pengadaan/:id/hps",
            get(get_pengadaan_hps).post(create_pengadaan_hps),
        )
        .route(
            "/pengadaan/:id/skppbj",
            get(get_pengadaan_skppbj).post(create_pengadaan_skppbj),
        )
        .route(
            "/pengadaan/:id/spk",
            get(get_pengadaan_spk).post(create_pengadaan_spk),
        )
        .route(
            "/pengadaan/:id/ringkasan",
            get(get_pengadaan_ringkasan).post(create_pengadaan_ringkasan),
        )
        .route(
            "/pengadaan/:id/kontrak",
            get(get_pengadaan_kontrak).post(create_pengadaan_kontrak),
        )
        .route(
            "/pengadaan/:id/bast",
            get(get_pengadaan_bast).post(create_pengadaan_bast),
        )
        .route(
            "/pengadaan/:id/nodis",
            get(get_pengadaan_nodis).post(create_pengadaan_nodis),
        )
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
        .route(
            "/penghapusan",
            get(get_all_penghapusan).post(create_penghapusan),
        )
        .route("/penghapusan/:id", get(get_penghapusan_by_id))
        // Pengalihan
        .route(
            "/pengalihan",
            get(get_all_pengalihan).post(create_pengalihan),
        )
        .route("/pengalihan/:id", get(get_pengalihan_by_id))
        // Pemeliharaan
        .route(
            "/pemeliharaan",
            get(get_all_pemeliharaan).post(create_pemeliharaan),
        )
        .route("/pemeliharaan/:id", get(get_pemeliharaan_by_id))
        // ============ Pakaian Dinas Routes ============
        // Master: Jenis Pakaian Dinas
        .route(
            "/pakaian-dinas/jenis",
            get(pakaian_dinas::get_all_jenis_pakaian).post(pakaian_dinas::create_jenis_pakaian),
        )
        .route(
            "/pakaian-dinas/jenis/:id",
            get(pakaian_dinas::get_jenis_pakaian_by_id)
                .put(pakaian_dinas::update_jenis_pakaian)
                .delete(pakaian_dinas::delete_jenis_pakaian),
        )
        // Master: Spesifikasi Pakaian Dinas
        .route(
            "/pakaian-dinas/spesifikasi",
            get(pakaian_dinas::get_all_spesifikasi).post(pakaian_dinas::create_spesifikasi),
        )
        .route(
            "/pakaian-dinas/spesifikasi/:id",
            get(pakaian_dinas::get_spesifikasi_by_id)
                .put(pakaian_dinas::update_spesifikasi)
                .delete(pakaian_dinas::delete_spesifikasi_handler),
        )
        // Master: SubSpesifikasi Pakaian Dinas
        .route(
            "/pakaian-dinas/subspesifikasi",
            get(pakaian_dinas::get_all_subspesifikasi).post(pakaian_dinas::create_subspesifikasi),
        )
        .route(
            "/pakaian-dinas/subspesifikasi/:id",
            get(pakaian_dinas::get_subspesifikasi_by_id)
                .delete(pakaian_dinas::delete_subspesifikasi_handler),
        )
        // Master: Ukuran
        .route("/pakaian-dinas/ukuran", get(pakaian_dinas::get_all_ukuran))
        // Pengajuan Pakaian Dinas
        .route(
            "/pakaian-dinas/pengajuan",
            get(pakaian_dinas::get_all_pengajuan_pakaian)
                .post(pakaian_dinas::create_pengajuan_pakaian),
        )
        .route(
            "/pakaian-dinas/pengajuan/:id",
            get(pakaian_dinas::get_pengajuan_pakaian_by_id)
                .delete(pakaian_dinas::delete_pengajuan_pakaian),
        )
        // Pengajuan Satker
        .route(
            "/pakaian-dinas/pengajuan/:pengajuan_id/satker",
            get(pakaian_dinas::get_pengajuan_satker_list),
        )
        .route(
            "/pakaian-dinas/satker/:id",
            get(pakaian_dinas::get_pengajuan_satker_by_id),
        )
        // Workflow Actions
        .route(
            "/pakaian-dinas/validator-action",
            post(pakaian_dinas::process_validator_action),
        )
        // Personal Uniform Sizes
        .route(
            "/pakaian-dinas/ukuran-pakaian-pegawai",
            get(pakaian_dinas::get_personal_ukuran).post(pakaian_dinas::update_personal_ukuran),
        )
        // MySIMKARI Integration
        .route(
            "/pakaian-dinas/pegawai-satker/:satker_id",
            get(pakaian_dinas::get_pegawai_by_satker),
        )
        .route(
            "/pakaian-dinas/pegawai-satker/:satker_id/with-sizes",
            get(pakaian_dinas::get_pegawai_with_sizes),
        )
        // Reports
        .route(
            "/pakaian-dinas/laporan/rekap-ukuran",
            get(pakaian_dinas::get_laporan_rekap_ukuran),
        )
        .route(
            "/pakaian-dinas/laporan/daftar-pegawai",
            get(pakaian_dinas::get_laporan_daftar_pegawai),
        )
        // ============ Kebutuhan BMN Routes ============
        // Dashboard
        .route(
            "/kebutuhan-bmn/dashboard",
            get(kebutuhan_bmn::get_dashboard_stats),
        )
        // Pengajuan CRUD
        .route(
            "/kebutuhan-bmn/pengajuan",
            get(kebutuhan_bmn::get_all_pengajuan).post(kebutuhan_bmn::create_pengajuan),
        )
        .route(
            "/kebutuhan-bmn/pengajuan/:id",
            get(kebutuhan_bmn::get_pengajuan_by_id)
                .put(kebutuhan_bmn::update_pengajuan)
                .delete(kebutuhan_bmn::delete_pengajuan),
        )
        // Pengajuan Workflow
        .route(
            "/kebutuhan-bmn/pengajuan/:id/transition",
            post(kebutuhan_bmn::transition_pengajuan_status),
        )
        .route(
            "/kebutuhan-bmn/pengajuan/:id/export",
            get(kebutuhan_bmn::export_pengajuan),
        )
        // Pengajuan Satker
        .route(
            "/kebutuhan-bmn/pengajuan/:id/satker",
            get(kebutuhan_bmn::get_pengajuan_satkers).post(kebutuhan_bmn::add_satker_to_pengajuan),
        )
        // Satker Operations
        .route(
            "/kebutuhan-bmn/satker/:id",
            get(kebutuhan_bmn::get_satker_detail),
        )
        .route(
            "/kebutuhan-bmn/satker/:id/transition",
            post(kebutuhan_bmn::transition_satker_status),
        )
        .route(
            "/kebutuhan-bmn/satker/:id/aktivitas",
            get(kebutuhan_bmn::get_satker_aktivitas),
        )
        .route(
            "/kebutuhan-bmn/satker/:id/analisis",
            get(kebutuhan_bmn::get_analisis_kelayakan),
        )
        // Barang Operations
        .route(
            "/kebutuhan-bmn/satker/:id/barang",
            post(kebutuhan_bmn::create_barang),
        )
        .route(
            "/kebutuhan-bmn/barang/:id/approval",
            put(kebutuhan_bmn::update_barang_approval),
        )
        .route(
            "/kebutuhan-bmn/barang/:id",
            delete(kebutuhan_bmn::delete_barang),
        )
        // Priority Setting
        .route(
            "/kebutuhan-bmn/prioritas",
            post(kebutuhan_bmn::set_barang_prioritas),
        )
        // SIMAN Integration
        .route(
            "/kebutuhan-bmn/siman/search",
            get(kebutuhan_bmn::search_siman_assets),
        )
        .route(
            "/kebutuhan-bmn/siman/summary/:satker_id",
            get(kebutuhan_bmn::get_siman_satker_summary),
        )
        .with_state(state)
}
