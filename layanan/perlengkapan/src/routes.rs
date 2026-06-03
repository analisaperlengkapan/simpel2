//! # API Routes
//!
//! Route definitions for the Perlengkapan service

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    AppState, dashboard, handlers::*, kebutuhan_bmn, mapping_kodefikasi, pakaian_dinas,
    pemakaian_bmn, roadmap_sarpras,
};

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        // Dashboard
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route(
            "/dashboard/perlengkapan",
            get(dashboard::handlers::get_perlengkapan_dashboard_metrics),
        )
        .route(
            "/dashboard/ws",
            get(dashboard::websocket::dashboard_websocket_handler),
        )
        .route(
            "/dashboard/perlengkapan/export/excel",
            get(dashboard::handlers::export_dashboard_excel),
        )
        .route(
            "/dashboard/perlengkapan/export/pdf",
            get(dashboard::handlers::export_dashboard_pdf),
        )
        // Bank Aset (unified SIMAN façade)
        .route("/bank-aset", get(crate::bank_aset::list_bank_aset))
        .route("/bank-aset/lookup", get(crate::bank_aset::lookup_bank_aset))
        .route(
            "/bank-aset/dashboard",
            get(crate::bank_aset::get_bank_aset_dashboard),
        )
        .route(
            "/bank-aset/sebaran",
            get(crate::bank_aset::get_bank_aset_sebaran),
        )
        .route(
            "/bank-aset/last-sync",
            get(crate::bank_aset::get_bank_aset_last_sync),
        )
        .route(
            "/bank-aset/{id}",
            get(crate::bank_aset::get_bank_aset_detail),
        )
        // Analisis Kebutuhan
        .route("/analisis", get(get_all_analisis).post(create_analisis))
        .route("/analisis/{id}", get(get_analisis_by_id))
        // Penghapusan BMN (Workflow-enabled)
        .route(
            "/penghapusan-bmn",
            get(crate::penghapusan_bmn::list_penghapusan_bmn)
                .post(crate::penghapusan_bmn::create_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/{id}",
            get(crate::penghapusan_bmn::get_penghapusan_bmn)
                .put(crate::penghapusan_bmn::update_penghapusan_bmn)
                .delete(crate::penghapusan_bmn::delete_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/{id}/detail",
            get(crate::penghapusan_bmn::get_penghapusan_bmn_detail),
        )
        .route(
            "/penghapusan-bmn/{id}/verifikasi-siman",
            get(crate::penghapusan_bmn::verify_penghapusan_asset_siman),
        )
        .route(
            "/penghapusan-bmn/{id}/transition",
            post(crate::penghapusan_bmn::transition_penghapusan_bmn),
        )
        .route(
            "/penghapusan-bmn/{id}/submit-wilayah",
            post(crate::penghapusan_bmn::submit_to_wilayah),
        )
        .route(
            "/penghapusan-bmn/{id}/validator-wilayah",
            post(crate::penghapusan_bmn::validator_wilayah_action),
        )
        .route(
            "/penghapusan-bmn/{id}/forward-pusat",
            post(crate::penghapusan_bmn::forward_to_pusat),
        )
        .route(
            "/penghapusan-bmn/{id}/return-operator",
            post(crate::penghapusan_bmn::return_to_operator),
        )
        .route(
            "/penghapusan-bmn/{id}/generate-sk",
            post(crate::penghapusan_bmn::generate_konsep_sk),
        )
        .route(
            "/penghapusan-bmn/{id}/generate-konsep-sk",
            post(crate::penghapusan_bmn::generate_konsep_sk),
        )
        // Stream the konsep SK in either DOCX (editable) or PDF (final) form.
        // {format} = "docx" | "pdf".
        .route(
            "/penghapusan-bmn/{id}/konsep-sk.{format}",
            get(crate::penghapusan_bmn::serve_konsep_sk),
        )
        .route(
            "/penghapusan-bmn/{id}/upload-signed-sk",
            post(crate::penghapusan_bmn::upload_signed_sk),
        )
        // V029 (Fase 1.9): SK Wilayah endpoints — Validator Wilayah
        // (mewakili Kepala Kejaksaan Tinggi) untuk kewenangan WILAYAH.
        .route(
            "/penghapusan-bmn/{id}/generate-sk-wilayah",
            post(crate::penghapusan_bmn::generate_konsep_sk_wilayah),
        )
        .route(
            "/penghapusan-bmn/{id}/upload-signed-sk-wilayah",
            post(crate::penghapusan_bmn::upload_signed_sk_wilayah),
        )
        .route(
            "/penghapusan-bmn/{id}/document",
            get(crate::penghapusan_bmn::get_penghapusan_document),
        )
        // File upload (Fase 0.6): Surat Usulan + Lampiran[]
        .route(
            "/penghapusan-bmn/{id}/lampiran",
            get(crate::penghapusan_bmn::list_lampiran)
                .post(crate::penghapusan_bmn::upload_lampiran),
        )
        // ============ Pemakaian BMN cek (Fase 1.11) ============
        .route(
            "/pemakaian-bmn/cek-pegawai/{nip}",
            get(crate::pemakaian_bmn::cek_pegawai),
        )
        .route("/pemakaian-bmn/cek-bmn", get(crate::pemakaian_bmn::cek_bmn))
        // ============ Pakaian Dinas Routes ============
        // Master: Jenis Pakaian Dinas
        .route(
            "/pakaian-dinas/jenis",
            get(pakaian_dinas::get_all_jenis_pakaian).post(pakaian_dinas::create_jenis_pakaian),
        )
        .route(
            "/pakaian-dinas/jenis/{id}",
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
            "/pakaian-dinas/spesifikasi/{id}",
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
            "/pakaian-dinas/subspesifikasi/{id}",
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
            "/pakaian-dinas/pengajuan/{id}",
            get(pakaian_dinas::get_pengajuan_pakaian_by_id)
                .delete(pakaian_dinas::delete_pengajuan_pakaian),
        )
        // Pengajuan Satker
        .route(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker",
            get(pakaian_dinas::get_pengajuan_satker_list),
        )
        .route(
            "/pakaian-dinas/satker/{id}",
            get(pakaian_dinas::get_pengajuan_satker_by_id),
        )
        .route(
            "/pakaian-dinas/satker/{id}/aktivitas",
            get(pakaian_dinas::get_pengajuan_satker_aktivitas),
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
        // Pegawai Profile (reporting fields + sizes)
        .route(
            "/pakaian-dinas/pegawai-profile",
            post(pakaian_dinas::upsert_pegawai_profile),
        )
        .route(
            "/pakaian-dinas/pegawai-profile/bulk",
            post(pakaian_dinas::bulk_upsert_pegawai_profiles),
        )
        // Spesifikasi by Jenis (sub-resource)
        .route(
            "/pakaian-dinas/jenis/{jenis_id}/spesifikasi",
            get(pakaian_dinas::get_spesifikasi_by_jenis),
        )
        // MySIMKARI Integration
        .route(
            "/pakaian-dinas/pegawai-satker/{satker_id}",
            get(pakaian_dinas::get_pegawai_by_satker),
        )
        .route(
            "/pakaian-dinas/pegawai-satker/{satker_id}/with-sizes",
            get(pakaian_dinas::get_pegawai_with_sizes),
        )
        .route(
            "/pakaian-dinas/pegawai-satker/{satker_id}/roster",
            get(pakaian_dinas::get_pegawai_roster_with_sync),
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
        .route(
            "/pakaian-dinas/laporan/cetak",
            get(pakaian_dinas::cetak_laporan),
        )
        // Workflow Transitions
        .route(
            "/pakaian-dinas/pengajuan/{id}/submit",
            post(pakaian_dinas::submit_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/{id}/approve",
            post(pakaian_dinas::approve_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/{id}/reject",
            post(pakaian_dinas::reject_pengajuan_handler),
        )
        .route(
            "/pakaian-dinas/pengajuan/{id}/rekapitulasi",
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
            "/kebutuhan-bmn/pengajuan/{id}",
            get(kebutuhan_bmn::get_pengajuan_by_id)
                .put(kebutuhan_bmn::update_pengajuan)
                .delete(kebutuhan_bmn::delete_pengajuan),
        )
        // Pengajuan Workflow
        .route(
            "/kebutuhan-bmn/pengajuan/{id}/transition",
            post(kebutuhan_bmn::transition_pengajuan_status),
        )
        .route(
            "/kebutuhan-bmn/pengajuan/{id}/export",
            get(kebutuhan_bmn::export_pengajuan),
        )
        // V029 (Fase 1.7): Daftar wilayah Kejaksaan Tinggi utk dropdown FE
        // saat user pilih pilihan_satker = wilayah.
        .route(
            "/kebutuhan-bmn/wilayah",
            get(kebutuhan_bmn::list_wilayah_kejati),
        )
        // Pengajuan Satker
        .route(
            "/kebutuhan-bmn/pengajuan/{id}/satker",
            get(kebutuhan_bmn::get_pengajuan_satkers).post(kebutuhan_bmn::add_satker_to_pengajuan),
        )
        // V029 (Fase 1.6): Daftar allowed-list BMN utk pengajuan
        .route(
            "/kebutuhan-bmn/pengajuan/{id}/bmn-referensi",
            get(kebutuhan_bmn::list_bmn_referensi_handler),
        )
        // Satker Operations
        .route(
            "/kebutuhan-bmn/satker/{id}",
            get(kebutuhan_bmn::get_satker_detail),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/transition",
            post(kebutuhan_bmn::transition_satker_status),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/aktivitas",
            get(kebutuhan_bmn::get_satker_aktivitas),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/analisis",
            get(kebutuhan_bmn::get_analisis_kelayakan),
        )
        // Laporan Hasil Analisis Kebutuhan BMN (Fase 0.8): preview inline (PDF
        // di-iframe FE) + download attachment (PDF; DOCX follow-up).
        .route(
            "/kebutuhan-bmn/satker/{id}/laporan/preview",
            get(kebutuhan_bmn::preview_laporan_analisis),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/laporan/download",
            get(kebutuhan_bmn::download_laporan_analisis),
        )
        // Satker Workflow Actions (Validator Wilayah & Pusat)
        .route(
            "/kebutuhan-bmn/satker/{id}/submit-wilayah",
            post(kebutuhan_bmn::submit_satker_to_wilayah),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/validator-wilayah",
            post(kebutuhan_bmn::validator_wilayah_action),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/keputusan-pusat",
            post(kebutuhan_bmn::validator_pusat_keputusan),
        )
        .route(
            "/kebutuhan-bmn/satker/{id}/validator-pusat",
            post(kebutuhan_bmn::validator_pusat_keputusan),
        )
        // Barang Operations
        .route(
            "/kebutuhan-bmn/satker/{id}/barang",
            post(kebutuhan_bmn::create_barang),
        )
        .route(
            "/kebutuhan-bmn/barang/{id}/approval",
            put(kebutuhan_bmn::update_barang_approval),
        )
        .route(
            "/kebutuhan-bmn/barang/{id}",
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
            "/kebutuhan-bmn/siman/summary/{satker_id}",
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
        .route("/export/jobs/{id}/status", get(get_export_job_status))
        .route("/export/jobs/{id}/download", get(download_export_job))
        // ============ Mapping Kodefikasi Routes (Read-Only + Export) ============
        .route(
            "/mapping/detect",
            get(mapping_kodefikasi::detect_non_standard_codes),
        )
        .route(
            "/mapping/standard",
            get(mapping_kodefikasi::list_standard_codes),
        )
        .route(
            "/mapping/suggestions",
            get(mapping_kodefikasi::get_mapping_suggestions),
        )
        .route(
            "/mapping/progress",
            get(mapping_kodefikasi::get_mapping_progress),
        )
        .route(
            "/mapping/progress/satker",
            get(mapping_kodefikasi::get_mapping_progress_by_satker),
        )
        .route("/mapping/export", get(mapping_kodefikasi::export_mapping))
        // ============ Predictive Analytics (Forecast) Routes ============
        .route("/forecast", get(roadmap_sarpras::get_forecast))
        .route(
            "/forecast/summary",
            get(roadmap_sarpras::get_forecast_summary),
        )
        .route(
            "/forecast/compare",
            get(roadmap_sarpras::get_forecast_compare),
        )
        .route("/forecast/export", get(roadmap_sarpras::export_forecast))
        // ============ Pemakaian BMN Routes ============
        // Permit CRUD
        .route(
            "/pemakaian-bmn",
            get(pemakaian_bmn::list_permits).post(pemakaian_bmn::create_permit),
        )
        .route(
            "/pemakaian-bmn/{id}",
            get(pemakaian_bmn::get_permit_by_id).put(pemakaian_bmn::update_permit),
        )
        // Workflow Actions
        .route(
            "/pemakaian-bmn/{id}/transition",
            post(pemakaian_bmn::transition_permit_status),
        )
        .route(
            "/pemakaian-bmn/{id}/activate",
            post(pemakaian_bmn::activate_permit),
        )
        // V035 (Fase 1.5): Alur internal-satker 3-step approval.
        .route(
            "/pemakaian-bmn/{id}/validator-satker-action",
            post(pemakaian_bmn::validator_satker_action),
        )
        .route(
            "/pemakaian-bmn/{id}/approver-satker-action",
            post(pemakaian_bmn::approver_satker_action),
        )
        .route(
            "/pemakaian-bmn/{id}/resubmit",
            post(pemakaian_bmn::operator_resubmit),
        )
        .route(
            "/pemakaian-bmn/{id}/document",
            get(pemakaian_bmn::get_permit_document),
        )
        // Document Generation & Upload
        .route(
            "/pemakaian-bmn/{id}/generate-konsep-surat",
            post(pemakaian_bmn::generate_konsep_surat),
        )
        // Stream the konsep surat in either DOCX (editable) or PDF (final)
        // form. {format} = "docx" | "pdf".
        .route(
            "/pemakaian-bmn/{id}/konsep-surat.{format}",
            get(pemakaian_bmn::serve_konsep_surat),
        )
        // Fase 1.10: SK Izin Pemakaian BMN 2-halaman PDF (info pegawai +
        // daftar BMN). Stream inline.
        .route(
            "/pemakaian-bmn/{id}/sk-izin.pdf",
            get(pemakaian_bmn::serve_sk_izin_pdf),
        )
        .route(
            "/pemakaian-bmn/{id}/upload-signed-pdf",
            post(pemakaian_bmn::upload_signed_pdf),
        )
        .route(
            "/pemakaian-bmn/{id}/revoke",
            post(pemakaian_bmn::revoke_permit),
        )
        .route(
            "/pemakaian-bmn/{id}/renew",
            post(pemakaian_bmn::renew_permit),
        )
        // BMN Availability & History
        .route(
            "/pemakaian-bmn/bmn/{bmn_nup}/availability",
            get(pemakaian_bmn::check_bmn_availability),
        )
        .route(
            "/pemakaian-bmn/bmn/{bmn_nup}/history",
            get(pemakaian_bmn::get_bmn_usage_history),
        )
        // Pegawai History
        .route(
            "/pemakaian-bmn/pegawai/{pegawai_nip}/history",
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
        // Monitoring Dashboard (read-only — Validator Wilayah & Pusat)
        .route(
            "/pemakaian-bmn/monitoring/summary",
            get(pemakaian_bmn::get_monitoring_summary),
        )
        .route(
            "/pemakaian-bmn/monitoring/active-usage",
            get(pemakaian_bmn::get_active_usage_dashboard),
        )
        .route(
            "/pemakaian-bmn/monitoring/utilization-report",
            get(pemakaian_bmn::get_bmn_utilization_report),
        )
        // ============ Workflow Definition Routes ============
        .route(
            "/workflow/definitions",
            get(crate::workflow::definition_handlers::get_workflow_definitions)
                .post(crate::workflow::definition_handlers::create_workflow_definition),
        )
        .route(
            "/workflow/definitions/{name}",
            get(crate::workflow::definition_handlers::get_workflow_definition_by_name)
                .put(crate::workflow::definition_handlers::update_workflow_definition)
                .delete(crate::workflow::definition_handlers::delete_workflow_definition),
        )
        .route(
            "/workflow/definitions/{name}/steps",
            post(crate::workflow::definition_handlers::upsert_workflow_step),
        )
        .route(
            "/workflow/definitions/{name}/steps/{state}",
            delete(crate::workflow::definition_handlers::delete_workflow_step),
        )
        // ============ Workflow Monitoring Routes ============
        .route(
            "/workflow/monitoring/metrics",
            get(crate::workflow::handlers::get_workflow_metrics),
        )
        .route(
            "/workflow/monitoring/active",
            get(crate::workflow::handlers::get_active_workflows),
        )
        .route(
            "/workflow/monitoring/sla-breaches",
            get(crate::workflow::handlers::get_sla_breaches),
        )
        .route(
            "/workflow/monitoring/bottlenecks",
            get(crate::workflow::handlers::get_bottlenecks),
        )
        // ============ Workflow Delegation Routes ============
        .route(
            "/workflow/delegations",
            get(crate::workflow::delegation_handlers::list_delegations_handler)
                .post(crate::workflow::delegation_handlers::create_delegation_handler),
        )
        .route(
            "/workflow/delegations/{id}/revoke",
            post(crate::workflow::delegation_handlers::revoke_delegation_handler),
        )
        // ============ Integrasi circuit-breaker status (FE banner) ============
        // Auth'd, lightweight view of SIMAN/MySIMKARI/MonSAKTI breaker state.
        .route(
            "/integrasi/circuit-status",
            get(crate::shared::health::integrasi_circuit_status),
        )
        // ============ Cross-module audit trail (BPK-ready) ============
        // Reads perlengkapan.audit_log (every module's AuditSink events),
        // unlike /admin/audit which only shows workflow transitions. Pull a
        // single record's full history via ?entity=<type>&resource_id=<id>.
        .route("/audit", get(crate::audit::list_audit_trail))
        // ============ Admin Routes (audit + master data + templates) ============
        .route("/admin/audit", get(crate::admin::list_audit_logs))
        .route("/admin/master", get(crate::admin::list_master_sources))
        .route(
            "/admin/master/{source}",
            get(crate::admin::list_master_records).post(crate::admin::create_master_record),
        )
        .route(
            "/admin/master/{source}/{id}",
            put(crate::admin::update_master_record).delete(crate::admin::delete_master_record),
        )
        // ============ Notifikasi Inbox ============
        // Per-user inbox surfaced from `notifikasi.in_app_notifications`.
        // `NotifikasiService::send` is the canonical writer; these endpoints
        // are the reader half.
        .route(
            "/notifikasi/unread-count",
            get(crate::notifikasi::api::unread_count),
        )
        .route("/notifikasi", get(crate::notifikasi::api::list_notifikasi))
        .route(
            "/notifikasi/{id}/read",
            axum::routing::patch(crate::notifikasi::api::mark_read),
        )
        .route(
            "/notifikasi/read-all",
            post(crate::notifikasi::api::mark_all_read),
        )
        .route("/admin/templates", get(crate::admin::list_templates))
        .route("/admin/templates/{id}", get(crate::admin::get_template))
        // POST renders a live preview of the template without persisting.
        // `?format=pdf|docx|xlsx|html|csv`; body `{ "data": {...} }` overrides
        // the template's stored sample_data.
        .route(
            "/admin/templates/{id}/preview",
            post(crate::admin::preview_template),
        )
        // User catalog + role assignments — backed by v_user_role_summary
        // (migration V018).
        .route("/admin/users", get(crate::admin::list_users))
        .route("/admin/users/{nip}", get(crate::admin::get_user))
        .route("/admin/users/{nip}/roles", post(crate::admin::assign_role))
        .route(
            "/admin/users/{nip}/roles/{role}",
            delete(crate::admin::unassign_role),
        )
        .with_state(state)
}
