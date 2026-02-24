#![recursion_limit = "512"]
//! SIMPEL Perlengkapan — Microfrontend entry point.
//!
//! Arsitektur:
//! - Framework : Leptos 0.8.x CSR/SPA WASM
//! - Router    : `base="/perlengkapan"`, inner routes di `/dashboard/*`
//! - Auth      : SSO via portal, token di localStorage
//! - UI        : Tailwind CSS + lib-ui components

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::*;
use lib_ui::components::auth::{LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile};
use lib_ui::prelude::*;
use wasm_bindgen::prelude::*;

// ── Crate modules ─────────────────────────────────────────────────────────
mod api;
mod components;
mod pages;
#[cfg(test)]
mod tests;
mod utils;

// ── Page imports ──────────────────────────────────────────────────────────
use pages::dashboard_home::DashboardHome;
use pages::not_found::NotFound;
use pages::under_construction::UnderConstruction;

// ── Component imports ─────────────────────────────────────────────────────
use components::admin_panel::{AdminRolesPage, AdminUsersPage};
use components::analisis_form::AnalisisForm;
use components::analisis_list::AnalisisList;
use components::aset_list::AsetList;
use components::hibah_form::HibahForm;
use components::hibah_list::HibahList;
use components::kebutuhan_bmn_dashboard::KebutuhanBmnDashboard;
use components::kebutuhan_bmn_detail::KebutuhanBmnDetail;
use components::kebutuhan_bmn_form::KebutuhanBmnForm;
use components::kebutuhan_bmn_list::KebutuhanBmnList;
use components::kebutuhan_bmn_satker::KebutuhanBmnSatkerDetail;
use components::mutasi_form::MutasiForm;
use components::mutasi_list::MutasiList;
use components::pakaian_dinas_jenis_list::PakaianDinasJenisList;
use components::pakaian_dinas_laporan::PakaianDinasLaporan;
use components::pakaian_dinas_pengajuan_list::PakaianDinasPengajuanList;
use components::pakaian_dinas_ukuran::UkuranPegawaiSatker;
use components::pemakaian_bmn_detail::PemakaianBmnDetail;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_list::PemakaianBmnList;
use components::pemakaian_form::PemakaianForm;
use components::pemakaian_list::PemakaianList;
use components::pemeliharaan_form::PemeliharaanForm;
use components::pemeliharaan_list::PemeliharaanList;
use components::pengalihan_form::PengalihanForm;
use components::pengalihan_list::PengalihanList;
use components::penghapusan_bmn_detail::PenghapusanBmnDetail;
use components::penghapusan_form::PenghapusanForm;
use components::penghapusan_list::PenghapusanList;
use components::role_switcher::RoleSwitcher;
use components::sidebar::Sidebar;

