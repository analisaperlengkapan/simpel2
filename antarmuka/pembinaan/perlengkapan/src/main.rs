// Binary entrypoint for Trunk so that data-bin works correctly.
// For wasm32 we just invoke the start function defined in lib.rs.

#[cfg(target_arch = "wasm32")]
use perlengkapan_microfrontend::start_app;

#[cfg(target_arch = "wasm32")]
fn main() {
    start_app();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This application only runs on wasm32 target");
}
