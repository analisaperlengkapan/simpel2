//! Route Guard Components
//!
//! ProtectedRoute and AdminRoute wrappers for authenticated/admin-only pages.

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

/// Redirect to login page
#[component]
fn RedirectToLogin() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/portal/login");
        }
    }

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-50">
            <div class="text-center">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-4"></div>
                <p class="text-gray-600">"Mengalihkan ke halaman login..."</p>
            </div>
        </div>
    }
}

/// Forbidden page for non-admin users trying to access admin routes
#[component]
fn ForbiddenPage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-50">
            <div class="text-center max-w-md">
                <div class="text-6xl mb-4">"🚫"</div>
                <h1 class="text-2xl font-bold text-gray-900 mb-2">"Akses Ditolak"</h1>
                <p class="text-gray-600 mb-6">
                    "Anda tidak memiliki izin untuk mengakses halaman ini. "
                    "Hubungi administrator jika Anda memerlukan akses."
                </p>
                <a
                    href="/portal/dashboard"
                    class="inline-flex items-center px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
                >
                    "Kembali ke Dashboard"
                </a>
            </div>
        </div>
    }
}
