//! Typed route enum for `antarmuka/perlengkapan`.
//!
//! Mirrors the constants in `antarmuka/perlengkapan/src/routes.rs`,
//! including its parameterized helpers in `mod url`. Once the
//! workspace build is healthy enough to verify a flag-day migration,
//! that file is deleted in favor of this enum.

use super::ToPath;

/// Every navigable route inside the Perlengkapan microfrontend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PerlengkapanRoute {
    Root,
    Login,
    /// Portal login redirect URL — kept here so cross-app navigation
    /// stays in one place rather than being recomputed in pages.
    PortalLogin,
    PortalLoginWithRedirect,

    Dashboard,
    DashboardSearch,
    /// Dashboard with an arbitrary querystring (e.g. filter args).
    DashboardWithQuery {
        query: String,
    },

    BankAsetDashboard,
    BankAsetDaftar,
    BankAsetSebaran,
    BankAsetQrcode,
    BankAsetDetail {
        id: String,
    },

    KebutuhanDaftar,
    KebutuhanBuat,
    KebutuhanLaporan,
    KebutuhanDetail {
        id: String,
    },
    KebutuhanEdit {
        id: String,
    },
    KebutuhanSatkerDetail {
        satker_id: String,
    },
    /// Legacy URLs preserved for redirect-table compatibility.
    KebutuhanDaftarLegacy,
    KebutuhanBuatLegacy,

    PakaianJenis,
    PakaianPengajuan,
    PakaianUkuran,
    PakaianLaporan,
    PakaianLaporanRekapLegacy,

    PengelolaanPemakaian,
    PengelolaanPenghapusan,
    PengelolaanPemakaianBuat,
    PengelolaanPenghapusanBuat,
    PengelolaanPenghapusanDaftarLegacy,

    PemakaianDaftarLegacy,
    PemakaianBuatLegacy,

    AnalitikRoadmap,
    AnalitikKodefikasi,
    AnalisisDaftarLegacy,
    AnalisisBuatLegacy,

    AdminUsers,
    AdminRoles,
    AdminAudit,
    AdminMaster,
    AdminWorkflow,
    AdminWorkflowMonitoring,

    BantuanPanduan,
    BantuanFaq,
    BantuanHelpdesk,
}

