//! # Role Switcher Component
//!
//! Dropdown component in the top-right profile area that allows users
//! to switch between their assigned perlengkapan roles:
//! - Operator Satker
//! - Validator Wilayah
//! - Validator Pusat
//! - Admin

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Role information
#[derive(Clone, Debug, PartialEq)]
pub struct PerlengkapanRole {
    pub key: String,
    pub label: String,
    pub description: String,
    pub icon: String,
    pub color: String,
}

impl PerlengkapanRole {
    pub fn all_roles() -> Vec<Self> {
        vec![
            Self {
                key: "operator_satker".to_string(),
                label: "Operator Satker".to_string(),
                description: "Input data kebutuhan, pemakaian, pemeliharaan BMN".to_string(),
                icon: "fas fa-keyboard".to_string(),
                color: "blue".to_string(),
            },
            Self {
                key: "validator_wilayah".to_string(),
                label: "Validator Wilayah".to_string(),
                description: "Verifikasi data dari operator satker tingkat wilayah".to_string(),
                icon: "fas fa-check-double".to_string(),
                color: "amber".to_string(),
            },
            Self {
                key: "validator_pusat".to_string(),
                label: "Validator Pusat".to_string(),
                description: "Persetujuan akhir & penerbitan SK tingkat pusat".to_string(),
                icon: "fas fa-stamp".to_string(),
                color: "emerald".to_string(),
            },
            Self {
                key: "admin".to_string(),
                label: "Admin".to_string(),
                description: "Pengelolaan pengguna, konfigurasi, master data".to_string(),
                icon: "fas fa-user-shield".to_string(),
                color: "red".to_string(),
            },
        ]
    }
}

/// Get active role from localStorage
pub fn get_active_role() -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("active_role").ok().flatten())
        .unwrap_or_else(|| "operator_satker".to_string())
}

/// Set active role in localStorage
pub fn set_active_role(role: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item("active_role", role);
    }
}

/// Get user's available roles from localStorage (set by auth flow)
pub fn get_available_roles() -> Vec<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("available_roles").ok().flatten())
        .map(|roles| roles.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| {
            vec![
                "operator_satker".to_string(),
                "validator_wilayah".to_string(),
                "validator_pusat".to_string(),
                "admin".to_string(),
            ]
        })
}

