//! Secreton Performance Benchmarks
//!
//! This module contains comprehensive performance benchmarks for secreton functionality,
//! focusing on the enhanced features for SIMKARI super app integration with authenc.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;
use serde_json::json;

use secreton_core::engines::EnhancedSecretEngine;
use secreton_core::auth::AuthencAuthProvider;
use secreton_core::models::{Secret, AuditEvent, AccessControl};
use secreton_core::crypto::HybridCrypto;
use secreton_core::config::SecretonConfig;

/// Benchmark secret retrieval based on role permissions
fn bench_secret_retrieval_by_role(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config();
    let secret_engine = rt.block_on(async {
        EnhancedSecretEngine::new(&config).await.unwrap()
    });

    // Create test secrets for different satker and roles
    let test_secrets = create_benchmark_secrets(1000);
    rt.block_on(async {
        for secret in &test_secrets {
            let _ = secret_engine.store_secret(secret).await;
        }
    });

    let test_tokens = create_benchmark_tokens(100);

    let mut group = c.benchmark_group("secret_retrieval_by_role");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark single secret retrieval
    for role_type in ["SecretonUser", "SecretonAdmin", "SecretonAuditor"].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_secret_retrieval", role_type),
            role_type,
            |b, &role_type| {
                let role_tokens: Vec<_> = test_tokens.iter()
                    .filter(|(_, token_role)| token_role == role_type)
                    .collect();

                b.to_async(&rt).iter(|| async {
                    for (token, _) in &role_tokens {
                        let secret_path = "secrets/KEJATI_DKI_JAKPUS/config";
                        let _result = secret_engine.get_secret_with_auth(token, secret_path).await;
                        black_box(_result);
                    }
                });
            },
        );
    }

    // Benchmark batch secret retrieval
    group.bench_function("batch_secret_retrieval", |b| {
        let admin_token = test_tokens.iter()
            .find(|(_, role)| role == "SecretonAdmin")
            .map(|(token, _)| token)
            .unwrap();

        let secret_paths: Vec<String> = test_secrets.iter()
            .take(50)
            .map(|s| s.path.clone())
            .collect();

        b.to_async(&rt).iter(|| async {
            let _results = secret_engine.batch_get_secrets_with_auth(admin_token, &secret_paths).await;
            black_box(_results);
        });
    });

    // Benchmark role-based filtering
    group.bench_function("role_based_filtering", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, role) in &test_tokens {
                let _filtered_secrets = secret_engine.get_secrets_by_role(token, role).await;
                black_box(_filtered_secrets);
            }
        });
    });

    group.finish();
}

/// Benchmark token validation performance
fn bench_token_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config();
    let auth_provider = rt.block_on(async {
        AuthencAuthProvider::new(&config.authenc).await.unwrap()
    });

    let test_tokens = create_various_test_tokens(1000);

    let mut group = c.benchmark_group("token_validation");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark single token validation
    group.bench_function("single_token_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, _) in &test_tokens {
                let _validation = auth_provider.validate_token(token).await;
                black_box(_validation);
            }
        });
    });

    // Benchmark batch token validation
    for batch_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_token_validation", batch_size),
            batch_size,
            |b, &batch_size| {
                let tokens: Vec<_> = test_tokens.iter()
                    .take(batch_size)
                    .map(|(token, _)| token.as_str())
                    .collect();

                b.to_async(&rt).iter(|| async {
                    let _validations = auth_provider.batch_validate_tokens(&tokens).await;
                    black_box(_validations);
                });
            },
        );
    }

    // Benchmark token validation with caching
    group.bench_function("cached_token_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, _) in &test_tokens {
                let _validation = auth_provider.validate_token_with_cache(token).await;
                black_box(_validation);
            }
        });
    });

    // Benchmark post-quantum token validation
    let pq_tokens = create_post_quantum_tokens(100);
    group.bench_function("post_quantum_token_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for token in &pq_tokens {
                let _validation = auth_provider.validate_post_quantum_token(token).await;
                black_box(_validation);
            }
        });
    });

    group.finish();
}

