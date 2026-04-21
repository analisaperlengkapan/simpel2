//! Route guards that reflect the JWT-backed session.
//!
//! The old guards read `user_session` as a prop. That prop is still accepted
//! for backwards-compat with call sites in `lib.rs`, but the source of truth
//! is now `AuthService::load_session()` which re-decodes the JWT each call —
//! so guards stay honest even if a tab went stale in the background.

use crate::features::auth::{AuthService, UserSession};
use crate::routes;
use leptos::prelude::*;
use leptos_router::components::A;

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
            let allowed = roles.with_value(|required| {
                required.iter().any(|r| session.has_role(r.as_str()))
            });
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
pub fn SatkerGuard(
    #[prop(into)] satker_code: String,
    children: ChildrenFn,
) -> impl IntoView {
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
