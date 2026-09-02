//! Perlengkapan Microfrontend — main app shell.
//!
//! Dark navy + gold Kejaksaan theme, single-page application.

#![recursion_limit = "512"]
// Kept deliberately: both lints' suggested rewrites shorten the drop scope of
// temporaries borrowed inside Leptos reactive closures, which fails the borrow
// checker here (E0597) — collapsing nested `if`s into edition-2024 let-chains,
// and replacing `move || f(x)` capturing closures with bare `f`. `cargo clippy
// --fix` produces non-compiling code for both, so they stay crate-wide.
#![allow(clippy::collapsible_if)]
#![allow(clippy::redundant_closure)]
mod api;
mod assets;
mod components;
mod features;
mod navigation;
mod pages;
mod routes;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use components::app_chrome::{AppFooter, AppHeader, IntegrasiHealthBanner};
use components::guards::{AdminLayout, AuthenticatedLayout};
use components::sidebar::Sidebar;
use features::auth::AuthService;
use lib_ui::components::app_shell::AppShell;
use pages::admin::{AdminAuditPage, AdminMasterDataPage, AdminTemplatesPage};
use pages::bank_aset::{
    BankAsetDashboardPage, BankAsetDetailPage, BankAsetListPage, BankAsetQrCodePage,
    BankAsetSebaranPage,
};
use pages::dashboard::DashboardHome;
use pages::kebutuhan_bmn::PeriodManagement;
use pages::login::LoginPage;
use pages::not_found::NotFound;
use pages::notifikasi::NotifikasiInboxPage;
use pages::pakaian_dinas::SpesifikasiPage;
use pages::pemakaian_bmn::{PemakaianBmnDetailPage, PemakaianBmnListPage};
use pages::penghapusan_bmn::{PenghapusanBmnDetailPage, PenghapusanBmnListPage};
use pages::workflow::config_management::WorkflowConfigManagement;
use pages::workflow::delegation::WorkflowDelegationPage;
use pages::workflow::monitoring::WorkflowMonitoring;

// Migrated business components
use components::admin_roles::AdminRolesPage;
use components::analisis_form::AnalisisForm;
use components::analisis_list::AnalisisList;
use components::faq::FaqPage;
use components::helpdesk::HelpdeskPage;
use components::kebutuhan_bmn_detail::KebutuhanBmnDetail;
use components::kebutuhan_bmn_form::KebutuhanBmnForm;
use components::kebutuhan_bmn_list::KebutuhanBmnList;
use components::kebutuhan_bmn_satker::KebutuhanBmnSatkerDetail;
use components::laporan_kebutuhan_bmn::LaporanKebutuhanBmn;
use components::pakaian_dinas_jenis_list::PakaianDinasJenisList;
use components::pakaian_dinas_laporan::PakaianDinasLaporan;
use components::pakaian_dinas_pengajuan_list::PakaianDinasPengajuanList;
use components::pakaian_dinas_pengisian::PakaianDinasPengisian;
use components::pakaian_dinas_satker::PakaianDinasSatkerDetail;
use components::pakaian_dinas_ukuran::UkuranPegawai;
use components::panduan::PanduanPengguna;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_monitoring::PemakaianBmnMonitoring;
use components::penghapusan_form::PenghapusanForm;