/// Benchmark batch operations for multi-satker scenarios
fn bench_satker_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config();
    let secret_engine = rt.block_on(async {
        EnhancedSecretEngine::new(&config).await.unwrap()
    });

    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    // Create secrets for each satker
    let satker_secrets = rt.block_on(async {
        let mut secrets = Vec::new();
        for satker_code in &satker_codes {
            for i in 0..20 {
                let secret = create_test_secret(
                    &format!("secrets/{}/config_{}", satker_code, i),
                    satker_code
                );
                let _ = secret_engine.store_secret(&secret).await;
                secrets.push(secret);
            }
        }
        secrets
    });

    let satker_tokens = create_satker_tokens(&satker_codes);

    let mut group = c.benchmark_group("satker_batch_operations");
    group.measurement_time(Duration::from_secs(20));

    // Benchmark batch secret creation per satker
    group.bench_function("batch_secret_creation", |b| {
        b.to_async(&rt).iter(|| async {
            for (satker_code, token) in &satker_tokens {
                let secrets_to_create: Vec<_> = (0..10)
                    .map(|i| create_test_secret(
                        &format!("secrets/{}/batch_secret_{}", satker_code, i),
                        satker_code
                    ))
                    .collect();

                let _results = secret_engine.batch_store_secrets_with_auth(token, &secrets_to_create).await;
                black_box(_results);
            }
        });
    });

    // Benchmark batch secret retrieval per satker
    group.bench_function("batch_secret_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            for (satker_code, token) in &satker_tokens {
                let secret_paths: Vec<String> = (0..20)
                    .map(|i| format!("secrets/{}/config_{}", satker_code, i))
                    .collect();

                let _results = secret_engine.batch_get_secrets_with_auth(token, &secret_paths).await;
                black_box(_results);
            }
        });
    });

    // Benchmark cross-satker batch operations (should be denied)
    group.bench_function("cross_satker_batch_denial", |b| {
        b.to_async(&rt).iter(|| async {
            for (requesting_satker, token) in &satker_tokens {
                let cross_satker_paths: Vec<String> = satker_codes.iter()
                    .filter(|&s| s != requesting_satker)
                    .flat_map(|s| (0..5).map(move |i| format!("secrets/{}/config_{}", s, i)))
                    .collect();

                let _results = secret_engine.batch_get_secrets_with_auth(token, &cross_satker_paths).await;
                black_box(_results);
            }
        });
    });

    // Benchmark concurrent satker operations
    group.bench_function("concurrent_satker_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let futures: Vec<_> = satker_tokens.iter().map(|(satker_code, token)| {
                let secret_paths: Vec<String> = (0..10)
                    .map(|i| format!("secrets/{}/config_{}", satker_code, i))
                    .collect();
                secret_engine.batch_get_secrets_with_auth(token, &secret_paths)
            }).collect();

            let _results = futures::future::join_all(futures).await;
            black_box(_results);
        });
    });

    group.finish();
}

/// Benchmark audit logging performance
fn bench_audit_logging_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config();
    let secret_engine = rt.block_on(async {
        EnhancedSecretEngine::new(&config).await.unwrap()
    });

    let test_audit_events = create_benchmark_audit_events(1000);
    let test_token = create_test_token("KEJATI_DKI_JAKPUS");

    let mut group = c.benchmark_group("audit_logging_performance");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark single audit event logging
    group.bench_function("single_audit_logging", |b| {
        b.to_async(&rt).iter(|| async {
            for event in &test_audit_events {
                let _result = secret_engine.log_audit_event(event).await;
                black_box(_result);
            }
        });
    });

    // Benchmark batch audit logging
    for batch_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_audit_logging", batch_size),
            batch_size,
            |b, &batch_size| {
                let events = &test_audit_events[..batch_size];
                b.to_async(&rt).iter(|| async {
                    let _result = secret_engine.batch_log_audit_events(events).await;
                    black_box(_result);
                });
            },
        );
    }

    // Benchmark audit trail retrieval
    group.bench_function("audit_trail_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            let _audit_trail = secret_engine.get_audit_trail_with_authenc_context(
                "KEJATI_DKI_JAKPUS",
                Some(chrono::Utc::now() - chrono::Duration::days(7))
            ).await;
            black_box(_audit_trail);
        });
    });

    // Benchmark audit filtering and search
    group.bench_function("audit_filtering_and_search", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = vec![
                ("by_satker", Some("KEJATI_DKI_JAKPUS".to_string()), None),
                ("by_nip", None, Some("198001012000011001".to_string())),
                ("by_operation", None, None),
            ];

            for (filter_type, satker_filter, nip_filter) in filters {
                let _filtered_audit = secret_engine.get_filtered_audit_trail(
                    satker_filter.as_deref(),
                    nip_filter.as_deref()
                ).await;
                black_box(_filtered_audit);
            }
        });
    });

    // Benchmark compliance audit generation
    group.bench_function("compliance_audit_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let _compliance_report = secret_engine.generate_compliance_audit_report(
                "KEJATI_DKI_JAKPUS",
                chrono::Utc::noration::days(30),
                chrono::Utc::now()
            ).await;
            black_box(_compliance_report);
        });
    });

    group.finish();
}

