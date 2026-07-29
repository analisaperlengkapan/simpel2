//! Centralized route constants for perlengkapan frontend.

pub mod path {
    #[cfg(target_arch = "wasm32")]
    pub const LOGIN: &str = "/perlengkapan/simpel/v2/login";
    pub const PORTAL_LOGIN_WITH_REDIRECT: &str =
        "/portal/login?redirect_uri=%2Fperlengkapan%2Fsimpel%2Fv2%2Fdashboard";

    pub const DASHBOARD: &str = "/perlengkapan/simpel/v2/dashboard";

    pub const BANK_ASET_DASHBOARD: &str = "/perlengkapan/simpel/v2/bank-aset/dashboard";
    pub const BANK_ASET_DAFTAR: &str = "/perlengkapan/simpel/v2/bank-aset/daftar";
    pub const BANK_ASET_SEBARAN: &str = "/perlengkapan/simpel/v2/bank-aset/sebaran";
    pub const BANK_ASET_QRCODE: &str = "/perlengkapan/simpel/v2/bank-aset/qrcode";

    pub const KEBUTUHAN_DAFTAR: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/daftar";
    pub const KEBUTUHAN_BUAT: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/buat";
    pub const KEBUTUHAN_LAPORAN: &str = "/perlengkapan/simpel/v2/kebutuhan-bmn/laporan";

    pub const PAKAIAN_JENIS: &str = "/perlengkapan/simpel/v2/pakaian-dinas/jenis";
    pub const PAKAIAN_PENGAJUAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/pengajuan";
    pub const PAKAIAN_UKURAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/ukuran";
    pub const PAKAIAN_LAPORAN: &str = "/perlengkapan/simpel/v2/pakaian-dinas/laporan";

    pub const PENGELOLAAN_PEMAKAIAN: &str = "/perlengkapan/simpel/v2/pengelolaan/pemakaian";
    pub const PENGELOLAAN_PENGHAPUSAN: &str = "/perlengkapan/simpel/v2/pengelolaan/penghapusan";
    pub const PENGELOLAAN_PEMAKAIAN_BUAT: &str =
        "/perlengkapan/simpel/v2/pengelolaan/pemakaian/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_BUAT: &str =
        "/perlengkapan/simpel/v2/pengelolaan/penghapusan/buat";

    pub const ANALITIK_ROADMAP: &str = "/perlengkapan/simpel/v2/analitik/roadmap";
    pub const ANALITIK_ROADMAP_BUAT: &str = "/perlengkapan/simpel/v2/analitik/roadmap/buat";

    pub const ADMIN_USERS: &str = "/perlengkapan/simpel/v2/admin/users";
    pub const ADMIN_ROLES: &str = "/perlengkapan/simpel/v2/admin/roles";
    pub const ADMIN_AUDIT: &str = "/perlengkapan/simpel/v2/admin/audit";
    pub const ADMIN_MASTER: &str = "/perlengkapan/simpel/v2/admin/master";
    pub const ADMIN_TEMPLATES: &str = "/perlengkapan/simpel/v2/admin/templates";
    pub const NOTIFIKASI: &str = "/perlengkapan/simpel/v2/notifikasi";
    pub const ADMIN_WORKFLOW: &str = "/perlengkapan/simpel/v2/admin/workflow";
    pub const ADMIN_WORKFLOW_MONITORING: &str = "/perlengkapan/simpel/v2/admin/workflow-monitoring";
    pub const ADMIN_WORKFLOW_DELEGATION: &str = "/perlengkapan/simpel/v2/admin/workflow-delegation";

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

    pub fn pakaian_satker_list(pengajuan_id: &str) -> String {
        format!(
            "/perlengkapan/simpel/v2/pakaian-dinas/pengajuan/{}/satker",
            pengajuan_id
        )
    }

    pub fn bank_aset_detail(id: &str) -> String {
        format!("/perlengkapan/simpel/v2/bank-aset/daftar/{}", id)
    }

    pub fn pakaian_spesifikasi(jenis_id: impl std::fmt::Display) -> String {
        format!(
            "/perlengkapan/simpel/v2/pakaian-dinas/jenis/{}/spesifikasi",
            jenis_id
        )
    }

    pub fn pemakaian_detail(id: impl std::fmt::Display) -> String {
        format!(
            "/perlengkapan/simpel/v2/pengelolaan/pemakaian/detail/{}",
            id
        )
    }

    pub fn penghapusan_detail(id: impl std::fmt::Display) -> String {
        format!(
            "/perlengkapan/simpel/v2/pengelolaan/penghapusan/detail/{}",
            id
        )
    }
}
