#![recursion_limit = "512"]

// SIMPEL Perlengkapan - Microfrontend
//
// Sistem Informasi Manajemen Perlengkapan untuk Kejaksaan RI
// menggunakan Leptos 0.8.x CSR SPA WASM
//
// ## Architecture
// - **Framework**: Leptos 0.8.x (CSR/SPA mode)
// - **UI**: Tailwind CSS + shared-microfrontend components
// - **Router**: Leptos Router dengan nested routes
// - **Auth**: SSO via portal-microfrontend
// - **State**: Leptos signals + localStorage
//
// ## Features
// - Modern authentication flow via Portal SSO
// - Comprehensive asset management (Bank Aset)
// - Supply chain integration
// - Procurement workflow (Pengadaan)
// - Maintenance tracking (Pemeliharaan)
// - Analytics & reporting (Dashboard)
// - Needs analysis (Analisis Kebutuhan)
// - BMN lifecycle management
//
// ## Best Practices Applied
// - Type-safe routing with path! macro
// - Error boundaries for graceful failures
// - Lazy loading untuk performa optimal
// - Accessibility (WCAG 2.1 AA)
// - SEO-friendly meta tags
// - Security headers & CSP compliance

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::*;
use lib_ui::components::auth::{LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile};
use lib_ui::prelude::*;
use wasm_bindgen::prelude::*;

use components::sidebar::Sidebar;

mod api;
mod components;
mod utils;
#[cfg(test)]
mod tests;

use api::fetch_dashboard_stats;
use components::admin_panel::{AdminUsersPage, AdminRolesPage};
use components::analisis_form::AnalisisForm;
use components::analisis_list::AnalisisList;
use components::aset_list::AsetList;
use components::hibah_form::HibahForm;
use components::hibah_list::HibahList;
// Kebutuhan BMN components
use components::kebutuhan_bmn_dashboard::KebutuhanBmnDashboard;
use components::kebutuhan_bmn_detail::KebutuhanBmnDetail;
use components::kebutuhan_bmn_form::KebutuhanBmnForm;
use components::kebutuhan_bmn_list::KebutuhanBmnList;
use components::kebutuhan_bmn_satker::KebutuhanBmnSatkerDetail;
use components::mutasi_form::MutasiForm;
use components::mutasi_list::MutasiList;
use components::pemakaian_form::PemakaianForm;
use components::pemakaian_list::PemakaianList;
use components::pemeliharaan_form::PemeliharaanForm;
use components::pemeliharaan_list::PemeliharaanList;
// Pengadaan components removed - not part of perlengkapan domain
use components::pengalihan_form::PengalihanForm;
use components::pengalihan_list::PengalihanList;
use components::penghapusan_form::PenghapusanForm;
use components::penghapusan_list::PenghapusanList;
use components::penghapusan_bmn_detail::PenghapusanBmnDetail;
// Pemakaian BMN components
use components::pemakaian_bmn_detail::PemakaianBmnDetail;
use components::pemakaian_bmn_form::PemakaianBmnForm;
use components::pemakaian_bmn_list::PemakaianBmnList;
// Pakaian Dinas components
use components::pakaian_dinas_jenis_list::PakaianDinasJenisList;
use components::pakaian_dinas_laporan::PakaianDinasLaporan;
use components::pakaian_dinas_pengajuan_list::PakaianDinasPengajuanList;
// UkuranPegawai removed - no pegawai user in perlengkapan domain
use components::pakaian_dinas_ukuran::UkuranPegawaiSatker;
// Role switcher
use components::role_switcher::RoleSwitcher;

// ============================================================================
// Constants & Configuration
// ============================================================================

/// Application version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "SIMPEL Perlengkapan";

// ============================================================================
// WASM Entry Point
// ============================================================================

/// Main App Component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL Perlengkapan - Kejaksaan RI" />
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
                <Route path=path!("/dashboard/*") view=DashboardRoutes />
            </Routes>
        </Router>
    }
}