/// Benchmark post-quantum cryptographic operations
fn bench_post_quantum_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config_with_post_quantum();
    let hybrid_crypto = rt.block_on(async {
        HybridCrypto::new(&config.crypto).await.unwrap()
    });
    let secret_engine = rt.block_on(async {
        EnhancedSecretEngine::new(&config).await.unwrap()
    });

    let test_data_sizes = vec![
        ("small", 1024),      // 1KB
        ("medium", 10240),    // 10KB
        ("large", 102400),    // 100KB
    ];

    let mut group = c.benchmark_group("post_quantum_operations");
    group.measurement_time(Duration::from_secs(30));

    // Benchmark ML-KEM key encapsulation
    group.bench_function("ml_kem_encapsulation", |b| {
        b.to_async(&rt).iter(|| async {
            let _kem_result = hybrid_crypto.ml_kem_encapsulate().await.unwrap();
            black_box(_kem_result);
        });
    });

    // Benchmark ML-KEM key decapsulation
    group.bench_function("ml_kem_decapsulation", |b| {
        let (ciphertext, _) = rt.block_on(async {
            hybrid_crypto.ml_kem_encapsulate().await.unwrap()
        });

        b.to_async(&rt).iter(|| async {
            let _shared_secret = hybrid_crypto.ml_kem_decapsulate(&ciphertext).await.unwrap();
            black_box(_shared_secret);
        });
    });

    // Benchmark ML-DSA signature generation
    group.bench_function("ml_dsa_signature_generation", |b| {
        let test_message = b"Test message for ML-DSA signature";

        b.to_async(&rt).iter(|| async {
            let _signature = hybrid_crypto.ml_dsa_sign(test_message).await.unwrap();
            black_box(_signature);
        });
    });

    // Benchmark ML-DSA signature verification
    group.bench_function("ml_dsa_signature_verification", |b| {
        let test_message = b"Test message for ML-DSA signature";
        let signature = rt.block_on(async {
            hybrid_crypto.ml_dsa_sign(test_message).await.unwrap()
        });

        b.to_async(&rt).iter(|| async {
            let _valid = hybrid_crypto.ml_dsa_verify(test_message, &signature).await.unwrap();
            black_box(_valid);
        });
    });

    // Benchmark hybrid encryption for different data sizes
    for (size_name, size_bytes) in test_data_sizes {
        let test_data = vec![0u8; size_bytes];

        group.bench_with_input(
            BenchmarkId::new("hybrid_encrypt", size_name),
            &test_data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    let _encrypted = hybrid_crypto.hybrid_encrypt(data).await.unwrap();
                    black_box(_encrypted);
                });
            },
        );

        let encrypted_data = rt.block_on(async {
            hybrid_crypto.hybrid_encrypt(&test_data).await.unwrap()
        });

        group.bench_with_input(
            BenchmarkId::new("hybrid_decrypt", size_name),
            &encrypted_data,
            |b, encrypted| {
                b.to_async(&rt).iter(|| async {
                    let _decrypted = hybrid_crypto.hybrid_decrypt(encrypted).await.unwrap();
                    black_box(_decrypted);
                });
            },
        );
    }

    // Benchmark post-quantum secret storage
    group.bench_function("pq_secret_storage", |b| {
        let pq_token = create_post_quantum_token("KEJATI_DKI_JAKPUS");

        b.to_async(&rt).iter(|| async {
            let secret = create_test_secret("secrets/KEJATI_DKI_JAKPUS/pq_test", "KEJATI_DKI_JAKPUS");
            let _result = secret_engine.store_pq_encrypted_secret(&pq_token, &secret).await;
            black_box(_result);
        });
    });

    // Benchmark post-quantum secret retrieval
    group.bench_function("pq_secret_retrieval", |b| {
        let pq_token = create_post_quantum_token("KEJATI_DKI_JAKPUS");

        // Pre-store some PQ secrets
        rt.block_on(async {
            for i in 0..10 {
                let secret = create_test_secret(
                    &format!("secrets/KEJATI_DKI_JAKPUS/pq_secret_{}", i),
                    "KEJATI_DKI_JAKPUS"
                );
                let _ = secret_engine.store_pq_encrypted_secret(&pq_token, &secret).await;
            }
        });

        b.to_async(&rt).iter(|| async {
            for i in 0..10 {
                let secret_path = format!("secrets/KEJATI_DKI_JAKPUS/pq_secret_{}", i);
                let _result = secret_engine.get_pq_encrypted_secret(&pq_token, &secret_path).await;
                black_box(_result);
            }
        });
    });

    group.finish();
}

