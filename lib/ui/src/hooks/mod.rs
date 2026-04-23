//! Reusable React-style hooks untuk Leptos.
//!
//! # Ekosistem `leptos-use`
//!
//! Modul ini membungkus dan menambah hook-hook kustom di atas
//! [`leptos_use`](https://leptos-use.rs). Seluruh ekosistem leptos-use
//! (90+ hook: `use_clipboard`, `use_scroll`, `use_resize_observer`,
//! `use_element_visibility`, `use_event_listener`, dsb.) tersedia lewat
//! re-export di bawah — downstream **tidak perlu** menambahkan
//! dependency `leptos-use` sendiri.
//!
//! ```rust,ignore
//! use lib_ui::hooks::leptos_use::use_clipboard;
//! use lib_ui::hooks::{is_mobile, use_debounce_fn};
//! ```

pub mod use_announcer;
pub mod use_auth;
pub mod use_debounce;
pub mod use_form;
pub mod use_keyboard;
pub mod use_media_query;
pub mod use_notifications;
pub mod use_search;
pub mod use_storage;
pub mod use_toast;

pub use use_announcer::*;
pub use use_auth::*;
pub use use_debounce::*;
pub use use_form::*;
pub use use_keyboard::*;
pub use use_media_query::*;
pub use use_notifications::*;
pub use use_search::*;
pub use use_storage::*;
pub use use_toast::*;

// Re-export seluruh ekosistem leptos-use. Akses via
// `lib_ui::hooks::leptos_use::*` untuk hook-hook yang belum kita bungkus
// (clipboard, scroll, resize observer, element visibility, dll).
pub use ::leptos_use;
