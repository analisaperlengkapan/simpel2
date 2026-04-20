//! Perlengkapan Microfrontend — main app shell.
//!
//! Dark navy + gold Kejaksaan theme, single-page application.

#![recursion_limit = "512"]
// Many API types and functions are defined for future use but not yet
// wired into components.  Suppress dead-code and related errors
// crate-wide until the remaining pages are connected.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_ok_err)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::collapsible_if)]
mod api;
mod components;
mod features;
mod navigation;
mod pages;
mod routes;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use components::app_chrome::{AppFooter, AppHeader};
use components::sidebar::Sidebar;
use components::guards::{SessionAdminGuard, SessionAuthGuard};
use features::auth::AuthService;
use pages::bank_aset::{
    BankAsetDashboardPage, BankAsetDetailPage, BankAsetListPage, BankAsetQrCodePage,
    BankAsetSebaranPage,
};
use pages::dashboard::DashboardHome;
use pages::dashboard_perlengkapan::DashboardPerlengkapan;
use pages::login::LoginPage;
use pages::kebutuhan_bmn::PeriodManagement;
use pages::not_found::NotFound;
use pages::pakaian_dinas::SpesifikasiPage;
use pages::pemakaian_bmn::{PemakaianBmnDetailPage, PemakaianBmnListPage};
use pages::penghapusan_bmn::{PenghapusanBmnDetailPage, PenghapusanBmnListPage};
use pages::placeholder::PlaceholderPage;
use pages::search_page::SearchPage;
use pages::workflow::config_management::WorkflowConfigManagement;
use pages::workflow::monitoring::WorkflowMonitoring;

// Migrated business components
use components::admin_users::{AdminRolesPage, AdminUsersPage};
use components::analisis_form::AnalisisForm;
use components::analisis_list::AnalisisList;
use components::aset_list::AsetList;
use components::faq::FaqPage;
use components::helpdesk::HelpdeskPage;
use components::kebutuhan_bmn_detail::KebutuhanBmnDetail;
use components::kebutuhan_bmn_form::KebutuhanBmnForm;
use components::kebutuhan_bmn_list::KebutuhanBmnList;
use components::kebutuhan_bmn_satker::KebutuhanBmnSatkerDetail;
use components::laporan_kebutuhan_bmn::LaporanKebutuhanBmn;
use components::mapping_kodefikasi_dashboard::MappingKodefikasiDashboard;
use components::pakaian_dinas_jenis_list::PakaianDinasJenisList;
use components::pakaian_dinas_laporan::PakaianDinasLaporan;
use components::pakaian_dinas_pengajuan_list::PakaianDinasPengajuanList;
use components::pakaian_dinas_ukuran::UkuranPegawai;
use components::panduan::PanduanPengguna;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_monitoring::PemakaianBmnMonitoring;
use components::pemakaian_bmn_renew::PemakaianBmnRenew;
use components::penghapusan_form::PenghapusanForm;
use components::qrcode_generator::QrCodeGenerator;

