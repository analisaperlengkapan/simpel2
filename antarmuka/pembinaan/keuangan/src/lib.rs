//! # SIMPEL Keuangan - Microfrontend
//!
//! Sistem Informasi Manajemen Keuangan untuk Kejaksaan RI
//! menggunakan Leptos CSR SPA WASM

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::*;
use shared_microfrontend::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use shared_microfrontend::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;

// Additional imports for async operations
use gloo::timers::future::TimeoutFuture;
use leptos::task::spawn_local;
use std::collections::HashMap;

pub mod components;

/// Main App Component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPEL Keuangan - Kejaksaan RI" />
        <Meta name="description" content="Sistem Informasi Manajemen Keuangan Kejaksaan Republik Indonesia" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=LoginRedirectPage />
                <Route path=path!("/dashboard/*") view=DashboardRoutes />
            </Routes>
        </Router>
    }
}

/// Home Page - Check authentication
#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();
    let (_show_loading, _set_show_loading) = signal(true);

    // Check authentication after a brief delay to show loading screen
    Effect::new(move |_| {
        let navigate_clone = navigate.clone();
        spawn_local(async move {
            // Show loading for at least 2 seconds for better UX
            TimeoutFuture::new(2000).await;

            let authenticated = is_authenticated();

            _set_show_loading.set(false);

            if authenticated {
                // Redirect to dashboard if authenticated
                navigate_clone("/dashboard", Default::default());
            } else {
                // Redirect to login page
                navigate_clone("/login", Default::default());
            }
        });
    });

    // Show beautiful loading screen
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-900 via-blue-800 to-blue-700">
            <div class="bg-white p-12 rounded-2xl shadow-2xl w-96 text-center">
                <div class="mb-8">
                    <div class="w-24 h-24 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-lg">
                        <i class="fas fa-money-bill-wave text-4xl text-white"></i>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 mb-2">"SIMPEL"</h1>
                    <p class="text-lg text-gray-700 font-medium mb-1">"KEJAKSAAN RI"</p>
                    <p class="text-gray-600">"Sistem Informasi Manajemen Keuangan"</p>
                </div>
                <div class="text-center">
                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
                    <p class="text-sm text-gray-500">"Memuat aplikasi..."</p>
                </div>
            </div>
        </div>
    }
}

/// Authentication callback handler
#[component]
pub fn AuthCallback() -> impl IntoView {
    let navigate = use_navigate();

    // Use Effect::new instead of deprecated create_effect
    Effect::new(move |_| {
        let navigate_clone = navigate.clone();
        spawn_local(async move {
            // Check for token in URL params or localStorage
            if let Some(token) = get_token_from_url() {
                // Store token in localStorage
                if let Some(window) = window()
                    && let Ok(storage) = window.local_storage()
                    && let Some(storage) = storage
                {
                    let _ = storage.set_item("jwt_token", &token);
                }

                // Redirect to dashboard
                navigate_clone("/dashboard", Default::default());
            } else {
                // No token found, redirect to login
                navigate_clone("/login", Default::default());
            }
        });
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-900 via-blue-800 to-blue-700">
            <div class="bg-white p-12 rounded-2xl shadow-2xl w-96 text-center">
                <div class="mb-8">
                    <div class="w-24 h-24 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-lg">
                        <i class="fas fa-check-circle text-4xl text-white"></i>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 mb-2">"Autentikasi Berhasil"</h1>
                    <p class="text-lg text-gray-700 font-medium mb-1">"KEJAKSAAN RI"</p>
                    <p class="text-gray-600">"Mengarahkan ke dashboard..."</p>
                </div>
                <div class="text-center">
                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-green-600 mb-4"></div>
                    <p class="text-sm text-gray-500">"Memproses autentikasi..."</p>
                </div>
            </div>
        </div>
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

                    // Anggaran routes
                    <Route path=path!("/anggaran/*") view=AnggaranRoutes />

                    // Pengeluaran routes
                    <Route path=path!("/pengeluaran/*") view=PengeluaranRoutes />

                    // Penerimaan routes
                    <Route path=path!("/penerimaan/*") view=PenerimaanRoutes />

                    // Laporan Keuangan routes
                    <Route path=path!("/laporan/*") view=LaporanRoutes />

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

// Anggaran Routes
#[component]
fn AnggaranRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/daftar") view=|| view! { <div>"Daftar Anggaran"</div> } />
            <Route path=path!("/rencana") view=|| view! { <div>"Rencana Anggaran"</div> } />
            <Route path=path!("/realisasi") view=|| view! { <div>"Realisasi Anggaran"</div> } />
        </Routes>
    }
}

// Pengeluaran Routes
#[component]
fn PengeluaranRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/operasional") view=|| view! { <div>"Pengeluaran Operasional"</div> } />
            <Route path=path!("/investasi") view=|| view! { <div>"Pengeluaran Investasi"</div> } />
            <Route path=path!("/approval") view=|| view! { <div>"Approval Pengeluaran"</div> } />
        </Routes>
    }
}

