#![no_main]

use authenc_crypto::keys::{sign_ed25519, verify_ed25519};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Try to sign data with Ed25519
    let signature = sign_ed25519(data);

    // Try to verify the signature
    let _ = verify_ed25519(data, &signature);
});
