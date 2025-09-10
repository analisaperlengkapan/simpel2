//! 🚀 SIMPelv2 Shared Component Library
//!
//! Library komponen bersama untuk aplikasi pemerintah Indonesia dengan fokus pada:
//! - Performance tinggi dengan optimasi Rust/WASM
//! - Accessibility standar WCAG 2.1 AA
//! - Government compliance dan branding Kejaksaan RI
//! - Type safety dan developer experience terbaik

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

// Core modules
pub mod components;
pub mod constants;
pub mod theme;
pub mod types;
pub mod utils;

// Optional modules
#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "styles")]
pub mod styles;

// ============================================================================
// PRELUDE - Common imports for easy usage
// ============================================================================

/// Prelude module untuk import yang mudah
pub mod prelude {
    // Re-export Leptos essentials (excluding conflicting items)
    pub use leptos::prelude::*;
    pub use leptos_meta::*;
    pub use leptos_router::*;

    // Re-export our components (excluding utility functions that conflict)
    pub use crate::components::{
        AppHeader, Breadcrumb, Button, CachedRender, Card, Form, Input, LazyLoad, Logo, Modal,
        Toast,
    };
    pub use crate::constants::*;
    pub use crate::theme::*;
    pub use crate::types::*;
    pub use crate::utils::*;

    // Re-export utility functions with specific names to avoid conflicts
    pub use crate::components::{
        event_target_checked as shared_event_target_checked,
        event_target_value as shared_event_target_value, set_timeout as shared_set_timeout,
        window as shared_window,
    };
}

// ============================================================================
// LIBRARY METADATA
// ============================================================================

/// Versi library saat ini
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Informasi library
pub const LIB_INFO: &str = concat!(
    "SIMPelv2 Shared Library v",
    env!("CARGO_PKG_VERSION"),
    " - High-performance components for Indonesian Government"
);

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// Custom error types untuk library
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum SharedError {
        #[error("Validation error: {message}")]
        ValidationError { message: String },

        #[error("Component error: {message}")]
        ComponentError { message: String },

        #[error("Theme error: {message}")]
        ThemeError { message: String },

        #[error("Government data error: {message}")]
        GovernmentDataError { message: String },
    }

    pub type Result<T> = std::result::Result<T, SharedError>;
}
