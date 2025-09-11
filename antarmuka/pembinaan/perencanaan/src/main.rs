// Binary entrypoint for Trunk so that data-bin works correctly.
// For wasm32 we just invoke the main function defined in lib.rs.

#[cfg(target_arch = "wasm32")]
fn main() {
    // main() function in lib.rs is already set to run with wasm_bindgen(start)
    // so we don't need to call anything here explicitly
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This application only runs on wasm32 target");
}
