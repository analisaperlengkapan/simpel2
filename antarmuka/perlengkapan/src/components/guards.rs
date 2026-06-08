//! Route guards that reflect the JWT-backed session.
//!
//! Thin wrappers over the shared [`lib_ui::components::guards::AuthGate`]
//! primitive (F0-B dedup): `AuthGate` owns the "authenticated → (authorized →
//! children | forbidden) | unauthenticated" skeleton; this module supplies the
//! Perlengkapan-specific bits — session source (`user_session` context or
//! `AuthService::load_session()`), the external-login **redirect** as the
//! unauthenticated view, and the Perlengkapan-styled forbidden page. Behavior is
//! preserved exactly; only the shared skeleton is factored out.
//!
//! ## Layout Guards (recommended)
//! ```rust,ignore
//! <ParentRoute path=path!("/") view=AuthenticatedLayout>
//!     <Route path=path!("/dashboard") view=DashboardHome />
//! </ParentRoute>
//! <ParentRoute path=path!("/admin") view=AdminLayout>
//!     <Route path=path!("/users") view=AdminUsersPage />
//! </ParentRoute>
//! ```

use crate::features::auth::{AuthService, UserSession};
use crate::routes;
use leptos::prelude::*;
use leptos_router::components::A;
use lib_ui::components::guards::AuthGate;

/// Resolve the active session: prefer the reactive context signal (so logout /
/// cross-tab storage events re-evaluate guards), fall back to localStorage.
fn current_session(ctx: Option<ReadSignal<Option<UserSession>>>) -> Option<UserSession> {
    match ctx {
        Some(sig) => sig.get(),
        None => AuthService::load_session(),
    }
}

fn redirect_view() -> ViewFn {
    ViewFn::from(|| view! { <RedirectToPerlengkapanLogin /> })
}

fn forbidden_view() -> ViewFn {
    ViewFn::from(|| view! { <ForbiddenPage /> })
}

// ============================================================================
// LAYOUT GUARDS — wrap all child routes automatically (Next.js / Laravel style)
// ============================================================================

/// Authenticated layout guard. Unauthenticated visitors are redirected to login.
#[component]
pub fn AuthenticatedLayout() -> impl IntoView {
    let ctx = use_context::<ReadSignal<Option<UserSession>>>();
    let authed = Signal::derive(move || current_session(ctx).is_some());

    view! {
        <AuthGate authenticated=authed unauthenticated=redirect_view()>
            <leptos_router::components::Outlet />
        </AuthGate>
    }
}

/// Admin layout guard. Non-admins see a forbidden page; unauthenticated visitors
/// are redirected to login.
#[component]
pub fn AdminLayout() -> impl IntoView {
    let ctx = use_context::<ReadSignal<Option<UserSession>>>();
    let authed = Signal::derive(move || current_session(ctx).is_some());
    let is_admin =
        Signal::derive(move || current_session(ctx).map(|s| s.is_admin()).unwrap_or(false));

    view! {
        <AuthGate
            authenticated=authed
            authorized=is_admin
            unauthenticated=redirect_view()
            forbidden=forbidden_view()
        >
            <leptos_router::components::Outlet />
        </AuthGate>
    }
}

// ============================================================================
// INLINE GUARDS — per-route wrappers
// ============================================================================

#[component]
pub fn SessionAuthGuard(
    user_session: ReadSignal<Option<UserSession>>,
    children: ChildrenFn,
) -> impl IntoView {
    let _ = user_session;
    let children = StoredValue::new_local(children);
    let authed = Signal::derive(move || AuthService::load_session().is_some());

    view! {
        <AuthGate authenticated=authed unauthenticated=redirect_view()>
            {move || children.with_value(|c| c())}
        </AuthGate>
    }
}

#[component]
pub fn SessionAdminGuard(
    user_session: ReadSignal<Option<UserSession>>,
    children: ChildrenFn,
) -> impl IntoView {
    let _ = user_session;
    let children = StoredValue::new_local(children);
    let authed = Signal::derive(move || AuthService::load_session().is_some());
    let is_admin = Signal::derive(move || {
        AuthService::load_session()
            .map(|s| s.is_admin())
            .unwrap_or(false)
    });

    view! {
        <AuthGate
            authenticated=authed
            authorized=is_admin
            unauthenticated=redirect_view()
            forbidden=forbidden_view()
        >
            {move || children.with_value(|c| c())}
        </AuthGate>
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
    let authed = Signal::derive(move || AuthService::load_session().is_some());
    let allowed = Signal::derive(move || {
        AuthService::load_session()
            .map(|s| roles.with_value(|req| req.iter().any(|r| s.has_role(r.as_str()))))
            .unwrap_or(false)
    });

    view! {
        <AuthGate
            authenticated=authed
            authorized=allowed
            unauthenticated=redirect_view()
            forbidden=forbidden_view()
        >
            {move || children.with_value(|c| c())}
        </AuthGate>
    }
}

/// Gate UI to a specific satker. Admins / validator pusat bypass the check so
/// pusat users can inspect any satker.
#[component]
pub fn SatkerGuard(#[prop(into)] satker_code: String, children: ChildrenFn) -> impl IntoView {
    let satker_code = StoredValue::new(satker_code);
    let children = StoredValue::new_local(children);
    let authed = Signal::derive(move || AuthService::load_session().is_some());
    let allowed = Signal::derive(move || {
        AuthService::load_session()
            .map(|s| {
                if s.is_admin() || s.is_validator_pusat() {
                    return true;
                }
                satker_code.with_value(|req| {
                    s.satker_code
                        .as_deref()
                        .map(|c| c == req.as_str())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    view! {
        <AuthGate
            authenticated=authed
            authorized=allowed
            unauthenticated=redirect_view()
            forbidden=forbidden_view()
        >
            {move || children.with_value(|c| c())}
        </AuthGate>
    }
}

// ============================================================================
// SHARED UI — Perlengkapan-specific redirect & forbidden pages (unchanged UX)
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
