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
use shared_microfrontend::prelude::*;
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
            .set_item("jwt_token", token)
            .map_err(|e| format!("Failed to store token: {:?}", e))
    }

    /// Check if user is authenticated
    #[allow(dead_code)]
    pub fn is_authenticated() -> bool {
        if let Some(window) = window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            return storage.get_item("jwt_token").ok().flatten().is_some();
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
        <div class="min-h-screen bg-gradient-to-br from-blue-900 via-blue-800 to-indigo-900 flex items-center justify-center p-4 relative overflow-hidden">
            // Animated background elements
            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                <div class="absolute w-96 h-96 bg-blue-400 rounded-full opacity-10 blur-3xl -top-20 -left-20 animate-pulse"></div>
                <div class="absolute w-96 h-96 bg-indigo-400 rounded-full opacity-10 blur-3xl -bottom-20 -right-20 animate-pulse" style="animation-delay: 2s"></div>
            </div>

            // Main card container dengan animasi entrance
            <div class="relative w-full max-w-md p-8 text-center bg-white/95 backdrop-blur-sm rounded-3xl shadow-2xl transform transition-all duration-500 hover:shadow-3xl hover:scale-[1.02]">

                // Header menggunakan Logo dari shared-microfrontend
                <div class="mb-8">
                    <div class="flex justify-center mb-6 animate-bounce-slow">
                        <div class="p-4 bg-gradient-to-br from-emerald-50 to-blue-50 rounded-2xl shadow-lg">
                            <Logo
                                show_text=false
                                size="large"
                            />
                        </div>
                    </div>

                    <h1 class="text-4xl font-bold bg-gradient-to-r from-emerald-700 to-blue-700 bg-clip-text text-transparent mb-3 tracking-tight" id="app-title">
                        "SIMPelv2"
                    </h1>

                    <div class="inline-block px-4 py-1 bg-gradient-to-r from-emerald-100 to-blue-100 rounded-full mb-3">
                        <p class="text-sm font-bold text-emerald-800">
                            "KEJAKSAAN REPUBLIK INDONESIA"
                        </p>
                    </div>

                    <div class="flex items-center justify-center gap-2 text-gray-600">
                        <i class="fas fa-boxes text-emerald-600"></i>
                        <p class="text-sm font-medium">
                            "Manajemen Aset & BMN"
                        </p>
                    </div>
                </div>

                // Divider dengan gradient
                <div class="relative my-8">
                    <div class="absolute inset-0 flex items-center">
                        <div class="w-full border-t border-gray-200"></div>
                    </div>
                    <div class="relative flex justify-center">
                        <span class="px-4 text-xs text-gray-500 bg-white">AKSES SISTEM</span>
                    </div>
                </div>

                // Login Description dengan icon
                <div class="mb-6 p-4 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl border border-blue-100">
                    <div class="flex items-start gap-3">
                        <i class="fas fa-info-circle text-blue-600 mt-1"></i>
                        <p class="text-sm text-gray-700 text-left leading-relaxed">
                            {move || if is_dev_mode {
                                "Mode Development: Klik tombol di bawah untuk simulasi login dan akses dashboard perlengkapan"
                            } else {
                                "Untuk mengakses sistem perlengkapan, Anda akan diarahkan ke portal login utama Kejaksaan RI dengan Single Sign-On (SSO)"
                            }}
                        </p>
                    </div>
                </div>

                // Error Message dengan animasi
                {move || error.get().map(|err| view! {
                    <div
                        class="mb-4 p-4 bg-red-50 border-l-4 border-red-500 rounded-r-lg text-sm text-red-700 animate-shake"
                        role="alert"
                        aria-live="polite"
                    >
                        <div class="flex items-center">
                            <i class="fas fa-exclamation-triangle text-red-500 mr-3 text-lg"></i>
                            <div>
                                <p class="font-semibold">"Gagal Autentikasi"</p>
                                <p class="mt-1">{err}</p>
                            </div>
                        </div>
                    </div>
                })}

                // Login Button dengan Loading State dan animasi
                <div class="space-y-4">
                    {move || {
                        if loading.get() {
                            view! {
                                <div
                                    class="flex flex-col items-center justify-center py-6 gap-3"
                                    role="status"
                                    aria-live="polite"
                                >
                                    <div class="relative">
                                        <div class="animate-spin rounded-full h-12 w-12 border-4 border-blue-200"></div>
                                        <div class="absolute inset-0 animate-spin rounded-full h-12 w-12 border-4 border-transparent border-t-blue-600"></div>
                                    </div>
                                    <span class="text-blue-700 font-medium animate-pulse">
                                        {if is_dev_mode { "🔐 Memproses autentikasi..." } else { "🔄 Mengarahkan ke Portal SSO..." }}
                                    </span>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="group relative w-full py-4 px-6 text-lg font-bold rounded-xl bg-gradient-to-r from-emerald-600 to-blue-600 text-white overflow-hidden transition-all duration-300 hover:shadow-2xl hover:shadow-blue-500/50 focus:outline-none focus:ring-4 focus:ring-blue-300 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
                                    on:click=handle_login
                                    aria-label="Masuk ke sistem perlengkapan"
                                    aria-describedby="app-title"
                                    disabled=false
                                    type="button"
                                >
                                    // Animated shine effect
                                    <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent transform -skew-x-12 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000"></div>

                                    <span class="relative flex items-center justify-center gap-2">
                                        <i class="fas fa-sign-in-alt" aria-hidden="true"></i>
                                        "Masuk ke Sistem"
                                        <i class="fas fa-arrow-right group-hover:translate-x-1 transition-transform" aria-hidden="true"></i>
                                    </span>
                                </button>
                            }.into_any()
                        }
                    }}
                </div>

                // Footer Info dengan badges
                <div class="mt-8 pt-6 border-t border-gray-100 space-y-3">
                    <div class="flex items-center justify-center gap-2 text-gray-600">
                        <i class="fas fa-shield-alt text-emerald-600" aria-hidden="true"></i>
                        <p class="text-xs font-medium">
                            "Gunakan akun SSO Kejaksaan RI Anda"
                        </p>
                    </div>

                    {move || if is_dev_mode {
                        view! {
                            <div class="inline-flex items-center gap-2 px-3 py-1.5 bg-amber-100 border border-amber-300 rounded-full">
                                <div class="w-2 h-2 bg-amber-500 rounded-full animate-pulse"></div>
                                <p class="text-xs font-semibold text-amber-700">
                                    <i class="fas fa-code mr-1" aria-hidden="true"></i>
                                    "Development Mode"
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        let _: () = view! { <></> };
                        ().into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
