#![recursion_limit = "512"]
//! # Perlengkapan Microfrontend — Entry Point

use lib_ui::utils::console_writer::ConsoleMakeWriter;
use perlengkapan_microfrontend::App;
use tracing_subscriber::fmt;

fn main() {
    console_error_panic_hook::set_once();

    fmt()
        .with_writer(ConsoleMakeWriter)
        .with_ansi(false)
        .without_time()
        .init();

    tracing::info!("🚀 Perlengkapan Microfrontend starting...");

    leptos::mount::mount_to_body(App);
}
