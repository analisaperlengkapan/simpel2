//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{AppState, dashboard, handlers::*, kebutuhan_bmn, mapping_kodefikasi, pakaian_dinas, pemakaian_bmn, roadmap_sarpras};

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        // Dashboard
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route("/dashboard/perlengkapan", get(dashboard::handlers::get_perlengkapan_dashboard_metrics))
        .route("/dashboard/ws", get(dashboard::websocket::dashboard_websocket_handler))
        .route("/dashboard/perlengkapan/export/excel", get(dashboard::handlers::export_dashboard_excel))
        .route("/dashboard/perlengkapan/export/pdf", get(dashboard::handlers::export_dashboard_pdf))
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
        // Penghapusan BMN (Workflow-enabled)
        .route(
            "/penghapusan-bmn",
            get(crate::penghapusan_bmn::list_penghapusan_bmn)
                .post(crate::penghapusan_bmn::create_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/:id",
            get(crate::penghapusan_bmn::get_penghapusan_bmn)
                .put(crate::penghapusan_bmn::update_penghapusan_bmn)
                .delete(crate::penghapusan_bmn::delete_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/:id/transition",
            post(crate::penghapusan_bmn::transition_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/:id/document",
            get(crate::penghapusan_bmn::get_penghapusan_document),
        )
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
        // Workflow Transitions
        .route(
            "/pakaian-dinas/pengajuan/:id/submit",
            post(pakaian_dinas::submit_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/:id/approve",
            post(pakaian_dinas::approve_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/:id/reject",
            post(pakaian_dinas::reject_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/:id/rekapitulasi",
            get(pakaian_dinas::download_rekapitulasi_handler),
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
        // Advanced Search
        .route(
            "/kebutuhan-bmn/search",
            get(kebutuhan_bmn::search_kebutuhan),
        )
        .route(
            "/kebutuhan-bmn/search/suggestions",
            get(kebutuhan_bmn::get_search_suggestions),
        )
        // Batch Operations
        .route(
            "/kebutuhan-bmn/batch/approve",
            post(kebutuhan_bmn::batch_approve_kebutuhan),
        )
        .route(
            "/kebutuhan-bmn/batch/reject",
            post(kebutuhan_bmn::batch_reject_kebutuhan),
        )
        .route(
            "/kebutuhan-bmn/batch/update-status",
            post(kebutuhan_bmn::batch_update_status),
        )
        // ============ Export Routes ============
        .route("/export/excel", get(export_to_excel))
        .route("/export/jobs/:id/status", get(get_export_job_status))
        .route("/export/jobs/:id/download", get(download_export_job))
        // ============ Mapping Kodefikasi Routes ============
        .route(
            "/mapping/detect",
            get(mapping_kodefikasi::detect_non_standard_codes),
        )
        .route(
            "/mapping/suggestions",
            get(mapping_kodefikasi::get_mapping_suggestions),
        )
        .route(
            "/mapping/proposals",
            get(mapping_kodefikasi::get_all_mapping_proposals)
                .post(mapping_kodefikasi::create_mapping_proposal),
        )
        .route(
            "/mapping/proposals/:id",
            get(mapping_kodefikasi::get_mapping_proposal_by_id),
        )
        .route(
            "/mapping/proposals/:id/verify",
            put(mapping_kodefikasi::verify_mapping_proposal),
        )
        .route(
            "/mapping/progress",
            get(mapping_kodefikasi::get_mapping_progress),
        )
        .route(
            "/mapping/progress/satker",
            get(mapping_kodefikasi::get_mapping_progress_by_satker),
        )
        .route(
            "/mapping/progress/wilayah",
            get(mapping_kodefikasi::get_mapping_progress_by_wilayah),
        )
        // ============ Roadmap Sarpras Routes ============
        .route(
            "/roadmap-sarpras",
            get(roadmap_sarpras::list_roadmaps).post(roadmap_sarpras::create_roadmap),
        )
        .route(
            "/roadmap-sarpras/batch",
            post(roadmap_sarpras::create_roadmap_batch),
        )
        .route(
            "/roadmap-sarpras/comparison",
            get(roadmap_sarpras::get_comparison),
        )
        .route(
            "/roadmap-sarpras/sync-realization",
            post(roadmap_sarpras::sync_realization_increment),
        )
        .route(
            "/roadmap-sarpras/:id",
            get(roadmap_sarpras::get_roadmap).delete(roadmap_sarpras::delete_roadmap),
        )
        .route(
            "/roadmap-sarpras/:id/realization",
            put(roadmap_sarpras::update_realization),
        )
        // ============ Pemakaian BMN Routes ============
        // Permit CRUD
        .route(
            "/pemakaian-bmn",
            get(pemakaian_bmn::list_permits).post(pemakaian_bmn::create_permit),
        )
        .route(
            "/pemakaian-bmn/:id",
            get(pemakaian_bmn::get_permit_by_id).put(pemakaian_bmn::update_permit),
        )
        // Workflow Actions
        .route(
            "/pemakaian-bmn/:id/transition",
            post(pemakaian_bmn::transition_permit_status),
        )
        .route(
            "/pemakaian-bmn/:id/activate",
            post(pemakaian_bmn::activate_permit),
        )
        .route(
            "/pemakaian-bmn/:id/document",
            get(pemakaian_bmn::get_permit_document),
        )
        .route(
            "/pemakaian-bmn/:id/revoke",
            post(pemakaian_bmn::revoke_permit),
        )
        .route(
            "/pemakaian-bmn/:id/renew",
            post(pemakaian_bmn::renew_permit),
        )
        // BMN Availability & History
        .route(
            "/pemakaian-bmn/bmn/:bmn_nup/availability",
            get(pemakaian_bmn::check_bmn_availability),
        )
        .route(
            "/pemakaian-bmn/bmn/:bmn_nup/history",
            get(pemakaian_bmn::get_bmn_usage_history),
        )
        // Pegawai History
        .route(
            "/pemakaian-bmn/pegawai/:pegawai_nip/history",
            get(pemakaian_bmn::get_pegawai_usage_history),
        )
        // Expiry Management
        .route(
            "/pemakaian-bmn/expiring",
            get(pemakaian_bmn::get_expiring_permits),
        )
        .route(
            "/pemakaian-bmn/auto-expire",
            post(pemakaian_bmn::auto_expire_permits),
        )
        // Monitoring Dashboard
        .route(
            "/pemakaian-bmn/monitoring/active-usage",
            get(pemakaian_bmn::get_active_usage_dashboard),
        )
        .route(
            "/pemakaian-bmn/monitoring/utilization-report",
            get(pemakaian_bmn::get_bmn_utilization_report),
        )
        .with_state(state)
}
