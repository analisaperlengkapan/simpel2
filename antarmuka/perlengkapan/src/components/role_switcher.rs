//! Role Switcher — inline role selector for embedding in profile dropdown.
//!
//! The real authoritative role comes from the JWT in `auth_token`. This
//! switcher only controls a UI preference key (`ui_active_role`) used by
//! views that preview how each role sees the app. It never mutates the
//! session — the JWT claims always win in guards, and the backend re-derives
//! every scope from the token regardless of what is picked here.
//!
//! # Reactivity
//!
//! The active role lives in an [`ActiveRole`] context signal, seeded from
//! `localStorage` once at app start. Switching role updates that signal, so
//! every reader re-renders immediately.
//!
//! It previously lived *only* in `localStorage`, read through a plain
//! `get_active_role() -> String`. Plain reads are not reactive, so pages that
//! branched on the role captured it once at mount and never saw a change; the
//! switcher also dispatched a `role-changed` CustomEvent that nothing ever
//! listened for. The result: picking a role did nothing until the browser was
//! refreshed (which remounted everything and re-read storage).

use crate::features::auth::AuthService;
use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::CHECK;

const UI_ACTIVE_ROLE_KEY: &str = "ui_active_role";

/// Reactive holder for the UI preview role. Provided once at the app root by
/// [`provide_active_role`]; read with [`use_active_role`].
#[derive(Clone, Copy)]
pub struct ActiveRole(pub RwSignal<String>);

/// Read the persisted UI preview role from storage, falling back to whichever
/// role the JWT reports (or `operator_satker` when no session is loaded).
///
/// Non-reactive by nature — it touches `localStorage`. Use it only to seed the
/// signal; readers that need to re-render want [`use_active_role`].
pub fn read_active_role_storage() -> String {
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

/// Install the active-role context. Call once, at the app root.
pub fn provide_active_role() {
    provide_context(ActiveRole(RwSignal::new(read_active_role_storage())));
}

/// The reactive active role. Reading `.get()` inside a view subscribes that
/// view, so a role switch re-renders it without a page refresh.
pub fn use_active_role() -> RwSignal<String> {
    expect_context::<ActiveRole>().0
}

/// Switch role: persist it *and* update the signal so live views follow.
/// Writing storage alone is what made the old switcher require a refresh.
pub fn set_active_role(role: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(UI_ACTIVE_ROLE_KEY, role);
    }
    if let Some(ActiveRole(signal)) = use_context::<ActiveRole>() {
        signal.set(role.to_string());
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
    // The shared context signal — NOT a local copy. A local signal is what
    // limited the old switcher's effect to its own checkmark.
    let active_key = use_active_role();

    view! {
        <div>
            {ROLES
                .iter()
                .map(|role| {
                    let key = role.key;
                    let label = role.label;
                    let icon = role.icon;
                    let accent = role.accent;

                    view! {
                        <button
                            // Persists + updates the shared signal, so every
                            // role-aware view re-renders on the spot. (This
                            // used to also dispatch a `role-changed`
                            // CustomEvent that nothing listened for.)
                            on:click=move |_| set_active_role(key)
                            style=move || {
                                format!(
                                    "width: 100%; display: flex; align-items: center; gap: 10px; padding: 8px 10px; border: none; background: {}; border-radius: 8px; cursor: pointer; transition: all 0.15s; text-align: left; border-left: 2px solid {};",
                                    if active_key.get() == key {
                                        "rgba(255,255,255,0.06)"
                                    } else {
                                        "transparent"
                                    },
                                    if active_key.get() == key { accent } else { "transparent" },
                                )
                            }
                            class="hover:bg-white/[0.04]"
                        >
                            <span style=format!(
                                "color: {}; width: 16px; display: inline-flex; justify-content: center;",
                                accent,
                            )>
                                <AppIcon icon=icon_from_fa_class(icon) size=12 />
                            </span>
                            <span style="font-size: 0.78rem; font-weight: 500; color: #cbd5e1;">
                                {label}
                            </span>
                            {move || {
                                (active_key.get() == key)
                                    .then(|| {
                                        view! {
                                            <span style=format!(
                                                "color: {}; margin-left: auto; display: inline-flex;",
                                                accent,
                                            )>
                                                <AppIcon icon=CHECK size=10 />
                                            </span>
                                        }
                                    })
                            }}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
