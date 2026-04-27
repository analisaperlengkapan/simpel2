//! Future home of `<RouteSuspense>` — phase 8b.
//!
//! When the workspace build is healthy enough to verify it, this
//! module will export a single component that bundles the standard
//! `<Suspense fallback=<LoadingPanel/>><ErrorBoundary
//! fallback=<ErrorPanel/>>...` pattern every page currently rewrites
//! by hand.
//!
//! The expected API:
//!
//! ```ignore
//! use lib_ui::prelude::*;
//! use leptos_fetch::use_query;
//!
//! #[component]
//! fn UserDetail(id: ReadSignal<String>) -> impl IntoView {
//!     let user = use_query(...);
//!     view! {
//!         <RouteSuspense>
//!             {move || user.data().map(|u| view! { <UserCard data=u /> })}
//!         </RouteSuspense>
//!     }
//! }
//! ```
//!
//! It composes cleanly with phase 6's `leptos-fetch` queries and
//! re-uses [`crate::components::error_boundary::LoadingPanel`] /
//! [`crate::components::error_boundary::ErrorPanel`], which already
//! pull their icons from `phosphor-leptos` (phase 3).
//!
//! Implementation is deferred until the 62 pre-existing errors in
//! `use_form.rs`, `error_boundary.rs`, and `shamir.rs` clear — see
//! `.claude/plans/coba-kritisi-uraian-berikut-pure-rossum.md`.
