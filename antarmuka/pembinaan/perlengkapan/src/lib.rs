//! # SIMPEL Perlengkapan - Microfrontend
//!
//! Sistem Informasi Manajemen Perlengkapan untuk Kejaksaan RI
//! menggunakan Leptos CSR SPA WASM

use components::LoginPage;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use web_sys::window;

pub mod components;

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
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/dashboard/*") view=DashboardRoutes />
            </Routes>
        </Router>
    }
}

/// Home Page - Check authentication
#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();

    // Check authentication token
    let authenticated = is_authenticated();

    if authenticated {
        // Redirect to dashboard if authenticated
        navigate("/dashboard", Default::default());
    } else {
        // Redirect to login page
        navigate("/login", Default::default());
    }

    // Loading state sementara redirect
    view! {
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
                <p class="text-gray-600">Loading...</p>
            </div>
        </div>
    }
}

/// Dashboard routes handler
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
        <div class="min-h-screen bg-gray-50">
            // Simple header
            <header class="bg-white shadow-sm border-b">
                <div class="px-6 py-4">
                    <h1 class="text-2xl font-bold text-gray-900">SIMPEL Perlengkapan</h1>
                    <p class="text-gray-600">Kejaksaan Republik Indonesia</p>
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

            // Simple footer
            <footer class="bg-gray-100 border-t py-6">
                <div class="container mx-auto px-6 text-center">
                    <p class="text-gray-600">"© 2024 Kejaksaan Republik Indonesia"</p>
                </div>
            </footer>
        </div>
    }
    .into_any()
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
        <div class="space-y-6">
            <div class="bg-white rounded-lg shadow p-6">
                <h1 class="text-2xl font-bold text-gray-900 mb-4">
                    "Dashboard SIMPEL Perlengkapan"
                </h1>
                <p class="text-gray-600">
                    "Selamat datang di Sistem Informasi Manajemen Perlengkapan Kejaksaan RI"
                </p>
            </div>

            // Quick stats cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="bg-blue-500 text-white p-6 rounded-lg">
                    <h3 class="text-lg font-medium">"Total Aset"</h3>
                    <p class="text-3xl font-bold">"1,234"</p>
                </div>
                <div class="bg-green-500 text-white p-6 rounded-lg">
                    <h3 class="text-lg font-medium">"Pengadaan Aktif"</h3>
                    <p class="text-3xl font-bold">"23"</p>
                </div>
                <div class="bg-yellow-500 text-white p-6 rounded-lg">
                    <h3 class="text-lg font-medium">"Pending Approval"</h3>
                    <p class="text-3xl font-bold">"12"</p>
                </div>
                <div class="bg-red-500 text-white p-6 rounded-lg">
                    <h3 class="text-lg font-medium">"Perlu Perhatian"</h3>
                    <p class="text-3xl font-bold">"5"</p>
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
    if let Some(window) = window() {
        if let Ok(storage) = window.local_storage() {
            if let Some(storage) = storage {
                return storage.get_item("jwt_token").unwrap_or(None).is_some();
            }
        }
    }
    false
}

/// Main function to mount the app
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