// ── Application version ───────────────────────────────────────────────────
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ═══════════════════════════════════════════════════════════════════════════
// Root component
// ═══════════════════════════════════════════════════════════════════════════

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL Perlengkapan — Kejaksaan RI" />
        <Meta name="description" content="Sistem Informasi Manajemen Perlengkapan Kejaksaan Republik Indonesia" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <Router base="/perlengkapan">
            <Routes fallback=|| view! { <NotFound /> }>
                <Route
                    path=path!("/")
                    view=move || view! {
                        <LoginRedirectPage
                            app_name="SIMPEL Perlengkapan"
                            app_description="Sistem Informasi Manajemen Perlengkapan"
                        />
                    }
                />
                <Route path=path!("/dashboard/*") view=DashboardLayout />
            </Routes>
        </Router>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Shell layout — header + sidebar + main + footer
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DashboardLayout() -> impl IntoView {
    let sidebar_open = RwSignal::new(false);

    let content = move || {
        view! {
            <div class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 flex flex-col">

                // ── Top header ────────────────────────────────────────────────
                <header class="bg-gradient-to-r from-blue-800 via-blue-700 to-indigo-800 text-white shadow-lg sticky top-0 z-50">
                    <div class="px-4 lg:px-6">
                        <div class="flex items-center justify-between h-14">
                            <div class="flex items-center gap-3">
                                <button
                                    class="lg:hidden p-2 rounded-lg text-white/80 hover:text-white hover:bg-white/10 transition-all"
                                    on:click=move |_| sidebar_open.update(|o| *o = !*o)
                                    aria-label="Toggle sidebar"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                              d="M4 6h16M4 12h16M4 18h16"/>
                                    </svg>
                                </button>
                                <a href="/perlengkapan/dashboard"
                                   class="flex items-center gap-3 hover:opacity-90 transition-opacity">
                                    <div class="w-9 h-9 bg-white/20 backdrop-blur-sm rounded-lg flex items-center justify-center">
                                        <svg class="w-5 h-5 text-yellow-300" fill="currentColor" viewBox="0 0 24 24">
                                            <path d="M20 7H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z"/>
                                            <path d="M16 3H8L6 7h12l-2-4zM6 13v6a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-6H6z"/>
                                        </svg>
                                    </div>
                                    <div class="hidden sm:block">
                                        <h1 class="text-lg font-bold tracking-tight leading-none">"SIMPelv2"</h1>
                                        <p class="text-[10px] text-blue-200 leading-none mt-0.5">"Perlengkapan"</p>
                                    </div>
                                </a>
                            </div>
                            <div class="flex items-center gap-2 md:gap-3">
                                <RoleSwitcher />
                                <div class="hidden md:block h-6 w-px bg-white/20"></div>
                                <UserProfile class="text-white".to_string() />
                                <LogoutButton
                                    class="text-white/80 hover:text-white hover:bg-white/10 px-3 py-1.5 rounded-lg text-sm transition-all".to_string()
                                />
                            </div>
                        </div>
                    </div>
                </header>

                // ── Body ──────────────────────────────────────────────────────
                <div class="flex flex-1 min-h-0">
                    <Sidebar sidebar_open=sidebar_open />

                    <main class="flex-1 overflow-y-auto px-4 lg:px-6 py-6">
                        // NOTE: inner <Routes> reads the FULL URL after base="/perlengkapan",
                        // so all paths here start with "/dashboard/".
                        <Routes fallback=|| view! { <DashboardHome /> }>

                            // ── Home ──────────────────────────────────────────
                            <Route path=path!("/dashboard") view=DashboardHome />

                            // ── Bank Aset ─────────────────────────────────────
                            <Route path=path!("/dashboard/bank-aset/daftar")  view=AsetList />
                            <Route path=path!("/dashboard/bank-aset/peta") view=|| view! {
                                <UnderConstruction title="Peta Sebaran Aset"
                                    description="Visualisasi lokasi aset BMN di seluruh satker." />
                            } />
                            <Route path=path!("/dashboard/bank-aset/qr-code") view=|| view! {
                                <UnderConstruction title="Cetak QR Code BMN"
                                    description="Pencetakan label QR Code untuk setiap aset." />
                            } />

                            // ── Analisis Kebutuhan ────────────────────────────
                            <Route path=path!("/dashboard/analisis")          view=AnalisisList />
                            <Route path=path!("/dashboard/analisis/daftar")   view=AnalisisList />
                            <Route path=path!("/dashboard/analisis/baru")     view=AnalisisForm />
                            <Route path=path!("/dashboard/analisis/pakaian") view=|| view! {
                                <UnderConstruction title="Analisis Kebutuhan Pakaian"
                                    description="Analisis kebutuhan pakaian dinas berdasarkan data kepegawaian." />
                            } />
                            <Route path=path!("/dashboard/analisis/standardisasi") view=|| view! {
                                <UnderConstruction title="Standardisasi BMN"
                                    description="Pengelolaan standar kebutuhan barang milik negara." />
                            } />

                            // ── Kebutuhan BMN ─────────────────────────────────
                            <Route path=path!("/dashboard/kebutuhan-bmn")               view=KebutuhanBmnDashboard />
                            <Route path=path!("/dashboard/kebutuhan-bmn/dashboard")     view=KebutuhanBmnDashboard />
                            <Route path=path!("/dashboard/kebutuhan-bmn/daftar")        view=KebutuhanBmnList />
                            <Route path=path!("/dashboard/kebutuhan-bmn/baru")          view=KebutuhanBmnForm />
                            <Route path=path!("/dashboard/kebutuhan-bmn/:id")           view=KebutuhanBmnDetail />
                            <Route path=path!("/dashboard/kebutuhan-bmn/:id/edit")      view=KebutuhanBmnForm />
                            <Route path=path!("/dashboard/kebutuhan-bmn/satker/:satker_id") view=KebutuhanBmnSatkerDetail />

                            // ── Pengelolaan BMN ───────────────────────────────
                            <Route path=path!("/dashboard/pengelolaan/pemakaian")       view=PemakaianList />
                            <Route path=path!("/dashboard/pengelolaan/pemakaian/baru")  view=PemakaianForm />
                            <Route path=path!("/dashboard/pengelolaan/hibah")           view=HibahList />
                            <Route path=path!("/dashboard/pengelolaan/hibah/baru")      view=HibahForm />
                            <Route path=path!("/dashboard/pengelolaan/pengalihan")      view=PengalihanList />
                            <Route path=path!("/dashboard/pengelolaan/pengalihan/baru") view=PengalihanForm />
                            <Route path=path!("/dashboard/pengelolaan/mutasi")          view=MutasiList />
                            <Route path=path!("/dashboard/pengelolaan/mutasi/baru")     view=MutasiForm />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan")     view=PenghapusanList />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan/baru") view=PenghapusanForm />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan/:id") view=PenghapusanBmnDetail />

                            // ── Pemakaian BMN workflow ─────────────────────────
                            <Route path=path!("/dashboard/pemakaian-bmn")          view=PemakaianBmnList />
                            <Route path=path!("/dashboard/pemakaian-bmn/daftar")   view=PemakaianBmnList />
                            <Route path=path!("/dashboard/pemakaian-bmn/baru")     view=PemakaianBmnForm />
                            <Route path=path!("/dashboard/pemakaian-bmn/:id")      view=PemakaianBmnDetail />

                            // ── Pemeliharaan ──────────────────────────────────
                            <Route path=path!("/dashboard/pemeliharaan/daftar")    view=PemeliharaanList />
                            <Route path=path!("/dashboard/pemeliharaan/baru")      view=PemeliharaanForm />

                            // ── Pakaian Dinas ─────────────────────────────────
                            <Route path=path!("/dashboard/pakaian-dinas/jenis")    view=PakaianDinasJenisList />
                            <Route path=path!("/dashboard/pakaian-dinas/jenis/:id/spesifikasi") view=|| view! {
                                <UnderConstruction title="Spesifikasi Pakaian"
                                    description="Detail spesifikasi tiap jenis pakaian dinas." />
                            } />
                            <Route path=path!("/dashboard/pakaian-dinas/pengajuan")         view=PakaianDinasPengajuanList />
                            <Route path=path!("/dashboard/pakaian-dinas/pengajuan/:id/satker") view=|| view! {
                                <UnderConstruction title="Pengajuan per Satker"
                                    description="Rincian pengajuan pakaian dinas per satuan kerja." />
                            } />
                            <Route path=path!("/dashboard/pakaian-dinas/ukuran/:satker_id") view=UkuranPegawaiSatkerFromRoute />
                            <Route path=path!("/dashboard/pakaian-dinas/laporan")   view=PakaianDinasLaporan />

                            // ── Bantuan ───────────────────────────────────────
                            <Route path=path!("/dashboard/bantuan/panduan") view=|| view! {
                                <UnderConstruction title="Panduan Penggunaan"
                                    description="Dokumentasi cara penggunaan sistem perlengkapan." />
                            } />
                            <Route path=path!("/dashboard/bantuan/faq") view=|| view! {
                                <UnderConstruction title="FAQ"
                                    description="Pertanyaan yang sering diajukan." />
                            } />
                            <Route path=path!("/dashboard/bantuan/helpdesk") view=|| view! {
                                <UnderConstruction title="Helpdesk"
                                    description="Hubungi tim dukungan teknis." />
                            } />

                            // ── Admin ─────────────────────────────────────────
                            <Route path=path!("/dashboard/admin/users")  view=AdminUsersPage />
                            <Route path=path!("/dashboard/admin/roles")  view=AdminRolesPage />
                            <Route path=path!("/dashboard/admin/audit") view=|| view! {
                                <UnderConstruction title="Audit Log"
                                    description="Riwayat seluruh aktivitas di dalam sistem." />
                            } />
                            <Route path=path!("/dashboard/admin/config") view=|| view! {
                                <UnderConstruction title="Konfigurasi Sistem"
                                    description="Pengaturan parameter aplikasi perlengkapan." />
                            } />
                            <Route path=path!("/dashboard/admin/master") view=|| view! {
                                <UnderConstruction title="Master Data"
                                    description="Pengelolaan data referensi dan kodefikasi BMN." />
                            } />
                        </Routes>
                    </main>
                </div>

                // ── Footer ────────────────────────────────────────────────────
                <footer class="bg-white border-t border-gray-200">
                    <div class="px-4 lg:px-6 py-3">
                        <div class="flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-gray-500">
                            <div class="flex items-center gap-2">
                                <i class="fas fa-shield-alt text-blue-500"></i>
                                <span>"© 2024 Kejaksaan Republik Indonesia"</span>
                            </div>
                            <div class="flex items-center gap-4">
                                <span>"v"{APP_VERSION}</span>
                                <span class="flex items-center gap-1">
                                    <i class="fas fa-lock"></i>
                                    " Encrypted"
                                </span>
                            </div>
                        </div>
                    </div>
                </footer>
            </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper — reads :satker_id from route params for UkuranPegawaiSatker
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn UkuranPegawaiSatkerFromRoute() -> impl IntoView {
    use leptos_router::hooks::use_params_map;
    let params = use_params_map();
    let satker_id = move || params.with(|p| p.get("satker_id").unwrap_or_default());

    view! {
        <UkuranPegawaiSatker
            satker_id=satker_id()
            satker_nama="".to_string()
        />
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WASM entry point
// ═══════════════════════════════════════════════════════════════════════════

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::logging::log!("🚀 Starting SIMPEL Perlengkapan v{APP_VERSION}…");

    if let Some(app_div) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("app"))
    {
        app_div.set_inner_html("");
    }

    leptos::mount::mount_to_body(App);
    leptos::logging::log!("✅ SIMPEL Perlengkapan mounted.");
}
