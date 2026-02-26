//! Route Guard Components
//!
//! ProtectedRoute and AdminRoute wrappers for authenticated/admin-only pages.
//! These guards use the global AppState to check authentication status.

use crate::utils::app_state::use_app_state;
use leptos::prelude::*;

/// A route wrapper that redirects to login if the user is not authenticated.
///
/// Usage:
/// ```rust,ignore
/// view! { <ProtectedRoute><DashboardPage /></ProtectedRoute> }
/// ```
#[component]
pub fn ProtectedRoute(children: Children) -> impl IntoView {
    let state = use_app_state();

    if state.get().is_authenticated() {
        children().into_any()
    } else {
        view! {
            <RedirectToLogin />
        }
        .into_any()
    }
}

/// A route wrapper that only allows admin users. Non-admins see a forbidden message.
#[component]
pub fn AdminRoute(children: Children) -> impl IntoView {
    let state = use_app_state();

    let current = state.get();
    if !current.is_authenticated() {
        view! { <RedirectToLogin /> }.into_any()
    } else if !current.is_admin() {
        view! { <ForbiddenPage /> }.into_any()
    } else {
        children().into_any()
    }
}

/// Redirect to login page with loading spinner
#[component]
fn RedirectToLogin() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/portal/login");
        }
    }

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="text-center" role="status" aria-live="polite">
                <div class="animate-spin rounded-full h-10 w-10 border-4 border-navy-200 dark:border-navy-700 border-t-navy-600 dark:border-t-gold-400 mx-auto mb-4"></div>
                <p class="text-gray-600 dark:text-gray-400 text-sm">"Mengalihkan ke halaman login..."</p>
            </div>
        </div>
    }
}

/// Forbidden page for non-admin users trying to access admin routes
#[component]
fn ForbiddenPage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="text-center max-w-md px-4">
                <div class="w-20 h-20 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                    <svg class="w-10 h-10 text-red-500 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                    </svg>
                </div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Akses Ditolak"</h1>
                <p class="text-gray-600 dark:text-gray-400 mb-8 leading-relaxed">
                    "Anda tidak memiliki izin untuk mengakses halaman ini. "
                    "Hubungi administrator jika Anda memerlukan akses."
                </p>
                <a
                    href="/portal/dashboard"
                    class="inline-flex items-center gap-2 px-5 py-2.5 bg-navy-700 hover:bg-navy-800 dark:bg-gold-500 dark:hover:bg-gold-600 text-white dark:text-navy-900 rounded-xl font-medium transition-colors shadow-sm"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                    </svg>
                    "Kembali ke Dashboard"
                </a>
            </div>
        </div>
    }
}
