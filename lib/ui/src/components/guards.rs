//! Session-agnostic route guard primitive (F0-B dedup).
//!
//! Portal and Perlengkapan each had their own `components/guards.rs` with
//! near-duplicate auth/admin/role logic but **different** unauthenticated UX
//! (portal renders `<LoginPage>` inline; perlengkapan redirects to an external
//! login) and **different** `UserSession` types. [`AuthGate`] factors out the
//! shared "show children iff authenticated (and authorized)" logic while keeping
//! the divergent parts as caller-supplied props, so each app preserves its exact
//! behavior:
//!
//! - `authenticated` — reactive auth state (each app derives it from its own session).
//! - `unauthenticated` — view shown when not logged in (inline `<LoginPage>` OR a
//!   redirect component — the app decides).
//! - `authorized` / `forbidden` (optional) — for admin/role/satker gating: when
//!   authenticated but not authorized, render `forbidden` (defaults to
//!   [`ForbiddenPage`]).
//!
//! This is **not** tied to `lib_ui::hooks::use_auth` (the apps don't use that
//! context) — it's pure props, so any session model can wrap it.

use leptos::prelude::*;

/// Two-tier guard: authentication first, then optional authorization.
#[component]
pub fn AuthGate(
    /// Reactive: is the user authenticated?
    #[prop(into)]
    authenticated: Signal<bool>,
    /// View rendered when NOT authenticated (caller supplies inline login or a
    /// redirect component).
    #[prop(into)]
    unauthenticated: ViewFn,
    /// Optional reactive authorization check (e.g. is_admin / has_role / has_satker).
    /// When `None`, authentication alone grants access.
    #[prop(optional, into)]
    authorized: Option<Signal<bool>>,
    /// View rendered when authenticated but NOT authorized. Defaults to
    /// [`ForbiddenPage`].
    #[prop(optional, into)]
    forbidden: Option<ViewFn>,
    /// Protected content.
    children: ChildrenFn,
) -> impl IntoView {
    let forbidden = forbidden.unwrap_or_else(|| ViewFn::from(|| view! { <ForbiddenPage /> }));
    let children = StoredValue::new(children);

    view! {
        <Show when=move || authenticated.get() fallback=move || unauthenticated.run()>
            {
                let forbidden = forbidden.clone();
                // Authenticated. Apply the optional authorization tier.
                view! {
                    <Show
                        when=move || authorized.map(|a| a.get()).unwrap_or(true)
                        fallback=move || forbidden.run()
                    >
                        {children.with_value(|c| c())}
                    </Show>
                }
            }
        </Show>
    }
}

/// Default 403 view (shared; previously portal-only). Apps may pass their own
/// `forbidden` view to [`AuthGate`] instead.
#[component]
pub fn ForbiddenPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="text-center p-8">
                <p class="text-6xl font-bold text-gray-300">"403"</p>
                <h1 class="mt-4 text-2xl font-semibold text-gray-800">"Akses Ditolak"</h1>
                <p class="mt-2 text-gray-600">
                    "Anda tidak memiliki izin untuk mengakses halaman ini."
                </p>
            </div>
        </div>
    }
}