/// Benchmark load testing for hierarchical operations
fn bench_load_testing_hierarchical_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = SecretonConfig::benchmark_config();
    let secret_engine = rt.block_on(async {
        EnhancedSecretEngine::new(&config).await.unwrap()
    });

    // Create hierarchical test data
    let admin_tokens = create_hierarchical_admin_tokens();
    let target_satker = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    let mut group = c.benchmark_group("load_testing_hierarchical_operations");
    group.measurement_time(Duration::from_secs(30));

    // Benchmark high-load admin operations
    for concurrent_ops in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_admin_operations", concurrent_ops),
            concurrent_ops,
            |b, &concurrent_ops| {
                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = (0..*concurrent_ops).map(|i| {
                        let admin_token = &admin_tokens[i % admin_tokens.len()];
                        let target = &target_satker[i % target_satker.len()];
                        let operation = match i % 4 {
                            0 => "read_secret",
                            1 => "create_secret",
                            2 => "update_secret",
                            3 => "delete_secret",
                            _ => "read_secret",
                        };
                        secret_engine.perform_admin_operation(admin_token, operation, target)
                    }).collect();

                    let _results = futures::future::join_all(futures).await;
                    black_box(_results);
                });
            },
        );
    }

    // Benchmark sustained load operations
    group.bench_function("sustained_load_operations", |b| {
        b.to_async(&rt).iter(|| async {
            for _ in 0..100 {
                let admin_token = &admin_tokens[0]; // Use AdminPusat for maximum access
                for target in &target_satker {
                    let _result = secret_engine.perform_admin_operation(admin_token, "read_secret", target).await;
                    black_box(_result);
                }
            }
        });
    });

    // Benchmark mixed workload (read/write operations)
    group.bench_function("mixed_workload_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let read_futures: Vec<_> = (0..50).map(|i| {
                let admin_token = &admin_tokens[i % admin_tokens.len()];
                let target = &target_satker[i % target_satker.len()];
                secret_engine.perform_admin_operation(admin_token, "read_secret", target)
            }).collect();

            let write_futures: Vec<_> = (0..20).map(|i| {
                let admin_token = &admin_tokens[i % admin_tokens.len()];
                let target = &target_satker[i % target_satker.len()];
                secret_engine.perform_admin_operation(admin_token, "create_secret", target)
            }).collect();

            let (read_results, write_results) = futures::future::join(
                futures::future::join_all(read_futures),
                futures::future::join_all(write_futures)
            ).await;

            black_box((read_results, write_results));
        });
    });

    group.finish();
}

// Helper functions for creating test data

fn create_benchmark_secrets(count: usize) -> Vec<Secret> {
    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    (0..count)
        .map(|i| {
            let satker_code = &satker_codes[i % satker_codes.len()];
            create_test_secret(
                &format!("secrets/{}/benchmark_secret_{}", satker_code, i),
                satker_code
            )
        })
        .collect()
}

