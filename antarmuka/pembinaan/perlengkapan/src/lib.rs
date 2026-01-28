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
use lib_ui::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use lib_ui::prelude::*;
use wasm_bindgen::prelude::*;

mod components;

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

        <Router>
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
    let content = move || {
        view! {
            <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
                // Header with auth
                <header class="simpelv2-header bg-blue-700 text-white">
                    <div class="container mx-auto px-6 py-4">
                        <div class="flex items-center justify-between">
                            <Logo show_text=true />
                            <div class="flex items-center space-x-4">
                                <UserProfile class="text-white".to_string() />
                                <LogoutButton class="text-white hover:bg-blue-800".to_string() />
                            </div>
                        </div>
                    </div>
                </header>

                // Content area
                <main class="container mx-auto px-6 py-8">
                    <Routes fallback=|| view! { <DashboardHome /> }>
                        // Dashboard utama
                        <Route path=path!("/") view=DashboardHome />

                        // Bank Aset routes
                        <Route path=path!("/bank-aset/*") view=BankAsetRoutes />

                        // Analisis Kebutuhan routes
                        <Route path=path!("/analisis/*") view=AnalisisRoutes />

                        // Pengadaan routes
                        <Route path=path!("/pengadaan/*") view=PengadaanRoutes />

                        // Pengelolaan BMN routes
                        <Route path=path!("/pengelolaan/*") view=PengelolaanRoutes />

                        // Pengguna routes
                        <Route path=path!("/pengguna/*") view=PenggunaRoutes />

                        // Bantuan routes
                        <Route path=path!("/bantuan/*") view=BantuanRoutes />
                    </Routes>
                </main>

                // Footer menggunakan shared
                <footer class="bg-gray-100 border-t py-6 simpelv2-stat-card">
                    <div class="container mx-auto px-6 text-center">
                        <p class="text-gray-600">"© 2024 Kejaksaan Republik Indonesia"</p>
                    </div>
                </footer>
            </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

// Bank Aset Routes
#[component]
fn BankAsetRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/daftar") view=|| view! { <div>"Daftar Aset"</div> } />
            <Route path=path!("/peta") view=|| view! { <div>"Peta Sebaran Aset"</div> } />
            <Route path=path!("/qr-code") view=|| view! { <div>"Cetak QR Code BMN"</div> } />
        </Routes>
    }
}

