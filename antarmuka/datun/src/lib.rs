#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

//! # SIMPelv2 Datun - Perdata dan Tata Usaha Negara
//!
//! Modern Leptos 0.7.8 microfrontend for Kejaksaan RI Civil & Administrative Law management.
//!
//! ## Features
//! - **Case Management**: Bantuan Hukum, Pertimbangan Hukum, Penegakan Hukum
//! - **Legal Services**: Pelayanan Hukum, Tindakan Hukum Lain
//! - **Performance Analytics**: Pemulihan Keuangan Negara tracking
//! - **Government Compliance**: WCAG 2.1 AA accessibility

use wasm_bindgen::prelude::wasm_bindgen;

// Import application modules
pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod types;

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