// ── Version ──────────────────────────────────────────────────────────────
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ═════════════════════════════════════════════════════════════════════════
// Root App — all routes are FLAT inside the layout shell
// ═════════════════════════════════════════════════════════════════════════

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let sidebar_open = RwSignal::new(false);
    let (user_session, set_user_session) = signal(AuthService::load_session());
    let is_login_page = move || {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .map(|path| {
                let normalized = path.trim_end_matches('/');
                normalized == "/perlengkapan/login" || normalized == "/login"
            })
            .unwrap_or(false)
    };

    let main_class = move || {
        if is_login_page() {
            "flex-1 overflow-y-auto p-0"
        } else {
            "flex-1 overflow-y-auto p-6 lg:ml-[250px]"
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            AuthService::setup_storage_listener(move |session| {
                set_user_session.set(session);
            });
        });
    }

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL — Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <Router base="/perlengkapan">
            <div class="flex min-h-screen flex-col bg-app-gradient font-sans text-slate-100">
                // Keep this static class token so Tailwind/JIT always emits the desktop offset utility.
                <div class="hidden lg:ml-[250px]"></div>

                {move || (!is_login_page()).then(|| {
                    let toggle = Callback::new(move |_: ()| sidebar_open.update(|o| *o = !*o));
                    view! { <AppHeader on_toggle_sidebar=toggle /> }
                })}

                <div class="flex min-h-0 flex-1">
                    {move || if is_login_page() {
                        view! { <></> }.into_any()
                    } else {
                        view! { <Sidebar sidebar_open=sidebar_open /> }.into_any()
                    }}

                        <main class=main_class>
                        <Routes fallback=move || view! { <NotFound /> }.into_any()>
                            // Dashboard home (default)
                            <Route path=path!("/login") view=move || {
                                if AuthService::load_session().is_some() {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href(routes::path::DASHBOARD);
                                    }
                                    view! { <div></div> }.into_any()
                                } else {
                                    view! { <LoginPage /> }.into_any()
                                }
                            } />
                            <Route path=path!("/") view=move || {
                                // Redirect root based on authentication status.
                                if AuthService::load_session().is_some() {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href(routes::path::DASHBOARD);
                                    }
                                } else if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href(routes::path::LOGIN);
                                }
                                view! { <div></div> }
                            } />
                            <Route path=path!("/dashboard") view=move || view! { <SessionAuthGuard user_session=user_session><DashboardHome /></SessionAuthGuard> } />
                            <Route path=path!("/dashboard/search") view=move || view! { <SessionAuthGuard user_session=user_session><SearchPage /></SessionAuthGuard> } />

                            // ── Bank Aset ────────────────────────────
                            <Route path=path!("/bank-aset") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetDashboardPage /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/dashboard") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetDashboardPage /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetListPage /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/daftar/:id") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetDetailPage /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/sebaran") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetSebaranPage /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/qrcode") view=move || view! { <SessionAuthGuard user_session=user_session><BankAsetQrCodePage /></SessionAuthGuard> } />

                            // ── Kebutuhan BMN ────────────────────────
                            <Route path=path!("/kebutuhan-bmn/periode") view=move || view! { <SessionAuthGuard user_session=user_session><PeriodManagement /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnList /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnList /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/buat") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/baru") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/:id/edit") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/detail/:id") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnDetail /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/:id") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnDetail /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/satker/:satker_id") view=move || view! { <SessionAuthGuard user_session=user_session><KebutuhanBmnSatkerDetail /></SessionAuthGuard> } />
                            <Route path=path!("/kebutuhan-bmn/laporan") view=move || view! { <SessionAuthGuard user_session=user_session><LaporanKebutuhanBmn /></SessionAuthGuard> } />

                            // ── Pakaian Dinas ────────────────────────
                            <Route path=path!("/pakaian-dinas/jenis") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasJenisList /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/jenis/:id/spesifikasi") view=move || view! {
                                <SessionAuthGuard user_session=user_session>
                                    <SpesifikasiPage />
                                </SessionAuthGuard>
                            } />
                            <Route path=path!("/pakaian-dinas/pengajuan") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasPengajuanList /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/ukuran") view=move || view! { <SessionAuthGuard user_session=user_session><UkuranPegawai pegawai_id="0".to_string() pegawai_nama="Pegawai".to_string() pegawai_nip="000".to_string() /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/laporan") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasLaporan /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/laporan/rekap") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasLaporan /></SessionAuthGuard> } />

                            // ── Pengelolaan BMN ──────────────────────
                            <Route path=path!("/pengelolaan/pemakaian") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnListPage /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnListPage /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/buat") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/baru") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/detail/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnDetailPage /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnDetailPage /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/:id/renew") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnRenew /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/monitoring") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnMonitoring /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnListPage /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnListPage /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/buat") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/baru") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/detail/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnDetailPage /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnDetailPage /></SessionAuthGuard> } />

                            // ── Analitik ─────────────────────────────
                            <Route path=path!("/analitik/roadmap") view=move || view! { <SessionAuthGuard user_session=user_session><AnalisisList /></SessionAuthGuard> } />
                            <Route path=path!("/analisis/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><AnalisisList /></SessionAuthGuard> } />
                            <Route path=path!("/analitik/roadmap/buat") view=move || view! { <SessionAuthGuard user_session=user_session><AnalisisForm /></SessionAuthGuard> } />
                            <Route path=path!("/analisis/baru") view=move || view! { <SessionAuthGuard user_session=user_session><AnalisisForm /></SessionAuthGuard> } />
                            <Route path=path!("/analitik/kodefikasi") view=move || view! { <SessionAuthGuard user_session=user_session><MappingKodefikasiDashboard /></SessionAuthGuard> } />

                            // ── Admin ────────────────────────────────
                            <Route path=path!("/admin/users") view=move || view! { <SessionAdminGuard user_session=user_session><AdminUsersPage /></SessionAdminGuard> } />
                            <Route path=path!("/admin/roles") view=move || view! { <SessionAdminGuard user_session=user_session><AdminRolesPage /></SessionAdminGuard> } />
                            <Route path=path!("/admin/audit") view=move || view! {
                                <SessionAdminGuard user_session=user_session>
                                    <PlaceholderPage title="Audit Log" icon="fas fa-history" description="Jejak audit seluruh aktivitas." />
                                </SessionAdminGuard>
                            } />
                            <Route path=path!("/admin/master") view=move || view! {
                                <SessionAdminGuard user_session=user_session>
                                    <PlaceholderPage title="Master Data" icon="fas fa-database" description="Pengelolaan data referensi." />
                                </SessionAdminGuard>
                            } />
                            <Route path=path!("/admin/workflow") view=move || view! { <SessionAdminGuard user_session=user_session><WorkflowConfigManagement /></SessionAdminGuard> } />
                            <Route path=path!("/admin/workflow-monitoring") view=move || view! { <SessionAdminGuard user_session=user_session><WorkflowMonitoring /></SessionAdminGuard> } />

                            // ── Bantuan ──────────────────────────────
                            <Route path=path!("/bantuan/panduan") view=move || view! { <SessionAuthGuard user_session=user_session><PanduanPengguna /></SessionAuthGuard> } />
                            <Route path=path!("/bantuan/faq") view=move || view! { <SessionAuthGuard user_session=user_session><FaqPage /></SessionAuthGuard> } />
                            <Route path=path!("/bantuan/helpdesk") view=move || view! { <SessionAuthGuard user_session=user_session><HelpdeskPage /></SessionAuthGuard> } />
                        </Routes>
                    </main>
                </div>

                {move || (!is_login_page()).then(|| view! { <AppFooter /> })}
            </div>
        </Router>
    }
}
