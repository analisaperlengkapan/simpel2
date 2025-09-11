//! # SIMPelv2 Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPelv2 dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

// Import local components directly
use crate::components::footer::Footer;
use crate::components::header::Header;

/// Authentication state
#[derive(Clone, Debug, PartialEq)]
pub enum AuthState {
    /// User is not authenticated and needs to log in
    LoggedOut,
    /// User is authenticated and can access the system
    LoggedIn,
}

/// User session data
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserSession {
    /// User's login username
    pub username: String,
    /// User's role in the system (e.g., admin, user, etc.)
    pub role: String,
    /// User's display name
    pub name: String,
}

/// Komponen halaman login
#[component]
pub fn LoginPage(
    /// Signal setter for authentication state
    set_auth_state: WriteSignal<AuthState>,
    /// Signal setter for user session
    set_user_session: WriteSignal<Option<UserSession>>
) -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (success_message, set_success_message) = signal(String::new());

    let login_action = Action::new(move |_: &()| {
        let username_val = username.get();
        let password_val = password.get();
        let set_auth_state_clone = set_auth_state;
        let set_user_session_clone = set_user_session;

        async move {
            // Clear previous error message
            set_error_message.set(String::new());

            // Validate input
            if username_val.trim().is_empty() {
                set_error_message.set("Username tidak boleh kosong".to_string());
                return;
            }

            if password_val.is_empty() {
                set_error_message.set("Password tidak boleh kosong".to_string());
                return;
            }

            // Simple authentication logic (in production, this would call an API)
            // For demo purposes, accept any non-empty credentials
            if !username_val.trim().is_empty() && !password_val.is_empty() {
                set_success_message.set("Login berhasil! Mengalihkan ke dashboard...".to_string());

                // Create user session
                let session = UserSession {
                    username: username_val.clone(),
                    role: "admin".to_string(),
                    name: "Administrator".to_string(),
                };

                // Set authentication state and user session
                set_auth_state_clone.set(AuthState::LoggedIn);
                set_user_session_clone.set(Some(session));

                leptos::logging::log!("✅ Login berhasil untuk user: {}", username_val);
            } else {
                set_error_message.set("Username dan password harus diisi dengan benar".to_string());
            }
        }
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-red-50 via-white to-blue-50 flex items-center justify-center p-4">
            <div class="max-w-md w-full">
                // Logo dan Header Kejaksaan
                <div class="flex items-center justify-center mb-8">
                    <div class="text-center">
                        <div class="text-4xl mb-4">"⚖"</div>
                        <h1 class="text-3xl font-bold text-red-600 mb-2">"Kejaksaan Agung"</h1>
                        <h2 class="text-xl text-gray-600">"Republik Indonesia"</h2>
                        <p class="text-sm text-gray-500 mt-2">"Sistem Pembinaan SIMPelv2"</p>
                    </div>
                </div>

                // Login Card
                <div class="bg-white rounded-xl shadow-2xl border border-gray-100 overflow-hidden">
                    // Header Card
                    <div class="bg-gradient-to-r from-red-600 to-red-700 px-6 py-4">
                        <h4 class="text-white text-lg font-semibold text-center">
                            "🔐 Masuk ke Sistem"
                        </h4>
                    </div>

                    // Form
                    <div class="p-6">
                        <form
                            on:submit=move |ev| {
                                ev.prevent_default();
                                login_action.dispatch(());
                            }
                            class="space-y-6"
                        >
                            <div>
                                <label for="username" class="block text-sm font-semibold text-gray-700 mb-2">
                                    <i class="fas fa-user mr-2 text-red-600"></i>
                                    "Username"
                                </label>
                                <input
                                    type="text"
                                    id="username"
                                    value=move || username.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    class="w-full px-4 py-3 border-2 border-gray-200 rounded-lg focus:outline-none focus:border-red-500 focus:ring-2 focus:ring-red-200 transition-all duration-200"
                                    placeholder="Masukkan username Anda"
                                    required
                                />
                            </div>

                            <div>
                                <label for="password" class="block text-sm font-semibold text-gray-700 mb-2">
                                    <i class="fas fa-lock mr-2 text-red-600"></i>
                                    "Password"
                                </label>
                                <input
                                    type="password"
                                    id="password"
                                    value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    class="w-full px-4 py-3 border-2 border-gray-200 rounded-lg focus:outline-none focus:border-red-500 focus:ring-2 focus:ring-red-200 transition-all duration-200"
                                    placeholder="Masukkan password Anda"
                                    required
                                />
                            </div>

                            // Error Message
                            {move || (!error_message.get().is_empty()).then(|| view! {
                                <div class="bg-red-50 border-l-4 border-red-500 text-red-700 px-4 py-3 rounded-r-lg">
                                    <div class="flex items-center">
                                        <i class="fas fa-exclamation-triangle mr-2 text-red-500"></i>
                                        <span class="font-medium">{error_message.get()}</span>
                                    </div>
                                </div>
                            })}

                            // Success Message
                            {move || (!success_message.get().is_empty()).then(|| view! {
                                <div class="bg-green-50 border-l-4 border-green-500 text-green-700 px-4 py-3 rounded-r-lg">
                                    <div class="flex items-center">
                                        <i class="fas fa-check-circle mr-2 text-green-500"></i>
                                        <span class="font-medium">{success_message.get()}</span>
                                    </div>
                                </div>
                            })}

                            // Loading Message
                            {move || (login_action.pending().get() && error_message.get().is_empty() && success_message.get().is_empty()).then(|| view! {
                                <div class="bg-blue-50 border-l-4 border-blue-500 text-blue-700 px-4 py-3 rounded-r-lg">
                                    <div class="flex items-center">
                                        <i class="fas fa-spinner fa-spin mr-2 text-blue-500"></i>
                                        <span class="font-medium">"Memverifikasi kredensial..."</span>
                                    </div>
                                </div>
                            })}

                            // Login Button
                            <button
                                type="submit"
                                disabled=(move || login_action.pending().get())()
                                class="w-full bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white font-bold py-3 px-4 rounded-lg transition-all duration-200 transform hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none shadow-lg"
                            >
                                {move || if login_action.pending().get() {
                                    view! {
                                        <div class="flex items-center justify-center">
                                            <i class="fas fa-spinner fa-spin mr-2"></i>
                                            <span>"Sedang Masuk..."</span>
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div class="flex items-center justify-center">
                                            <i class="fas fa-sign-in-alt mr-2"></i>
                                            <span>"Masuk ke Sistem"</span>
                                        </div>
                                    }
                                }}
                            </button>
                        </form>
                    </div>
                </div>

                // Demo Info
                <div class="mt-6 text-center">
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
                        <p class="text-sm text-blue-800 font-medium">
                            <i class="fas fa-info-circle mr-1"></i>
                            "Untuk demo, masukkan username dan password apa saja"
                        </p>
                    </div>
                </div>

                // Footer
                <div class="mt-8 text-center text-xs text-gray-400">
                    <p>"© 2025 Kejaksaan Agung Republik Indonesia"</p>
                    <p class="mt-1">"SIMPelv2 v2.0 - Sistem Terintegrasi"</p>
                </div>
            </div>
        </div>
    }
}

