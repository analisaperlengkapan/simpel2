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
/// Reads session from `use_context()` — no props needed.
/// Provides navbar, sidebar, footer with Kejaksaan RI branding.
#[component]
pub fn MainLayout(
    /// Page content
    children: Children,
) -> impl IntoView {
    // Read session from context (provided by App root)
    let session = use_context::<ReadSignal<Option<UserSession>>>()
        .and_then(|sig| sig.get_untracked());

    // Build logout handler that broadcasts + clears, then does a full page
    // reload to discard all WASM memory (session signals, provide_context).
    let on_logout: Option<Box<dyn Fn()>> = Some(Box::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            // AuthService::logout() already calls broadcast_logout() + clear_session()
            // and fires a background POST to /api/v1/auth/logout.
            AuthService::logout();

            // Full page reload to clear WASM memory — SPA navigation would
            // leave reactive session state in the linear memory.
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/portal/login");
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            AuthService::clear_session();
            if let Some(set_session) = use_context::<WriteSignal<Option<UserSession>>>() {
                set_session.set(None);
            }
        }
    }));

    view! {
        <div class="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
            <Navbar user_session=session.clone() on_logout=on_logout />

            <div class="flex flex-1 min-h-0">
                <SidebarNavigation user_session=session.clone() />

                <main id="main-content" class="flex-1 min-w-0" role="main">
                    {children()}
                </main>
            </div>

            <footer class="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 mt-auto" role="contentinfo">
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
                            <a href="/portal/apps" class="hover:text-navy-600 dark:hover:text-gold-400 transition-colors">
                                "Aplikasi"
                            </a>
                            <span class="text-gray-300 dark:text-gray-600" aria-hidden="true">"|"</span>
                            <a href="/portal/settings" class="hover:text-navy-600 dark:hover:text-gold-400 transition-colors">
                                "Pengaturan"
                            </a>
                        </div>
                    </div>
                </div>
            </footer>
        </div>
    }
}
