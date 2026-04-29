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
use components::guards::{AdminLayout, AuthenticatedLayout};
use components::sidebar::Sidebar;
use features::auth::AuthService;
use lib_ui::components::app_shell::AppShell;
use pages::admin::{AdminAuditPage, AdminMasterDataPage};
use pages::bank_aset::{
    BankAsetDashboardPage, BankAsetDetailPage, BankAsetListPage, BankAsetQrCodePage,
    BankAsetSebaranPage,
};
use pages::dashboard::DashboardHome;
use pages::dashboard_perlengkapan::DashboardPerlengkapan;
use pages::kebutuhan_bmn::PeriodManagement;
use pages::login::LoginPage;
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
// Root App — routes grouped by auth level using ParentRoute layout guards
// (similar to Next.js layout.tsx / Laravel Route::middleware()->group())
// ═════════════════════════════════════════════════════════════════════════

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let sidebar_open = RwSignal::new(false);
    let (user_session, set_user_session) = signal(AuthService::load_session());

    // Global session context — any child can access via use_context()
    // (like Laravel Auth::user() or Next.js useSession())
    provide_context(user_session);
    provide_context(set_user_session);

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

        <AppShell>
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
                        ().into_any()
                    } else {
                        view! { <Sidebar sidebar_open=sidebar_open /> }.into_any()
                    }}

                        <main class=main_class>
                        <Routes fallback=move || view! { <NotFound /> }.into_any()>
                            // ══════════════════════════════════════════
                            // PUBLIC ROUTES (no auth required)
                            // ══════════════════════════════════════════
                            <Route path=path!("/login") view=move || {
                                if AuthService::load_session().is_some() {
                                    let nav = leptos_router::hooks::use_navigate();
                                    nav(routes::path::DASHBOARD, Default::default());
                                    view! { <div></div> }.into_any()
                                } else {
                                    view! { <LoginPage /> }.into_any()
                                }
                            } />
                            <Route path=path!("/") view=move || {
                                // SPA redirect based on authentication status.
                                let nav = leptos_router::hooks::use_navigate();
                                if AuthService::load_session().is_some() {
                                    nav(routes::path::DASHBOARD, Default::default());
                                } else {
                                    nav(routes::path::LOGIN, Default::default());
                                }
                                view! { <div></div> }
                            } />

                            // ══════════════════════════════════════════
                            // AUTHENTICATED ROUTES — AuthenticatedLayout
                            // guards ALL children automatically (like
                            // Next.js layout.tsx / Laravel middleware)
                            // ══════════════════════════════════════════
                            <ParentRoute path=path!("/") view=AuthenticatedLayout>
                                <Route path=path!("/dashboard") view=DashboardHome />
                                <Route path=path!("/dashboard/search") view=SearchPage />

                                // ── Bank Aset ────────────────────────
                                <Route path=path!("/bank-aset") view=BankAsetDashboardPage />
                                <Route path=path!("/bank-aset/dashboard") view=BankAsetDashboardPage />
                                <Route path=path!("/bank-aset/daftar") view=BankAsetListPage />
                                <Route path=path!("/bank-aset/daftar/:id") view=BankAsetDetailPage />
                                <Route path=path!("/bank-aset/sebaran") view=BankAsetSebaranPage />
                                <Route path=path!("/bank-aset/qrcode") view=BankAsetQrCodePage />

                                // ── Kebutuhan BMN ────────────────────
                                <Route path=path!("/kebutuhan-bmn/periode") view=PeriodManagement />
                                <Route path=path!("/kebutuhan-bmn/daftar") view=KebutuhanBmnList />
                                <Route path=path!("/kebutuhan-bmn") view=KebutuhanBmnList />
                                <Route path=path!("/kebutuhan-bmn/buat") view=KebutuhanBmnForm />
                                <Route path=path!("/kebutuhan-bmn/baru") view=KebutuhanBmnForm />
                                <Route path=path!("/kebutuhan-bmn/:id/edit") view=KebutuhanBmnForm />
                                <Route path=path!("/kebutuhan-bmn/detail/:id") view=KebutuhanBmnDetail />
                                <Route path=path!("/kebutuhan-bmn/:id") view=KebutuhanBmnDetail />
                                <Route path=path!("/kebutuhan-bmn/satker/:satker_id") view=KebutuhanBmnSatkerDetail />
                                <Route path=path!("/kebutuhan-bmn/laporan") view=LaporanKebutuhanBmn />

                                // ── Pakaian Dinas ────────────────────
                                <Route path=path!("/pakaian-dinas/jenis") view=PakaianDinasJenisList />
                                <Route path=path!("/pakaian-dinas/jenis/:id/spesifikasi") view=SpesifikasiPage />
                                <Route path=path!("/pakaian-dinas/pengajuan") view=PakaianDinasPengajuanList />
                                <Route path=path!("/pakaian-dinas/ukuran") view=move || view! { <UkuranPegawai pegawai_id="0".to_string() pegawai_nama="Pegawai".to_string() pegawai_nip="000".to_string() /> } />
                                <Route path=path!("/pakaian-dinas/laporan") view=PakaianDinasLaporan />
                                <Route path=path!("/pakaian-dinas/laporan/rekap") view=PakaianDinasLaporan />

                                // ── Pengelolaan BMN ──────────────────
                                <Route path=path!("/pengelolaan/pemakaian") view=PemakaianBmnListPage />
                                <Route path=path!("/pemakaian-bmn") view=PemakaianBmnListPage />
                                <Route path=path!("/pengelolaan/pemakaian/buat") view=PemakaianBmnForm />
                                <Route path=path!("/pemakaian-bmn/baru") view=PemakaianBmnForm />
                                <Route path=path!("/pengelolaan/pemakaian/detail/:id") view=PemakaianBmnDetailPage />
                                <Route path=path!("/pemakaian-bmn/:id") view=PemakaianBmnDetailPage />
                                <Route path=path!("/pemakaian-bmn/:id/renew") view=PemakaianBmnRenew />
                                <Route path=path!("/pengelolaan/pemakaian/monitoring") view=PemakaianBmnMonitoring />
                                <Route path=path!("/pengelolaan/penghapusan") view=PenghapusanBmnListPage />
                                <Route path=path!("/pengelolaan/penghapusan/daftar") view=PenghapusanBmnListPage />
                                <Route path=path!("/pengelolaan/penghapusan/buat") view=PenghapusanForm />
                                <Route path=path!("/pengelolaan/penghapusan/baru") view=PenghapusanForm />
                                <Route path=path!("/pengelolaan/penghapusan/detail/:id") view=PenghapusanBmnDetailPage />
                                <Route path=path!("/pengelolaan/penghapusan/:id") view=PenghapusanBmnDetailPage />

                                // ── Analitik ─────────────────────────
                                <Route path=path!("/analitik/roadmap") view=AnalisisList />
                                <Route path=path!("/analisis/daftar") view=AnalisisList />
                                <Route path=path!("/analitik/roadmap/buat") view=AnalisisForm />
                                <Route path=path!("/analisis/baru") view=AnalisisForm />
                                <Route path=path!("/analitik/kodefikasi") view=MappingKodefikasiDashboard />

                                // ── Bantuan ──────────────────────────
                                <Route path=path!("/bantuan/panduan") view=PanduanPengguna />
                                <Route path=path!("/bantuan/faq") view=FaqPage />
                                <Route path=path!("/bantuan/helpdesk") view=HelpdeskPage />
                            </ParentRoute>

                            // ══════════════════════════════════════════
                            // ADMIN ROUTES — AdminLayout guards all
                            // children (admin role required)
                            // ══════════════════════════════════════════
                            <ParentRoute path=path!("/admin") view=AdminLayout>
                                <Route path=path!("/users") view=AdminUsersPage />
                                <Route path=path!("/roles") view=AdminRolesPage />
                                <Route path=path!("/audit") view=AdminAuditPage />
                                <Route path=path!("/master") view=AdminMasterDataPage />
                                <Route path=path!("/workflow") view=WorkflowConfigManagement />
                                <Route path=path!("/workflow-monitoring") view=WorkflowMonitoring />
                            </ParentRoute>
                        </Routes>
                    </main>
                </div>

                {move || (!is_login_page()).then(|| view! { <AppFooter /> })}
            </div>
        </Router>
        </AppShell>
    }
}
