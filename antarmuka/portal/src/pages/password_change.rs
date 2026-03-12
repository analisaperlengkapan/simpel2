//! Password Change Page
//!
//! Self-service password change for authenticated users.
//! REQ-PORTAL-006

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::ChangePasswordRequest;
use leptos::prelude::*;
use leptos::task::spawn_local;

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
    let state = use_app_state();
    let api = use_api_client();

    let (current_password, set_current_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (show_current, set_show_current) = signal(false);
    let (show_new, set_show_new) = signal(false);

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
                    set_success.set(Some("Kata sandi berhasil diubah".to_string()));
                    set_current_password.set(String::new());
                    set_new_password.set(String::new());
                    set_confirm_password.set(String::new());
                }
                Err(e) => set_error.set(Some(format!("Gagal mengubah kata sandi: {}", e))),
            }
            set_loading.set(false);
        });
    };

    let on_logout = {
        let state = state;
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-lg mx-auto px-4 py-8">
                <h1 class="text-2xl font-bold text-gray-900 mb-2">"Ubah Kata Sandi"</h1>
                <p class="text-gray-600 mb-6">"Pastikan kata sandi baru Anda kuat dan unik."</p>

                {move || success.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">
                        "✅ " {msg}
                    </div>
                })}
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">
                        "❌ " {msg}
                    </div>
                })}

                <form on:submit=handle_submit class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 space-y-5">
                    // Current password
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Kata Sandi Saat Ini"</label>
                        <div class="relative">
                            <input
                                type={move || if show_current.get() { "text" } else { "password" }}
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
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Kata Sandi Baru"</label>
                        <div class="relative">
                            <input
                                type={move || if show_new.get() { "text" } else { "password" }}
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
                        {move || password_strength.get().map(|strength| view! {
                            <div class="mt-2">
                                <div class="h-2 bg-gray-200 rounded-full overflow-hidden">
                                    <div class={format!("h-full {} transition-all duration-300", format!("{} {}", strength.color(), strength.width()))}></div>
                                </div>
                                <p class="text-xs text-gray-500 mt-1">"Kekuatan: " <strong>{strength.label()}</strong></p>
                            </div>
                        })}

                        // Password requirements
                        <div class="mt-2 text-xs text-gray-500 space-y-1">
                            <p class={move || if new_password.get().len() >= 8 { "text-green-600" } else { "text-gray-400" }}>
                                {move || if new_password.get().len() >= 8 { "✅" } else { "○" }}
                                " Minimal 8 karakter"
                            </p>
                            <p class={move || if new_password.get().chars().any(|c| c.is_uppercase()) { "text-green-600" } else { "text-gray-400" }}>
                                {move || if new_password.get().chars().any(|c| c.is_uppercase()) { "✅" } else { "○" }}
                                " Huruf besar"
                            </p>
                            <p class={move || if new_password.get().chars().any(|c| c.is_ascii_digit()) { "text-green-600" } else { "text-gray-400" }}>
                                {move || if new_password.get().chars().any(|c| c.is_ascii_digit()) { "✅" } else { "○" }}
                                " Angka"
                            </p>
                            <p class={move || if new_password.get().chars().any(|c| !c.is_alphanumeric()) { "text-green-600" } else { "text-gray-400" }}>
                                {move || if new_password.get().chars().any(|c| !c.is_alphanumeric()) { "✅" } else { "○" }}
                                " Karakter khusus"
                            </p>
                        </div>
                    </div>

                    // Confirm password
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Konfirmasi Kata Sandi Baru"</label>
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
                                Some(view! {
                                    <p class="text-red-500 text-xs mt-1">"Kata sandi tidak cocok"</p>
                                })
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
