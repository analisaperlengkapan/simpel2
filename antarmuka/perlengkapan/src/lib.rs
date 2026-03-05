//! Perlengkapan Microfrontend — main app shell.
//!
//! Dark navy + gold Kejaksaan theme, single-page application.

#![recursion_limit="512"]
mod api;
mod components;
mod pages;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use components::profile_menu::ProfileMenu;
use components::sidebar::Sidebar;
use pages::dashboard::DashboardHome;
use pages::not_found::NotFound;
use pages::placeholder::PlaceholderPage;

// Migrated business components
use components::aset_list::AsetList;
use components::kebutuhan_bmn_list::KebutuhanBmnList;
use components::kebutuhan_bmn_form::KebutuhanBmnForm;
use components::kebutuhan_bmn_detail::KebutuhanBmnDetail;
use components::kebutuhan_bmn_satker::KebutuhanBmnSatkerDetail;
use components::pakaian_dinas_jenis_list::PakaianDinasJenisList;
use components::pakaian_dinas_pengajuan_list::PakaianDinasPengajuanList;
use components::pakaian_dinas_ukuran::UkuranPegawai;
use components::pakaian_dinas_laporan::PakaianDinasLaporan;
use components::pemakaian_bmn_list::PemakaianBmnList;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_detail::PemakaianBmnDetail;
use components::pemakaian_bmn_monitoring::PemakaianBmnMonitoring;
use components::penghapusan_list::PenghapusanList;
use components::penghapusan_bmn_detail::PenghapusanBmnDetail;
use components::penghapusan_form::PenghapusanForm;
use components::analisis_list::AnalisisList;
use components::analisis_form::AnalisisForm;
use components::qrcode_generator::QrCodeGenerator;
use components::laporan_kebutuhan_bmn::LaporanKebutuhanBmn;
use components::admin_users::{AdminUsersPage, AdminRolesPage};
use components::mapping_kodefikasi_dashboard::MappingKodefikasiDashboard;
use components::panduan::PanduanPengguna;
use components::faq::FaqPage;
use components::helpdesk::HelpdeskPage;

// ── Version ──────────────────────────────────────────────────────────────
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ═════════════════════════════════════════════════════════════════════════
// Root App — all routes are FLAT inside the layout shell
// ═════════════════════════════════════════════════════════════════════════

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let sidebar_open = RwSignal::new(false);

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL — Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <Router base="/perlengkapan">
            // ══ Full-page dark shell ═════════════════════════════════
            <div style="min-height: 100vh; display: flex; flex-direction: column; background: linear-gradient(180deg, #0f172a 0%, #111c36 50%, #0a1020 100%); font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;">

                // ── Header ───────────────────────────────────────────
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
                            <a href="/perlengkapan/dashboard" style="display: flex; align-items: center; gap: 10px; text-decoration: none;">
                                <img
                                    src="/perlengkapan/assets/kejaksaan-logo.png"
                                    alt="Kejaksaan RI"
                                    style="width: 30px; height: 30px; object-fit: contain;"
                                />
                                <span style="font-size: 1.1rem; font-weight: 800; color: #ffffff; letter-spacing: 0.02em;">
                                    "SIMPEL"
                                </span>
                            </a>
                        </div>
                        // Right: profile avatar
                        <ProfileMenu />
                    </div>
                </header>

                // ── Body (sidebar + main) ────────────────────────────
                <div style="display: flex; flex: 1; min-height: 0;">
                    <Sidebar sidebar_open=sidebar_open />

                    <main style="flex: 1; overflow-y: auto; padding: 1.5rem; margin-left: 0;" class="lg:ml-[250px]">
                        <Routes fallback=move || view! { <NotFound /> }.into_any()>
                            // Dashboard home (default)
                            <Route path=path!("/") view=move || {
                                // Redirect root to dashboard
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/perlengkapan/dashboard");
                                }
                                view! { <div></div> }
                            } />
                            <Route path=path!("/dashboard") view=DashboardHome />

                            // ── Bank Aset ────────────────────────────
                            <Route path=path!("/dashboard/bank-aset/daftar") view=AsetList />
                            <Route path=path!("/dashboard/bank-aset/qrcode") view=QrCodeGenerator />

                            // ── Kebutuhan BMN ────────────────────────
                            <Route path=path!("/dashboard/kebutuhan-bmn/daftar") view=KebutuhanBmnList />
                            <Route path=path!("/dashboard/kebutuhan-bmn/buat") view=KebutuhanBmnForm />
                            <Route path=path!("/dashboard/kebutuhan-bmn/detail/:id") view=KebutuhanBmnDetail />
                            <Route path=path!("/dashboard/kebutuhan-bmn/satker/:satker_id") view=KebutuhanBmnSatkerDetail />
                            <Route path=path!("/dashboard/kebutuhan-bmn/laporan") view=LaporanKebutuhanBmn />

                            // ── Pakaian Dinas ────────────────────────
                            <Route path=path!("/dashboard/pakaian-dinas/jenis") view=PakaianDinasJenisList />
                            <Route path=path!("/dashboard/pakaian-dinas/pengajuan") view=PakaianDinasPengajuanList />
                            <Route path=path!("/dashboard/pakaian-dinas/ukuran") view=move || view! { <UkuranPegawai pegawai_id="0".to_string() pegawai_nama="Pegawai".to_string() pegawai_nip="000".to_string() /> } />
                            <Route path=path!("/dashboard/pakaian-dinas/laporan") view=PakaianDinasLaporan />

                            // ── Pengelolaan BMN ──────────────────────
                            <Route path=path!("/dashboard/pengelolaan/pemakaian") view=PemakaianBmnList />
                            <Route path=path!("/dashboard/pengelolaan/pemakaian/buat") view=PemakaianBmnForm />
                            <Route path=path!("/dashboard/pengelolaan/pemakaian/detail/:id") view=PemakaianBmnDetail />
                            <Route path=path!("/dashboard/pengelolaan/pemakaian/monitoring") view=PemakaianBmnMonitoring />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan") view=PenghapusanList />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan/buat") view=PenghapusanForm />
                            <Route path=path!("/dashboard/pengelolaan/penghapusan/detail/:id") view=PenghapusanBmnDetail />

                            // ── Analitik ─────────────────────────────
                            <Route path=path!("/dashboard/analitik/roadmap") view=AnalisisList />
                            <Route path=path!("/dashboard/analitik/roadmap/buat") view=AnalisisForm />
                            <Route path=path!("/dashboard/analitik/kodefikasi") view=MappingKodefikasiDashboard />

                            // ── Admin ────────────────────────────────
                            <Route path=path!("/dashboard/admin/users") view=AdminUsersPage />
                            <Route path=path!("/dashboard/admin/roles") view=AdminRolesPage />
                            <Route path=path!("/dashboard/admin/audit") view=move || view! {
                                <PlaceholderPage title="Audit Log" icon="fas fa-history" description="Jejak audit seluruh aktivitas." />
                            } />
                            <Route path=path!("/dashboard/admin/master") view=move || view! {
                                <PlaceholderPage title="Master Data" icon="fas fa-database" description="Pengelolaan data referensi." />
                            } />

                            // ── Bantuan ──────────────────────────────
                            <Route path=path!("/dashboard/bantuan/panduan") view=PanduanPengguna />
                            <Route path=path!("/dashboard/bantuan/faq") view=FaqPage />
                            <Route path=path!("/dashboard/bantuan/helpdesk") view=HelpdeskPage />
                        </Routes>
                    </main>
                </div>

                // ── Footer ───────────────────────────────────────────
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
            </div>
        </Router>
    }
}
