//! Authentication layout component
//!
//! Layout wrapper for public/authentication pages (login, home, 404, etc.)
//! Uses navy/gold government branding with dark-mode support.

use leptos::prelude::*;

/// Layout for authentication pages (login, register, etc.)
///
/// Features:
/// - Navy-to-dark gradient background consistent with Kejaksaan RI branding
/// - Gold accent top bar
/// - Responsive padding and centering
/// - Proper dark mode support
/// - Accessible landmark roles
#[component]
pub fn AuthLayout(
    /// Page content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col bg-gradient-to-br from-navy-800 via-navy-900 to-navy-950 overflow-x-hidden">
            // Decorative top accent bar — gold branding
            <div class="h-1 w-full bg-gradient-to-r from-gold-400 via-gold-500 to-gold-400 flex-shrink-0"></div>

            <main class="flex-1 flex items-center justify-center px-4 py-8 sm:py-12 w-full" role="main">
                <div class="w-full">
                    {children()}
                </div>
            </main>

            <footer class="flex-shrink-0 border-t border-white/10" role="contentinfo">
                <div class="max-w-7xl mx-auto px-4 py-4">
                    <p class="text-center text-sm text-slate-400">
                        "© 2026 Kejaksaan Republik Indonesia — Portal SIMPEL v2.0"
                    </p>
                </div>
            </footer>
        </div>
    }
}
