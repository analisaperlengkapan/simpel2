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

use components::profile_menu::ProfileMenu;
use components::sidebar::Sidebar;
use components::guards::{SessionAdminGuard, SessionAuthGuard};
use features::auth::AuthService;
use pages::dashboard::DashboardHome;
use pages::dashboard_perlengkapan::DashboardPerlengkapan;
use pages::login::LoginPage;
use pages::kebutuhan_bmn::PeriodManagement;
use pages::not_found::NotFound;
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
use components::pemakaian_bmn_detail::PemakaianBmnDetail;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_list::PemakaianBmnList;
use components::pemakaian_bmn_monitoring::PemakaianBmnMonitoring;
use components::pemakaian_bmn_renew::PemakaianBmnRenew;
use components::penghapusan_bmn_detail::PenghapusanBmnDetail;
use components::penghapusan_form::PenghapusanForm;
use components::penghapusan_list::PenghapusanList;
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

    let main_style = move || {
        if is_login_page() {
            "flex: 1; overflow-y: auto; padding: 0; margin-left: 0;"
        } else {
            "flex: 1; overflow-y: auto; padding: 1.5rem; margin-left: 0;"
        }
    };

    let main_class = move || {
        if is_login_page() {
            ""
        } else {
            "lg:ml-[250px]"
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
            // ══ Full-page dark shell ═════════════════════════════════
            <div style="min-height: 100vh; display: flex; flex-direction: column; background: linear-gradient(180deg, #0f172a 0%, #111c36 50%, #0a1020 100%); font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;">
                // Keep this static class token so Tailwind/JIT always emits the desktop offset utility.
                <div class="hidden lg:ml-[250px]" style="display: none;"></div>

                // ── Header ───────────────────────────────────────────
                {move || if is_login_page() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <header style="background: #0c1425; border-bottom: 1px solid rgba(255,255,255,0.06); position: sticky; top: 0; z-index: 50;">
                            <div style="display: flex; align-items: center; justify-content: space-between; padding: 0 1.25rem; height: 56px;">
                                // Left: mobile toggle + brand
                                <div style="display: flex; align-items: center; gap: 12px;">
                                    <button
                                        on:click=move |_| sidebar_open.update(|o| *o = !*o)
                                        style="background: none; border: none; color: #94a3b8; font-size: 1.1rem; cursor: pointer; padding: 4px;"
                                        class="lg:hidden"
                                    >
                                        <i class="fas fa-bars"></i>
                                    </button>
                                    <A href=routes::path::DASHBOARD attr:style="display: flex; align-items: center; gap: 10px; text-decoration: none;">
                                        <img
                                            src="/perlengkapan/assets/kejaksaan-logo.png"
                                            alt="Kejaksaan RI"
                                            style="width: 30px; height: 30px; object-fit: contain;"
                                        />
                                        <span style="font-size: 1.1rem; font-weight: 800; color: #ffffff; letter-spacing: 0.02em;">
                                            "SIMPEL"
                                        </span>
                                    </A>
                                </div>
                                // Right: profile avatar
                                <ProfileMenu />
                            </div>
                        </header>
                    }.into_any()
                }}

                // ── Body (sidebar + main) ────────────────────────────
                <div style="display: flex; flex: 1; min-height: 0;">
                    {move || if is_login_page() {
                        view! { <></> }.into_any()
                    } else {
                        view! { <Sidebar sidebar_open=sidebar_open /> }.into_any()
                    }}

                        <main style=main_style class=main_class>
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
                            <Route path=path!("/bank-aset/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><AsetList /></SessionAuthGuard> } />
                            <Route path=path!("/bank-aset/qrcode") view=move || view! { <SessionAuthGuard user_session=user_session><QrCodeGenerator /></SessionAuthGuard> } />

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
                                    <PlaceholderPage title="Spesifikasi Pakaian Dinas" icon="fas fa-list" description="Halaman spesifikasi per jenis sedang dalam pengembangan." />
                                </SessionAuthGuard>
                            } />
                            <Route path=path!("/pakaian-dinas/pengajuan") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasPengajuanList /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/ukuran") view=move || view! { <SessionAuthGuard user_session=user_session><UkuranPegawai pegawai_id="0".to_string() pegawai_nama="Pegawai".to_string() pegawai_nip="000".to_string() /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/laporan") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasLaporan /></SessionAuthGuard> } />
                            <Route path=path!("/pakaian-dinas/laporan/rekap") view=move || view! { <SessionAuthGuard user_session=user_session><PakaianDinasLaporan /></SessionAuthGuard> } />

                            // ── Pengelolaan BMN ──────────────────────
                            <Route path=path!("/pengelolaan/pemakaian") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnList /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnList /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/buat") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/baru") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/detail/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnDetail /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnDetail /></SessionAuthGuard> } />
                            <Route path=path!("/pemakaian-bmn/:id/renew") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnRenew /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/pemakaian/monitoring") view=move || view! { <SessionAuthGuard user_session=user_session><PemakaianBmnMonitoring /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanList /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/daftar") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanList /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/buat") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/baru") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanForm /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/detail/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnDetail /></SessionAuthGuard> } />
                            <Route path=path!("/pengelolaan/penghapusan/:id") view=move || view! { <SessionAuthGuard user_session=user_session><PenghapusanBmnDetail /></SessionAuthGuard> } />

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

                // ── Footer ───────────────────────────────────────────
                {move || if is_login_page() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <footer style="background: rgba(10,16,32,0.9); border-top: 1px solid rgba(255,255,255,0.05); padding: 10px 1.25rem;" class="lg:ml-[250px]">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <span style="font-size: 0.7rem; color: #475569;">
                                    "SIMPEL v" {APP_VERSION} " · Kejaksaan Agung RI"
                                </span>
                                <span style="font-size: 0.65rem; color: #334155;">
                                    "© 2025 Biro Perlengkapan"
                                </span>
                            </div>
                        </footer>
                    }.into_any()
                }}
            </div>
        </Router>
    }
}
