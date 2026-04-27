//! Future home of `<AppShell>` — phase 8e.
//!
//! When the workspace build is healthy enough to verify it, this
//! module will export a single component that consolidates every
//! provider each app currently mounts by hand at the top of its
//! `App()` function:
//!
//! - [`tracing_subscriber_wasm`] init (phase 1)
//! - [`leptos_fetch::QueryClient::new().provide()`] (phase 6)
//! - [`crate::hooks::use_toast::ToastProvider`] (phase 7, upgraded
//!   with aria-live + pause-on-hover + optional progress bar)
//! - [`crate::components::custom_branding::BrandingProvider`]
//! - Session monitor + cross-tab sync (app-specific, opt-in)
//! - `config.json` runtime loader (app-specific, opt-in)
//!
//! Expected API:
//!
//! ```ignore
//! use lib_ui::prelude::*;
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     view! {
//!         <AppShell unit="portal">
//!             <Router base="/portal">
//!                 <Routes fallback=|| view! { <NotFoundPage /> }>
//!                     // route registration only — no provider plumbing
//!                 </Routes>
//!             </Router>
//!         </AppShell>
//!     }
//! }
//! ```
//!
//! Once landed, [`antarmuka/portal/src/app.rs`] and
//! [`antarmuka/perlengkapan/src/lib.rs`] both shrink to under
//! ~60 LoC, with route registration as the bulk of the remaining
//! code.
//!
//! Implementation is deferred until the 62 pre-existing errors in
//! `use_form.rs`, `error_boundary.rs`, and `shamir.rs` clear — see
//! `.claude/plans/coba-kritisi-uraian-berikut-pure-rossum.md`.
