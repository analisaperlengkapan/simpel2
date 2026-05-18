//! Centralized route constants for perlengkapan frontend.

pub mod path {
    pub const ROOT: &str = "/perlengkapan/simpel/v2/";
    pub const LOGIN: &str = "/perlengkapan/simpel/v2/login";
    pub const PORTAL_LOGIN: &str = "/portal/login";
    pub const PORTAL_LOGIN_WITH_REDIRECT: &str =
        "/portal/login?redirect_uri=%2Fperlengkapan%2Fsimpel%2Fv2%2Fdashboard";

    pub const DASHBOARD: &str = "/perlengkapan/simpel/v2/dashboard";
    pub const DASHBOARD_SEARCH: &str = "/perlengkapan/simpel/v2/dashboard/search";

    pub const BANK_ASET_DASHBOARD: &str = "/perlengkapan/simpel/v2/bank-aset/dashboard";
    pub const BANK_ASET_DAFTAR: &str = "/perlengkapan/simpel/v2/bank-aset/daftar";
    pub const BANK_ASET_SEBARAN: &str = "/perlengkapan/simpel/v2/bank-aset/sebaran";
    pub const BANK_ASET_QRCODE: &str = "/perlengkapan/simpel/v2/bank-aset/qrcode";

    pub const KEBUTUHAN_DAFTAR: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/daftar";
    pub const KEBUTUHAN_BUAT: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/buat";
    pub const KEBUTUHAN_LAPORAN: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/laporan";
    pub const KEBUTUHAN_DAFTAR_LEGACY: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn";
    pub const KEBUTUHAN_BUAT_LEGACY: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/baru";

    pub const PAKAIAN_JENIS: &str = "/perlengkapan/simpel/v2/pakaian-dinas/jenis";
    pub const PAKAIAN_PENGAJUAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/pengajuan";
    pub const PAKAIAN_UKURAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/ukuran";
    pub const PAKAIAN_LAPORAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/laporan";
    pub const PAKAIAN_LAPORAN_REKAP_LEGACY: &str = "/perlengkapan/simpel/v2/pakaian-dinas/laporan/rekap";

    pub const PENGELOLAAN_PEMAKAIAN: &str = "/perlengkapan/simpel/v2/pengelolaan/pemakaian";
    pub const PENGELOLAAN_PENGHAPUSAN: &str = "/perlengkapan/simpel/v2/pengelolaan/penghapusan";
    pub const PENGELOLAAN_PEMAKAIAN_BUAT: &str = "/perlengkapan/simpel/v2/pengelolaan/pemakaian/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_BUAT: &str = "/perlengkapan/simpel/v2/pengelolaan/penghapusan/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_DAFTAR_LEGACY: &str =
        "/perlengkapan/simpel/v2/pengelolaan/penghapusan/daftar";

    pub const PEMAKAIAN_DAFTAR_LEGACY: &str = "/perlengkapan/simpel/v2/pemakaian-bmn";
    pub const PEMAKAIAN_BUAT_LEGACY: &str = "/perlengkapan/simpel/v2/pemakaian-bmn/baru";

    pub const ANALITIK_ROADMAP: &str = "/perlengkapan/simpel/v2/analitik/roadmap";
    pub const ANALITIK_KODEFIKASI: &str = "/perlengkapan/simpel/v2/analitik/kodefikasi";
    pub const ANALISIS_DAFTAR_LEGACY: &str = "/perlengkapan/simpel/v2/analisis/daftar";
    pub const ANALISIS_BUAT_LEGACY: &str = "/perlengkapan/simpel/v2/analisis/baru";

    pub const ADMIN_USERS: &str = "/perlengkapan/simpel/v2/admin/users";
    pub const ADMIN_ROLES: &str = "/perlengkapan/simpel/v2/admin/roles";
    pub const ADMIN_AUDIT: &str = "/perlengkapan/simpel/v2/admin/audit";
    pub const ADMIN_MASTER: &str = "/perlengkapan/simpel/v2/admin/master";
    pub const ADMIN_WORKFLOW: &str = "/perlengkapan/simpel/v2/admin/workflow";
    pub const ADMIN_WORKFLOW_MONITORING: &str = "/perlengkapan/simpel/v2/admin/workflow-monitoring";

    pub const BANTUAN_PANDUAN: &str = "/perlengkapan/simpel/v2/bantuan/panduan";
    pub const BANTUAN_FAQ: &str = "/perlengkapan/simpel/v2/bantuan/faq";
    pub const BANTUAN_HELPDESK: &str = "/perlengkapan/simpel/v2/bantuan/helpdesk";
}

pub mod url {
    pub fn kebutuhan_detail(id: &str) -> String {
        format!("/perlengkapan/simpel/v2/kebutuhan-bmn/detail/{}", id)
    }

    pub fn kebutuhan_edit(id: &str) -> String {
        format!("/perlengkapan/simpel/v2/kebutuhan-bmn/{}/edit", id)
    }

    pub fn kebutuhan_satker_detail(satker_id: &str) -> String {
        format!("/perlengkapan/simpel/v2/kebutuhan-bmn/satker/{}", satker_id)
    }

    pub fn bank_aset_detail(id: &str) -> String {
        format!("/perlengkapan/simpel/v2/bank-aset/daftar/{}", id)
    }

    pub fn dashboard_perlengkapan_with_query(query: &str) -> String {
        if query.is_empty() {
            super::path::DASHBOARD.to_string()
        } else {
            format!("{}?{}", super::path::DASHBOARD, query)
        }
    }
}
