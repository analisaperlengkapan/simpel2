#![no_main]

use authenc::crypto::aes_gcm::AesGcmService;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let service = AesGcmService::new();

    // Test encryption
    if let Ok(encrypted) = service.encrypt(data) {
        // Test decryption
        let _ = service.decrypt(&encrypted);
    }
});
