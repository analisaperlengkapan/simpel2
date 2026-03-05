//! Logged out confirmation page
//!
//! Displayed after successful logout to confirm session termination

use leptos::prelude::*;

/// Logged out confirmation page
///
/// Displayed after successful logout to confirm session termination.
/// Provides clear feedback to users and options to login again or return home.
///
/// # Features
/// - Success confirmation message
/// - Security reminder for shared computers
/// - Quick actions (login again, return home)
/// - Responsive design with dark mode support
///
/// # Example
/// ```rust
/// use portal_microfrontend::pages::logged_out::LoggedOutPage;
/// use leptos::prelude::*;
/// use leptos_router::{components::{Route, Router, Routes}, StaticSegment};
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     view! {
///         <Router>
///             <Routes fallback=|| "Not Found".into_view()>
///                 <Route path=StaticSegment("/logged-out") view=LoggedOutPage />
///             </Routes>
///         </Router>
///     }
/// }
/// ```
#[component]
pub fn LoggedOutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    // Success icon (SVG — no FontAwesome dependency)
                    <div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-emerald-100 dark:bg-emerald-900/30 mb-6">
                        <svg class="w-8 h-8 text-emerald-600 dark:text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                    </div>

                    // Title
                    <h2 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Anda Telah Keluar"
                    </h2>

                    // Description
                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                        "Sesi Anda telah berakhir dengan aman. Terima kasih telah menggunakan SIMPEL."
                    </p>

                    // Security notice
                    <div class="bg-navy-50 dark:bg-navy-900/20 border border-navy-200 dark:border-navy-800 rounded-xl p-4 mb-8">
                        <div class="flex items-start">
                            <svg class="w-5 h-5 text-navy-500 dark:text-navy-400 mt-0.5 mr-3 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <div class="text-left">
                                <p class="text-sm text-navy-800 dark:text-navy-300">
                                    "Untuk keamanan, pastikan Anda menutup browser jika menggunakan komputer bersama."
                                </p>
                            </div>
                        </div>
                    </div>

                    // Action buttons
                    <div class="space-y-3">
                        <a
                            href="/portal/login"
                            class="w-full inline-flex items-center justify-center gap-2 px-6 py-3 text-base font-medium rounded-xl text-white bg-navy-700 hover:bg-navy-800 dark:bg-gold-500 dark:hover:bg-gold-600 dark:text-navy-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-navy-500 transition-colors shadow-sm"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
                            </svg>
                            "Masuk Kembali"
                        </a>

                        <a
                            href="/portal"
                            class="w-full inline-flex items-center justify-center gap-2 px-6 py-3 border border-gray-300 dark:border-gray-600 text-base font-medium rounded-xl text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-navy-500 transition-colors"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                            </svg>
                            "Kembali ke Beranda"
                        </a>
                    </div>

                    // Additional info
                    <div class="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
                        <p class="text-xs text-gray-500 dark:text-gray-400">
                            "Jika Anda mengalami masalah saat login, silakan hubungi administrator sistem."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
