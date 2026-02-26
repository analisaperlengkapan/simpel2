#![recursion_limit = "512"]
//! # Perlengkapan Microfrontend — Entry Point

use perlengkapan_microfrontend::App;

fn main() {
    // Set up panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // Initialize logging
    let _ = console_log::init_with_level(log::Level::Debug);

    log::info!("🚀 Perlengkapan Microfrontend starting...");

    // Mount the app to the body
    leptos::mount::mount_to_body(App);
}
