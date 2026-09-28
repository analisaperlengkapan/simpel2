//! Read-only role identity for the profile dropdown.
//!
//! # What this replaced, and why
//!
//! This module was a **role switcher**: a "Ganti Role" list in the profile
//! menu where any authenticated user could pick `admin`, `validator_pusat`, or
//! any other role, and the choice was persisted to `localStorage` under
//! `ui_active_role`. The sidebar then showed the admin section based on that
//! value, the helpdesk rendered staff triage based on it, and the dashboard
//! chose its scope wording from it.
//!
//! The previous module docstring defended this as harmless because "the real
//! authoritative role comes from the JWT … the backend re-derives every scope
//! from the token". That reasoning is correct about *authorization* and wrong
//! about the *product*. A dropdown that lets an operator select "Administrator"
//! tells them they hold a role they do not hold. They then meet `403` on every
//! admin link — or, in the other direction, an administrator who selects
//! "Operator Satker" is told they lack access they actually have. Either way
//! the UI is lying about identity, and the fix is not to document the lie but
//! to stop offering the choice.
//!
//! Roles are now **displayed**, not chosen. They come from the JWT via
//! [`crate::components::session_authz::use_authz`], so what the badge shows is
//! what the server will enforce.

use crate::components::session_authz::use_session;
use leptos::prelude::*;
use lib_core::authz::{is_admin_role, role_label};
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};

/// Accent colour per role family, so identity reads at a glance.
fn accent_for(role: &str) -> &'static str {
    let r = role.trim().to_lowercase();
    if is_admin_role(&r) {
        "#f87171"
    } else if r.starts_with("validator_") {
        "#fbbf24"
    } else if r.starts_with("operator") {
        "#60a5fa"
    } else if r.contains("pusat") {
        "#34d399"
    } else {
        "#94a3b8"
    }
}

/// Icon per role family.
fn icon_for(role: &str) -> &'static str {
    let r = role.trim().to_lowercase();
    if is_admin_role(&r) {
        "fas fa-user-shield"
    } else if r.starts_with("validator_") {
        "fas fa-check-double"
    } else if r.starts_with("operator") {
        "fas fa-keyboard"
    } else {
        "fas fa-user"
    }
}

/// The caller's roles, as granted by the token. Read-only.
///
/// Renders nothing when there is no session — the profile menu only appears
/// for authenticated callers, and an empty badge would be noise.
#[component]
pub fn RoleIdentity() -> impl IntoView {
    let session = use_session();

    view! {
        <div>
            {move || {
                let Some(s) = session.and_then(|sig| sig.get()) else {
                    return view! { <div /> }.into_any();
                };
                if s.roles.is_empty() {
                    return view! {
                        <div style="padding: 6px 10px; font-size: 0.72rem; color: #64748b;">
                            "Tidak ada role pada token."
                        </div>
                    }
                    .into_any();
                }
                let primary = s.role.clone();
                s
                    .roles
                    .clone()
                    .into_iter()
                    .map(|role| {
                        let is_primary = role == primary;
                        let accent = accent_for(&role);
                        let icon = icon_for(&role);
                        view! {
                            <div
                                style=format!(
                                    "display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 8px; border-left: 2px solid {}; background: {};",
                                    if is_primary { accent } else { "transparent" },
                                    if is_primary { "rgba(255,255,255,0.05)" } else { "transparent" },
                                )
                                title=role.clone()
                            >
                                <span style=format!(
                                    "color: {}; width: 16px; display: inline-flex; justify-content: center;",
                                    accent,
                                )>
                                    <AppIcon icon=icon_from_fa_class(icon) size=12 />
                                </span>
                                <span style=format!(
                                    "font-size: 0.78rem; font-weight: {}; color: {};",
                                    if is_primary { "600" } else { "400" },
                                    if is_primary { "#e2e8f0" } else { "#cbd5e1" },
                                )>{role_label(&role)}</span>
                            </div>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </div>
    }
}
