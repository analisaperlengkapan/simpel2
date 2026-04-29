#![recursion_limit = "512"]
//! # Perlengkapan Microfrontend — Entry Point

use perlengkapan_microfrontend::App;
use tracing_subscriber::fmt;
use tracing_subscriber_wasm::MakeConsoleWriter;

fn main() {
    console_error_panic_hook::set_once();

    fmt()
        .with_writer(MakeConsoleWriter::default())
        .with_ansi(false)
        .without_time()
        .init();

    tracing::info!("🚀 Perlengkapan Microfrontend starting...");

    leptos::mount::mount_to_body(App);
}
