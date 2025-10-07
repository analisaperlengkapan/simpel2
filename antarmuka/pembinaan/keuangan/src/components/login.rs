//! # SIMPEL Keuangan - Login Component
//!
//! Halaman login khusus untuk SIMPEL Keuangan Kejaksaan RI
//! Menggunakan shared-microfrontend components untuk konsistensi

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_microfrontend::components::{Card, Logo};
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Login Page Component untuk SIMPEL Keuangan
///
/// Menampilkan halaman login dengan:
/// - Logo SIMPEL KEJAKSAAN RI
/// - Judul aplikasi
/// - Tombol login (redirect ke portal)
/// - Loading state
#[component]
pub fn LoginPage() -> impl IntoView {
    let (loading, set_loading) = signal(false);

    // Handle login button click
    let handle_login = move |_| {
        set_loading.set(true);
        console_log!("Redirecting to portal for authentication...");

        spawn_local(async move {
            // Redirect ke portal microfrontend untuk authentication
            if let Some(window) = window() {
                let location = window.location();
                let portal_url = match location.origin() {
                    Ok(origin) => format!("{}/portal/login?redirect=keuangan", origin),
                    Err(_) => "http://localhost:3000/portal/login?redirect=keuangan".to_string(),
                };

                console_log!("Redirecting to: {}", portal_url);
                let _ = location.set_href(&portal_url);
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-blue-900 via-blue-800 to-blue-700 flex items-center justify-center p-4">
            <Card class="w-full max-w-md p-8 text-center bg-white rounded-2xl shadow-2xl">

                // Header dengan Logo Kejaksaan
                <div class="mb-8">
                    <Logo
                        size=96
                        show_text=true
                        class="mx-auto mb-6"
                    />

                    <h1 class="text-3xl font-bold text-gray-900 mb-2">
                        "SIMPEL"
                    </h1>

                    <p class="text-lg text-gray-700 font-semibold mb-1">
                        "KEJAKSAAN RI"
                    </p>

                    <p class="text-gray-600">
                        "Sistem Informasi Manajemen Keuangan"
                    </p>
                </div>

                // Divider
                <div class="border-t border-gray-200 my-6"></div>

                // Login Description
                <div class="mb-6">
                    <p class="text-sm text-gray-600 mb-4">
                        "Untuk mengakses sistem keuangan, Anda akan diarahkan ke portal login utama Kejaksaan RI"
                    </p>
                </div>

                // Login Button
                <div class="space-y-4">
                    {move || {
                        if loading.get() {
                            view! {
                                <div class="flex items-center justify-center py-3">
                                    <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600 mr-2"></div>
                                    <span class="text-blue-600">"Mengarahkan ke portal..."</span>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="w-full py-3 px-6 text-lg font-semibold rounded-lg bg-blue-600 text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 transition-all duration-200 flex items-center justify-center"
                                    on:click=handle_login
                                    aria-label="Masuk ke sistem keuangan"
                                    disabled=false
                                >
                                    <i class="fas fa-sign-in-alt mr-2"></i>
                                    "Masuk ke Sistem"
                                </button>
                            }.into_any()
                        }
                    }}
                </div>

                // Footer Info
                <div class="mt-6 pt-4 border-t border-gray-100">
                    <p class="text-xs text-gray-500">
                        "Gunakan akun SSO Kejaksaan RI Anda"
                    </p>
                </div>
            </Card>
        </div>
    }
}
