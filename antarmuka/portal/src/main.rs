#![recursion_limit = "256"]
//! # SIMPEL Portal — Entry Point

use portal_microfrontend::app::App;

fn main() {
    // Set up panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // Initialize logging
    let _ = console_log::init_with_level(log::Level::Debug);

    log::info!("🚀 Portal SIMPEL starting...");

    leptos::mount::mount_to_body(App);
}
