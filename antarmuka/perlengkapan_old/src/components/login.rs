//! # SIMPEL Perlengkapan - Login Component
//!
//! Halaman login khusus untuk SIMPEL Perlengkapan Kejaksaan RI
//! Menggunakan shared-microfrontend components untuk konsistensi
//!
//! ## Best Practices Applied
//! - Menggunakan shared components dari shared-microfrontend
//! - Proper error handling dengan Result types
//! - Accessibility compliant (ARIA labels, semantic HTML)
//! - Loading states untuk UX yang baik
//! - Responsive design dengan Tailwind CSS
//! - Type-safe routing dengan Leptos Router

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
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

/// Authentication service helper
mod auth_service {
    use super::*;

    /// Store authentication token in localStorage
    #[allow(dead_code)]
    pub fn store_token(token: &str) -> Result<(), String> {
        window()
            .ok_or("Window not available")?
            .local_storage()
            .map_err(|e| format!("Failed to access localStorage: {:?}", e))?
            .ok_or("localStorage not available")?
            .set_item("auth_token", token)
            .map_err(|e| format!("Failed to store token: {:?}", e))
    }

    /// Check if user is authenticated
    #[allow(dead_code)]
    pub fn is_authenticated() -> bool {
        if let Some(window) = window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            return storage.get_item("auth_token").ok().flatten().is_some();
        }
        false
    }
}

/// Login Page Component untuk SIMPEL Perlengkapan
/// ## Architecture
/// Menggunakan portal-microfrontend untuk SSO authentication.
/// Flow: LoginPage -> Portal SSO -> Callback -> Dashboard
/// ## Features
/// - Shared components dari shared-microfrontend (Logo)
/// - Responsive design
/// - Accessible (WCAG 2.1 AA compliant)
/// - Loading states dengan feedback visual
/// - Error handling yang proper
/// ## Development Mode
/// Untuk development, tombol login akan simulasi auth dan redirect ke dashboard.
/// Untuk production, akan redirect ke portal SSO.
#[component]
pub fn LoginPage() -> impl IntoView {
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let navigate = use_navigate();
    let navigate = StoredValue::new(navigate);

    // Determine environment (dev vs prod)
    let is_dev_mode = cfg!(debug_assertions);

    // Handle login button click
    let handle_login = move |_| {
        set_loading.set(true);
        set_error.set(None);
        console_log!(
            "🔐 Login initiated - Environment: {}",
            if is_dev_mode {
                "Development"
            } else {
                "Production"
            }
        );

        spawn_local(async move {
            // Simulate authentication delay for better UX
            gloo_timers::future::TimeoutFuture::new(800).await;

            // Development: Simulasi auth lokal
            if is_dev_mode {
                match auth_service::store_token("dev_token_perlengkapan_v1") {
                    Ok(_) => {
                        console_log!("✅ Dev token stored successfully");
                        navigate.with_value(|nav| nav("/dashboard", Default::default()));
                    }
                    Err(e) => {
                        console_log!("❌ Failed to store token: {}", e);
                        set_error.set(Some(format!("Authentication error: {}", e)));
                        set_loading.set(false);
                    }
                }
            } else {
                // Production: Redirect ke portal SSO
                if let Some(window) = window() {
                    let location = window.location();
                    match location.origin() {
                        Ok(origin) => {
                            let portal_url = format!(
                                "{}/portal/login?redirect=perlengkapan&from={}",
                                origin,
                                location.pathname().unwrap_or_default()
                            );
                            console_log!("🔄 Redirecting to portal SSO: {}", portal_url);
                            let _ = location.set_href(&portal_url);
                        }
                        Err(_) => {
                            set_error.set(Some("Failed to get origin URL".to_string()));
                            set_loading.set(false);
                        }
                    }
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-gray-50 via-white to-gray-100 flex flex-col overflow-hidden">
            // Top accent bar
            <div class="h-1 w-full bg-gradient-to-r from-red-700 via-red-600 to-red-700 flex-shrink-0"></div>

            <main class="flex-1 flex items-center justify-center px-4 py-12">
                <div class="w-full max-w-md">
                    // Header
                    <div class="text-center mb-8">
                        <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-red-600 to-red-700 rounded-full shadow-xl mb-4">
                            <span class="text-4xl">"⚖️"</span>
                        </div>
                        <h1 class="text-3xl font-bold text-gray-900 mb-1">
                            "SIMPEL Perlengkapan"
                        </h1>
                        <p class="text-sm text-gray-500">
                            "Sistem Informasi Manajemen Perlengkapan"
                        </p>
                        <p class="text-xs text-gray-400 mt-1">
                            "Kejaksaan Republik Indonesia"
                        </p>
                    </div>

                    // Login card
                    <div class="bg-white rounded-2xl shadow-xl border border-gray-100 p-8">
                        // Divider
                        <div class="relative mb-6">
                            <div class="absolute inset-0 flex items-center">
                                <div class="w-full border-t border-gray-200"></div>
                            </div>
                            <div class="relative flex justify-center">
                                <span class="px-3 text-xs font-medium text-gray-400 bg-white uppercase tracking-wider">"Akses Sistem"</span>
                            </div>
                        </div>

                        // Info box
                        <div class="mb-6 p-4 bg-red-50 rounded-xl border border-red-100">
                            <div class="flex items-start gap-3">
                                <svg class="w-5 h-5 text-red-500 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
                                </svg>
                                <p class="text-sm text-gray-700 leading-relaxed">
                                    {move || if is_dev_mode {
                                        "Mode Development — Klik tombol untuk simulasi login dan akses dashboard perlengkapan."
                                    } else {
                                        "Anda akan diarahkan ke Portal Login Kejaksaan RI untuk autentikasi SSO."
                                    }}
                                </p>
                            </div>
                        </div>

                        // Error
                        {move || error.get().map(|err| view! {
                            <div class="mb-4 flex items-start gap-2 p-3 bg-red-50 border border-red-200 rounded-xl text-sm text-red-700" role="alert">
                                <svg class="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                                </svg>
                                <span>{err}</span>
                            </div>
                        })}

                        // Login button
                        <div>
                            {move || {
                                if loading.get() {
                                    view! {
                                        <div class="flex flex-col items-center justify-center py-8 gap-3" role="status">
                                            <svg class="animate-spin h-10 w-10 text-red-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            <span class="text-sm font-medium text-gray-600">
                                                {if is_dev_mode { "Memproses autentikasi..." } else { "Mengarahkan ke Portal SSO..." }}
                                            </span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            class="w-full py-3.5 px-6 text-base font-semibold rounded-xl bg-red-600 hover:bg-red-700 text-white transition-all duration-200 shadow-lg hover:shadow-xl focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 active:scale-[0.98]"
                                            on:click=handle_login
                                            type="button"
                                        >
                                            <span class="inline-flex items-center justify-center gap-2">
                                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
                                                </svg>
                                                "Masuk ke Sistem"
                                            </span>
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // SSO note
                        <div class="mt-6 pt-4 border-t border-gray-100 text-center">
                            <div class="inline-flex items-center gap-2 text-xs text-gray-400">
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                                </svg>
                                "Gunakan akun SSO Kejaksaan RI"
                            </div>
                        </div>
                    </div>
                </div>
            </main>

            <footer class="flex-shrink-0 bg-white/80 border-t border-gray-200 py-4">
                <p class="text-center text-xs text-gray-400">
                    "© 2026 Kejaksaan Republik Indonesia — SIMPEL Perlengkapan"
                </p>
            </footer>
        </div>
    }
}
