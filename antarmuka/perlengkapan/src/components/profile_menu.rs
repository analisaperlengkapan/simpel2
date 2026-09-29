//! Profile Menu — avatar dropdown with identity, admin link, and logout.

use super::role_identity::RoleIdentity;
use crate::components::session_authz::use_authz;
use crate::features::auth::{AuthService, UserSession};
use crate::routes;
use leptos::prelude::*;
use leptos_node_ref::AnyNodeRef;
use leptos_router::components::A;
use lib_core::authz::Capability;
use lib_ui::components::floating::Popover;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{GEAR, SIGN_OUT, USER};

/// Profile avatar button + dropdown showing identity, roles, and logout.
///
/// Positioning, outside-click dismiss, and Escape-key dismiss come from
/// [`Popover`] (backed by `floating-ui-leptos`). The avatar trigger and
/// the panel content stay declarative — no manual z-index, no manual
/// click-outside handler, no opacity-transform open/close trickery.
#[component]
pub fn ProfileMenu() -> impl IntoView {
    let is_open = RwSignal::new(false);
    let trigger_ref = AnyNodeRef::new();

    // Read user info from the global session context (provided in App root).
    let session = use_context::<ReadSignal<Option<UserSession>>>();
    let display_name = move || {
        session
            .and_then(|s| s.get())
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Pengguna".to_string())
    };
    let display_email = move || {
        session
            .and_then(|s| s.get())
            .and_then(|s| s.email.clone())
            .unwrap_or_else(|| "-".to_string())
    };

    view! {
        // ── Avatar trigger ────────────────────────────────────
        <button
            node_ref=trigger_ref
            on:click=move |_| is_open.update(|o| *o = !*o)
            style="width: 36px; height: 36px; border-radius: 50%; border: 2px solid rgba(212,168,67,0.5); background: rgba(255,255,255,0.06); display: flex; align-items: center; justify-content: center; cursor: pointer; transition: all 0.2s;"
            class="hover:border-gold-400"
            title="Profil"
            aria-haspopup="menu"
            aria-expanded=move || is_open.get().to_string()
        >
            <span style="color: #d4a843; display: inline-flex;">
                <AppIcon icon=USER size=14 />
            </span>
        </button>

        // ── Floating panel ────────────────────────────────────
        <Popover
            open=is_open.read_only()
            set_open=is_open.write_only()
            trigger_ref=trigger_ref
            offset=10.0
            role="menu"
            class="w-[300px] rounded-[18px] border border-white/10 bg-surface-panel overflow-hidden shadow-panel"
                .to_string()
        >
            // ── User info header ─────────────────────────────
            <div style="padding: 20px; border-bottom: 1px solid rgba(255,255,255,0.06);">
                <div style="display: flex; align-items: center; gap: 14px;">
                    <div style="width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #d4a843, #facc15); display: flex; align-items: center; justify-content: center; flex-shrink: 0;">
                        <span style="color: #0f172a; display: inline-flex;">
                            <AppIcon icon=USER size=20 />
                        </span>
                    </div>
                    <div>
                        <div style="font-size: 0.9rem; font-weight: 700; color: #e2e8f0;">
                            {display_name}
                        </div>
                        <div style="font-size: 0.72rem; color: #7b8ba1; margin-top: 2px;">
                            {display_email}
                        </div>
                    </div>
                </div>
            </div>

            // ── Identity: roles as granted by the token ──────
            // Read-only. There is no role to choose here: the server
            // re-derives every scope from the bearer token, so a selectable
            // role could only ever misrepresent authority.
            <div style="padding: 10px 12px; border-bottom: 1px solid rgba(255,255,255,0.06);">
                <div style="font-size: 0.68rem; font-weight: 600; color: #7b8ba1; text-transform: uppercase; letter-spacing: 0.08em; padding: 4px 8px; margin-bottom: 4px;">
                    "Role Anda"
                </div>
                <RoleIdentity />
            </div>

            // ── Actions ──────────────────────────────────────
            <div style="padding: 8px;">
                <Show when=move || use_authz().get().can(Capability::Administer)>
                <A
                    href=routes::path::ADMIN_MASTER
                    attr:style="display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; text-decoration: none; color: #94a3b8; font-size: 0.82rem; transition: all 0.15s;"
                    attr:class="hover:bg-white/[0.04] hover:text-white"
                >
                    <span style="width: 18px; display: inline-flex; justify-content: center;">
                        <AppIcon icon=GEAR size=14 />
                    </span>
                    <span>"Pengaturan"</span>
                </A>
                </Show>
                <button
                    style="width: 100%; display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; border: none; background: none; color: #f87171; font-size: 0.82rem; cursor: pointer; transition: all 0.15s; text-align: left;"
                    class="hover:bg-red-500/[0.1]"
                    on:click=move |_| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            AuthService::logout();
                            if let Some(window) = web_sys::window() {
                                let origin = window
                                    .location()
                                    .origin()
                                    .unwrap_or_else(|_| String::new());
                                let _ = window
                                    .location()
                                    .set_href(&format!("{}/portal/login", origin));
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))] AuthService::clear_session();
                    }
                >
                    <span style="width: 18px; display: inline-flex; justify-content: center;">
                        <AppIcon icon=SIGN_OUT size=14 />
                    </span>
                    <span>"Keluar"</span>
                </button>
            </div>
        </Popover>
    }
}
