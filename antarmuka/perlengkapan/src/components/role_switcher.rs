//! Role Switcher — inline role selector for embedding in profile dropdown.
//!
//! The real authoritative role comes from the JWT in `auth_token`. This
//! switcher only controls a UI preference key (`ui_active_role`) used by
//! development views that preview how each role sees the app. It never
//! mutates the session — the JWT claims always win in guards.

use crate::features::auth::AuthService;
use leptos::prelude::*;

const UI_ACTIVE_ROLE_KEY: &str = "ui_active_role";

/// Read the UI preview role. Defaults to whichever role the JWT reports, or
/// `operator_satker` when no session is loaded.
pub fn get_active_role() -> String {
    if let Some(stored) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(UI_ACTIVE_ROLE_KEY).ok().flatten())
    {
        return stored;
    }

    AuthService::load_session()
        .map(|s| s.role)
        .unwrap_or_else(|| "operator_satker".to_string())
}

fn set_active_role_storage(role: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(UI_ACTIVE_ROLE_KEY, role);
    }
}

#[derive(Clone, Debug)]
pub struct PerlengkapanRole {
    pub key: String,
    pub label: String,
    pub description: String,
    pub color: String,
    pub icon: String,
}

impl PerlengkapanRole {
    pub fn all_roles() -> Vec<Self> {
        vec![
            Self {
                key: "operator_satker".to_string(),
                label: "Operator Satker".to_string(),
                description: "Pengelola perlengkapan di tingkat Satuan Kerja".to_string(),
                color: "blue".to_string(),
                icon: "fas fa-keyboard".to_string(),
            },
            Self {
                key: "validator_wilayah".to_string(),
                label: "Validator Wilayah".to_string(),
                description: "Verifikator di tingkat Kejaksaan Tinggi".to_string(),
                color: "amber".to_string(),
                icon: "fas fa-check-double".to_string(),
            },
            Self {
                key: "validator_pusat".to_string(),
                label: "Validator Pusat".to_string(),
                description: "Verifikator akhir di Kejaksaan Agung".to_string(),
                color: "emerald".to_string(),
                icon: "fas fa-stamp".to_string(),
            },
            Self {
                key: "admin".to_string(),
                label: "Admin".to_string(),
                description: "Administrator sistem perlengkapan".to_string(),
                color: "red".to_string(),
                icon: "fas fa-user-shield".to_string(),
            },
        ]
    }
}

struct RoleDef {
    key: &'static str,
    label: &'static str,
    icon: &'static str,
    accent: &'static str,
}

const ROLES: &[RoleDef] = &[
    RoleDef {
        key: "operator_satker",
        label: "Operator Satker",
        icon: "fas fa-keyboard",
        accent: "#60a5fa",
    },
    RoleDef {
        key: "validator_wilayah",
        label: "Validator Wilayah",
        icon: "fas fa-check-double",
        accent: "#fbbf24",
    },
    RoleDef {
        key: "validator_pusat",
        label: "Validator Pusat",
        icon: "fas fa-stamp",
        accent: "#34d399",
    },
    RoleDef {
        key: "admin",
        label: "Admin",
        icon: "fas fa-user-shield",
        accent: "#f87171",
    },
];

/// Inline role switcher — renders role buttons as a flat list (for embedding in ProfileMenu).
#[component]
pub fn RoleSwitcher() -> impl IntoView {
    let active_key = RwSignal::new(get_active_role());

    view! {
        <div>
            {ROLES.iter().map(|role| {
                let key = role.key;
                let label = role.label;
                let icon = role.icon;
                let accent = role.accent;

                view! {
                    <button
                        on:click=move |_| {
                            set_active_role_storage(key);
                            active_key.set(key.to_string());
                            if let Some(window) = web_sys::window() {
                                if let Ok(event) = web_sys::CustomEvent::new("role-changed") {
                                    let _ = window.dispatch_event(&event);
                                }
                            }
                        }
                        style=move || format!(
                            "width: 100%; display: flex; align-items: center; gap: 10px; padding: 8px 10px; border: none; background: {}; border-radius: 8px; cursor: pointer; transition: all 0.15s; text-align: left; border-left: 2px solid {};",
                            if active_key.get() == key { "rgba(255,255,255,0.06)" } else { "transparent" },
                            if active_key.get() == key { accent } else { "transparent" },
                        )
                        class="hover:bg-white/[0.04]"
                    >
                        <i class=icon style=format!("font-size: 0.75rem; color: {}; width: 16px; text-align: center;", accent)></i>
                        <span style="font-size: 0.78rem; font-weight: 500; color: #cbd5e1;">{label}</span>
                        {move || (active_key.get() == key).then(|| view! {
                            <i class="fas fa-check" style=format!("font-size: 0.6rem; color: {}; margin-left: auto;", accent)></i>
                        })}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
