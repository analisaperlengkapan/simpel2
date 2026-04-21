//! Centralized route constants for perlengkapan frontend.

pub mod path {
    pub const ROOT: &str = "/perlengkapan/";
    pub const LOGIN: &str = "/perlengkapan/login";
    pub const PORTAL_LOGIN: &str = "/portal/login";
    pub const PORTAL_LOGIN_WITH_REDIRECT: &str =
        "/portal/login?redirect_uri=%2Fperlengkapan%2Fdashboard";

    pub const DASHBOARD: &str = "/perlengkapan/dashboard";
    pub const DASHBOARD_SEARCH: &str = "/perlengkapan/dashboard/search";

    pub const BANK_ASET_DASHBOARD: &str = "/perlengkapan/bank-aset/dashboard";
    pub const BANK_ASET_DAFTAR: &str = "/perlengkapan/bank-aset/daftar";
    pub const BANK_ASET_SEBARAN: &str = "/perlengkapan/bank-aset/sebaran";
    pub const BANK_ASET_QRCODE: &str = "/perlengkapan/bank-aset/qrcode";

    pub const KEBUTUHAN_DAFTAR: &str = "/perlengkapan/kebutuhan-bmn/daftar";
    pub const KEBUTUHAN_BUAT: &str = "/perlengkapan/kebutuhan-bmn/buat";
    pub const KEBUTUHAN_LAPORAN: &str = "/perlengkapan/kebutuhan-bmn/laporan";
    pub const KEBUTUHAN_DAFTAR_LEGACY: &str = "/perlengkapan/kebutuhan-bmn";
    pub const KEBUTUHAN_BUAT_LEGACY: &str = "/perlengkapan/kebutuhan-bmn/baru";

    pub const PAKAIAN_JENIS: &str = "/perlengkapan/pakaian-dinas/jenis";
    pub const PAKAIAN_PENGAJUAN: &str = "/perlengkapan/pakaian-dinas/pengajuan";
    pub const PAKAIAN_UKURAN: &str = "/perlengkapan/pakaian-dinas/ukuran";
    pub const PAKAIAN_LAPORAN: &str = "/perlengkapan/pakaian-dinas/laporan";
    pub const PAKAIAN_LAPORAN_REKAP_LEGACY: &str =
        "/perlengkapan/pakaian-dinas/laporan/rekap";

    pub const PENGELOLAAN_PEMAKAIAN: &str = "/perlengkapan/pengelolaan/pemakaian";
    pub const PENGELOLAAN_PENGHAPUSAN: &str = "/perlengkapan/pengelolaan/penghapusan";
    pub const PENGELOLAAN_PEMAKAIAN_BUAT: &str =
        "/perlengkapan/pengelolaan/pemakaian/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_BUAT: &str =
        "/perlengkapan/pengelolaan/penghapusan/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_DAFTAR_LEGACY: &str =
        "/perlengkapan/pengelolaan/penghapusan/daftar";

    pub const PEMAKAIAN_DAFTAR_LEGACY: &str = "/perlengkapan/pemakaian-bmn";
    pub const PEMAKAIAN_BUAT_LEGACY: &str = "/perlengkapan/pemakaian-bmn/baru";

    pub const ANALITIK_ROADMAP: &str = "/perlengkapan/analitik/roadmap";
    pub const ANALITIK_KODEFIKASI: &str = "/perlengkapan/analitik/kodefikasi";
    pub const ANALISIS_DAFTAR_LEGACY: &str = "/perlengkapan/analisis/daftar";
    pub const ANALISIS_BUAT_LEGACY: &str = "/perlengkapan/analisis/baru";

    pub const ADMIN_USERS: &str = "/perlengkapan/admin/users";
    pub const ADMIN_ROLES: &str = "/perlengkapan/admin/roles";
    pub const ADMIN_AUDIT: &str = "/perlengkapan/admin/audit";
    pub const ADMIN_MASTER: &str = "/perlengkapan/admin/master";
    pub const ADMIN_WORKFLOW: &str = "/perlengkapan/admin/workflow";
    pub const ADMIN_WORKFLOW_MONITORING: &str = "/perlengkapan/admin/workflow-monitoring";

    pub const BANTUAN_PANDUAN: &str = "/perlengkapan/bantuan/panduan";
    pub const BANTUAN_FAQ: &str = "/perlengkapan/bantuan/faq";
    pub const BANTUAN_HELPDESK: &str = "/perlengkapan/bantuan/helpdesk";
}

pub mod url {
    pub fn kebutuhan_detail(id: &str) -> String {
        format!("/perlengkapan/kebutuhan-bmn/detail/{}", id)
    }

    pub fn kebutuhan_edit(id: &str) -> String {
        format!("/perlengkapan/kebutuhan-bmn/{}/edit", id)
    }

    pub fn kebutuhan_satker_detail(satker_id: &str) -> String {
        format!("/perlengkapan/kebutuhan-bmn/satker/{}", satker_id)
    }

    pub fn bank_aset_detail(id: &str) -> String {
        format!("/perlengkapan/bank-aset/daftar/{}", id)
    }

    pub fn dashboard_perlengkapan_with_query(query: &str) -> String {
        if query.is_empty() {
            super::path::DASHBOARD.to_string()
        } else {
            format!("{}?{}", super::path::DASHBOARD, query)
        }
    }
}
