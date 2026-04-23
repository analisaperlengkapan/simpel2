//! Route Guard Components
//!
//! ## Layout Guards (recommended for new routes)
//!
//! Use `PortalAuthLayout` and `PortalAdminLayout` as `ParentRoute` views
//! to protect all nested children automatically — like Next.js `layout.tsx`
//! or Laravel `Route::middleware('auth')->group(...)`.
//!
//! ## Inline Guards (legacy)
//!
//! `SessionAuthGuard` and `SessionAdminGuard` wrap individual route views.
//! Prefer the layout approach for new routes.

use crate::features::auth::UserSession;
use crate::pages::LoginPage;
use crate::routes;
use crate::utils::app_state::use_app_state;
use leptos::prelude::*;

// ============================================================================
// LAYOUT GUARDS — wrap all child routes automatically (Next.js / Laravel style)
// ============================================================================

/// Authenticated layout guard for portal. Reads `user_session` from context
/// (provided in App root). Unauthenticated visitors see the login page.
/// Password-change redirect is enforced automatically.
#[component]
pub fn PortalAuthLayout(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for fallback login page
    on_login_success: WriteSignal<Option<UserSession>>,
) -> impl IntoView {
    move || match user_session.get() {
        Some(session) => {
            if session.require_password_change {
                let nav = leptos_router::hooks::use_navigate();
                nav(
                    &format!("/{}", routes::segment::PASSWORD),
                    Default::default(),
                );
                view! { <div /> }.into_any()
            } else {
                view! { <leptos_router::components::Outlet /> }.into_any()
            }
        }
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}

/// Admin layout guard for portal. Enforces admin role + password-change policy.
#[component]
pub fn PortalAdminLayout(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for fallback login page
    on_login_success: WriteSignal<Option<UserSession>>,
) -> impl IntoView {
    move || match user_session.get() {
        Some(session) => {
            if session.require_password_change {
                let nav = leptos_router::hooks::use_navigate();
                nav(
                    &format!("/{}", routes::segment::PASSWORD),
                    Default::default(),
                );
                return view! { <div /> }.into_any();
            }
            if session.role.is_admin() {
                view! { <leptos_router::components::Outlet /> }.into_any()
            } else {
                view! { <ForbiddenPage /> }.into_any()
            }
        }
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}

// ============================================================================
// INLINE GUARDS — legacy per-route wrappers (kept for backward compat)
// ============================================================================

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

/// Redirect to login page with loading spinner (SPA navigation)
#[component]
fn RedirectToLogin() -> impl IntoView {
    let nav = leptos_router::hooks::use_navigate();
    nav("/portal/login", Default::default());

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
pub fn ForbiddenPage() -> impl IntoView {
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
                    on:click=move |ev| {
                        ev.prevent_default();
                        let nav = leptos_router::hooks::use_navigate();
                        nav("/portal/dashboard", Default::default());
                    }
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

/// Session-aware auth guard for pages that use signal-based session state.
#[component]
pub fn SessionAuthGuard(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for fallback login page
    on_login_success: WriteSignal<Option<UserSession>>,
    /// Child content rendered when authenticated
    children: ChildrenFn,
    /// Skip password-change redirect for password page itself
    #[prop(optional)]
    allow_password_change: bool,
) -> impl IntoView {
    let children = StoredValue::new_local(children);

    move || match user_session.get() {
        Some(session) => {
            if !allow_password_change && session.require_password_change {
                let nav = leptos_router::hooks::use_navigate();
                nav(
                    &format!("/{}", routes::segment::PASSWORD),
                    Default::default(),
                );
                view! { <div /> }.into_any()
            } else {
                children.with_value(|c| c().into_any())
            }
        }
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}

/// Session-aware admin guard that enforces admin role and password-change policy.
#[component]
pub fn SessionAdminGuard(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for fallback login page
    on_login_success: WriteSignal<Option<UserSession>>,
    /// Child content rendered when authenticated admin
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new_local(children);

    move || match user_session.get() {
        Some(session) => {
            if session.require_password_change {
                let nav = leptos_router::hooks::use_navigate();
                nav(
                    &format!("/{}", routes::segment::PASSWORD),
                    Default::default(),
                );
                return view! { <div /> }.into_any();
            }
            if session.role.is_admin() {
                children.with_value(|c| c().into_any())
            } else {
                view! { <ForbiddenPage /> }.into_any()
            }
        }
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}
