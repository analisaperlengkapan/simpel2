//! # SIMPelv2 Datun - Criminal Prosecution System
//!
//! Modern Leptos 0.7.8 microfrontend for Kejaksaan RI criminal prosecution management.
//!
//! ## Features
//! - **Case Management**: Comprehensive criminal case tracking and management
//! - **Investigation Support**: Tools for investigation coordination and evidence tracking
//! - **Prosecution Planning**: Strategic prosecution planning and legal analysis
//! - **Government Compliance**: WCAG 2.1 AA accessibility and security standards
//! - **Performance Analytics**: Case resolution metrics and prosecution effectiveness

// Modern Leptos imports for 0.7.8
use wasm_bindgen::prelude::wasm_bindgen;

// Import application modules
pub mod app;
pub mod components;

// Re-export the main App component
pub use app::App;

/// Entry point for client-side rendering (CSR)
#[wasm_bindgen(start)]
pub fn hydrate() {
    // Enable better error messages in debug mode
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Mount the Leptos application
    leptos::mount::hydrate_body(App);
}