/// Role Switcher Component
#[component]
pub fn RoleSwitcher() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (active_role, set_active_role_sig) = signal(get_active_role());
    let available_roles = get_available_roles();

    let all_roles = PerlengkapanRole::all_roles();
    let user_roles: Vec<PerlengkapanRole> = all_roles
        .into_iter()
        .filter(|r| available_roles.contains(&r.key))
        .collect();

    let current_role = move || {
        let active = active_role.get();
        PerlengkapanRole::all_roles()
            .into_iter()
            .find(|r| r.key == active)
            .unwrap_or_else(|| PerlengkapanRole::all_roles()[0].clone())
    };

    let handle_switch = move |role_key: String| {
        log(&format!("Switching role to: {}", role_key));
        set_active_role(&role_key);
        set_active_role_sig.set(role_key);
        set_is_open.set(false);
        // Dispatch a custom event so other components can react to the role change
        // without a full page reload (preserves SPA state)
        if let Some(window) = web_sys::window() {
            if let Ok(event) = web_sys::CustomEvent::new("role-changed") {
                let _ = window.dispatch_event(&event);
            }
        }
    };

    let role_color_class = move |color: &str| -> String {
        match color {
            "blue" => "bg-blue-100 text-blue-700 border-blue-200".to_string(),
            "amber" => "bg-amber-100 text-amber-700 border-amber-200".to_string(),
            "emerald" => "bg-emerald-100 text-emerald-700 border-emerald-200".to_string(),
            "red" => "bg-red-100 text-red-700 border-red-200".to_string(),
            _ => "bg-gray-100 text-gray-700 border-gray-200".to_string(),
        }
    };

    let role_bg_class = move |color: &str| -> String {
        match color {
            "blue" => "from-blue-500 to-blue-600".to_string(),
            "amber" => "from-amber-500 to-amber-600".to_string(),
            "emerald" => "from-emerald-500 to-emerald-600".to_string(),
            "red" => "from-red-500 to-red-600".to_string(),
            _ => "from-gray-500 to-gray-600".to_string(),
        }
    };

    view! {
        <div class="relative">
            // Current role badge (clickable)
            <button
                class=move || format!("flex items-center gap-2 px-3 py-1.5 rounded-lg border transition-all duration-200 hover:shadow-md {}", role_color_class(&current_role().color))
                on:click=move |_| set_is_open.update(|o| *o = !*o)
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        set_is_open.set(false);
                    }
                }
                title="Ganti Role"
                aria-haspopup="true"
                aria-expanded=move || if is_open.get() { "true" } else { "false" }
            >
                <i class=move || current_role().icon.clone()></i>
                <span class="text-xs font-semibold hidden md:inline">{move || current_role().label.clone()}</span>
                <i class=move || format!(
                    "fas fa-chevron-down text-xs transition-transform {}",
                    if is_open.get() { "rotate-180" } else { "" }
                )></i>
            </button>

            // Dropdown
            <div
                class=move || format!(
                    "absolute right-0 mt-2 w-72 bg-white rounded-xl shadow-2xl border border-gray-100 z-50 transition-all duration-200 overflow-hidden {}",
                    if is_open.get() { "opacity-100 visible scale-100" } else { "opacity-0 invisible scale-95" }
                )
                role="menu"
                aria-label="Pilih role aktif"
            >
                // Header
                <div class="px-4 py-3 bg-gradient-to-r from-gray-50 to-gray-100 border-b">
                    <div class="flex items-center gap-2">
                        <i class="fas fa-exchange-alt text-gray-500"></i>
                        <span class="text-sm font-semibold text-gray-700">"Ganti Role Aktif"</span>
                    </div>
                    <p class="text-xs text-gray-500 mt-1">"Pilih role untuk mengubah tampilan dan akses menu"</p>
                </div>

                // Role options
                <div class="py-2">
                    {user_roles.into_iter().map(|role| {
                        let role_key = role.key.clone();
                        let role_key_for_click = role.key.clone();
                        let role_label = role.label.clone();
                        let role_desc = role.description.clone();
                        let role_icon = role.icon.clone();
                        let role_color = role.color.clone();
                        let is_active = Signal::derive(move || active_role.get() == role_key);
                        let handle = handle_switch.clone();
                        let bg_cls = role_bg_class(&role_color);

                        view! {
                            <button
                                class=move || format!(
                                    "w-full text-left px-4 py-3 flex items-start gap-3 transition-all duration-150 {}",
                                    if is_active.get() { "bg-gradient-to-r from-gray-50 to-blue-50 border-l-4 border-blue-500" }
                                    else { "hover:bg-gray-50 border-l-4 border-transparent" }
                                )
                                on:click={
                                    let key = role_key_for_click.clone();
                                    move |_| handle(key.clone())
                                }
                            >
                                <div class=format!("w-8 h-8 rounded-lg bg-gradient-to-br {} flex items-center justify-center flex-shrink-0 shadow-sm", bg_cls)>
                                    <i class=format!("{} text-white text-xs", role_icon)></i>
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <span class="text-sm font-medium text-gray-800">{role_label}</span>
                                        {move || is_active.get().then(|| view! {
                                            <span class="px-1.5 py-0.5 bg-blue-100 text-blue-600 text-[10px] font-bold rounded">
                                                "AKTIF"
                                            </span>
                                        })}
                                    </div>
                                    <p class="text-xs text-gray-500 mt-0.5 line-clamp-2">{role_desc}</p>
                                </div>
                                {move || is_active.get().then(|| view! {
                                    <i class="fas fa-check-circle text-blue-500 mt-1 flex-shrink-0"></i>
                                })}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>

        // Backdrop to close dropdown
        {move || is_open.get().then(|| view! {
            <div
                class="fixed inset-0 z-40"
                on:click=move |_| set_is_open.set(false)
            ></div>
        })}
    }
}
