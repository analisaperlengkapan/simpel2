//! Login page component
//!
//! Authentication page with WebAuthn/Passkey as PRIMARY login method (MANDATORY),
//! plus traditional username+password with CAPTCHA as fallback.

use crate::components::layout::AuthLayout;
use crate::features::auth::{AuthService, LoginCredentials, LoginResult, UserSession};
use crate::utils::app_state::use_api_client;
use crate::utils::webauthn;
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

    // WebAuthn / Passkey state
    let (passkey_loading, set_passkey_loading) = signal(false);
    let (show_password_form, set_show_password_form) = signal(false);
    let webauthn_supported = webauthn::is_webauthn_supported();

    let navigate = leptos_router::hooks::use_navigate();
    let api = use_api_client();

    // ── Passkey authentication trigger ────────────────────────────────────
    let (passkey_trigger, set_passkey_trigger) = signal(0u32);

    {
        let api = api.clone();
        let nav = navigate.clone();
        Effect::new(move || {
            let count = passkey_trigger.get();
            if count == 0 {
                return;
            }
            let api = api.clone();
            let nav = nav.clone();
            set_passkey_loading.set(true);
            set_error_message.set(String::new());

            spawn_local(async move {
                match api.webauthn_authenticate_start().await {
                    Ok(start_resp) => {
                        match webauthn::get_credential(&start_resp.challenge).await {
                            Ok(assertion) => {
                                match api
                                    .webauthn_authenticate_finish(
                                        &start_resp.session_id,
                                        &assertion,
                                    )
                                    .await
                                {
                                    Ok(token_resp) => {
                                        // Store tokens
                                        AuthService::save_token(&token_resp.access_token);
                                        if let Some(refresh) = &token_resp.refresh_token {
                                            AuthService::save_refresh_token(refresh);
                                        }

                                        // Decode JWT and create session
                                        match AuthService::decode_jwt_claims(
                                            &token_resp.access_token,
                                        ) {
                                            Ok(session) => {
                                                AuthService::save_session(&session);
                                                on_login_success.set(Some(session));
                                                set_passkey_loading.set(false);
                                                nav("/dashboard", Default::default());
                                            }
                                            Err(e) => {
                                                set_error_message
                                                    .set(format!("Token tidak valid: {}", e));
                                                set_passkey_loading.set(false);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        leptos::logging::warn!("Passkey finish error: {}", e);
                                        set_error_message
                                            .set("Autentikasi passkey gagal. Silakan coba lagi atau gunakan password.".to_string());
                                        set_passkey_loading.set(false);
                                    }
                                }
                            }
                            Err(e) => {
                                let msg = {
                                    let err_str = format!("{}", e);
                                    if err_str.contains("NotAllowedError")
                                        || err_str.contains("cancelled")
                                    {
                                        "Autentikasi passkey dibatalkan oleh pengguna.".to_string()
                                    } else if err_str.contains("SecurityError")
                                        || err_str.contains("invalid domain")
                                    {
                                        "Passkey tidak dapat digunakan pada domain ini. Hubungi administrator.".to_string()
                                    } else if err_str.contains("NotSupportedError") {
                                        "Browser Anda tidak mendukung passkey. Silakan gunakan password.".to_string()
                                    } else {
                                        "Autentikasi passkey gagal. Silakan coba lagi atau gunakan password.".to_string()
                                    }
                                };
                                leptos::logging::warn!("Passkey error: {}", e);
                                set_error_message.set(msg);
                                set_passkey_loading.set(false);
                            }
                        }
                    }
                    Err(e) => {
                        leptos::logging::warn!("Passkey start error: {}", e);
                        set_error_message.set(
                            "Gagal memulai autentikasi passkey. Silakan coba lagi.".to_string(),
                        );
                        set_passkey_loading.set(false);
                    }
                }
            });
        });
    }

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
            set_error_message
                .set("Silakan selesaikan verifikasi keamanan terlebih dahulu.".to_string());
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
                LoginResult::PasswordChangeRequired(session) => {
                    // Login succeeded but user must change password first
                    set_failed_attempts.set(0);
                    AuthService::save_session(&session);
                    on_login_success.set(Some(*session));
                    set_is_loading.set(false);

                    // Redirect to password change page
                    nav("/password", Default::default());
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
                    <div class="relative w-24 h-24 sm:w-28 sm:h-28 mx-auto mb-4 animate-float">
                        <div class="absolute inset-0 bg-gold-400/20 blur-2xl rounded-full"></div>
                        <img
                            src="/portal/assets/kejaksaan-logo.png"
                            alt="Logo Kejaksaan RI"
                            class="relative w-full h-full object-contain drop-shadow-xl"
                        />
                    </div>
                    <h1 class="text-2xl font-bold text-white">
                        "Masuk ke " <span class="text-gold-400">"SIMPEL"</span>
                    </h1>
                    <p class="text-sm text-slate-400">
                        "Kejaksaan Agung Republik Indonesia"
                    </p>
                </div>

                // Login Card
                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-700 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-center text-gray-900 dark:text-white mb-6">
                        "Masuk ke Sistem"
                    </h2>

                    // ═══ PRIMARY: Passkey / WebAuthn ═══
                    <Show when=move || webauthn_supported>
                        <div class="mb-6">
                            <button
                                type="button"
                                on:click=move |_| set_passkey_trigger.set(passkey_trigger.get() + 1)
                                class="w-full flex items-center justify-center gap-3 py-3 px-6 text-sm font-semibold rounded-xl bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 text-white shadow-lg hover:shadow-xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                disabled=move || passkey_loading.get() || is_loading.get()
                            >
                                <Show
                                    when=move || passkey_loading.get()
                                    fallback=|| view! {
                                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                                        </svg>
                                        "Masuk dengan Passkey"
                                    }
                                >
                                    <svg class="animate-spin h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Memverifikasi Passkey..."
                                </Show>
                            </button>
                            <p class="text-xs text-center text-gray-400 dark:text-gray-500 mt-2">
                                "Gunakan sidik jari, wajah, atau kunci keamanan"
                            </p>
                        </div>

                        // Divider and Toggle
                        <div class="relative my-6">
                            <div class="absolute inset-0 flex items-center">
                                <div class="w-full border-t border-gray-200 dark:border-gray-700"></div>
                            </div>
                            <div class="relative flex justify-center">
                                <button
                                    type="button"
                                    on:click=move |_| set_show_password_form.set(!show_password_form.get())
                                    class="bg-white dark:bg-gray-800 px-4 py-1.5 text-sm font-medium text-gold-600 dark:text-gold-400 border border-gold-300 dark:border-gold-600/50 rounded-full shadow-sm hover:bg-gold-50 dark:hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-gold-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800"
                                >
                                    {move || if show_password_form.get() {
                                        "Tutup login password"
                                    } else {
                                        "Atau masuk dengan password"
                                    }}
                                </button>
                            </div>
                        </div>
                    </Show>

                    // ═══ SECONDARY: Username + Password + CAPTCHA ═══
                    <div class=move || {
                        if !webauthn_supported || show_password_form.get() {
                            "space-y-4"
                        } else {
                            "hidden"
                        }
                    }>
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
                                class="w-full px-4 py-2.5 text-sm border border-slate-200 dark:border-navy-600 rounded-xl focus:ring-2 focus:ring-navy-500 focus:border-navy-500 dark:focus:ring-gold-500 dark:focus:border-gold-500 bg-white dark:bg-navy-700 dark:text-white transition-colors"
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

                        // Submit Button
                        <button
                            type="submit"
                            class=move || format!(
                                "w-full py-3 px-6 text-sm font-semibold rounded-xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed {}",
                                if captcha_token.get().is_none() {
                                    "bg-slate-300 dark:bg-slate-600 text-slate-500 dark:text-slate-400 cursor-not-allowed"
                                } else {
                                    "bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-600 hover:to-gold-700 text-navy-900 shadow-lg hover:shadow-xl focus:ring-gold-500"
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

                        // Info: Admin-only password reset
                        <div class="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-xl">
                            <p class="text-xs text-blue-700 dark:text-blue-400 text-center">
                                "Lupa password? Hubungi administrator unit kerja Anda."
                            </p>
                        </div>
                    </form>
                    </div> // end password form wrapper

                    // ═══ Error messages (shared between passkey and password) ═══
                    {move || (!error_message.get().is_empty()).then(|| view! {
                        <div class="mt-4 flex items-start gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl text-sm text-red-700 dark:text-red-400">
                            <svg class="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                            </svg>
                            <span>{error_message.get()}</span>
                        </div>
                    })}
                </div>
            </div>
        </AuthLayout>
    }
}