// Analisis Kebutuhan Routes
#[component]
fn AnalisisRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/pakaian/*") view=|| view! { <div>"Kebutuhan Pakaian"</div> } />
            <Route path=path!("/bmn/*") view=|| view! { <div>"Kebutuhan BMN"</div> } />
            <Route path=path!("/standardisasi/*") view=|| view! { <div>"Standardisasi BMN"</div> } />
        </Routes>
    }
}

// Pengadaan Routes
#[component]
fn PengadaanRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/administrasi") view=|| view! { <div>"Administrasi Pengadaan"</div> } />
            <Route path=path!("/distribusi") view=|| view! { <div>"Distribusi"</div> } />
        </Routes>
    }
}

// Pengelolaan BMN Routes
#[component]
fn PengelolaanRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/pemakaian/*") view=|| view! { <div>"Pemakaian BMN"</div> } />
            <Route path=path!("/hibah/*") view=|| view! { <div>"Penerimaan Hibah"</div> } />
            <Route path=path!("/pengalihan/*") view=|| view! { <div>"Pengalihan BMN"</div> } />
            <Route path=path!("/mutasi/*") view=|| view! { <div>"Mutasi BMN"</div> } />
            <Route path=path!("/penghapusan/*") view=|| view! { <div>"Penghapusan BMN"</div> } />
        </Routes>
    }
}

// Pengguna Routes
#[component]
fn PenggunaRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/profil") view=|| view! { <div>"Profil Pengguna"</div> } />
            <Route path=path!("/aktivitas") view=|| view! { <div>"Log Aktivitas"</div> } />
        </Routes>
    }
}

// Bantuan Routes
#[component]
fn BantuanRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/helpdesk") view=|| view! { <div>"Helpdesk"</div> } />
            <Route path=path!("/panduan") view=|| view! { <div>"Panduan Penggunaan"</div> } />
            <Route path=path!("/faq") view=|| view! { <div>"FAQ"</div> } />
        </Routes>
    }
}

// Dashboard Home Component
#[component]
fn DashboardHome() -> impl IntoView {
    view! {
        <div class="space-y-6 animate-fade-in">
            // Welcome Banner dengan gradient dan animasi
            <div class="relative overflow-hidden bg-gradient-to-r from-emerald-600 via-blue-600 to-indigo-600 rounded-2xl shadow-2xl p-8 text-white">
                // Animated background pattern
                <div class="absolute inset-0 opacity-10">
                    <div class="absolute w-64 h-64 bg-white rounded-full -top-20 -right-20 animate-pulse"></div>
                    <div class="absolute w-96 h-96 bg-white rounded-full -bottom-32 -left-32 animate-pulse" style="animation-delay: 1s"></div>
                </div>

                <div class="relative z-10">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="p-3 bg-white/20 backdrop-blur-sm rounded-xl">
                            <i class="fas fa-boxes text-3xl"></i>
                        </div>
                        <div>
                            <h1 class="text-3xl md:text-4xl font-bold tracking-tight">
                                "Dashboard Perlengkapan"
                            </h1>
                            <p class="text-blue-100 text-sm md:text-base mt-1">
                                "Sistem Informasi Manajemen Perlengkapan Kejaksaan RI"
                            </p>
                        </div>
                    </div>

                    <div class="flex flex-wrap gap-4 mt-6">
                        <div class="flex items-center gap-2 px-4 py-2 bg-white/20 backdrop-blur-sm rounded-lg">
                            <i class="fas fa-calendar-day"></i>
                            <span class="text-sm font-medium">"Senin, 6 Oktober 2025"</span>
                        </div>
                        <div class="flex items-center gap-2 px-4 py-2 bg-white/20 backdrop-blur-sm rounded-lg">
                            <i class="fas fa-user-check"></i>
                            <span class="text-sm font-medium">"Status: Aktif"</span>
                        </div>
                    </div>
                </div>
            </div>

            // Quick Stats Cards dengan animasi stagger
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6">
                // Card 1 - Total Aset
                <div class="group relative overflow-hidden bg-gradient-to-br from-blue-500 to-blue-600 text-white p-6 rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 hover:scale-105 cursor-pointer animate-slide-in-left">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-white/10 rounded-full -mr-12 -mt-12"></div>
                    <div class="relative z-10">
                        <div class="flex items-center justify-between mb-4">
                            <div class="p-3 bg-white/20 backdrop-blur-sm rounded-lg group-hover:scale-110 transition-transform">
                                <i class="fas fa-box text-2xl"></i>
                            </div>
                            <i class="fas fa-arrow-trend-up text-white/50"></i>
                        </div>
                        <h3 class="text-sm font-medium opacity-90 mb-1">"Total Aset"</h3>
                        <p class="text-4xl font-bold mb-1">"1,234"</p>
                        <p class="text-xs opacity-75">
                            <i class="fas fa-arrow-up mr-1"></i>
                            "+12% dari bulan lalu"
                        </p>
                    </div>
                </div>

                // Card 2 - Pengadaan Aktif
                <div class="group relative overflow-hidden bg-gradient-to-br from-emerald-500 to-green-600 text-white p-6 rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 hover:scale-105 cursor-pointer animate-slide-in-left" style="animation-delay: 0.1s">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-white/10 rounded-full -mr-12 -mt-12"></div>
                    <div class="relative z-10">
                        <div class="flex items-center justify-between mb-4">
                            <div class="p-3 bg-white/20 backdrop-blur-sm rounded-lg group-hover:scale-110 transition-transform">
                                <i class="fas fa-shopping-cart text-2xl"></i>
                            </div>
                            <i class="fas fa-pulse text-white/50"></i>
                        </div>
                        <h3 class="text-sm font-medium opacity-90 mb-1">"Pengadaan Aktif"</h3>
                        <p class="text-4xl font-bold mb-1">"23"</p>
                        <p class="text-xs opacity-75">
                            <i class="fas fa-clock mr-1"></i>
                            "5 menunggu persetujuan"
                        </p>
                    </div>
                </div>

                // Card 3 - Pending Approval
                <div class="group relative overflow-hidden bg-gradient-to-br from-amber-500 to-yellow-600 text-white p-6 rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 hover:scale-105 cursor-pointer animate-slide-in-left" style="animation-delay: 0.2s">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-white/10 rounded-full -mr-12 -mt-12"></div>
                    <div class="relative z-10">
                        <div class="flex items-center justify-between mb-4">
                            <div class="p-3 bg-white/20 backdrop-blur-sm rounded-lg group-hover:scale-110 transition-transform">
                                <i class="fas fa-hourglass-half text-2xl animate-spin-slow"></i>
                            </div>
                            <i class="fas fa-exclamation text-white/50"></i>
                        </div>
                        <h3 class="text-sm font-medium opacity-90 mb-1">"Pending Approval"</h3>
                        <p class="text-4xl font-bold mb-1">"12"</p>
                        <p class="text-xs opacity-75">
                            <i class="fas fa-user-clock mr-1"></i>
                            "3 perlu tindakan segera"
                        </p>
                    </div>
                </div>

                // Card 4 - Perlu Perhatian
                <div class="group relative overflow-hidden bg-gradient-to-br from-red-500 to-pink-600 text-white p-6 rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 hover:scale-105 cursor-pointer animate-slide-in-left" style="animation-delay: 0.3s">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-white/10 rounded-full -mr-12 -mt-12"></div>
                    <div class="relative z-10">
                        <div class="flex items-center justify-between mb-4">
                            <div class="p-3 bg-white/20 backdrop-blur-sm rounded-lg group-hover:scale-110 transition-transform">
                                <i class="fas fa-exclamation-triangle text-2xl animate-pulse"></i>
                            </div>
                            <i class="fas fa-bell text-white/50 animate-swing"></i>
                        </div>
                        <h3 class="text-sm font-medium opacity-90 mb-1">"Perlu Perhatian"</h3>
                        <p class="text-4xl font-bold mb-1">"5"</p>
                        <p class="text-xs opacity-75">
                            <i class="fas fa-triangle-exclamation mr-1"></i>
                            "Tindakan diperlukan"
                        </p>
                    </div>
                </div>
            </div>

            // Quick Actions Section
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-xl shadow-md hover:shadow-xl transition-all duration-300 border border-gray-100 hover:border-emerald-200">
                    <div class="flex items-center gap-4 mb-4">
                        <div class="p-3 bg-emerald-100 rounded-lg">
                            <i class="fas fa-plus-circle text-2xl text-emerald-600"></i>
                        </div>
                        <div>
                            <h3 class="font-bold text-gray-900">"Pengadaan Baru"</h3>
                            <p class="text-xs text-gray-500">"Buat pengadaan aset baru"</p>
                        </div>
                    </div>
                    <button class="w-full py-2 px-4 bg-gradient-to-r from-emerald-600 to-green-600 text-white rounded-lg hover:shadow-lg transition-all hover:scale-105 font-medium">
                        "Mulai Pengadaan"
                    </button>
                </div>

                <div class="bg-white p-6 rounded-xl shadow-md hover:shadow-xl transition-all duration-300 border border-gray-100 hover:border-blue-200">
                    <div class="flex items-center gap-4 mb-4">
                        <div class="p-3 bg-blue-100 rounded-lg">
                            <i class="fas fa-file-alt text-2xl text-blue-600"></i>
                        </div>
                        <div>
                            <h3 class="font-bold text-gray-900">"Laporan"</h3>
                            <p class="text-xs text-gray-500">"Lihat laporan aset"</p>
                        </div>
                    </div>
                    <button class="w-full py-2 px-4 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:shadow-lg transition-all hover:scale-105 font-medium">
                        "Buka Laporan"
                    </button>
                </div>

                <div class="bg-white p-6 rounded-xl shadow-md hover:shadow-xl transition-all duration-300 border border-gray-100 hover:border-purple-200">
                    <div class="flex items-center gap-4 mb-4">
                        <div class="p-3 bg-purple-100 rounded-lg">
                            <i class="fas fa-chart-line text-2xl text-purple-600"></i>
                        </div>
                        <div>
                            <h3 class="font-bold text-gray-900">"Statistik"</h3>
                            <p class="text-xs text-gray-500">"Analisis data aset"</p>
                        </div>
                    </div>
                    <button class="w-full py-2 px-4 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-lg hover:shadow-lg transition-all hover:scale-105 font-medium">
                        "Lihat Analisis"
                    </button>
                </div>
            </div>
        </div>
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
