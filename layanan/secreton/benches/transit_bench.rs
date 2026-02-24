use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use secreton_crypto::transit::keys::{KeyOptions, KeyType, TransitKey};

/// Benchmark transit engine encryption operations
fn bench_transit_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("transit_encrypt");

    // Test different key types
    let key_types = vec![
        ("aes256-gcm", KeyType::Aes256Gcm),
        ("chacha20", KeyType::ChaCha20Poly1305),
        ("xchacha20", KeyType::XChaCha20Poly1305),
    ];

    // Test different data sizes
    let data_sizes = vec![(128, "128B"), (1024, "1KB"), (10240, "10KB")];

    for (key_name, key_type) in &key_types {
        for (size, size_name) in &data_sizes {
            let key = TransitKey::new(
                format!("bench-{}", key_name),
                key_type.clone(),
                KeyOptions::default(),
            )
            .unwrap();

            let plaintext = vec![0u8; *size];

            group.throughput(Throughput::Bytes(*size as u64));
            group.bench_with_input(
                BenchmarkId::new(*key_name, size_name),
                &plaintext,
                |b, data| {
                    b.iter(|| {
                        let encrypted = key.encrypt(black_box(data), None, None).unwrap();
                        black_box(encrypted)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark transit engine decryption operations
fn bench_transit_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("transit_decrypt");

    let key = TransitKey::new(
        "bench-decrypt".to_string(),
        KeyType::Aes256Gcm,
        KeyOptions::default(),
    )
    .unwrap();

    let data_sizes = vec![(128, "128B"), (1024, "1KB"), (10240, "10KB")];

    for (size, size_name) in &data_sizes {
        let plaintext = vec![0u8; *size];
        let ciphertext = key.encrypt(&plaintext, None, None).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size_name),
            &ciphertext,
            |b, ct| {
                b.iter(|| {
                    let decrypted = key.decrypt(black_box(ct), None).unwrap();
                    black_box(decrypted)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark signature operations
fn bench_transit_sign(c: &mut Criterion) {
    let mut group = c.benchmark_group("transit_sign");

    let sign_key_types = vec![
        ("ed25519", KeyType::Ed25519),
        ("ecdsa-p256", KeyType::EcdsaP256),
        ("ecdsa-secp256k1", KeyType::EcdsaSecp256k1),
    ];

    let message = b"The quick brown fox jumps over the lazy dog";

    for (key_name, key_type) in &sign_key_types {
        let mut key_options = KeyOptions::default();
        key_options.usage = vec![
            secreton_crypto::transit::keys::KeyUsage::Sign,
            secreton_crypto::transit::keys::KeyUsage::Verify,
        ];

        let key =
            TransitKey::new(format!("sign-{}", key_name), key_type.clone(), key_options).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(key_name), &key, |b, k| {
            b.iter(|| {
                let signature = k.sign(black_box(message), None, None).unwrap();
                black_box(signature)
            })
        });
    }

    group.finish();
}

/// Benchmark key derivation
fn bench_key_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_derivation");

    let mut key_options = KeyOptions::default();
    key_options.usage = vec![secreton_crypto::transit::keys::KeyUsage::Derive];

    let key = TransitKey::new("kdf-key".to_string(), KeyType::Aes256Gcm, key_options).unwrap();

    let derived_key_sizes = vec![16, 32, 64];

    for size in &derived_key_sizes {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, s| {
            b.iter(|| {
                let derived = key.derive_key(b"context", *s).unwrap();
                black_box(derived)
            })
        });
    }

    group.finish();
}

criterion_group!(
    transit_benches,
    bench_transit_encrypt,
    bench_transit_decrypt,
    bench_transit_sign,
    bench_key_derivation
);

criterion_main!(transit_benches);
