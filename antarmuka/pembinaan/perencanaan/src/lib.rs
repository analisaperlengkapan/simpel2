//! # SIMPelv2 Perencanaan Microfrontend
//!
//! Sistem Informasi Manajemen Perencanaan BMN (Barang Milik Negara)
//! Mengelola perencanaan pengadaan, pemeliharaan, dan pengembangan aset negara

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::*;
use shared_microfrontend::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use shared_microfrontend::prelude::*;

pub mod components;
pub mod pages;

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
                <Route path=path!("/") view=LoginRedirectPage />
                <Route path=path!("/dashboard/*") view=DashboardRoutes />
            </Routes>
        </Router>
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
    };

    view! {
        <ProtectedRoute children=content />
    }
}
