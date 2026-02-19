//! Login page component
//!
//! Authentication page for user login with CAPTCHA integration

use crate::components::layout::AuthLayout;
use crate::features::auth::{AuthService, LoginCredentials, LoginResult, UserSession};
use leptos::prelude::*;
use lib_ui::components::captcha::Captcha;
use wasm_bindgen_futures::spawn_local;
use web_sys;

/// Login page
#[component]
pub fn LoginPage(
    /// Callback when login succeeds
    on_login_success: WriteSignal<Option<UserSession>>,
) -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (captcha_token, setcaptcha_token) = signal(None::<String>);
    let (_show_captcha, _set_show_captcha) = signal(true); // Always show CAPTCHA from start
    let (failed_attempts, set_failed_attempts) = signal(0u32);

    let navigate = leptos_router::hooks::use_navigate();

    // Handle CAPTCHA completion
    let handle_captcha_success = move |token: String| {
        setcaptcha_token.set(Some(token));
        set_error_message.set(String::new());
    };

    let handle_captcha_failure = move |error: String| {
        set_error_message.set(error);
        setcaptcha_token.set(None);
    };

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        // CAPTCHA is always required
        if captcha_token.get().is_none() {
            set_error_message.set("Silakan selesaikan verifikasi keamanan terlebih dahulu.".to_string());
            return;
        }

        set_is_loading.set(true);
        set_error_message.set(String::new());

        let credentials = LoginCredentials {
            username: username.get(),
            password: password.get(),
            captcha_token: captcha_token.get(),
        };

        // Resolve redirect target here (synchronous, in reactive context) before entering async
        let redirect_to_perlengkapan = {
            let query_map = leptos_router::hooks::use_query_map();
            query_map.with(|params| params.get("redirect").map(|s| s.to_string()))
                == Some("perlengkapan".to_string())
        };

        let nav = navigate.clone();

        spawn_local(async move {
            match AuthService::login(credentials).await {
                LoginResult::Success(session) => {
                    // Reset failed attempts on successful login
                    set_failed_attempts.set(0);

                    // Full authentication complete
                    AuthService::save_session(&session);
                    on_login_success.set(Some(*session));

                    // Hide loading spinner before navigating
                    set_is_loading.set(false);

                    if redirect_to_perlengkapan {
                        // Hard redirect to Perlengkapan root
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/");
                        }
                        return;
                    }

                    nav("/dashboard", Default::default());
                }
                LoginResult::MfaSetupRequired(temp_token) => {
                    // Reset failed attempts
                    set_failed_attempts.set(0);

                    // Store temp token for MFA setup
                    AuthService::save_temp_token(&temp_token);

                    // Hide loading spinner before navigating
                    set_is_loading.set(false);

                    // Redirect to MFA setup
                    nav("/mfa/setup", Default::default());
                }
                LoginResult::MfaVerificationRequired(temp_token) => {
                    // Reset failed attempts
                    set_failed_attempts.set(0);

                    // Store temp token for MFA verification
                    AuthService::save_temp_token(&temp_token);

                    // Hide loading spinner before navigating
                    set_is_loading.set(false);

                    // Redirect to MFA verification
                    nav("/mfa/verify", Default::default());
                }
                LoginResult::Error(msg) => {
                    // Increment failed attempts
                    let new_attempts = failed_attempts.get() + 1;
                    set_failed_attempts.set(new_attempts);

                    set_error_message.set(msg);
                    set_is_loading.set(false);
                    setcaptcha_token.set(None); // Reset CAPTCHA on failure
                }
            }
        });
    };

    view! {
        <AuthLayout>
            <div class="max-w-md mx-auto">
                // Logo dan Header
                <div class="text-center mb-6">
                    <div class="inline-flex items-center justify-center w-16 h-16 bg-gradient-to-br from-red-600 to-red-700 rounded-full shadow-lg mb-3">
                        <span class="text-3xl">"⚖️"</span>
                    </div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        "Portal SIMPelv2"
                    </h1>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Kejaksaan Republik Indonesia"
                    </p>
                </div>

                // Login Card
                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-700 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-center text-gray-900 dark:text-white mb-6">
                        "Masuk ke Sistem"
                    </h2>

                    <form on:submit=handle_submit class="space-y-4">
                        // Username
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                                "Username"
                            </label>
                            <input
                                type="text"
                                id="username"
                                name="username"
                                class="w-full px-4 py-2.5 text-sm border border-gray-300 dark:border-gray-600 rounded-xl focus:ring-2 focus:ring-red-500 focus:border-red-500 dark:bg-gray-700 dark:text-white transition-colors"
                                placeholder="Masukkan username"
                                prop:value=move || username.get()
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                required
                                disabled=move || is_loading.get()
                            />
                        </div>

                        // Password
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                                "Password"
                            </label>
                            <input
                                type="password"
                                id="password"
                                name="password"
                                class="w-full px-4 py-2.5 text-sm border border-gray-300 dark:border-gray-600 rounded-xl focus:ring-2 focus:ring-red-500 focus:border-red-500 dark:bg-gray-700 dark:text-white transition-colors"
                                placeholder="Masukkan password"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                                disabled=move || is_loading.get()
                            />
                        </div>

                        // CAPTCHA — compact wrapper
                        <div class="bg-gray-50 dark:bg-gray-700/50 rounded-xl p-3 border border-gray-200 dark:border-gray-600">
                            <Captcha
                                on_success=Callback::new(handle_captcha_success)
                                on_failure=Callback::new(handle_captcha_failure)
                                difficulty=3u8
                                accessibility_enabled=true
                                behavioral_analysis=true
                                class="captcha-login"
                            />
                        </div>

                        // Error
                        {move || (!error_message.get().is_empty()).then(|| view! {
                            <div class="flex items-start gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl text-sm text-red-700 dark:text-red-400">
                                <svg class="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                </svg>
                                <span>{error_message.get()}</span>
                            </div>
                        })}

                        // Submit Button
                        <button
                            type="submit"
                            class=move || format!(
                                "w-full py-3 px-6 text-sm font-semibold rounded-xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed {}",
                                if captcha_token.get().is_none() {
                                    "bg-gray-300 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed"
                                } else {
                                    "bg-red-600 hover:bg-red-700 text-white shadow-lg hover:shadow-xl focus:ring-red-500"
                                }
                            )
                            disabled=move || is_loading.get() || captcha_token.get().is_none()
                        >
                            <Show
                                when=move || is_loading.get()
                                fallback=move || {
                                    if captcha_token.get().is_none() {
                                        view! { "Selesaikan Verifikasi Keamanan" }
                                    } else {
                                        view! { "Masuk ke Portal" }
                                    }
                                }
                            >
                                <span class="inline-flex items-center gap-2">
                                    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Memverifikasi..."
                                </span>
                            </Show>
                        </button>

                        // Forgot Password
                        <div class="text-center pt-2">
                            <a href="/portal/password-reset" class="text-sm text-red-600 hover:text-red-500 dark:text-red-400 dark:hover:text-red-300 font-medium transition-colors">
                                "Lupa Password?"
                            </a>
                        </div>
                    </form>
                </div>
            </div>
        </AuthLayout>
    }
}