/// Komponen halaman dashboard utama
#[component]
pub fn DashboardPage(
    /// User session data
    user_session: UserSession
) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gradient-to-br from-red-50 via-white to-blue-50">
            // Header dengan tema Kejaksaan
            <header class="bg-gradient-to-r from-red-600 to-red-700 shadow-lg">
                <div class="container mx-auto px-4 py-4">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-4">
                            <div class="bg-white p-2 rounded-lg shadow-md">
                                <div class="text-2xl">"⚖"</div>
                            </div>
                            <div>
                                <h1 class="text-white text-xl font-bold">"Kejaksaan Agung RI"</h1>
                                <p class="text-red-100 text-sm">"Sistem Pembinaan SIMPelv2"</p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-4">
                            <span class="text-white text-sm">"Selamat datang, "{user_session.name}</span>
                            <button
                                class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
                                on:click=move |_| {
                                    // Logout functionality will be handled by parent component
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().reload();
                                    }
                                }
                            >
                                <i class="fas fa-sign-out-alt mr-2"></i>
                                "Logout"
                            </button>
                        </div>
                    </div>
                </div>
            </header>

            <main class="container mx-auto px-4 py-8">
                // Welcome Section
                <div class="bg-white rounded-xl shadow-lg p-8 mb-8 border-l-4 border-red-600">
                    <div class="flex items-center justify-between">
                        <div>
                            <h1 class="text-3xl font-bold text-gray-900 mb-2">
                                "Selamat Datang di Portal SIMPelv2"
                            </h1>
                            <p class="text-gray-600">"Sistem Pembinaan Terintegrasi Kejaksaan"</p>
                        </div>
                        <div class="text-right">
                            <div class="text-4xl">"🏛"</div>
                        </div>
                    </div>
                </div>

                // Quick Stats
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-green-500">
                        <div class="flex items-center">
                            <div class="bg-green-100 p-3 rounded-lg mr-4">
                                <i class="fas fa-server text-green-600 text-xl"></i>
                            </div>
                            <div>
                                <p class="text-2xl font-bold text-gray-900">"9"</p>
                                <p class="text-gray-600 text-sm">"Total Sistem"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-blue-500">
                        <div class="flex items-center">
                            <div class="bg-blue-100 p-3 rounded-lg mr-4">
                                <i class="fas fa-users text-blue-600 text-xl"></i>
                            </div>
                            <div>
                                <p class="text-2xl font-bold text-gray-900">"1,234"</p>
                                <p class="text-gray-600 text-sm">"Pengguna Aktif"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-yellow-500">
                        <div class="flex items-center">
                            <div class="bg-yellow-100 p-3 rounded-lg mr-4">
                                <i class="fas fa-clock text-yellow-600 text-xl"></i>
                            </div>
                            <div>
                                <p class="text-2xl font-bold text-gray-900">"99.9%"</p>
                                <p class="text-gray-600 text-sm">"Uptime Sistem"</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-red-500">
                        <div class="flex items-center">
                            <div class="bg-red-100 p-3 rounded-lg mr-4">
                                <i class="fas fa-shield-alt text-red-600 text-xl"></i>
                            </div>
                            <div>
                                <p class="text-2xl font-bold text-gray-900">"A+"</p>
                                <p class="text-gray-600 text-sm">"Tingkat Keamanan"</p>
                            </div>
                        </div>
                    </div>
                </div>

                // Main Content
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div class="bg-white rounded-xl shadow-lg p-6 border-l-4 border-blue-600">
                        <h3 class="text-xl font-bold text-gray-900 mb-4 flex items-center">
                            <i class="fas fa-cogs text-blue-600 mr-2"></i>
                            "Sistem Utama"
                        </h3>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <span class="text-gray-700">"Portal SIMPelv2"</span>
                                <span class="bg-green-100 text-green-800 text-xs font-semibold px-2 py-1 rounded-full">"Online"</span>
                            </div>
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <span class="text-gray-700">"AI Service"</span>
                                <span class="bg-green-100 text-green-800 text-xs font-semibold px-2 py-1 rounded-full">"Online"</span>
                            </div>
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <span class="text-gray-700">"Security Service"</span>
                                <span class="bg-green-100 text-green-800 text-xs font-semibold px-2 py-1 rounded-full">"Online"</span>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-xl shadow-lg p-6 border-l-4 border-green-600">
                        <h3 class="text-xl font-bold text-gray-900 mb-4 flex items-center">
                            <i class="fas fa-chart-line text-green-600 mr-2"></i>
                            "Aktivitas Terbaru"
                        </h3>
                        <div class="space-y-3">
                            <div class="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
                                <div class="bg-blue-100 p-2 rounded-lg">
                                    <i class="fas fa-user-plus text-blue-600"></i>
                                </div>
                                <div class="flex-1">
                                    <p class="text-sm font-medium text-gray-900">"User baru terdaftar"</p>
                                    <p class="text-xs text-gray-500">"2 menit yang lalu"</p>
                                </div>
                            </div>
                            <div class="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
                                <div class="bg-green-100 p-2 rounded-lg">
                                    <i class="fas fa-check-circle text-green-600"></i>
                                </div>
                                <div class="flex-1">
                                    <p class="text-sm font-medium text-gray-900">"Backup sistem berhasil"</p>
                                    <p class="text-xs text-gray-500">"1 jam yang lalu"</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </main>

            // Footer
            <footer class="bg-gray-800 text-white mt-12">
                <div class="container mx-auto px-4 py-6 text-center">
                    <p class="text-gray-400 text-sm">
                        "© 2025 Kejaksaan Agung Republik Indonesia - Sistem Pembinaan SIMPelv2"
                    </p>
                </div>
            </footer>
        </div>
    }
}

/// Komponen utama aplikasi Portal SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    // Authentication state
    let (auth_state, set_auth_state) = signal(AuthState::LoggedOut);
    let (user_session, set_user_session) = signal(None::<UserSession>);

    view! {
        <Html attr:lang="id"/>
        <Title text="Portal SIMPelv2 - Kejaksaan Agung RI"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Portal Sistem Informasi Manajemen Pengelolaan Barang Milik Negara Kejaksaan Agung Republik Indonesia"/>
        <Meta name="keywords" content="kejaksaan, simpel, bmn, indonesia, government"/>
        <Meta property="og:title" content="Portal SIMPelv2"/>
        <Meta property="og:description" content="Gateway to Justice Technology - Kejaksaan Agung RI"/>

        <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            <Header />
            {move || match auth_state.get() {
                AuthState::LoggedOut => view! {
                    <LoginPage set_auth_state=set_auth_state set_user_session=set_user_session />
                }.into_any(),
                AuthState::LoggedIn => view! {
                    <DashboardPage user_session=user_session.get().unwrap_or_default() />
                }.into_any(),
            }}
            <Footer />
        </div>
    }
}
