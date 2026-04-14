use crate::features::auth::{AuthService, UserSession};
use crate::routes;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn SessionAuthGuard(
    user_session: ReadSignal<Option<UserSession>>,
    children: ChildrenFn,
) -> impl IntoView {
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
    let children = StoredValue::new_local(children);

    move || match AuthService::load_session() {
        Some(session) if session.is_admin() => children.with_value(|c| c().into_any()),
        Some(_) => view! { <ForbiddenPage /> }.into_any(),
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
                <p class="mt-2 text-sm text-slate-300">"Halaman ini hanya untuk admin."</p>
                <A href=routes::path::DASHBOARD attr:class="mt-4 inline-flex rounded-lg bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600">
                    "Kembali ke Dashboard"
                </A>
            </div>
        </div>
    }
}
