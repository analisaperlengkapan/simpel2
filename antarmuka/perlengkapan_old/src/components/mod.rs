// ── Admin ──────────────────────────────────────────────────────────────────
pub mod admin_panel;

// ── Analisis / Aset ────────────────────────────────────────────────────────
pub mod analisis_form;
pub mod analisis_list;
pub mod aset_list;
pub mod batch_operations_toolbar;
pub mod dashboard_filters;

// ── Hibah ──────────────────────────────────────────────────────────────────
pub mod hibah_form;
pub mod hibah_list;

// ── Kebutuhan BMN ──────────────────────────────────────────────────────────
pub mod kebutuhan_bmn_dashboard;
pub mod kebutuhan_bmn_detail;
pub mod kebutuhan_bmn_form;
pub mod kebutuhan_bmn_list;
pub mod kebutuhan_bmn_satker;
pub mod kebutuhan_bmn_search;

// ── Mapping Kodefikasi ─────────────────────────────────────────────────────
pub mod mapping_kodefikasi_dashboard;
pub mod mapping_kodefikasi_form;
pub mod mapping_kodefikasi_verification;

// ── Mutasi ─────────────────────────────────────────────────────────────────
pub mod mutasi_form;
pub mod mutasi_list;

// ── Pakaian Dinas ──────────────────────────────────────────────────────────
pub mod pakaian_dinas_jenis_list;
pub mod pakaian_dinas_laporan;
pub mod pakaian_dinas_pengajuan_list;
pub mod pakaian_dinas_ukuran;

// ── Pemakaian (legacy pengelolaan) ─────────────────────────────────────────
pub mod pemakaian_form;
pub mod pemakaian_list;

// ── Pemakaian BMN (workflow) ────────────────────────────────────────────────
pub mod pemakaian_bmn_detail;
pub mod pemakaian_bmn_form;
pub mod pemakaian_bmn_list;
pub mod pemakaian_bmn_monitoring;
pub mod pemakaian_bmn_renew;

// ── Pemeliharaan ───────────────────────────────────────────────────────────
pub mod pemeliharaan_form;
pub mod pemeliharaan_list;

// ── Pengalihan ─────────────────────────────────────────────────────────────
pub mod pengalihan_form;
pub mod pengalihan_list;

// ── Penghapusan ────────────────────────────────────────────────────────────
pub mod penghapusan_bmn_detail;
pub mod penghapusan_form;
pub mod penghapusan_list;

// ── Perlengkapan Dashboard (analytics) ────────────────────────────────────
pub mod perlengkapan_dashboard;

// ── Roadmap / Predictive Analytics ────────────────────────────────────────
pub mod roadmap_sarpras_timeline;

// ── Shell ──────────────────────────────────────────────────────────────────
pub mod auth;
pub mod content;
pub mod login;
pub mod role_switcher;
pub mod sidebar;
pub mod sidebar_section;
pub mod user_menu;

// ── SIMAN Integration ─────────────────────────────────────────────────────
pub mod siman_asset_search;
