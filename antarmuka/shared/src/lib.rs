//! # Kejaksaan Shared Components Library
//!
//! **Optimized** shared library untuk sistem informasi Kejaksaan Agung RI (SIMPelv2).
//!
//! ## 🎯 **Design Philosophy**
//! - **Performance First**: Optimized for fast compilation and runtime
//! - **Type Safety**: Comprehensive type system untuk government-grade reliability  
//! - **Accessibility**: WCAG 2.1 AA compliant components
//! - **Consistent Branding**: Unified Kejaksaan RI visual identity
//!
//! ## 📦 **Core Modules**
//! - [`components`] - UI components dengan branding Kejaksaan
//! - [`types`] - Type definitions dan data structures
//! - [`utils`] - Helper functions dan utilities
//! - [`constants`] - System constants dan konfigurasi
//! - [`theme`] - Visual styling dan themes
//! - [`api`] - HTTP client dan service layer
//!
//! ## 🚀 **Quick Start**
//! ```rust
//! use shared_microfrontend::prelude::*;
//!
//! #[component]  
//! fn MyApp() -> impl IntoView {
//!     view! {
//!         <AppHeader title="My Application" />
//!         <main class="container">
//!             <p>"Content goes here"</p>
//!         </main>
//!         <KejaksaanFooter />
//!     }
//! }
//! ```
//!
//! ## 🛡️ **Standards & Compliance**
//! - **Security**: Government security standards
//! - **Performance**: Sub-second load times
//! - **Accessibility**: Screen reader compatible
//! - **Responsive**: Mobile-first design approach

// ============================================================================
// CORE MODULES - Organized by functionality
// ============================================================================

pub mod api;
pub mod components;
pub mod constants;
#[cfg(feature = "serde")]
pub mod styles;
pub mod theme;
pub mod types;
pub mod utils;

// ============================================================================
// PRELUDE - Commonly used imports for convenience
// ============================================================================

/// Common imports for typical usage
pub mod prelude {
    // Core components - most frequently used
    // === UI Components - Modern Leptos 0.7.8 components ===
    pub use crate::components::{
        KejBreadcrumb, KejButton, KejCard, KejInput, KejModal, KejNotification, KejSpinner,
        KejTable,
    }; // Essential types
    pub use crate::types::{ButtonVariant, LoadingState, MessageType, NavItem};

    // Key utilities
    // === Utilities - Core helper functions ===
    pub use crate::utils::{
        check_permission, format_currency, format_date_id, generate_component_id, validate_nip,
    };

    // Theme and styling
    pub use crate::constants::KejaksaanColors;
    pub use crate::theme::{apply_unit_theme, KejaksaanTheme};
}

// ============================================================================
// SELECTIVE RE-EXPORTS - Only essential items at root level
// ============================================================================

// Core UI Components (most commonly used)
pub use components::{
    KejBreadcrumb, KejButton, KejCard, KejInput, KejModal, KejNotification, KejSpinner, KejTable,
}; // Essential Types
pub use types::{ApiResponse, ButtonVariant, LoadingState, MessageType, NavItem, User, UserRole};

// API Layer
pub use api::{ApiClient, ApiError};

// ============================================================================
// MODULE RE-EXPORTS - For advanced usage
// ============================================================================

/// All component types and functions
pub mod component_types {
    pub use crate::components::*;
}

/// All utility functions organized by category
pub mod utilities {
    pub use crate::utils::*;
}

/// All type definitions
pub mod type_definitions {
    pub use crate::types::*;
}

/// System constants and configurations
pub mod system_constants {
    pub use crate::constants::*;
}

/// Theming and styling utilities  
pub mod styling {
    #[cfg(feature = "serde")]
    pub use crate::styles::*;
    pub use crate::theme::*;
}
