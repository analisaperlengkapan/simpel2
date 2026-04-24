//! Reusable React-style hooks for Leptos.
//!
//! Primitives for debounce, throttle, local/session storage, media queries,
//! keyboard events, and similar concerns live in `leptos_use` — re-exported
//! from `lib_ui::prelude`. This module hosts only hooks with domain-specific
//! behavior (auth, toast, form, search, announcer, notifications).

pub mod use_announcer;
pub mod use_auth;
pub mod use_form;
pub mod use_notifications;
pub mod use_search;
pub mod use_toast;

pub use use_announcer::*;
pub use use_auth::*;
pub use use_form::*;
pub use use_notifications::*;
pub use use_search::*;
pub use use_toast::*;
