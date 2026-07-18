//! Route Guard Components
//!
//! `PortalAuthLayout` and `PortalAdminLayout` are `ParentRoute` views that
//! protect all nested children automatically — like Next.js `layout.tsx`
//! or Laravel `Route::middleware('auth')->group(...)`.

use crate::features::auth::LoginPage;
use crate::features::auth::UserSession;
use crate::routes;
use leptos::prelude::*;

// ============================================================================
// LAYOUT GUARDS — wrap all child routes automatically (Next.js / Laravel style)
// ============================================================================

/// Authenticated layout guard for portal. Reads `user_session` from context
/// (provided in App root). Unauthenticated visitors see the login page
/// rendered inline at the originally-requested URL — this is intentional so
/// that after a successful login the reactive signal update re-renders the
/// guard into the protected `<Outlet />` at the same URL, preserving the
/// user's intended destination without needing return-URL plumbing.
/// Password-change redirect is enforced automatically.
#[component]
pub fn PortalAuthLayout(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for fallback login page
    on_login_success: WriteSignal<Option<UserSession>>,
) -> impl IntoView {
    // Capture router hooks at component render time (inside a reactive Owner
    // scope). Reading them inside the lazy guard closure would be fragile if
    // the Owner has been disposed.
    let location = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();

    move || match user_session.get() {
        Some(session) => {
            if session.require_password_change {
                // Allow the password-change page itself through to avoid an
                // infinite redirect loop (the password route is a child of
                // this same layout guard). Use the router's reactive
                // `use_location()` rather than `window.location.pathname` —
                // the former is updated synchronously during SPA navigation
                // and is less fragile to path-structure changes.
                let is_password_page = location.pathname.with(|p| {
                    let normalized = p.trim_end_matches('/');
                    normalized.ends_with(&format!("/{}", routes::segment::PASSWORD))
                });

                if !is_password_page {
                    navigate(
                        &format!("/{}", routes::segment::PASSWORD),
                        Default::default(),
                    );
                    return view! { <div /> }.into_any();
                }
            }
            view! { <leptos_router::components::Outlet /> }.into_any()
        }
        None => view! { <LoginPage on_login_success=on_login_success /> }.into_any(),
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
    // Capture navigate at render time so the closure below doesn't depend on
    // a reactive Owner scope that may be disposed during rapid navigation.
    let navigate = leptos_router::hooks::use_navigate();

    move || match user_session.get() {
        Some(session) => {
            if session.require_password_change {
                navigate(
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
        None => view! { <LoginPage on_login_success=on_login_success /> }.into_any(),
    }
}

/// Forbidden page for non-admin users trying to access admin routes
#[component]
pub fn ForbiddenPage() -> impl IntoView {
    // Capture `use_navigate()` at component render time (inside a reactive
    // Owner scope). Calling it inside the `on:click` closure directly would
    // be fragile — if the reactive Owner has been disposed (e.g. during
    // rapid navigation), `use_context` inside `use_navigate` would panic.
    let nav = leptos_router::hooks::use_navigate();

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="text-center max-w-md px-4">
                <div class="w-20 h-20 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                    <svg
                        class="w-10 h-10 text-red-500 dark:text-red-400"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"
                        />
                    </svg>
                </div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                    "Akses Ditolak"
                </h1>
                <p class="text-gray-600 dark:text-gray-400 mb-8 leading-relaxed">
                    "Anda tidak memiliki izin untuk mengakses halaman ini. "
                    "Hubungi administrator jika Anda memerlukan akses."
                </p>
                <a
                    href="/portal/dashboard"
                    on:click=move |ev| {
                        ev.prevent_default();
                        // navigate() resolves against the router base ("/portal"),
                        // so the path here must be base-relative — a "/portal/…"
                        // path would navigate to /portal/portal/… (404).
                        nav("/dashboard", Default::default());
                    }
                    class="inline-flex items-center gap-2 px-5 py-2.5 bg-navy-700 hover:bg-navy-800 dark:bg-gold-500 dark:hover:bg-gold-600 text-white dark:text-navy-900 rounded-xl font-medium transition-colors shadow-sm"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10 19l-7-7m0 0l7-7m-7 7h18"
                        />
                    </svg>
                    "Kembali ke Dashboard"
                </a>
            </div>
        </div>
    }
}