impl ToPath for PerlengkapanRoute {
    fn to_path(&self) -> String {
        use PerlengkapanRoute::*;
        match self {
            Root => "/perlengkapan/".to_string(),
            Login => "/perlengkapan/login".to_string(),
            PortalLogin => "/portal/login".to_string(),
            PortalLoginWithRedirect => {
                "/portal/login?redirect_uri=%2Fperlengkapan%2Fdashboard".to_string()
            }

            Dashboard => "/perlengkapan/dashboard".to_string(),
            DashboardSearch => "/perlengkapan/dashboard/search".to_string(),
            DashboardWithQuery { query } => {
                if query.is_empty() {
                    "/perlengkapan/dashboard".to_string()
                } else {
                    format!("/perlengkapan/dashboard?{query}")
                }
            }

            BankAsetDashboard => "/perlengkapan/bank-aset/dashboard".to_string(),
            BankAsetDaftar => "/perlengkapan/bank-aset/daftar".to_string(),
            BankAsetSebaran => "/perlengkapan/bank-aset/sebaran".to_string(),
            BankAsetQrcode => "/perlengkapan/bank-aset/qrcode".to_string(),
            BankAsetDetail { id } => format!("/perlengkapan/bank-aset/daftar/{id}"),

            KebutuhanDaftar => "/perlengkapan/kebutuhan-bmn/daftar".to_string(),
            KebutuhanBuat => "/perlengkapan/kebutuhan-bmn/buat".to_string(),
            KebutuhanLaporan => "/perlengkapan/kebutuhan-bmn/laporan".to_string(),
            KebutuhanDetail { id } => format!("/perlengkapan/kebutuhan-bmn/detail/{id}"),
            KebutuhanEdit { id } => format!("/perlengkapan/kebutuhan-bmn/{id}/edit"),
            KebutuhanSatkerDetail { satker_id } => {
                format!("/perlengkapan/kebutuhan-bmn/satker/{satker_id}")
            }
            KebutuhanDaftarLegacy => "/perlengkapan/kebutuhan-bmn".to_string(),
            KebutuhanBuatLegacy => "/perlengkapan/kebutuhan-bmn/baru".to_string(),

            PakaianJenis => "/perlengkapan/pakaian-dinas/jenis".to_string(),
            PakaianPengajuan => "/perlengkapan/pakaian-dinas/pengajuan".to_string(),
            PakaianUkuran => "/perlengkapan/pakaian-dinas/ukuran".to_string(),
            PakaianLaporan => "/perlengkapan/pakaian-dinas/laporan".to_string(),
            PakaianLaporanRekapLegacy => {
                "/perlengkapan/pakaian-dinas/laporan/rekap".to_string()
            }

            PengelolaanPemakaian => "/perlengkapan/pengelolaan/pemakaian".to_string(),
            PengelolaanPenghapusan => "/perlengkapan/pengelolaan/penghapusan".to_string(),
            PengelolaanPemakaianBuat => "/perlengkapan/pengelolaan/pemakaian/buat".to_string(),
            PengelolaanPenghapusanBuat => "/perlengkapan/pengelolaan/penghapusan/buat".to_string(),
            PengelolaanPenghapusanDaftarLegacy => {
                "/perlengkapan/pengelolaan/penghapusan/daftar".to_string()
            }

            PemakaianDaftarLegacy => "/perlengkapan/pemakaian-bmn".to_string(),
            PemakaianBuatLegacy => "/perlengkapan/pemakaian-bmn/baru".to_string(),

            AnalitikRoadmap => "/perlengkapan/analitik/roadmap".to_string(),
            AnalitikKodefikasi => "/perlengkapan/analitik/kodefikasi".to_string(),
            AnalisisDaftarLegacy => "/perlengkapan/analisis/daftar".to_string(),
            AnalisisBuatLegacy => "/perlengkapan/analisis/baru".to_string(),

            AdminUsers => "/perlengkapan/admin/users".to_string(),
            AdminRoles => "/perlengkapan/admin/roles".to_string(),
            AdminAudit => "/perlengkapan/admin/audit".to_string(),
            AdminMaster => "/perlengkapan/admin/master".to_string(),
            AdminWorkflow => "/perlengkapan/admin/workflow".to_string(),
            AdminWorkflowMonitoring => "/perlengkapan/admin/workflow-monitoring".to_string(),

            BantuanPanduan => "/perlengkapan/bantuan/panduan".to_string(),
            BantuanFaq => "/perlengkapan/bantuan/faq".to_string(),
            BantuanHelpdesk => "/perlengkapan/bantuan/helpdesk".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_resolves() {
        assert_eq!(PerlengkapanRoute::Dashboard.to_path(), "/perlengkapan/dashboard");
    }

    #[test]
    fn parameterized_kebutuhan_detail() {
        let url = PerlengkapanRoute::KebutuhanDetail {
            id: "kbn-42".into(),
        }
        .to_path();
        assert_eq!(url, "/perlengkapan/kebutuhan-bmn/detail/kbn-42");
    }

    #[test]
    fn dashboard_with_empty_query_drops_prefix() {
        assert_eq!(
            PerlengkapanRoute::DashboardWithQuery {
                query: String::new(),
            }
            .to_path(),
            "/perlengkapan/dashboard"
        );
    }

    #[test]
    fn dashboard_with_query_appends() {
        assert_eq!(
            PerlengkapanRoute::DashboardWithQuery {
                query: "satker=jakarta".into(),
            }
            .to_path(),
            "/perlengkapan/dashboard?satker=jakarta"
        );
    }
}
