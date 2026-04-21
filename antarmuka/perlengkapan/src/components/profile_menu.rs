//! Profile Menu — avatar dropdown with role switcher + logout.

use crate::features::auth::AuthService;
use crate::routes;
use super::role_switcher::{RoleSwitcher, get_active_role};
use leptos::prelude::*;
use leptos_router::components::A;

/// Profile avatar button + dropdown with role switcher and logout.
#[component]
pub fn ProfileMenu() -> impl IntoView {
    let is_open = RwSignal::new(false);

    view! {
        <div style="position: relative;">
            // ── Avatar trigger ────────────────────────────────────
            <button
                on:click=move |_| is_open.update(|o| *o = !*o)
                style="width: 36px; height: 36px; border-radius: 50%; border: 2px solid rgba(212,168,67,0.5); background: rgba(255,255,255,0.06); display: flex; align-items: center; justify-content: center; cursor: pointer; transition: all 0.2s;"
                class="hover:border-gold-400"
                title="Profil"
            >
                <i class="fas fa-user" style="font-size: 0.85rem; color: #d4a843;"></i>
            </button>

            // ── Dropdown panel ────────────────────────────────────
            <div style=move || format!(
                "position: absolute; right: 0; top: 100%; margin-top: 10px; width: 300px; background: #0c1425; border: 1px solid rgba(255,255,255,0.1); border-radius: 18px; z-index: 70; overflow: hidden; {} box-shadow: 0 24px 64px rgba(0,0,0,0.6);",
                if is_open.get() { "opacity: 1; transform: translateY(0); pointer-events: auto;" } else { "opacity: 0; transform: translateY(-8px); pointer-events: none;" }
            )>
                // ── User info header ─────────────────────────────
                <div style="padding: 20px; border-bottom: 1px solid rgba(255,255,255,0.06);">
                    <div style="display: flex; align-items: center; gap: 14px;">
                        <div style="width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #d4a843, #facc15); display: flex; align-items: center; justify-content: center; flex-shrink: 0;">
                            <i class="fas fa-user" style="font-size: 1.2rem; color: #0f172a;"></i>
                        </div>
                        <div>
                            <div style="font-size: 0.9rem; font-weight: 700; color: #e2e8f0;">"Administrator"</div>
                            <div style="font-size: 0.72rem; color: #64748b; margin-top: 2px;">"admin@kejaksaan.go.id"</div>
                        </div>
                    </div>
                </div>

                // ── Role switcher section ────────────────────────
                <div style="padding: 10px 12px; border-bottom: 1px solid rgba(255,255,255,0.06);">
                    <div style="font-size: 0.68rem; font-weight: 600; color: #64748b; text-transform: uppercase; letter-spacing: 0.08em; padding: 4px 8px; margin-bottom: 4px;">"Ganti Role"</div>
                    <RoleSwitcher />
                </div>

                // ── Actions ──────────────────────────────────────
                <div style="padding: 8px;">
                    <A href=routes::path::ADMIN_MASTER attr:style="display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; text-decoration: none; color: #94a3b8; font-size: 0.82rem; transition: all 0.15s;" attr:class="hover:bg-white/[0.04] hover:text-white">
                        <i class="fas fa-cog" style="width: 18px; text-align: center; font-size: 0.8rem;"></i>
                        <span>"Pengaturan"</span>
                    </A>
                    <button
                        style="width: 100%; display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; border: none; background: none; color: #f87171; font-size: 0.82rem; cursor: pointer; transition: all 0.15s; text-align: left;"
                        class="hover:bg-red-500/[0.1]"
                        on:click=move |_| {
                            #[cfg(target_arch = "wasm32")]
                            AuthService::logout();

                            #[cfg(not(target_arch = "wasm32"))]
                            AuthService::clear_session();

                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(routes::path::LOGIN);
                            }
                        }
                    >
                        <i class="fas fa-sign-out-alt" style="width: 18px; text-align: center; font-size: 0.8rem;"></i>
                        <span>"Keluar"</span>
                    </button>
                </div>
            </div>

            // ── Backdrop ─────────────────────────────────────────
            {move || is_open.get().then(|| view! {
                <div
                    style="position: fixed; inset: 0; z-index: 60;"
                    on:click=move |_| is_open.set(false)
                ></div>
            })}
        </div>
    }
}