fn create_test_secret(path: &str, satker_owner: &str) -> Secret {
    Secret {
        path: path.to_string(),
        value: format!("encrypted_value_for_{}", satker_owner).into(),
        metadata: json!({
            "description": format!("Benchmark secret for {}", satker_owner),
            "created_by": "benchmark_test",
            "classification": "secret"
        }),
        access_control: AccessControl {
            required_roles: vec!["SecretonUser".to_string()],
            required_satker: vec![satker_owner.to_string()],
            nip_whitelist: None,
            nip_blacklist: None,
            time_based_access: None,
            audit_required: true,
        },
        audit_trail: vec![],
        version: 1,
        created_by_nip: Some("198001012000011001".to_string()),
        satker_owner: satker_owner.to_string(),
        last_accessed: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    }
}

fn create_benchmark_tokens(count: usize) -> Vec<(String, String)> {
    let roles = vec!["SecretonUser", "SecretonAdmin", "SecretonAuditor"];

    (0..count)
        .map(|i| {
            let role = &roles[i % roles.len()];
            let token = format!("benchmark_token_{}_{}", role, i);
            (token, role.to_string())
        })
        .collect()
}

fn create_various_test_tokens(count: usize) -> Vec<(String, String)> {
    let token_types = vec!["valid", "expired", "malformed", "cross_satker"];

    (0..count)
        .map(|i| {
            let token_type = &token_types[i % token_types.len()];
            let token = format!("test_token_{}_{}", token_type, i);
            (token, token_type.to_string())
        })
        .collect()
}

fn create_post_quantum_tokens(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("pq_token_ml_dsa_{}", i))
        .collect()
}

fn create_satker_tokens(satker_codes: &[&str]) -> Vec<(String, String)> {
    satker_codes.iter()
        .map(|&satker_code| {
            let token = format!("valid_token_for_{}", satker_code);
            (satker_code.to_string(), token)
        })
        .collect()
}

fn create_test_token(satker_code: &str) -> String {
    format!("valid_token_for_{}", satker_code)
}

fn create_post_quantum_token(satker_code: &str) -> String {
    format!("pq_token_ml_dsa_for_{}", satker_code)
}

fn create_benchmark_audit_events(count: usize) -> Vec<AuditEvent> {
    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
    ];

    let operations = vec!["get_secret", "create_secret", "update_secret", "delete_secret"];

    (0..count)
        .map(|i| {
            let satker_code = &satker_codes[i % satker_codes.len()];
            let operation = &operations[i % operations.len()];

            AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                event_type: secreton_core::models::AuditEventType::SecretAccess,
                nip: Some(format!("19800101200001{:04}", i)),
                satker_code: Some(satker_code.to_string()),
                authenc_session_id: Some(Uuid::new_v4().to_string()),
                resource_path: format!("secrets/{}/benchmark_resource_{}", satker_code, i),
                operation: operation.to_string().into(),
                result: secreton_core::models::OperationResult::Success,
                security_context: Default::default(),
                risk_score: Some(0.1),
                compliance_flags: vec!["KEJAKSAAN_AUDIT".to_string()],
                admin_level: None,
            }
        })
        .collect()
}

fn create_hierarchical_admin_tokens() -> Vec<String> {
    vec![
        "admin_token_AdminPusat_KEJAGUNG".to_string(),
        "admin_token_AdminEselonI_KEJAGUNG".to_string(),
        "admin_token_AdminWilayah_KEJATI_DKI".to_string(),
        "admin_token_AdminWilayah_KEJATI_JABAR".to_string(),
        "admin_token_AdminSatker_KEJATI_DKI_JAKPUS".to_string(),
        "admin_token_AdminSatker_KEJATI_DKI_JAKSEL".to_string(),
    ]
}

criterion_group!(
    benches,
    bench_secret_retrieval_by_role,
    bench_token_validation,
    bench_satker_batch_operations,
    bench_audit_logging_performance,
    bench_post_quantum_operations,
    bench_load_testing_hierarchical_operations
);

criterion_main!(benches);
