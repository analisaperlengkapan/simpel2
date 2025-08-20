//! # SIMPelv2 Portal Utama - Gateway to Justice Technology
//!
//! Portal Utama sistem SIMPelv2 yang menyediakan:
//! - **Dashboard Terpadu**: Overview semua layanan kejaksaan
//! - **SSO Integration**: Single Sign-On dengan sistem pemerintahan
//! - **Microfrontend Router**: Gateway ke semua aplikasi SIMPelv2
//! - **Government Compliance**: Sesuai standar keamanan siber nasional
//! - **Responsive Design**: Optimized untuk semua device
//!
//! ## Architecture
//! Portal menggunakan Leptos 0.7.8 dengan pattern:
//! - **Component-Based**: Reusable UI components dari shared library
//! - **Type-Safe Routing**: Leptos Router dengan compile-time checks
//! - **Performance First**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//!
//! ## Usage
//! ```rust
//! use portal_microfrontend::App;
//! // leptos::mount::mount_to_body(App);
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]
#![forbid(unsafe_code)]

pub mod app;
pub mod components;

// Re-exports untuk kemudahan penggunaan
pub use app::*;
pub use components::*;

/// Prelude module untuk import yang sering digunakan
pub mod prelude {
    pub use crate::app::*;
    pub use crate::components::*;
    pub use leptos::prelude::*;
    pub use leptos_router::*;
    pub use shared_microfrontend::prelude::*;
}
