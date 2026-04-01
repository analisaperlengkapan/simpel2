//! Main authenticated layout component
//!
//! Layout wrapper for authenticated pages with navbar and footer.
//! Uses navy/gold government branding consistent with Kejaksaan RI design system.

use crate::components::navigation::Navbar;
use crate::features::auth::UserSession;
use leptos::prelude::*;

/// Main layout for authenticated pages
///
/// Provides:
/// - Skip-to-content accessibility link
/// - Responsive navbar with user session
/// - Consistent footer with correct year and branding
#[component]
pub fn MainLayout(
    /// User session data
    user_session: UserSession,
    /// On logout callback
    on_logout: Box<dyn Fn()>,
    /// Page content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
            <Navbar user_session=user_session on_logout=on_logout />

            <main id="main-content" class="flex-1" role="main">
                {children()}
            </main>

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
