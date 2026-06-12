//! Main authenticated layout component
//!
//! Layout wrapper for authenticated pages with navbar and footer.
//! Uses navy/gold government branding consistent with Kejaksaan RI design system.
//!
//! Reads `UserSession` from context (provided by App root) instead of props,
//! enabling use with `ParentRoute` layout guards.

use crate::components::navigation::Navbar;
use crate::components::navigation::SidebarNavigation;
use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;

/// Main layout for authenticated pages.
///
/// Reads session from `use_context()` — no props needed in new code.
/// Provides navbar, sidebar, footer with Kejaksaan RI branding.
///
/// The `user_session` and `on_logout` props are accepted for backward
/// compatibility with pages that haven't migrated to the context-based
/// pattern yet (e.g. most `pages/admin/*.rs`). When supplied, the
/// `user_session` prop overrides the context-provided session and
/// `on_logout` overrides the built-in logout handler.
#[component]
pub fn MainLayout(
    /// Page content
    children: Children,
    /// Legacy: current user session (ignored if `None`; context is used instead).
    #[prop(optional)]
    user_session: Option<UserSession>,
    /// Legacy: logout callback (ignored if `None`; built-in handler is used instead).
    #[prop(optional)]
    on_logout: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    // Prefer the prop when supplied (legacy call sites); otherwise read
    // from context (provided by App root).
    let session = user_session.or_else(|| {
        use_context::<ReadSignal<Option<UserSession>>>().and_then(|sig| sig.get_untracked())
    });

    // Capture the session-timeout writers (provided by App root) so we can
    // dismiss the warning modal before navigating away at logout time —
    // otherwise the modal could flash on the login page. Captured at
    // component render time; the closure below uses them without
    // re-calling `use_context`.
    //
    // NOTE: Leptos context lookup is type-based. These generic `WriteSignal<bool>`
    // / `WriteSignal<i64>` types are only provided once (in `app.rs`), so this
    // lookup is unambiguous today. If another `WriteSignal<bool>` or
    // `WriteSignal<i64>` is ever provided higher in the tree, the lookup could
    // silently shadow. Wrap in a newtype (e.g. `struct ShowTimeoutWarning(...)`)
    // if that becomes a concern.
    let set_show_timeout_warning = use_context::<WriteSignal<bool>>();
    let set_timeout_countdown = use_context::<WriteSignal<i64>>();

    // Capture the session writer at render time as well. `use_context` must
    // run inside a reactive Owner scope, but the logout closure below may
    // fire long after render (e.g. from a click handler), outside that scope.
    // Capturing here ensures the non-WASM branch can always clear the signal.
    #[cfg(not(target_arch = "wasm32"))]
    let set_user_session = use_context::<WriteSignal<Option<UserSession>>>();

    // Build logout handler that broadcasts + clears, then does a full page
    // reload to discard all WASM memory (session signals, provide_context).
    // If a legacy `on_logout` prop was passed, prefer that.
    let on_logout: Option<Box<dyn Fn()>> = on_logout.or_else(|| {
        Some(Box::new(move || {
            // Clear the session-timeout warning first so it cannot flash on
            // the login page after navigation.
            if let Some(setter) = set_show_timeout_warning {
                setter.set(false);
            }
            if let Some(setter) = set_timeout_countdown {
                setter.set(0);
            }

            #[cfg(target_arch = "wasm32")]
            {
                // Explicitly broadcast before `logout()` clears localStorage, to
                // match the pattern used by Perlengkapan's `AuthService::logout()`
                // (which calls `broadcast_logout()` internally). Portal's
                // `AuthService::logout()` does NOT broadcast on its own, so we
                // must do it here — otherwise peer tabs only learn about the
                // logout via the `user_session` removal storage event, which is
                // a weaker signal than the dedicated `logout_event` broadcast.
                AuthService::broadcast_logout();
                AuthService::logout();

                // Full page reload to clear WASM memory — SPA navigation would
                // leave reactive session state in the linear memory.
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/portal/login");
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                // logout() membersihkan localStorage + state — under cfg(wasm32) di
                // dalamnya jadi no-op untuk native build, cukup signal state reset.
                AuthService::logout();
                if let Some(set_session) = set_user_session {
                    set_session.set(None);
                }
            }
        }))
    });

    view! {
        <div class="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
            <Navbar user_session=session.clone() on_logout=on_logout />

            <div class="flex flex-1 min-h-0">
                <SidebarNavigation user_session=session.clone() />

                <main id="main-content" class="flex-1 min-w-0" role="main">
                    {children()}
                </main>
            </div>

            <footer
                class="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 mt-auto"
                role="contentinfo"
            >
                <div class="container mx-auto px-4 py-6">
                    <div class="flex flex-col md:flex-row items-center justify-between space-y-4 md:space-y-0">
                        <div class="text-center md:text-left">
                            <p class="text-sm text-gray-600 dark:text-gray-400">
                                "© 2026 Kejaksaan Agung Republik Indonesia"
                            </p>
                            <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                                "SIMPEL Portal v2 — Sistem Terintegrasi"
                            </p>
                        </div>
                        <div class="flex items-center space-x-4 text-sm text-gray-600 dark:text-gray-400">
                            <a
                                href="/portal/apps"
                                class="hover:text-navy-600 dark:hover:text-gold-400 transition-colors"
                            >
                                "Aplikasi"
                            </a>
                            <span class="text-gray-300 dark:text-gray-600" aria-hidden="true">
                                "|"
                            </span>
                            <a
                                href="/portal/settings"
                                class="hover:text-navy-600 dark:hover:text-gold-400 transition-colors"
                            >
                                "Pengaturan"
                            </a>
                        </div>
                    </div>
                </div>
            </footer>
        </div>
    }
}
