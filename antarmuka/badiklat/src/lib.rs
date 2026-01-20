#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

//! # SIMPelv2 Badiklat - Training & Education System
//!
//! Modern Leptos 0.7.8 microfrontend for Kejaksaan RI training management.
//!
//! ## Features
//! - **Training Program Management**: Comprehensive program catalog and enrollment
//! - **Digital Learning Platform**: Modern e-learning with progress tracking
//! - **Competency Framework**: Skills assessment and certification tracking
//! - **Government Compliance**: WCAG 2.1 AA accessibility and security standards
//! - **Performance Analytics**: Training effectiveness and participant metrics

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
