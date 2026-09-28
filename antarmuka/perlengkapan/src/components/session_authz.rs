//! Reactive authorization derived from the JWT-backed session.
//!
//! # Replaces the role switcher
//!
//! Perlengkapan used to ship a "Ganti Role" dropdown that let any authenticated
//! user select `admin`, `validator_pusat`, or any other role from the profile
//! menu. The pick was persisted to `localStorage` (`ui_active_role`) and then
//! consulted by the sidebar (which section to show), the helpdesk (whether to
//! render staff triage), and the dashboard (which scope wording to use).
//!
//! The server never honoured it — every scope is re-derived from the bearer
//! token — so it was not privilege *escalation*. It was something arguably
//! worse for an operator: the UI asserted an identity the user does not hold.
//! An operator could select "Admin", watch the admin menu appear, and then meet
//! `403` on every link. Worse in the other direction, an admin could select
//! "Operator Satker" and be told they had no permission.
//!
//! Authorization here is now a pure projection of the session, which is itself
//! a projection of the JWT. There is no writable role.
//!
//! # Usage
//!
//! ```rust,ignore
//! let authz = use_authz();
//! view! {
//!     <Show when=move || authz.get().can(Capability::Administer)>
//!         <AdminPanel />
//!     </Show>
//! }
//! ```

use crate::features::auth::UserSession;
use leptos::prelude::*;
use lib_core::authz::Authorization;

/// The current session signal, if the app root provided one.
pub fn use_session() -> Option<ReadSignal<Option<UserSession>>> {
    use_context::<ReadSignal<Option<UserSession>>>()
}

/// The caller's authorization, reactive to login/logout and to cross-tab
/// `storage` events (which update the session signal).
///
/// An absent session resolves to [`Authorization::default`] — no roles, no
/// capabilities — so a surface gated on a capability fails closed while the
/// session is still loading rather than flashing privileged content.
pub fn use_authz() -> Signal<Authorization> {
    let session = use_session();
    Signal::derive(move || {
        session
            .and_then(|s| s.get())
            .map(|s| s.authz())
            .unwrap_or_default()
    })
}