// ── Version ──────────────────────────────────────────────────────────────
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Route view for `/pakaian-dinas/ukuran` — the "my own size" page.
///
/// The previous declaration hard-coded `pegawai_id="0"`, which made the
/// page render someone else's identity and broke the per-user cache.
/// Here we read the live `UserSession` from context (the same value the
/// guards check) and project NIP/name straight from the JWT claims.
#[component]
fn UkuranPegawaiCurrentUser() -> impl IntoView {
    let session = expect_context::<ReadSignal<Option<features::auth::UserSession>>>();
    move || match session.get() {
        Some(s) => {
            let nip = s.nip.clone().unwrap_or_default();
            // Use the authenc user_id as the QueryClient cache key — it is
            // unique per identity and never empty, so we never collide
            // across two pegawai sharing a workstation.
            let cache_key = s.user_id.clone();
            view! { <UkuranPegawai pegawai_id=cache_key pegawai_nama=s.name.clone() pegawai_nip=nip /> }
            .into_any()
        }
        None => view! {
            <div class="p-6 text-center text-slate-300">
                "Sesi tidak tersedia. Silakan login kembali."
            </div>
        }
        .into_any(),
    }
}

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

    // Reactive UI preview role. Must be provided before any role-aware view
    // mounts: readers call `use_active_role()` (expect_context) so that
    // switching role re-renders them instead of needing a browser refresh.
    crate::components::role_switcher::provide_active_role();

    // is_login_page + main_class dipindah ke <AppRouterShell> component
    // di bawah supaya bisa pakai use_location() (HOOK) yang reactive ke
    // history.pushState navigation. Sebelumnya pakai
    // `web_sys::window().location()` langsung — DOM access non-reactive,
    // closure tidak re-evaluate saat router nav → sidebar tetap muncul
    // di /login.

    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            AuthService::setup_storage_listener(move |session| {
                set_user_session.set(session);
            });
        });

        // Self-refresh: don't rely on Portal being open. See
        // `features::session_monitor` for the rationale.
        Effect::new(move |_| {
            features::session_monitor::spawn_refresh_loop(set_user_session);
        });
    }

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL — Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <AppShell>
            // Router base harus match canonical mount point `/perlengkapan/simpel/v2/`
            // sesuai routing spec — supaya path `/perlengkapan/simpel/v2/` ke router
            // dianggap path `/` (root), trigger redirect ke `/login` jika belum
            // authenticated. Sebelumnya base="/perlengkapan" tidak match path baru
            // sehingga semua route fall through ke NotFound.
            <Router base="/perlengkapan/simpel/v2">
                {
                    let location = leptos_router::hooks::use_location();
                    let is_login_page = move || {
                        let path = location.pathname.get();
                        let normalized = path.trim_end_matches('/');
                        normalized == "/perlengkapan/simpel/v2/login"
                            || normalized == "/perlengkapan/login" || normalized == "/login"
                            || normalized.ends_with("/login")
                    };
                    let main_class = move || {
                        if is_login_page() {
                            "flex-1 overflow-y-auto p-0"
                        } else {
                            "flex-1 overflow-y-auto p-6 lg:ml-[250px]"
                        }
                    };
                    // use_location() HARUS dipanggil di dalam Router scope.
                    // location.pathname adalah reactive signal → closures
                    // di bawahnya re-evaluate setiap kali router nav.

                    view! {
                        <div class="flex min-h-screen flex-col bg-app-gradient font-sans text-slate-100">
                            // Keep this static class token so Tailwind/JIT always emits the desktop offset utility.
                            <div class="hidden lg:ml-[250px]"></div>

                            {move || {
                                (!is_login_page())
                                    .then(|| {
                                        let toggle = Callback::new(move |_: ()| {
                                            sidebar_open.update(|o| *o = !*o)
                                        });
                                        view! {
                                            <AppHeader on_toggle_sidebar=toggle />
                                            <IntegrasiHealthBanner />
                                        }
                                    })
                            }}

                            <div class="flex min-h-0 flex-1">
                                {move || {
                                    if is_login_page() {
                                        ().into_any()
                                    } else {
                                        view! { <Sidebar sidebar_open=sidebar_open /> }.into_any()
                                    }
                                }} <main class=main_class>
                                    <Routes fallback=move || view! { <NotFound /> }.into_any()>
                                        // ══════════════════════════════════════════
                                        // PUBLIC ROUTES (no auth required)
                                        // ══════════════════════════════════════════
                                        <Route
                                            path=path!("/login")
                                            view=move || {
                                                if AuthService::load_session().is_some() {
                                                    let nav = leptos_router::hooks::use_navigate();
                                                    nav("/dashboard", Default::default());
                                                    // Leptos Router base "/perlengkapan/simpel/v2" otomatis
                                                    // prepend ke nav() path — pakai relative path /dashboard
                                                    // (tanpa prefix) supaya tidak jadi
                                                    // "/perlengkapan/simpel/v2/perlengkapan/simpel/v2/dashboard".
                                                    view! { <div></div> }
                                                        .into_any()
                                                } else {
                                                    view! { <LoginPage /> }.into_any()
                                                }
                                            }
                                        />
                                        <Route
                                            path=path!("/")
                                            view=move || {
                                                let nav = leptos_router::hooks::use_navigate();
                                                if AuthService::load_session().is_some() {
                                                    nav("/dashboard", Default::default());
                                                } else {
                                                    nav("/login", Default::default());
                                                }
                                                // SPA redirect based on authentication status.
                                                // Lihat catatan di atas: nav() relative ke router base.
                                                view! { <div></div> }
                                            }
                                        />

                                        // ══════════════════════════════════════════
                                        // AUTHENTICATED ROUTES — AuthenticatedLayout
                                        // guards ALL children automatically (like
                                        // Next.js layout.tsx / Laravel middleware)
                                        // ══════════════════════════════════════════
                                        <ParentRoute path=path!("/") view=AuthenticatedLayout>
                                            <Route path=path!("/dashboard") view=DashboardHome />

                                            // ── Bank Aset ────────────────────────
                                            <Route
                                                path=path!("/bank-aset/dashboard")
                                                view=BankAsetDashboardPage
                                            />
                                            <Route
                                                path=path!("/bank-aset/daftar")
                                                view=BankAsetListPage
                                            />
                                            <Route
                                                path=path!("/bank-aset/daftar/:id")
                                                view=BankAsetDetailPage
                                            />
                                            <Route
                                                path=path!("/bank-aset/sebaran")
                                                view=BankAsetSebaranPage
                                            />
                                            <Route
                                                path=path!("/bank-aset/qrcode")
                                                view=BankAsetQrCodePage
                                            />

                                            // ── Kebutuhan BMN ────────────────────
                                            <Route
                                                path=path!("/kebutuhan-bmn/periode")
                                                view=PeriodManagement
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/daftar")
                                                view=KebutuhanBmnList
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/buat")
                                                view=KebutuhanBmnForm
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/:id/edit")
                                                view=KebutuhanBmnForm
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/detail/:id")
                                                view=KebutuhanBmnDetail
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/satker/:satker_id")
                                                view=KebutuhanBmnSatkerDetail
                                            />
                                            <Route
                                                path=path!("/kebutuhan-bmn/laporan")
                                                view=LaporanKebutuhanBmn
                                            />

                                            // ── Pakaian Dinas ────────────────────
                                            <Route
                                                path=path!("/pakaian-dinas/jenis")
                                                view=PakaianDinasJenisList
                                            />
                                            <Route
                                                path=path!("/pakaian-dinas/jenis/:id/spesifikasi")
                                                view=SpesifikasiPage
                                            />
                                            <Route
                                                path=path!("/pakaian-dinas/pengajuan")
                                                view=PakaianDinasPengajuanList
                                            />
                                            <Route
                                                path=path!("/pakaian-dinas/pengajuan/:pengajuan_id/satker")
                                                view=PakaianDinasSatkerDetail
                                            />
                                            <Route
                                                path=path!(
                                                    "/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi"
                                                )
                                                view=PakaianDinasPengisian
                                            />
                                            <Route
                                                path=path!("/pakaian-dinas/ukuran")
                                                view=UkuranPegawaiCurrentUser
                                            />
                                            <Route
                                                path=path!("/pakaian-dinas/laporan")
                                                view=PakaianDinasLaporan
                                            />

                                            // ── Pengelolaan BMN ──────────────────
                                            <Route
                                                path=path!("/pengelolaan/pemakaian")
                                                view=PemakaianBmnListPage
                                            />
                                            <Route
                                                path=path!("/pengelolaan/pemakaian/buat")
                                                view=PemakaianBmnForm
                                            />
                                            <Route
                                                path=path!("/pengelolaan/pemakaian/detail/:id")
                                                view=PemakaianBmnDetailPage
                                            />
                                            <Route
                                                path=path!("/pengelolaan/pemakaian/monitoring")
                                                view=PemakaianBmnMonitoring
                                            />
                                            <Route
                                                path=path!("/pengelolaan/penghapusan")
                                                view=PenghapusanBmnListPage
                                            />
                                            <Route
                                                path=path!("/pengelolaan/penghapusan/buat")
                                                view=PenghapusanForm
                                            />
                                            <Route
                                                path=path!("/pengelolaan/penghapusan/detail/:id")
                                                view=PenghapusanBmnDetailPage
                                            />

                                            // ── Analitik ─────────────────────────
                                            <Route path=path!("/analitik/roadmap") view=AnalisisList />
                                            <Route
                                                path=path!("/analitik/roadmap/buat")
                                                view=AnalisisForm
                                            />

                                            // ── Notifikasi ───────────────────────
                                            <Route path=path!("/notifikasi") view=NotifikasiInboxPage />

                                            // ── Bantuan ──────────────────────────
                                            <Route
                                                path=path!("/bantuan/panduan")
                                                view=PanduanPengguna
                                            />
                                            <Route path=path!("/bantuan/faq") view=FaqPage />
                                            <Route path=path!("/bantuan/helpdesk") view=HelpdeskPage />
                                        </ParentRoute>

                                        // ══════════════════════════════════════════
                                        // ADMIN ROUTES — AdminLayout guards all
                                        // children (admin role required)
                                        // ══════════════════════════════════════════
                                        <ParentRoute path=path!("/admin") view=AdminLayout>
                                            // No `/admin/users`: it was backed by
                                            // relations no migration creates, and
                                            // user/role administration belongs to
                                            // authenc (portal already does it).
                                            <Route path=path!("/roles") view=AdminRolesPage />
                                            <Route path=path!("/audit") view=AdminAuditPage />
                                            <Route path=path!("/master") view=AdminMasterDataPage />
                                            <Route path=path!("/templates") view=AdminTemplatesPage />
                                            <Route
                                                path=path!("/workflow")
                                                view=WorkflowConfigManagement
                                            />
                                            <Route
                                                path=path!("/workflow-monitoring")
                                                view=WorkflowMonitoring
                                            />
                                            <Route
                                                path=path!("/workflow-delegation")
                                                view=WorkflowDelegationPage
                                            />
                                        </ParentRoute>
                                    </Routes>
                                </main>
                            </div>

                            {move || (!is_login_page()).then(|| view! { <AppFooter /> })}
                        </div>
                    }
                }
            </Router>
        </AppShell>
    }
}
