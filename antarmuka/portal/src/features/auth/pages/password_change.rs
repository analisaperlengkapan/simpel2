//! Password Change Page
//!
//! Self-service password change for authenticated users.
//! REQ-PORTAL-006

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::use_api_client;
use crate::utils::authenc_api::ChangePasswordRequest;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Password strength level
#[derive(Clone, Copy, PartialEq)]
enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    fn label(&self) -> &'static str {
        match self {
            Self::Weak => "Lemah",
            Self::Fair => "Cukup",
            Self::Good => "Baik",
            Self::Strong => "Kuat",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            Self::Weak => "bg-red-500",
            Self::Fair => "bg-yellow-500",
            Self::Good => "bg-blue-500",
            Self::Strong => "bg-green-500",
        }
    }

    fn width(&self) -> &'static str {
        match self {
            Self::Weak => "w-1/4",
            Self::Fair => "w-2/4",
            Self::Good => "w-3/4",
            Self::Strong => "w-full",
        }
    }
}

/// Calculate password strength
fn calculate_strength(password: &str) -> PasswordStrength {
    let mut score = 0;
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_lowercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }

    match score {
        0..=2 => PasswordStrength::Weak,
        3 => PasswordStrength::Fair,
        4..=5 => PasswordStrength::Good,
        _ => PasswordStrength::Strong,
    }
}

