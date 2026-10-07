#![recursion_limit = "256"]
//! # SIMPEL Portal — Entry Point

use lib_ui::utils::console_writer::ConsoleMakeWriter;
use portal_microfrontend::app::App;
use tracing_subscriber::fmt;

fn main() {
    console_error_panic_hook::set_once();

    fmt()
        .with_writer(ConsoleMakeWriter)
        .with_ansi(false)
        .without_time()
        .init();

    tracing::info!("🚀 Portal SIMPEL starting...");

    leptos::mount::mount_to_body(App);
}