#[component]
pub fn DashboardRoutes() -> impl IntoView {
    let sidebar_open = RwSignal::new(false);

    let content = move || {
        view! {
            <div class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 flex flex-col">
                // Header with auth and role switcher
                <header class="bg-gradient-to-r from-blue-800 via-blue-700 to-indigo-800 text-white shadow-lg sticky top-0 z-50">
                    <div class="px-4 lg:px-6">
                        <div class="flex items-center justify-between h-14">
                            // Left: hamburger + Logo
                            <div class="flex items-center gap-3">
                                // Hamburger toggle (mobile only)
                                <button
                                    class="lg:hidden p-2 rounded-lg text-white/80 hover:text-white hover:bg-white/10 transition-all"
                                    on:click=move |_| sidebar_open.update(|o| *o = !*o)
                                    aria-label="Toggle sidebar"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                                    </svg>
                                </button>
                                <a href="/perlengkapan/dashboard" class="flex items-center gap-3 hover:opacity-90 transition-opacity">
                                    <div class="w-9 h-9 bg-white/20 backdrop-blur-sm rounded-lg flex items-center justify-center">
                                        <i class="fas fa-boxes text-yellow-300 text-lg"></i>
                                    </div>
                                    <div class="hidden sm:block">
                                        <h1 class="text-lg font-bold tracking-tight leading-none">"SIMPelv2"</h1>
                                        <p class="text-[10px] text-blue-200 leading-none mt-0.5">"Perlengkapan"</p>
                                    </div>
                                </a>
                            </div>

                            // Right: Role switcher + Profile + Logout
                            <div class="flex items-center gap-2 md:gap-3">
                                <RoleSwitcher />
                                <div class="hidden md:block h-6 w-px bg-white/20"></div>
                                <UserProfile class="text-white".to_string() />
                                <LogoutButton class="text-white/80 hover:text-white hover:bg-white/10 px-3 py-1.5 rounded-lg text-sm transition-all".to_string() />
                            </div>
                        </div>
                    </div>
                </header>

                // Body: sidebar + main content
                <div class="flex flex-1 min-h-0">
                    <Sidebar sidebar_open=sidebar_open />

                    // Main content area
                    <main class="flex-1 overflow-y-auto px-4 lg:px-6 py-6">
                    // NOTE: Routes are prefixed with /dashboard because this inner <Routes>
                    // reads the full URL path (after base="/perlengkapan"), not the remaining
                    // path after the parent route's /dashboard/* match.
                    <Routes fallback=|| view! { <DashboardHome /> }>
                        // Dashboard utama
                        <Route path=path!("/dashboard") view=DashboardHome />

                        // Bank Aset
                        <Route path=path!("/dashboard/bank-aset/daftar") view=AsetList />
                        <Route path=path!("/dashboard/bank-aset/peta") view=|| view! { <div>"Peta Sebaran Aset"</div> } />
                        <Route path=path!("/dashboard/bank-aset/qr-code") view=|| view! { <div>"Cetak QR Code BMN"</div> } />

                        // Analisis Kebutuhan
                        <Route path=path!("/dashboard/analisis") view=AnalisisList />
                        <Route path=path!("/dashboard/analisis/daftar") view=AnalisisList />
                        <Route path=path!("/dashboard/analisis/baru") view=AnalisisForm />
                        <Route path=path!("/dashboard/analisis/pakaian") view=|| view! { <div>"Kebutuhan Pakaian"</div> } />
                        <Route path=path!("/dashboard/analisis/bmn") view=KebutuhanBmnDashboard />
                        <Route path=path!("/dashboard/analisis/bmn/dashboard") view=KebutuhanBmnDashboard />
                        <Route path=path!("/dashboard/analisis/bmn/daftar") view=KebutuhanBmnList />
                        <Route path=path!("/dashboard/analisis/bmn/baru") view=KebutuhanBmnForm />
                        <Route path=path!("/dashboard/analisis/standardisasi") view=|| view! { <div>"Standardisasi BMN"</div> } />

                        // Kebutuhan BMN (direct access)
                        <Route path=path!("/dashboard/kebutuhan-bmn") view=KebutuhanBmnDashboard />
                        <Route path=path!("/dashboard/kebutuhan-bmn/dashboard") view=KebutuhanBmnDashboard />
                        <Route path=path!("/dashboard/kebutuhan-bmn/daftar") view=KebutuhanBmnList />
                        <Route path=path!("/dashboard/kebutuhan-bmn/baru") view=KebutuhanBmnForm />
                        <Route path=path!("/dashboard/kebutuhan-bmn/:id") view=KebutuhanBmnDetail />
                        <Route path=path!("/dashboard/kebutuhan-bmn/:id/edit") view=KebutuhanBmnForm />
                        <Route path=path!("/dashboard/kebutuhan-bmn/satker/:satker_id") view=KebutuhanBmnSatkerDetail />

                        // Pengelolaan BMN
                        <Route path=path!("/dashboard/pengelolaan/pemakaian") view=PemakaianList />
                        <Route path=path!("/dashboard/pengelolaan/pemakaian/daftar") view=PemakaianList />
                        <Route path=path!("/dashboard/pengelolaan/pemakaian/baru") view=PemakaianForm />
                        <Route path=path!("/dashboard/pengelolaan/hibah") view=HibahList />
                        <Route path=path!("/dashboard/pengelolaan/hibah/daftar") view=HibahList />
                        <Route path=path!("/dashboard/pengelolaan/hibah/baru") view=HibahForm />
                        <Route path=path!("/dashboard/pengelolaan/pengalihan") view=PengalihanList />
                        <Route path=path!("/dashboard/pengelolaan/pengalihan/daftar") view=PengalihanList />
                        <Route path=path!("/dashboard/pengelolaan/pengalihan/baru") view=PengalihanForm />
                        <Route path=path!("/dashboard/pengelolaan/pemeliharaan") view=PemeliharaanList />
                        <Route path=path!("/dashboard/pengelolaan/pemeliharaan/daftar") view=PemeliharaanList />
                        <Route path=path!("/dashboard/pengelolaan/pemeliharaan/baru") view=PemeliharaanForm />
                        <Route path=path!("/dashboard/pengelolaan/mutasi") view=MutasiList />
                        <Route path=path!("/dashboard/pengelolaan/mutasi/daftar") view=MutasiList />
                        <Route path=path!("/dashboard/pengelolaan/mutasi/baru") view=MutasiForm />
                        <Route path=path!("/dashboard/pengelolaan/penghapusan") view=PenghapusanList />
                        <Route path=path!("/dashboard/pengelolaan/penghapusan/daftar") view=PenghapusanList />
                        <Route path=path!("/dashboard/pengelolaan/penghapusan/baru") view=PenghapusanForm />
                        <Route path=path!("/dashboard/pengelolaan/penghapusan/:id") view=PenghapusanBmnDetail />

                        // Pemakaian BMN (workflow with konsep surat & signed PDF)
                        <Route path=path!("/dashboard/pemakaian-bmn") view=PemakaianBmnList />
                        <Route path=path!("/dashboard/pemakaian-bmn/daftar") view=PemakaianBmnList />
                        <Route path=path!("/dashboard/pemakaian-bmn/baru") view=PemakaianBmnForm />
                        <Route path=path!("/dashboard/pemakaian-bmn/:id") view=PemakaianBmnDetail />

                        // Pemeliharaan
                        <Route path=path!("/dashboard/pemeliharaan") view=PemeliharaanList />
                        <Route path=path!("/dashboard/pemeliharaan/daftar") view=PemeliharaanList />
                        <Route path=path!("/dashboard/pemeliharaan/baru") view=PemeliharaanForm />

                        // Pakaian Dinas
                        <Route path=path!("/dashboard/pakaian-dinas/jenis") view=PakaianDinasJenisList />
                        <Route path=path!("/dashboard/pakaian-dinas/jenis/:id/spesifikasi") view=|| view! { <div>"Spesifikasi Pakaian"</div> } />
                        <Route path=path!("/dashboard/pakaian-dinas/pengajuan") view=PakaianDinasPengajuanList />
                        <Route path=path!("/dashboard/pakaian-dinas/pengajuan/:id/satker") view=|| view! { <div>"Pengajuan per Satker"</div> } />
                        <Route path=path!("/dashboard/pakaian-dinas/ukuran/:satker_id") view=|| {
                            view! {
                                <UkuranPegawaiSatker
                                    satker_id="satker-placeholder".to_string()
                                    satker_nama="Satker Placeholder".to_string()
                                />
                            }
                        } />
                        <Route path=path!("/dashboard/pakaian-dinas/laporan") view=PakaianDinasLaporan />
                        <Route path=path!("/dashboard/pakaian-dinas/laporan/rekap") view=PakaianDinasLaporan />
                        <Route path=path!("/dashboard/pakaian-dinas/laporan/pegawai") view=PakaianDinasLaporan />

                        // Pengguna
                        <Route path=path!("/dashboard/pengguna/profil") view=|| view! { <div>"Profil Pengguna"</div> } />
                        <Route path=path!("/dashboard/pengguna/aktivitas") view=|| view! { <div>"Log Aktivitas"</div> } />

                        // Bantuan
                        <Route path=path!("/dashboard/bantuan/helpdesk") view=|| view! { <div>"Helpdesk"</div> } />
                        <Route path=path!("/dashboard/bantuan/panduan") view=|| view! { <div>"Panduan Penggunaan"</div> } />
                        <Route path=path!("/dashboard/bantuan/faq") view=|| view! { <div>"FAQ"</div> } />

                        // Admin (admin only)
                        <Route path=path!("/dashboard/admin/users") view=AdminUsersPage />
                        <Route path=path!("/dashboard/admin/roles") view=AdminRolesPage />
                        <Route path=path!("/dashboard/admin/config") view=|| view! {
                            <div class="bg-white rounded-xl shadow-sm border p-8 text-center">
                                <i class="fas fa-cogs text-4xl text-gray-300 mb-4"></i>
                                <h2 class="text-xl font-bold text-gray-700">"Konfigurasi Sistem"</h2>
                                <p class="text-gray-500 mt-2">"Pengaturan aplikasi perlengkapan"</p>
                            </div>
                        } />
                        <Route path=path!("/dashboard/admin/audit") view=|| view! {
                            <div class="bg-white rounded-xl shadow-sm border p-8 text-center">
                                <i class="fas fa-clipboard-list text-4xl text-gray-300 mb-4"></i>
                                <h2 class="text-xl font-bold text-gray-700">"Audit Log"</h2>
                                <p class="text-gray-500 mt-2">"Riwayat aktivitas sistem"</p>
                            </div>
                        } />
                        <Route path=path!("/dashboard/admin/master") view=|| view! {
                            <div class="bg-white rounded-xl shadow-sm border p-8 text-center">
                                <i class="fas fa-database text-4xl text-gray-300 mb-4"></i>
                                <h2 class="text-xl font-bold text-gray-700">"Master Data"</h2>
                                <p class="text-gray-500 mt-2">"Pengelolaan data referensi"</p>
                            </div>
                        } />
                    </Routes>
                    </main>
                </div>

                // Footer
                <footer class="bg-white border-t border-gray-200">
                    <div class="px-4 lg:px-6 py-3">
                        <div class="flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-gray-500">
                            <div class="flex items-center gap-2">
                                <i class="fas fa-shield-alt text-blue-500"></i>
                                <span>"© 2024 Kejaksaan Republik Indonesia"</span>
                            </div>
                            <div class="flex items-center gap-4">
                                <span class="flex items-center gap-1"><i class="fas fa-code-branch"></i>" v0.1.0"</span>
                                <span class="flex items-center gap-1"><i class="fas fa-lock"></i>" Encrypted"</span>
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

// NOTE: All route component functions (BankAsetRoutes, AnalisisRoutes, etc.)
// have been removed. Routes are now flattened directly in DashboardRoutes
// with /dashboard prefix, because each <Routes> component independently reads
// the full URL (after base), not the remaining path after the parent route match.

// Dashboard Home Component
#[component]
fn DashboardHome() -> impl IntoView {
    let stats_resource = LocalResource::new(move || async move {
        match fetch_dashboard_stats().await {
            Ok(response) => Some(response.data),
            Err(e) => {
                leptos::logging::error!("Failed to fetch stats: {:?}", e);
                None
            }
        }
    });

    // Get active role for role-aware rendering
    let active_role = components::role_switcher::get_active_role();

    view! {
        <div class="space-y-6">
            // Welcome Banner
            <div class="relative overflow-hidden bg-gradient-to-r from-blue-700 via-indigo-700 to-purple-700 rounded-2xl shadow-xl p-6 md:p-8 text-white">
                <div class="absolute inset-0 opacity-10">
                    <div class="absolute w-64 h-64 bg-white rounded-full -top-20 -right-20"></div>
                    <div class="absolute w-48 h-48 bg-white rounded-full -bottom-16 -left-16"></div>
                </div>
                <div class="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
                    <div>
                        <h1 class="text-2xl md:text-3xl font-bold tracking-tight">"Dashboard Perlengkapan"</h1>
                        <p class="text-blue-200 text-sm mt-1">"Sistem Informasi Manajemen Perlengkapan Kejaksaan RI"</p>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-white/15 backdrop-blur-sm rounded-lg text-xs font-medium">
                            <i class="fas fa-user-shield"></i>
                            {active_role.clone()}
                        </span>
                        <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-white/15 backdrop-blur-sm rounded-lg text-xs font-medium">
                            <i class="fas fa-calendar-day"></i>
                            "Tahun Anggaran 2025"
                        </span>
                    </div>
                </div>
            </div>

            // Stats Cards
            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                    {(0..4).map(|_| view! {
                        <div class="bg-white rounded-xl shadow-sm border p-6 animate-pulse">
                            <div class="h-4 bg-gray-200 rounded w-20 mb-3"></div>
                            <div class="h-8 bg-gray-200 rounded w-16 mb-2"></div>
                            <div class="h-3 bg-gray-200 rounded w-24"></div>
                        </div>
                    }).collect_view()
                    }
                </div>
            }>
                {move || {
                    stats_resource.get().flatten().map(|stats| {
                        view! {
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                                <StatCard
                                    icon="fas fa-box"
                                    label="Total Aset"
                                    value=stats.total_aset.to_string()
                                    subtitle="Terintegrasi SIMAN"
                                    color="blue"
                                />
                                <StatCard
                                    icon="fas fa-check-circle"
                                    label="Kondisi Baik"
                                    value=stats.aset_baik.to_string()
                                    subtitle="Siap Pakai"
                                    color="green"
                                />
                                <StatCard
                                    icon="fas fa-exclamation-triangle"
                                    label="Kondisi Rusak"
                                    value=stats.aset_rusak.to_string()
                                    subtitle="Perlu Tindakan"
                                    color="amber"
                                />
                                <StatCard
                                    icon="fas fa-building"
                                    label="Total Satker"
                                    value=stats.total_satker.to_string()
                                    subtitle="Unit Terdata"
                                    color="red"
                                />
                            </div>
                        }
                    })
                }}
            </Suspense>

            // Quick Navigation Grid
            <div>
                <h2 class="text-lg font-bold text-gray-800 mb-4">"Menu Utama"</h2>
                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                    <QuickNav href="/dashboard/bank-aset/daftar" icon="fas fa-boxes" label="Bank Aset" description="Daftar aset BMN" color="emerald" />
                    <QuickNav href="/dashboard/kebutuhan-bmn/dashboard" icon="fas fa-clipboard-list" label="Kebutuhan BMN" description="Analisis kebutuhan" color="blue" />
                    <QuickNav href="/dashboard/pemakaian-bmn/daftar" icon="fas fa-file-signature" label="Pemakaian BMN" description="Pengajuan pemakaian" color="indigo" />
                    <QuickNav href="/dashboard/pakaian-dinas/pengajuan" icon="fas fa-tshirt" label="Pakaian Dinas" description="Kebutuhan pakaian" color="purple" />
                    <QuickNav href="/dashboard/pengelolaan/penghapusan" icon="fas fa-trash-alt" label="Penghapusan BMN" description="Proses penghapusan" color="red" />
                    <QuickNav href="/dashboard/pengelolaan/mutasi" icon="fas fa-exchange-alt" label="Mutasi BMN" description="Transfer antar satker" color="amber" />
                    <QuickNav href="/dashboard/pemeliharaan/daftar" icon="fas fa-tools" label="Pemeliharaan" description="Jadwal & riwayat" color="teal" />
                    <QuickNav href="/dashboard/pengelolaan/hibah" icon="fas fa-gift" label="Hibah BMN" description="Pengelolaan hibah" color="pink" />
                </div>
            </div>

            // Admin Quick Access (only for admin)
            {
                let is_admin = active_role == "admin";
                view! {
                    <Show when=move || is_admin>
                        <div>
                            <h2 class="text-lg font-bold text-gray-800 mb-4 flex items-center gap-2">
                                <i class="fas fa-shield-alt text-red-500"></i>
                                "Panel Admin"
                            </h2>
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                <QuickNav href="/dashboard/admin/users" icon="fas fa-users-cog" label="Manajemen User" description="Kelola pengguna" color="red" />
                                <QuickNav href="/dashboard/admin/roles" icon="fas fa-user-tag" label="Manajemen Role" description="Kelola hak akses" color="orange" />
                                <QuickNav href="/dashboard/admin/audit" icon="fas fa-clipboard-list" label="Audit Log" description="Riwayat aktivitas" color="gray" />
                                <QuickNav href="/dashboard/admin/master" icon="fas fa-database" label="Master Data" description="Data referensi" color="cyan" />
                            </div>
                        </div>
                    </Show>
                }
            }
        </div>
    }
}

// Reusable Stat Card Component
#[component]
fn StatCard(
    icon: &'static str,
    label: &'static str,
    value: String,
    subtitle: &'static str,
    color: &'static str,
) -> impl IntoView {
    let (bg, _text, _icon_bg) = match color {
        "blue" => ("from-blue-500 to-blue-600", "text-blue-600", "bg-blue-100"),
        "green" => ("from-emerald-500 to-green-600", "text-emerald-600", "bg-emerald-100"),
        "amber" => ("from-amber-500 to-yellow-600", "text-amber-600", "bg-amber-100"),
        "red" => ("from-red-500 to-pink-600", "text-red-600", "bg-red-100"),
        _ => ("from-gray-500 to-gray-600", "text-gray-600", "bg-gray-100"),
    };
    let gradient = format!("bg-gradient-to-br {bg}");

    view! {
        <div class=format!("group relative overflow-hidden {gradient} text-white p-5 rounded-xl shadow-md hover:shadow-lg transition-all duration-300 cursor-pointer")>
            <div class="absolute top-0 right-0 w-20 h-20 bg-white/10 rounded-full -mr-10 -mt-10"></div>
            <div class="relative z-10">
                <div class="flex items-center justify-between mb-3">
                    <div class="p-2 bg-white/20 backdrop-blur-sm rounded-lg">
                        <i class=format!("{icon} text-xl")></i>
                    </div>
                </div>
                <p class="text-xs font-medium opacity-80 mb-0.5">{label}</p>
                <p class="text-3xl font-bold">{value}</p>
                <p class="text-[11px] opacity-70 mt-1">
                    <i class="fas fa-info-circle mr-1"></i>
                    {subtitle}
                </p>
            </div>
        </div>
    }
}

// Reusable Quick Navigation Card
#[component]
fn QuickNav(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    color: &'static str,
) -> impl IntoView {
    let icon_bg = match color {
        "emerald" => "bg-emerald-100 text-emerald-600",
        "blue" => "bg-blue-100 text-blue-600",
        "indigo" => "bg-indigo-100 text-indigo-600",
        "purple" => "bg-purple-100 text-purple-600",
        "red" => "bg-red-100 text-red-600",
        "amber" => "bg-amber-100 text-amber-600",
        "teal" => "bg-teal-100 text-teal-600",
        "pink" => "bg-pink-100 text-pink-600",
        "orange" => "bg-orange-100 text-orange-600",
        "gray" => "bg-gray-100 text-gray-600",
        "cyan" => "bg-cyan-100 text-cyan-600",
        _ => "bg-gray-100 text-gray-600",
    };

    view! {
        <a href=href class="group bg-white p-4 rounded-xl shadow-sm border border-gray-100 hover:border-blue-200 hover:shadow-md transition-all duration-200">
            <div class="flex items-start gap-3">
                <div class=format!("p-2.5 rounded-lg {icon_bg} group-hover:scale-110 transition-transform")>
                    <i class=format!("{icon} text-lg")></i>
                </div>
                <div class="min-w-0">
                    <h3 class="font-semibold text-gray-800 text-sm leading-tight">{label}</h3>
                    <p class="text-[11px] text-gray-500 mt-0.5">{description}</p>
                </div>
            </div>
        </a>
    }
}

// 404 Not Found Component
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-gray-900 mb-4">"404"</h1>
                <p class="text-gray-600 mb-4">"Halaman tidak ditemukan"</p>
                <a href="/" class="text-blue-600 hover:text-blue-800">
                    "Kembali ke Dashboard"
                </a>
            </div>
        </div>
    }
}

/// Main entry point untuk WASM application
/// #[wasm_bindgen(start)] tells wasm-bindgen to automatically call this function
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error reporting
    console_error_panic_hook::set_once();

    // Log that we're starting
    leptos::logging::log!("🚀 Starting SIMPEL Perlengkapan...");

    // Clear fallback loading content
    if let Some(app_div) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("app"))
    {
        app_div.set_inner_html("");
    }

    // Mount app to body
    leptos::mount::mount_to_body(App);

    leptos::logging::log!("✅ SIMPEL Perlengkapan mounted successfully!");
}
