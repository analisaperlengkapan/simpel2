//! Shared route helpers for portal app.

use crate::features::auth::UserSession;
use crate::components::guards::{SessionAdminGuard, SessionAuthGuard};
use crate::pages::LoginPage;
use crate::routes;
use leptos::prelude::*;

/// Shared renderer for layout-based protected pages.
pub fn render_session_layout_page<F>(
    user_session: ReadSignal<Option<UserSession>>,
    on_login_success: WriteSignal<Option<UserSession>>,
    make_logout: &dyn Fn() -> Box<dyn Fn()>,
    page: F,
) -> AnyView
where
    F: Fn(UserSession, Box<dyn Fn()>) -> AnyView,
{
    match user_session.get() {
        Some(session) if session.require_password_change => {
            let nav = leptos_router::hooks::use_navigate();
            nav(&format!("/{}", routes::segment::PASSWORD), Default::default());
            view! { <div /> }.into_any()
        }
        Some(session) => page(session, make_logout()),
        None => view! { <LoginPage on_login_success=on_login_success /> }.into_any(),
    }
}

/// Shared renderer for session-authenticated pages (non-layout routes).
pub fn render_session_auth_page<F>(
    user_session: ReadSignal<Option<UserSession>>,
    on_login_success: WriteSignal<Option<UserSession>>,
    allow_password_change: bool,
    page: F,
) -> AnyView
where
    F: Fn() -> AnyView + Send + Sync + 'static,
{
    view! {
        <SessionAuthGuard
            user_session=user_session
            on_login_success=on_login_success
            allow_password_change=allow_password_change
        >
            {page()}
        </SessionAuthGuard>
    }
    .into_any()
}

/// Shared renderer for session-authenticated admin pages.
pub fn render_session_admin_page<F>(
    user_session: ReadSignal<Option<UserSession>>,
    on_login_success: WriteSignal<Option<UserSession>>,
    page: F,
) -> AnyView
where
    F: Fn() -> AnyView + Send + Sync + 'static,
{
    view! {
        <SessionAdminGuard user_session=user_session on_login_success=on_login_success>
            {page()}
        </SessionAdminGuard>
    }
    .into_any()
}
