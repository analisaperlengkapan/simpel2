//! # SIMPelv2 Perencanaan Microfrontend
//!
//! Sistem Informasi Manajemen Perencanaan BMN (Barang Milik Negara)
//! Mengelola perencanaan pengadaan, pemeliharaan, dan pengembangan aset negara

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::*;
use shared_microfrontend::prelude::*;
use web_sys::window;

// Additional imports for async operations
use gloo::timers::future::TimeoutFuture;
use leptos::task::spawn_local;

pub mod components;
pub mod pages;

use components::LoginPage;

/// Main App Component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="SIMPelv2 - Perencanaan BMN" />
        <Meta name="description" content="Sistem Informasi Manajemen Perencanaan Barang Milik Negara" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />

        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/auth/callback") view=AuthCallback />
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
                        <i class="fas fa-chart-line text-4xl text-white"></i>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 mb-2">"SIMPEL"</h1>
                    <p class="text-lg text-gray-700 font-medium mb-1">"PERENCANAAN"</p>
                    <p class="text-gray-600">"Sistem Informasi Manajemen Perencanaan"</p>
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

    Effect::new(move |_| {
        let navigate_clone = navigate.clone();
        spawn_local(async move {
            // Simulate authentication callback processing
            TimeoutFuture::new(1000).await;

            // For now, just redirect to dashboard
            navigate_clone("/dashboard", Default::default());
        });
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-900 via-blue-800 to-blue-700">
            <div class="bg-white p-8 rounded-xl shadow-xl text-center">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
                <p class="text-gray-700">"Memproses autentikasi..."</p>
            </div>
        </div>
    }
}

/// Not Found Page
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
            <div class="bg-white p-12 rounded-2xl shadow-2xl text-center w-96">
                <div class="w-24 h-24 bg-gradient-to-br from-red-400 to-red-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-lg">
                    <i class="fas fa-exclamation-triangle text-4xl text-white"></i>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 mb-4">"404"</h1>
                <p class="text-gray-600 mb-6">"Halaman tidak ditemukan"</p>
                <a href="/dashboard" class="bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition-colors">
                    "Kembali ke Dashboard"
                </a>
            </div>
        </div>
    }
}

/// Dashboard Routes - Nested routing for dashboard pages
#[component]
pub fn DashboardRoutes() -> impl IntoView {
    let navigate = use_navigate();

    // Check authentication before showing dashboard
    if !is_authenticated() {
        navigate("/login", Default::default());
        return view! {
            <div class="min-h-screen flex items-center justify-center">
                <div class="text-center">
                    <p class="text-red-600">Authentication required. Redirecting...</p>
                </div>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            // Header menggunakan shared
            <header class="simpelv2-header">
                <div class="container mx-auto px-6 py-4">
                    <Logo show_text=true />
                </div>
            </header>

            // Content area
            <main class="container mx-auto px-6 py-8">
                <Routes fallback=|| view! { <pages::dashboard_home::DashboardHome /> }>
                    // Dashboard utama
                    <Route path=path!("/") view=pages::dashboard_home::DashboardHome />

                    // Rencana Pengadaan routes
                    <Route path=path!("/pengadaan") view=pages::rencana_pengadaan::RencanaPengadaan />

                    // Rencana Pemeliharaan routes
                    <Route path=path!("/pemeliharaan") view=pages::rencana_pemeliharaan::RencanaPemeliharaan />

                    // Rencana Pengembangan routes
                    <Route path=path!("/pengembangan") view=pages::rencana_pengembangan::RencanaPengembangan />

                    // Laporan Perencanaan routes
                    <Route path=path!("/laporan") view=pages::laporan_perencanaan::LaporanPerencanaan />
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
    .into_any()
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
