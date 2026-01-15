#![recursion_limit = "1024"]

//! Portal SIMPelv2 - Main Entry Point
//!
//! This is the main entry point for the portal application.
//! For CSR (Client-Side Rendering) with Trunk, this file is optional
//! as the WASM module is loaded directly from index.html.
//!
//! However, we include it for:
//! - Consistency with Rust project structure
//! - Potential SSR (Server-Side Rendering) support in the future
//! - Better IDE support and tooling

fn main() {
    // This main function is not used in CSR mode with Trunk
    // The actual entry point is in lib.rs with #[wasm_bindgen(start)]
    // or through Trunk's automatic WASM initialization

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("This application is designed to run in a web browser via WebAssembly.");
        eprintln!("Please use 'trunk serve' to run the development server.");
        std::process::exit(1);
    }

    #[cfg(target_arch = "wasm32")]
    {
        // For WASM target, mount the Leptos app
        console_error_panic_hook::set_once();
        leptos::mount::mount_to_body(portal_microfrontend::App);
    }
}
