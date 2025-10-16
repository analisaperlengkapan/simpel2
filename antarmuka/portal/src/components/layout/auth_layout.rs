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
        <div class="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">

            <main class="flex-1 flex items-center justify-center p-4">
                {children()}
            </main>

            <footer class="bg-white/50 dark:bg-gray-800/50 backdrop-blur-sm border-t border-gray-200 dark:border-gray-700">
                <div class="container mx-auto px-4 py-4">
                    <p class="text-center text-sm text-gray-600 dark:text-gray-400">
                        "© 2025 Kejaksaan Agung Republik Indonesia - Portal SIMPelv2"
                    </p>
                </div>
            </footer>
        </div>
    }
}
