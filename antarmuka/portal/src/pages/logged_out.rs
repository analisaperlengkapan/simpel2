//! Logged out confirmation page
//!
//! Displayed after successful logout to confirm session termination

use leptos::prelude::*;

/// Logged out confirmation page
/// Displayed after successful logout to confirm session termination.
/// Provides clear feedback to users and options to login again or return home.
/// # Features
/// - Success confirmation message
/// - Security reminder for shared computers
/// - Quick actions (login again, return home)
/// - Responsive design with dark mode support
/// # Example
/// ```rust
/// use portal::pages::logged_out::LoggedOutPage;
/// #[component]
/// pub fn App() -> impl IntoView {
///     view! {
///         <Route path="/logged-out" view=LoggedOutPage />
///     }
/// }
/// ```
#[component]
pub fn LoggedOutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    // Success icon
                    <div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100 dark:bg-green-900 mb-4">
                        <i class="fas fa-check-circle text-3xl text-green-600 dark:text-green-400"></i>
                    </div>

                    // Title
                    <h2 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Anda Telah Keluar"
                    </h2>

                    // Description
                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                        "Sesi Anda telah berakhir dengan aman. Terima kasih telah menggunakan SIMPelv2."
                    </p>

                    // Security notice
                    <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-8">
                        <div class="flex items-start">
                            <i class="fas fa-info-circle text-blue-500 mt-1 mr-3 flex-shrink-0"></i>
                            <div class="text-left">
                                <p class="text-sm text-blue-800 dark:text-blue-300">
                                    "Untuk keamanan, pastikan Anda menutup browser jika menggunakan komputer bersama."
                                </p>
                            </div>
                        </div>
                    </div>

                    // Action buttons
                    <div class="space-y-3">
                        <a
                            href="/login"
                            class="w-full inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                        >
                            <i class="fas fa-sign-in-alt mr-2"></i>
                            "Masuk Kembali"
                        </a>

                        <a
                            href="/"
                            class="w-full inline-flex items-center justify-center px-6 py-3 border border-gray-300 dark:border-gray-600 text-base font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                        >
                            <i class="fas fa-home mr-2"></i>
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
