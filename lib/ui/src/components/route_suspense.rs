//! `<RouteSuspense>` — declarative loading + error wrapper.
//!
//! Bundles the standard
//! `<Suspense fallback=…><ErrorBoundary fallback=…>…` triple every
//! page used to repeat by hand. Uses
//! [`crate::components::error_boundary::LoadingPanel`] and
//! [`crate::components::error_boundary::ErrorPanel`] under the
//! hood so the Loading + Error visuals match the rest of the app.
//!
//! Pairs naturally with phase 6's `leptos-fetch` queries — the
//! resource is awaited inside the inner `<Suspense>`, and any
//! panic / error inside the children bubbles up to the outer
//! `<ErrorBoundary>`.
//!
//! ```ignore
//! use lib_ui::prelude::*;
//!
//! #[component]
//! fn UserDetail(id: ReadSignal<String>) -> impl IntoView {
//!     let client: leptos_fetch::QueryClient = expect_context();
//!     let user = client.local_resource(get_user, move || id.get());
//!     view! {
//!         <RouteSuspense loading_message="Memuat profil…">
//!             {move || user.get().map(|u| view! { <UserCard data=u /> })}
//!         </RouteSuspense>
//!     }
//! }
//! ```

use crate::components::error_boundary::{ErrorPanel, LoadingPanel};
use leptos::prelude::*;

#[component]
pub fn RouteSuspense(
    /// Message shown while the underlying resource is pending.
    #[prop(default = "Memuat data...".to_string(), into)]
    loading_message: String,
    /// Title shown when an error bubbles up to the outer
    /// `<ErrorBoundary>`. The error message itself is rendered
    /// inside `ErrorPanel`.
    #[prop(default = "Terjadi Kesalahan".to_string(), into)]
    error_title: String,
    /// The view function — typically returns a `view!` that reads
    /// from a `Resource` and may `.await` it inside a `Suspend`.
    children: ChildrenFn,
) -> impl IntoView {
    let error_title = StoredValue::new_local(error_title);
    let loading_message = StoredValue::new_local(loading_message);

    let fallback = move |errors: ArcRwSignal<Errors>| {
        let title = error_title.with_value(|t| t.clone());
        let message = errors
            .get_untracked()
            .iter()
            .next()
            .map(|(_, e)| e.to_string())
            .unwrap_or_default();
        view! { <ErrorPanel title=title message=message /> }
    };

    let suspense_fallback = move || {
        let msg = loading_message.with_value(|m| m.clone());
        view! { <LoadingPanel message=msg /> }
    };

    view! {
        <ErrorBoundary fallback=fallback>
            <Suspense fallback=suspense_fallback>
                {children()}
            </Suspense>
        </ErrorBoundary>
    }
}