// Penerimaan Routes
#[component]
fn PenerimaanRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/pendapatan") view=|| view! { <div>"Penerimaan Pendapatan"</div> } />
            <Route path=path!("/hibah") view=|| view! { <div>"Penerimaan Hibah"</div> } />
            <Route path=path!("/dana") view=|| view! { <div>"Dana Cadangan"</div> } />
        </Routes>
    }
}

// Laporan Keuangan Routes
#[component]
fn LaporanRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound /> }>
            <Route path=path!("/keuangan") view=|| view! { <div>"Laporan Keuangan"</div> } />
            <Route path=path!("/pertanggungjawaban") view=|| view! { <div>"Laporan Pertanggungjawaban"</div> } />
            <Route path=path!("/audit") view=|| view! { <div>"Laporan Audit"</div> } />
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
        <div class="space-y-6">
            <div class="bg-white rounded-lg shadow p-6 simpelv2-stat-card">
                <h1 class="text-2xl font-bold gradient-text mb-4">
                    "Dashboard SIMPEL Keuangan"
                </h1>
                <p class="text-gray-600">
                    "Selamat datang di Sistem Informasi Manajemen Keuangan Kejaksaan RI"
                </p>
            </div>

            // Quick stats cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="simpelv2-stat-card bg-green-500 text-white p-6 rounded-lg">
                    <div class="icon-wrapper mb-3">
                        <i class="fas fa-money-bill-wave text-2xl"></i>
                    </div>
                    <h3 class="text-lg font-medium">"Total Anggaran"</h3>
                    <p class="text-3xl font-bold animate-glow">"Rp 2.5B"</p>
                </div>
                <div class="simpelv2-stat-card bg-blue-500 text-white p-6 rounded-lg">
                    <div class="icon-wrapper mb-3">
                        <i class="fas fa-chart-line text-2xl"></i>
                    </div>
                    <h3 class="text-lg font-medium">"Realisasi"</h3>
                    <p class="text-3xl font-bold animate-glow">"78%"</p>
                </div>
                <div class="simpelv2-stat-card bg-yellow-500 text-white p-6 rounded-lg">
                    <div class="icon-wrapper mb-3">
                        <i class="fas fa-clock text-2xl"></i>
                    </div>
                    <h3 class="text-lg font-medium">"Pending Approval"</h3>
                    <p class="text-3xl font-bold animate-glow">"15"</p>
                </div>
                <div class="simpelv2-stat-card bg-red-500 text-white p-6 rounded-lg">
                    <div class="icon-wrapper mb-3">
                        <i class="fas fa-exclamation-triangle text-2xl"></i>
                    </div>
                    <h3 class="text-lg font-medium">"Perlu Perhatian"</h3>
                    <p class="text-3xl font-bold animate-glow">"3"</p>
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

/// Check if user is authenticated by looking for JWT token
fn is_authenticated() -> bool {
    if let Some(window) = window()
        && let Ok(storage) = window.local_storage()
        && let Some(storage) = storage
    {
        return storage.get_item("jwt_token").unwrap_or(None).is_some();
    }
    false
}

/// Get token from URL parameters
fn get_token_from_url() -> Option<String> {
    if let Some(window) = window() {
        let location = window.location();
        if let Ok(search) = location.search() {
            // Parse URL parameters
            let params: HashMap<String, String> =
                url::form_urlencoded::parse(search.trim_start_matches('?').as_bytes())
                    .into_owned()
                    .collect();

            return params.get("token").cloned();
        }
    }
    None
}

/// Main function to mount the app
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