/// Password change page
#[component]
pub fn PasswordChangePage() -> impl IntoView {
    let api = use_api_client();

    // Obtain the top-level user_session writer so we can update the reactive
    // signal after a successful password change.  Without this, route guards
    // that read the signal would still see `require_password_change = true`
    // and redirect the user back here in a loop.
    let set_user_session = use_context::<WriteSignal<Option<crate::features::auth::UserSession>>>();

    let (current_password, set_current_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (show_current, set_show_current) = signal(false);
    let (show_new, set_show_new) = signal(false);

    let navigate = leptos_router::hooks::use_navigate();

    // Track whether this component is still mounted so the timer callback
    // inside spawn_local can skip navigation after the user left the page.
    let mounted = Arc::new(AtomicBool::new(true));
    let mounted_cleanup = Arc::clone(&mounted);
    on_cleanup(move || {
        mounted_cleanup.store(false, Ordering::SeqCst);
    });

    let password_strength = Signal::derive(move || {
        let pw = new_password.get();
        if pw.is_empty() {
            None
        } else {
            Some(calculate_strength(&pw))
        }
    });

    let passwords_match = Signal::derive(move || {
        let np = new_password.get();
        let cp = confirm_password.get();
        np == cp
    });

    let can_submit = Signal::derive(move || {
        !current_password.get().is_empty()
            && new_password.get().len() >= 8
            && passwords_match.get()
            && !loading.get()
    });

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let api = api.clone();
        let nav = navigate.clone();
        let is_mounted = mounted.clone();
        set_loading.set(true);
        set_error.set(None);
        set_success.set(None);

        spawn_local(async move {
            let req = ChangePasswordRequest {
                current_password: current_password.get(),
                new_password: new_password.get(),
            };

            match api.change_password(&req).await {
                Ok(()) => {
                    set_success.set(Some(
                        "Kata sandi berhasil diubah. Mengalihkan ke dashboard...".to_string(),
                    ));
                    set_current_password.set(String::new());
                    set_new_password.set(String::new());
                    set_confirm_password.set(String::new());

                    // Clear the require_password_change flag from the local
                    // session so the user is no longer redirected back here.
                    // The backend already cleared the flag in the DB; a token
                    // refresh will eventually embed the updated claim, but we
                    // update localStorage immediately for a responsive UX.
                    if let Some(mut session) = crate::features::auth::AuthService::load_session() {
                        session.require_password_change = false;

                        // Immediately refresh the JWT so the new token embeds
                        // `require_password_change: false`.  Without this, the
                        // stale JWT still carries the old claim and backend
                        // endpoints like GET /me will return 403.
                        if let Some(ref refresh_token) = session.refresh_token {
                            match crate::features::auth::AuthService::refresh_token(refresh_token)
                                .await
                            {
                                Ok(token_response) => {
                                    crate::features::auth::AuthService::update_session_token(
                                        &token_response,
                                    );
                                    // Reload the session which now has the fresh JWT
                                    if let Some(refreshed) =
                                        crate::features::auth::AuthService::load_session()
                                    {
                                        session = refreshed;
                                        // Ensure the flag is cleared even if the
                                        // new JWT hasn't propagated the DB change
                                        // yet (edge case with replication lag).
                                        session.require_password_change = false;
                                    }
                                }
                                Err(e) => {
                                    // Token refresh failed — continue with the
                                    // local-only flag clear.  The next automatic
                                    // refresh cycle will pick up the new JWT.
                                    leptos::logging::warn!(
                                        "Token refresh after password change failed: {}",
                                        e
                                    );
                                }
                            }
                        }

                        crate::features::auth::AuthService::save_session(&session);

                        // Also update the reactive user_session signal so that
                        // route guards (WithAuth, WithAdminAuth, inline guards)
                        // see the updated flag immediately without waiting for
                        // a token refresh or page reload.
                        if let Some(setter) = set_user_session {
                            setter.set(Some(session.clone()));
                        }
                        crate::utils::app_state::app_state_login(session);
                    }

                    // Navigate to dashboard after a short delay so the user
                    // sees the success message before being redirected.
                    // Guard: if the component unmounted during the 1.5-second
                    // timer (e.g. user navigated away), skip the navigation.
                    gloo_timers::future::TimeoutFuture::new(1_500).await;
                    if is_mounted.load(Ordering::SeqCst) {
                        if let Some(target) =
                            crate::features::auth::AuthService::take_post_login_redirect()
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(&target);
                            }
                        } else {
                            nav("/dashboard", Default::default());
                        }
                    }
                }
                Err(e) => set_error.set(Some(format!("Gagal mengubah kata sandi: {}", e))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <MainLayout>
            <div class="max-w-lg mx-auto px-4 py-8">
                <h1 class="text-2xl font-bold text-gray-900 mb-2">"Ubah Kata Sandi"</h1>
                <p class="text-gray-600 mb-6">"Pastikan kata sandi baru Anda kuat dan unik."</p>

                {move || {
                    success
                        .get()
                        .map(|msg| {
                            view! {
                                <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">
                                    "✅ " {msg}
                                </div>
                            }
                        })
                }}
                {move || {
                    error
                        .get()
                        .map(|msg| {
                            view! {
                                <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">
                                    "❌ " {msg}
                                </div>
                            }
                        })
                }}

                <form
                    on:submit=handle_submit
                    class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 space-y-5"
                >
                    // Current password
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Kata Sandi Saat Ini"
                        </label>
                        <div class="relative">
                            <input
                                type=move || if show_current.get() { "text" } else { "password" }
                                prop:value=current_password
                                on:input=move |ev| set_current_password.set(event_target_value(&ev))
                                required=true
                                class="w-full px-3 py-2 pr-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                            />
                            <button
                                type="button"
                                on:click=move |_| set_show_current.set(!show_current.get())
                                class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                            >
                                {move || if show_current.get() { "🙈" } else { "👁️" }}
                            </button>
                        </div>
                    </div>

                    // New password
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Kata Sandi Baru"
                        </label>
                        <div class="relative">
                            <input
                                type=move || if show_new.get() { "text" } else { "password" }
                                prop:value=new_password
                                on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                required=true
                                minlength=8
                                class="w-full px-3 py-2 pr-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                            />
                            <button
                                type="button"
                                on:click=move |_| set_show_new.set(!show_new.get())
                                class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                            >
                                {move || if show_new.get() { "🙈" } else { "👁️" }}
                            </button>
                        </div>

                        // Password strength indicator
                        {move || {
                            password_strength
                                .get()
                                .map(|strength| {
                                    view! {
                                        <div class="mt-2">
                                            <div class="h-2 bg-gray-200 rounded-full overflow-hidden">
                                                <div class=format!(
                                                    "h-full {} {} transition-all duration-300",
                                                    strength.color(),
                                                    strength.width(),
                                                )></div>
                                            </div>
                                            <p class="text-xs text-gray-500 mt-1">
                                                "Kekuatan: " <strong>{strength.label()}</strong>
                                            </p>
                                        </div>
                                    }
                                })
                        }}

                        // Password requirements
                        <div class="mt-2 text-xs text-gray-500 space-y-1">
                            <p class=move || {
                                if new_password.get().len() >= 8 {
                                    "text-green-600"
                                } else {
                                    "text-gray-400"
                                }
                            }>
                                {move || if new_password.get().len() >= 8 { "✅" } else { "○" }}
                                " Minimal 8 karakter"
                            </p>
                            <p class=move || {
                                if new_password.get().chars().any(|c| c.is_uppercase()) {
                                    "text-green-600"
                                } else {
                                    "text-gray-400"
                                }
                            }>
                                {move || {
                                    if new_password.get().chars().any(|c| c.is_uppercase()) {
                                        "✅"
                                    } else {
                                        "○"
                                    }
                                }} " Huruf besar"
                            </p>
                            <p class=move || {
                                if new_password.get().chars().any(|c| c.is_ascii_digit()) {
                                    "text-green-600"
                                } else {
                                    "text-gray-400"
                                }
                            }>
                                {move || {
                                    if new_password.get().chars().any(|c| c.is_ascii_digit()) {
                                        "✅"
                                    } else {
                                        "○"
                                    }
                                }} " Angka"
                            </p>
                            <p class=move || {
                                if new_password.get().chars().any(|c| !c.is_alphanumeric()) {
                                    "text-green-600"
                                } else {
                                    "text-gray-400"
                                }
                            }>
                                {move || {
                                    if new_password.get().chars().any(|c| !c.is_alphanumeric()) {
                                        "✅"
                                    } else {
                                        "○"
                                    }
                                }} " Karakter khusus"
                            </p>
                        </div>
                    </div>

                    // Confirm password
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Konfirmasi Kata Sandi Baru"
                        </label>
                        <input
                            type="password"
                            prop:value=confirm_password
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            required=true
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                        />
                        {move || {
                            let cp = confirm_password.get();
                            if !cp.is_empty() && !passwords_match.get() {
                                Some(
                                    view! {
                                        <p class="text-red-500 text-xs mt-1">
                                            "Kata sandi tidak cocok"
                                        </p>
                                    },
                                )
                            } else {
                                None
                            }
                        }}
                    </div>

                    <div class="flex justify-end pt-2">
                        <button
                            type="submit"
                            disabled=move || !can_submit.get()
                            class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        >
                            {move || if loading.get() { "Menyimpan..." } else { "Ubah Kata Sandi" }}
                        </button>
                    </div>
                </form>
            </div>
        </MainLayout>
    }
}
