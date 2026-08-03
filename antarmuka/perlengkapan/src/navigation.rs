//! Centralized navigation registry for Perlengkapan sidebar.

use crate::routes;

#[derive(Clone, Copy)]
pub struct NavItem {
    pub href: &'static str,
    pub icon: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy)]
pub struct NavGroup {
    pub icon: &'static str,
    pub label: &'static str,
    pub accent: &'static str,
    /// Shown only when the active role is `admin` (pages are additionally
    /// guarded server-side + by `AdminLayout`; this only declutters the nav).
    pub admin_only: bool,
    pub items: &'static [NavItem],
}

#[derive(Clone, Copy)]
pub struct NavSection {
    pub title: &'static str,
    pub groups: &'static [NavGroup],
}

pub const DASHBOARD_ITEM: NavItem = NavItem {
    href: routes::path::DASHBOARD,
    icon: "fas fa-home",
    label: "Dashboard",
};

const MODUL_UTAMA_GROUPS: &[NavGroup] = &[
    NavGroup {
        icon: "fas fa-boxes",
        label: "Bank Aset",
        accent: "#34d399",
        admin_only: false,
        items: &[
            NavItem {
                href: routes::path::BANK_ASET_DASHBOARD,
                icon: "fas fa-chart-simple",
                label: "Dashboard",
            },
            NavItem {
                href: routes::path::BANK_ASET_DAFTAR,
                icon: "fas fa-list",
                label: "Daftar Aset",
            },
            NavItem {
                href: routes::path::BANK_ASET_SEBARAN,
                icon: "fas fa-map-location-dot",
                label: "Sebaran",
            },
            NavItem {
                href: routes::path::BANK_ASET_QRCODE,
                icon: "fas fa-qrcode",
                label: "Cetak QR Code",
            },
        ],
    },
    NavGroup {
        icon: "fas fa-clipboard-list",
        label: "Kebutuhan BMN",
        accent: "#fb923c",
        admin_only: false,
        items: &[
            NavItem {
                href: routes::path::KEBUTUHAN_DAFTAR,
                icon: "fas fa-list",
                label: "Daftar",
            },
            NavItem {
                href: routes::path::KEBUTUHAN_BUAT,
                icon: "fas fa-plus-circle",
                label: "Buat Baru",
            },
            NavItem {
                href: routes::path::KEBUTUHAN_LAPORAN,
                icon: "fas fa-chart-bar",
                label: "Laporan",
            },
        ],
    },
    NavGroup {
        icon: "fas fa-tshirt",
        label: "Pakaian Dinas",
        accent: "#c084fc",
        admin_only: false,
        items: &[
            NavItem {
                href: routes::path::PAKAIAN_JENIS,
                icon: "fas fa-tags",
                label: "Jenis",
            },
            NavItem {
                href: routes::path::PAKAIAN_PENGAJUAN,
                icon: "fas fa-paper-plane",
                label: "Pengajuan",
            },
            NavItem {
                href: routes::path::PAKAIAN_UKURAN,
                icon: "fas fa-ruler",
                label: "Ukuran",
            },
            NavItem {
                href: routes::path::PAKAIAN_LAPORAN,
                icon: "fas fa-chart-bar",
                label: "Laporan",
            },
        ],
    },
    NavGroup {
        icon: "fas fa-cogs",
        label: "Pengelolaan BMN",
        accent: "#60a5fa",
        admin_only: false,
        items: &[
            NavItem {
                href: routes::path::PENGELOLAAN_PEMAKAIAN,
                icon: "fas fa-file-signature",
                label: "Pemakaian BMN",
            },
            NavItem {
                href: routes::path::PENGELOLAAN_PENGHAPUSAN,
                icon: "fas fa-trash-alt",
                label: "Penghapusan",
            },
        ],
    },
];

const ANALITIK_GROUPS: &[NavGroup] = &[NavGroup {
    icon: "fas fa-chart-line",
    label: "Analitik",
    accent: "#2dd4bf",
    admin_only: false,
    items: &[NavItem {
        href: routes::path::ANALITIK_ROADMAP,
        icon: "fas fa-road",
        label: "Roadmap Sarpras",
    }],
}];

const ADMINISTRASI_GROUPS: &[NavGroup] = &[
    NavGroup {
        icon: "fas fa-shield-alt",
        label: "Admin",
        accent: "#f87171",
        admin_only: true,
        items: &[
            // No "Pengguna" entry: that page was removed (user administration
            // lives in portal, against authenc).
            NavItem {
                href: routes::path::ADMIN_ROLES,
                icon: "fas fa-user-tag",
                label: "Otorisasi",
            },
            NavItem {
                href: routes::path::ADMIN_AUDIT,
                icon: "fas fa-history",
                label: "Audit Log",
            },
            NavItem {
                href: routes::path::ADMIN_MASTER,
                icon: "fas fa-database",
                label: "Master Data",
            },
            NavItem {
                href: routes::path::ADMIN_TEMPLATES,
                icon: "fas fa-file-alt",
                label: "Template Dokumen",
            },
            NavItem {
                href: routes::path::ADMIN_WORKFLOW,
                icon: "fas fa-project-diagram",
                label: "Konfigurasi Workflow",
            },
            NavItem {
                href: routes::path::ADMIN_WORKFLOW_MONITORING,
                icon: "fas fa-wave-square",
                label: "Monitoring Workflow",
            },
            NavItem {
                href: routes::path::ADMIN_WORKFLOW_DELEGATION,
                icon: "fas fa-people-arrows",
                label: "Delegasi Workflow",
            },
        ],
    },
    NavGroup {
        icon: "fas fa-life-ring",
        label: "Bantuan",
        accent: "#94a3b8",
        admin_only: false,
        items: &[
            NavItem {
                href: routes::path::BANTUAN_PANDUAN,
                icon: "fas fa-book",
                label: "Panduan",
            },
            NavItem {
                href: routes::path::BANTUAN_FAQ,
                icon: "fas fa-question-circle",
                label: "FAQ",
            },
            NavItem {
                href: routes::path::BANTUAN_HELPDESK,
                icon: "fas fa-headset",
                label: "Helpdesk",
            },
        ],
    },
];

pub const SIDEBAR_SECTIONS: &[NavSection] = &[
    NavSection {
        title: "MODUL UTAMA",
        groups: MODUL_UTAMA_GROUPS,
    },
    NavSection {
        title: "ANALITIK",
        groups: ANALITIK_GROUPS,
    },
    NavSection {
        title: "ADMINISTRASI",
        groups: ADMINISTRASI_GROUPS,
    },
];
