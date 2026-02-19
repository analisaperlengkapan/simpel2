//! Authentication layout component
//!
//! Layout wrapper for public/authentication pages

use leptos::prelude::*;

/// Layout for authentication pages (login, register, etc.)
#[component]
pub fn AuthLayout(
    /// Page content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col bg-gradient-to-br from-gray-50 via-white to-gray-100 dark:from-gray-900 dark:via-gray-850 dark:to-gray-800 overflow-x-hidden">
            // Decorative top accent bar
            <div class="h-1 w-full bg-gradient-to-r from-red-700 via-red-600 to-red-700 flex-shrink-0"></div>

            <main class="flex-1 flex items-center justify-center px-4 py-8 sm:py-12 w-full">
                <div class="w-full">
                    {children()}
                </div>
            </main>

            <footer class="flex-shrink-0 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-t border-gray-200 dark:border-gray-700">
                <div class="max-w-7xl mx-auto px-4 py-4">
                    <p class="text-center text-sm text-gray-500 dark:text-gray-400">
                        "© 2026 Kejaksaan Republik Indonesia — Portal SIMPEL v2.0"
                    </p>
                </div>
            </footer>
        </div>
    }
}
