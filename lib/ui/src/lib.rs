//! 🚀 SIMPEL Shared Component Library
//!
//! Clean, focused, production-ready component library untuk aplikasi Kejaksaan RI.
//!
//! ## Philosophy
//! - **KISS**: Keep It Simple, Stupid
//! - **YAGNI**: You Aren't Gonna Need It
//! - **DRY**: Don't Repeat Yourself
//! - **Production-First**: Battle-tested components only
//!
//! ## Structure
//! ```text
//! shared/
//! |-- core/          # Types, constants, theme
//! |-- components/    # UI components (layout, forms, feedback, navigation, display)
//! |-- hooks/         # Domain hooks (auth, toast, form, search, announcer)
//! `-- utils/         # Utilities (validation, formatters, helpers, storage)
//! ```
//!
//! Generic primitives (debounce, throttle, storage, media query, keyboard events)
//! come from `leptos_use` — re-exported via `lib_ui::prelude`.

// Allow clippy warnings for common patterns in this crate
#![allow(clippy::collapsible_if)]
#![allow(clippy::useless_vec)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

pub mod components;
pub mod core;
pub mod hooks;
pub mod routes;
pub mod utils;

// ============================================================================
// PRELUDE - Ergonomic imports
// ============================================================================

/// Prelude module untuk import yang mudah
#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    // Leptos essentials
    pub use leptos::prelude::*;
    pub use leptos_meta::*;
    pub use leptos_router::*;

    // Core
    pub use crate::core::constants::*;
    pub use crate::core::theme::*;
    pub use crate::core::types::*;

    // Components
    pub use crate::components::captcha::*;
    pub use crate::components::display::*;
    pub use crate::components::feedback::*;
    pub use crate::components::forms::*;
    pub use crate::components::icon::*;
    pub use crate::components::layout::*;
    pub use crate::components::navigation::*;

    // Icon catalogue — typed phosphor constants + weight enum
    pub use phosphor_leptos;

    // Hooks (domain-specific)
    pub use crate::hooks::*;

    // Typed routes (Phase 8a — pure-data layer)
    pub use crate::routes::{PerlengkapanRoute, PortalRoute, ToPath};

    // Generic reactive primitives from leptos-use
    pub use leptos_use::storage::{
        UseStorageOptions, use_local_storage, use_local_storage_with_options, use_session_storage,
        use_session_storage_with_options,
    };
    pub use leptos_use::{
        on_click_outside, use_debounce_fn, use_event_listener, use_interval_fn, use_media_query,
        use_throttle_fn, use_window_focus,
    };

    // Storage codec for use with `use_local_storage` / `use_session_storage`
    pub use codee::string::JsonSerdeCodec;

    // Utils
    pub use crate::utils::*;
}

// ============================================================================
// LIBRARY METADATA
// ============================================================================

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library info
pub const INFO: &str = concat!(
    "SIMPEL Shared Library v",
    env!("CARGO_PKG_VERSION"),
    " - Production-ready components for Kejaksaan RI"
);

// ============================================================================
// RE-EXPORTS (Top-level convenience)
// ============================================================================

// Core exports
pub use core::{constants, theme, types as core_types};

// Component exports
pub use components::{
    accessibility, auth, captcha, display, error_boundary, feedback, forms, layout,
    monitoring_dashboard, navigation, security_meta,
};

// Hook exports
pub use hooks::*;

// Utility exports (specific to avoid conflicts)
pub use utils::{
    analytics, caching, code_splitting, csrf, error_tracking, formatters, helpers, monitoring,
    secure_storage, security, validation,
};
