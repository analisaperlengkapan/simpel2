//! Centralized route constants for perlengkapan frontend.

pub mod path {
    #[cfg(target_arch = "wasm32")]
    pub const LOGIN: &str = "/perlengkapan/simpel/login";
    pub const PORTAL_LOGIN_WITH_REDIRECT: &str =
        "/portal/login?redirect_uri=%2Fperlengkapan%2Fsimpel%2Fv2%2Fdashboard";

    pub const DASHBOARD: &str = "/perlengkapan/simpel/dashboard";

    pub const BANK_ASET_DASHBOARD: &str = "/perlengkapan/simpel/bank-aset/dashboard";
    pub const BANK_ASET_DAFTAR: &str = "/perlengkapan/simpel/bank-aset/daftar";
    pub const BANK_ASET_SEBARAN: &str = "/perlengkapan/simpel/bank-aset/sebaran";
    pub const BANK_ASET_QRCODE: &str = "/perlengkapan/simpel/bank-aset/qrcode";

    pub const KEBUTUHAN_DAFTAR: &str = "/perlengkapan/simpel/kebutuhan-bmn/daftar";
    pub const KEBUTUHAN_BUAT: &str = "/perlengkapan/simpel/kebutuhan-bmn/buat";
    pub const KEBUTUHAN_LAPORAN: &str = "/perlengkapan/simpel/kebutuhan-bmn/laporan";

    pub const PAKAIAN_JENIS: &str = "/perlengkapan/simpel/pakaian-dinas/jenis";
    pub const PAKAIAN_PENGAJUAN: &str = "/perlengkapan/simpel/pakaian-dinas/pengajuan";
    pub const PAKAIAN_UKURAN: &str = "/perlengkapan/simpel/pakaian-dinas/ukuran";
    pub const PAKAIAN_LAPORAN: &str = "/perlengkapan/simpel/pakaian-dinas/laporan";

    pub const PENGELOLAAN_PEMAKAIAN: &str = "/perlengkapan/simpel/pengelolaan/pemakaian";
    pub const PENGELOLAAN_PENGHAPUSAN: &str = "/perlengkapan/simpel/pengelolaan/penghapusan";
    pub const PENGELOLAAN_PEMAKAIAN_BUAT: &str = "/perlengkapan/simpel/pengelolaan/pemakaian/buat";
    pub const PENGELOLAAN_PENGHAPUSAN_BUAT: &str =
        "/perlengkapan/simpel/pengelolaan/penghapusan/buat";

    pub const ANALITIK_ROADMAP: &str = "/perlengkapan/simpel/analitik/roadmap";
    pub const ANALITIK_ROADMAP_BUAT: &str = "/perlengkapan/simpel/analitik/roadmap/buat";

    // No ADMIN_USERS: the page was removed — it queried relations no migration
    // creates, and user administration belongs to authenc (portal provides it).
    pub const ADMIN_ROLES: &str = "/perlengkapan/simpel/admin/roles";
    pub const ADMIN_AUDIT: &str = "/perlengkapan/simpel/admin/audit";
    pub const ADMIN_MASTER: &str = "/perlengkapan/simpel/admin/master";
    pub const ADMIN_TEMPLATES: &str = "/perlengkapan/simpel/admin/templates";
    pub const ADMIN_SYNC_STATUS: &str = "/perlengkapan/simpel/admin/status-sinkronisasi";
    pub const NOTIFIKASI: &str = "/perlengkapan/simpel/notifikasi";
    pub const ADMIN_WORKFLOW: &str = "/perlengkapan/simpel/admin/workflow";
    pub const ADMIN_WORKFLOW_MONITORING: &str = "/perlengkapan/simpel/admin/workflow-monitoring";

    pub const BANTUAN_PANDUAN: &str = "/perlengkapan/simpel/bantuan/panduan";
    pub const BANTUAN_FAQ: &str = "/perlengkapan/simpel/bantuan/faq";
    pub const BANTUAN_HELPDESK: &str = "/perlengkapan/simpel/bantuan/helpdesk";
}

pub mod url {
    pub fn kebutuhan_detail(id: &str) -> String {
        format!("/perlengkapan/simpel/kebutuhan-bmn/detail/{}", id)
    }

    pub fn kebutuhan_edit(id: &str) -> String {
        format!("/perlengkapan/simpel/kebutuhan-bmn/{}/edit", id)
    }

    pub fn kebutuhan_satker_detail(satker_id: &str) -> String {
        format!("/perlengkapan/simpel/kebutuhan-bmn/satker/{}", satker_id)
    }

    pub fn pakaian_satker_list(pengajuan_id: &str) -> String {
        format!(
            "/perlengkapan/simpel/pakaian-dinas/pengajuan/{}/satker",
            pengajuan_id
        )
    }

    pub fn bank_aset_detail(id: &str) -> String {
        format!("/perlengkapan/simpel/bank-aset/daftar/{}", id)
    }

    pub fn pakaian_pengisian(pengajuan_id: &str, satker_code: &str) -> String {
        format!(
            "/perlengkapan/simpel/pakaian-dinas/pengajuan/{}/satker/{}/isi",
            pengajuan_id, satker_code
        )
    }

    pub fn pakaian_spesifikasi(jenis_id: impl std::fmt::Display) -> String {
        format!(
            "/perlengkapan/simpel/pakaian-dinas/jenis/{}/spesifikasi",
            jenis_id
        )
    }

    pub fn pemakaian_detail(id: impl std::fmt::Display) -> String {
        format!("/perlengkapan/simpel/pengelolaan/pemakaian/detail/{}", id)
    }

    pub fn penghapusan_detail(id: impl std::fmt::Display) -> String {
        format!("/perlengkapan/simpel/pengelolaan/penghapusan/detail/{}", id)
    }
}
