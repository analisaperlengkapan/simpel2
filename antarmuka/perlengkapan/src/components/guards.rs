//! Route guards that reflect the JWT-backed session.
//!
//! ## Layout Guards (recommended)
//!
//! Use `AuthenticatedLayout` and `AdminLayout` as parent route views to
//! protect all nested children automatically — similar to Next.js `layout.tsx`
//! or Laravel `Route::middleware('auth')->group(...)`.
//!
//! ```rust,ignore
//! <ParentRoute path=path!("/") view=AuthenticatedLayout>
//!     <Route path=path!("/dashboard") view=DashboardHome />
//!     <Route path=path!("/bank-aset/daftar") view=BankAsetListPage />
//! </ParentRoute>
//! <ParentRoute path=path!("/admin") view=AdminLayout>
//!     <Route path=path!("/users") view=AdminUsersPage />
//! </ParentRoute>
//! ```
//!
//! ## Inline Guards (legacy)
//!
//! `SessionAuthGuard` and `SessionAdminGuard` wrap individual route views.
//! Prefer the layout approach for new routes.

use crate::features::auth::{AuthService, UserSession};
use crate::routes;
use leptos::prelude::*;
use leptos_router::components::A;

// ============================================================================
// LAYOUT GUARDS — wrap all child routes automatically (Next.js / Laravel style)
// ============================================================================

/// Authenticated layout guard. Use as a `ParentRoute` view to protect all
/// nested children. Unauthenticated visitors are redirected to login.
///
/// This replaces wrapping every `<Route>` with `<SessionAuthGuard>`.
#[component]
pub fn AuthenticatedLayout() -> impl IntoView {
    move || match AuthService::load_session() {
        Some(_session) => {
            view! { <leptos_router::components::Outlet /> }.into_any()
        }
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

/// Admin layout guard. Use as a `ParentRoute` view to protect all nested
/// children. Non-admin users see a forbidden page; unauthenticated visitors
/// are redirected to login.
#[component]
pub fn AdminLayout() -> impl IntoView {
    move || match AuthService::load_session() {
        Some(session) if session.is_admin() => {
            view! { <leptos_router::components::Outlet /> }.into_any()
        }
        Some(_) => view! { <ForbiddenPage /> }.into_any(),
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

// ============================================================================
// INLINE GUARDS — legacy per-route wrappers (kept for backward compat)
// ============================================================================

#[component]
pub fn SessionAuthGuard(
    user_session: ReadSignal<Option<UserSession>>,
    children: ChildrenFn,
) -> impl IntoView {
    let _ = user_session;
    let children = StoredValue::new_local(children);

    move || match AuthService::load_session() {
        Some(_) => children.with_value(|c| c().into_any()),
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

#[component]
pub fn SessionAdminGuard(
    user_session: ReadSignal<Option<UserSession>>,
    children: ChildrenFn,
) -> impl IntoView {
    let _ = user_session;
    let children = StoredValue::new_local(children);

    move || match AuthService::load_session() {
        Some(session) if session.is_admin() => children.with_value(|c| c().into_any()),
        Some(_) => view! { <ForbiddenPage /> }.into_any(),
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

/// Allow rendering only if the active session holds one of the given roles.
#[component]
pub fn RoleGuard(
    /// Realm roles accepted by this guard (OR semantics).
    #[prop(into)]
    roles: Vec<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let roles = StoredValue::new(roles);
    let children = StoredValue::new_local(children);

    move || match AuthService::load_session() {
        Some(session) => {
            let allowed =
                roles.with_value(|required| required.iter().any(|r| session.has_role(r.as_str())));
            if allowed {
                children.with_value(|c| c().into_any())
            } else {
                view! { <ForbiddenPage /> }.into_any()
            }
        }
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

/// Gate UI to a specific satker. Admins bypass the check so pusat users can
/// still inspect any satker without needing a cross-role hack.
#[component]
pub fn SatkerGuard(#[prop(into)] satker_code: String, children: ChildrenFn) -> impl IntoView {
    let satker_code = StoredValue::new(satker_code);
    let children = StoredValue::new_local(children);

    move || match AuthService::load_session() {
        Some(session) => {
            if session.is_admin() || session.is_validator_pusat() {
                return children.with_value(|c| c().into_any());
            }
            let ok = satker_code.with_value(|required| {
                session
                    .satker_code
                    .as_deref()
                    .map(|s| s == required.as_str())
                    .unwrap_or(false)
            });
            if ok {
                children.with_value(|c| c().into_any())
            } else {
                view! { <ForbiddenPage /> }.into_any()
            }
        }
        None => view! { <RedirectToPerlengkapanLogin /> }.into_any(),
    }
}

// ============================================================================
// SHARED UI — redirect & forbidden pages
// ============================================================================

#[component]
fn RedirectToPerlengkapanLogin() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(routes::path::LOGIN);
        }
    }

    view! {
        <div class="flex min-h-[280px] items-center justify-center rounded-xl border border-slate-700/60 bg-slate-900/80 p-6 text-slate-200">
            "Mengalihkan ke halaman login Perlengkapan..."
        </div>
    }
}

#[component]
fn ForbiddenPage() -> impl IntoView {
    view! {
        <div class="flex min-h-[280px] items-center justify-center rounded-xl border border-red-500/20 bg-slate-900/80 p-6 text-center text-slate-200">
            <div>
                <p class="text-lg font-semibold text-red-300">"Akses Ditolak"</p>
                <p class="mt-2 text-sm text-slate-300">"Anda tidak memiliki izin untuk membuka halaman ini."</p>
                <A href=routes::path::DASHBOARD attr:class="mt-4 inline-flex rounded-lg bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600">
                    "Kembali ke Dashboard"
                </A>
            </div>
        </div>
    }
}
