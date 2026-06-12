//! `<AppShell>` — consolidates per-app provider plumbing.
//!
//! Mounts the providers every microfrontend would otherwise have to
//! repeat at the top of `App()`:
//!
//! - [`leptos_fetch::QueryClient`] — phase 6 cache layer.
//! - [`crate::hooks::use_toast::ToastProvider`] — upgraded toast system
//!   from phase 7 (aria-live, pause-on-hover, optional progress bar).
//!
//! Tracing-subscriber init stays in each app's `main.rs` because it
//! must run before any Leptos code, and `BrandingProvider` plus
//! session-monitor wiring stay in the App because their config is
//! per-app.
//!
//! ```ignore
//! use lib_ui::prelude::*;
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     view! {
//!         <AppShell>
//!             <Router base="/portal">
//!                 <Routes fallback=|| view! { <NotFoundPage /> }>
//!                     /* route registration only */
//!                 </Routes>
//!             </Router>
//!         </AppShell>
//!     }
//! }
//! ```

use crate::hooks::use_toast::ToastProvider;
use leptos::prelude::*;
use leptos_fetch::QueryClient;

#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    // Mount the leptos-fetch cache so descendants can opt into
    // `use_query` / `client.local_resource` / `client.resource`.
    QueryClient::new().provide();

    view! { <ToastProvider>{children()}</ToastProvider> }
}
