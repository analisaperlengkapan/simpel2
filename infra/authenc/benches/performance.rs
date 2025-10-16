//! Authenc Performance Benchmarks
//!
//! This module contains comprehensive performance benchmarks for authenc functionality,
//! focusing on the enhanced features for SIMKARI super app integration with secreton.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;
use serde_json::json;

use authenc::crypto::CryptoEngine;
use authenc::vault::SecretonClient;
use authenc::models::{User, OptimizedToken, Role, RoleScope, AdminLevel};
use authenc::config::AuthencConfig;

/// Benchmark JWT signing for pegawai authentication
fn bench_pegawai_jwt_signing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    let test_users = create_benchmark_users(100);

    let mut group = c.benchmark_group("pegawai_jwt_signing");
    group.measurement_time(Duration::from_secs(10));

    for user_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_jwt_sign", user_count),
            user_count,
            |b, &user_count| {
                let users = &test_users[..user_count];
                b.to_async(&rt).iter(|| async {
                    for user in users {
                        let claims = create_pegawai_claims(user);
                        let _token = crypto_engine.sign_jwt_for_government(&claims).await.unwrap();
                        black_box(_token);
                    }
                });
            },
        );
    }

    // Benchmark batch JWT signing
    group.bench_function("batch_jwt_signing_100", |b| {
        b.to_async(&rt).iter(|| async {
            let claims_batch: Vec<_> = test_users.iter()
                .map(|user| create_pegawai_claims(user))
                .collect();

            let _tokens = crypto_engine.batch_sign_jwt_for_government(&claims_batch).await.unwrap();
            black_box(_tokens);
        });
    });

    group.finish();
}

/// Benchmark batch NIP validation for multiple pegawai
fn bench_batch_nip_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    let test_tokens = rt.block_on(async {
        let users = create_benchmark_users(1000);
        let mut tokens = Vec::new();

        for user in &users {
            let claims = create_pegawai_claims(user);
            let token = crypto_engine.sign_jwt_for_government(&claims).await.unwrap();
            tokens.push(token);
        }

        tokens
    });

    let mut group = c.benchmark_group("batch_nip_validation");
    group.measurement_time(Duration::from_secs(15));

    for batch_size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("validate_tokens", batch_size),
            batch_size,
            |b, &batch_size| {
                let tokens = &test_tokens[..batch_size];
                b.to_async(&rt).iter(|| async {
                    let _results = crypto_engine.verify_token_batch(tokens).await.unwrap();
                    black_box(_results);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark secreton integration performance per satker
fn bench_satker_secreton_integration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let secreton_client = rt.block_on(async {
        SecretonClient::new(&config.secreton).await.unwrap()
    });
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    let test_tokens = rt.block_on(async {
        let mut tokens = Vec::new();
        for satker_code in &satker_codes {
            for i in 0..20 {
                let user = create_test_user(&format!("19800101200001100{}", i), satker_code);
                let claims = create_pegawai_claims(&user);
                let token = crypto_engine.sign_jwt_for_government(&claims).await.unwrap();
                tokens.push((token, satker_code.to_string()));
            }
        }
        tokens
    });

    let mut group = c.benchmark_group("satker_secreton_integration");
    group.measurement_time(Duration::from_secs(20));

    // Benchmark secret retrieval per satker
    group.bench_function("secret_retrieval_per_satker", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, satker_code) in &test_tokens {
                let secret_path = format!("secrets/{}/config", satker_code);
                let _result = secreton_client.get_secret_with_token(token, &secret_path).await;
                black_box(_result);
            }
        });
    });

    // Benchmark concurrent secret retrieval
    group.bench_function("concurrent_secret_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            let futures: Vec<_> = test_tokens.iter().map(|(token, satker_code)| {
                let secret_path = format!("secrets/{}/config", satker_code);
                secreton_client.get_secret_with_token(token, &secret_path)
            }).collect();

            let _results = futures::future::join_all(futures).await;
            black_box(_results);
;
    });

    // Benchmark application config retrieval
    group.bench_function("application_config_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            for satker_code in &satker_codes {
                let app_id = format!("SIMKARI_{}", satker_code);
                let _result = secreton_client.get_application_config(&app_id).await;
                black_box(_result);
            }
        });
    });

    group.finish();
}

/// Benchmark role-based access control validation
fn bench_role_based_access_control(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    // Create users with different role combinations
    let test_users = create_users_with_various_roles(500);
    let test_tokens = rt.block_on(async {
        let mut tokens = Vec::new();
        for user in &test_users {
            let claims = create_pegawai_claims(user);
            let token = crypto_engine.sign_jwt_for_government(&claims).await.unwrap();
            tokens.push((token, user.clone()));
        }
        tokens
    });

    let mut group = c.benchmark_group("role_based_access_control");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark role validation
    group.bench_function("role_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, user) in &test_tokens {
                let _validation = crypto_engine.validate_user_roles(token, &user.roles).await;
                black_box(_validation);
            }
        });
    });

    // Benchmark permission checking
    group.bench_function("permission_checking", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, user) in &test_tokens {
                let resource_path = format!("secrets/{}/sensitive_data", user.satker_code);
                let _has_permission = crypto_engine.check_resource_permission(token, &resource_path).await;
                black_box(_has_permission);
            }
        });
    });

    // Benchmark hierarchical access validation
    group.bench_function("hierarchical_access_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, user) in &test_tokens {
                for target_satker in ["KEJATI_DKI_JAKPUS", "KEJATI_DKI_JAKSEL", "KEJARI_SOLO"] {
                    let _can_access = crypto_engine.validate_hierarchical_access(
                        token, &user.satker_code, target_satker
                    ).await;
                    black_box(_can_access);
                }
            }
        });
    });

    group.finish();
}

/// Benchmark hierarchical admin operations across satker/wilayah/pusat levels
fn bench_hierarchical_admin_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let secreton_client = rt.block_on(async {
        SecretonClient::new(&config.secreton).await.unwrap()
    });
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    // Create admin users at different levels
    let admin_users = create_admin_users_at_all_levels();
    let admin_tokens = rt.block_on(async {
        let mut tokens = Vec::new();
        for user in &admin_users {
            let claims = create_pegawai_claims(user);
            let token = crypto_engine.sign_jwt_for_government(&claims).await.unwrap();
            tokens.push((token, user.clone()));
        }
        tokens
    });

    let target_satker = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    let mut group = c.benchmark_group("hierarchical_admin_operations");
    group.measurement_time(Duration::from_secs(20));

    // Benchmark admin secret creation
    group.bench_function("admin_secret_creation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, admin_user) in &admin_tokens {
                for target in &target_satker {
                    let operation = "create_secret";
                    let _result = secreton_client.perform_admin_operation(token, operation, target).await;
                    black_box(_result);
                }
            }
        });
    });

    // Benchmark admin access validation
    group.bench_function("admin_access_validation", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, admin_user) in &admin_tokens {
                for target in &target_satker {
                    let _can_admin = crypto_engine.validate_admin_access(
                        token, &admin_user.satker_code, target
                    ).await;
                    black_box(_can_admin);
                }
            }
        });
    });

    // Benchmark cross-level admin operations
    group.bench_function("cross_level_admin_operations", |b| {
        b.to_async(&rt).iter(|| async {
            for (token, admin_user) in &admin_tokens {
                let operations = ["read_secret", "update_secret", "delete_secret"];
                for operation in &operations {
                    for target in &target_satker {
                        let _result = secreton_client.perform_admin_operation(token, operation, target).await;
                        black_box(_result);
                    }
                }
            }
        });
    });

    group.finish();
}

/// Benchmark post-quantum cryptographic operations
fn bench_post_quantum_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config_with_post_quantum();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });
    let secreton_client = rt.block_on(async {
        SecretonClient::new(&config.secreton).await.unwrap()
    });

    let test_user = create_test_user("198001012000011001", "KEJATI_DKI_JAKPUS");

    let mut group = c.benchmark_group("post_quantum_operations");
    group.measurement_time(Duration::from_secs(30));

    // Benchmark ML-DSA signature generation
    group.bench_function("ml_dsa_signature_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let claims = create_pegawai_claims(&test_user);
            let _pq_token = crypto_engine.sign_jwt_with_ml_dsa(&claims).await.unwrap();
            black_box(_pq_token);
        });
    });

    // Benchmark ML-DSA signature verification
    group.bench_function("ml_dsa_signature_verification", |b| {
        let pq_token = rt.block_on(async {
            let claims = create_pegawai_claims(&test_user);
            crypto_engine.sign_jwt_with_ml_dsa(&claims).await.unwrap()
        });

        b.to_async(&rt).iter(|| async {
            let _valid = crypto_engine.verify_ml_dsa_token(&pq_token).await.unwrap();
            black_box(_valid);
        });
    });

    // Benchmark ML-KEM key encapsulation
    group.bench_function("ml_kem_key_encapsulation", |b| {
        b.to_async(&rt).iter(|| async {
            let _kem_result = crypto_engine.ml_kem_encapsulate().await.unwrap();
            black_box(_kem_result);
        });
    });

    // Benchmark ML-KEM key decapsulation
    group.bench_function("ml_kem_key_decapsulation", |b| {
        let (ciphertext, _) = rt.block_on(async {
            crypto_engine.ml_kem_encapsulate().await.unwrap()
        });

        b.to_async(&rt).iter(|| async {
            let _shared_secret = crypto_engine.ml_kem_decapsulate(&ciphertext).await.unwrap();
            black_box(_shared_secret);
        });
    });

    // Benchmark hybrid cryptographic operations
    group.bench_function("hybrid_crypto_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let claims = create_pegawai_claims(&test_user);
            let _hybrid_token = crypto_engine.sign_jwt_hybrid(&claims).await.unwrap();
            black_box(_hybrid_token);
        });
    });

    // Benchmark post-quantum secret operations
    group.bench_function("pq_secret_operations", |b| {
        let pq_token = rt.block_on(async {
            let claims = create_pegawai_claims(&test_user);
            crypto_engine.sign_jwt_with_ml_dsa(&claims).await.unwrap()
        });

        b.to_async(&rt).iter(|| async {
            let _pq_key = secreton_client.get_post_quantum_key(&pq_token, "ML-KEM").await;
            black_box(_pq_key);
        });
    });

    group.finish();
}

/// Benchmark session data encryption/decryption
fn bench_session_data_encryption(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    let session_data_sizes = vec![
        ("small", 1024),      // 1KB
        ("medium", 10240),    // 10KB
        ("large", 102400),    // 100KB
        ("xlarge", 1048576),  // 1MB
    ];

    let mut group = c.benchmark_group("session_data_encryption");
    group.measurement_time(Duration::from_secs(15));

    for (size_name, size_bytes) in session_data_sizes {
        let test_data = create_test_session_data(size_bytes);

        group.bench_with_input(
            BenchmarkId::new("encrypt", size_name),
            &test_data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    let _encrypted = crypto_engine.encrypt_session_data(data).await.unwrap();
                    black_box(_encrypted);
                });
            },
        );

        let encrypted_data = rt.block_on(async {
            crypto_engine.encrypt_session_data(&test_data).await.unwrap()
        });

        group.bench_with_input(
            BenchmarkId::new("decrypt", size_name),
            &encrypted_data,
            |b, encrypted| {
                b.to_async(&rt).iter(|| async {
                    let _decrypted = crypto_engine.decrypt_session_data(encrypted).await.unwrap();
                    black_box(_decrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark audit signature generation for compliance
fn bench_audit_signature_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = AuthencConfig::benchmark_config();
    let crypto_engine = rt.block_on(async {
        CryptoEngine::new(&config.crypto).await.unwrap()
    });

    let audit_data_samples = create_audit_data_samples(1000);

    let mut group = c.benchmark_group("audit_signature_generation");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark single audit signature
    group.bench_function("single_audit_signature", |b| {
        b.to_async(&rt).iter(|| async {
            for audit_data in &audit_data_samples {
                let _signature = crypto_engine.generate_government_audit_signature(audit_data).await.unwrap();
                black_box(_signature);
            }
        });
    });

    // Benchmark batch audit signatures
    group.bench_function("batch_audit_signatures", |b| {
        b.to_async(&rt).iter(|| async {
            let _signatures = crypto_engine.batch_generate_audit_signatures(&audit_data_samples).await.unwrap();
            black_box(_signatures);
        });
    });

    group.finish();
}

// Helper functions for creating test data

fn create_benchmark_users(count: usize) -> Vec<User> {
    (0..count)
        .map(|i| create_test_user(&format!("19800101200001{:04}", i), "KEJATI_DKI_JAKPUS"))
        .collect()
}

fn create_test_user(nip: &str, satker_code: &str) -> User {
    User {
        id: Uuid::new_v4(),
        nip: nip.to_string(),
        nama: format!("Test User {}", nip),
        email: format!("test.{}@kejaksaan.go.id", nip),
        satker_code: satker_code.to_string(),
        jabatan: "Jaksa Muda".to_string(),
        roles: vec![create_basic_role(satker_code)],
        permissions: vec![],
        session_data: Default::default(),
        secreton_access_policy: Default::default(),
        last_auth: chrono::Utc::now(),
        security_context: Default::default(),
    }
}

fn create_basic_role(satker_code: &str) -> Role {
    Role {
        id: Uuid::new_v4(),
        name: "SecretonUser".to_string(),
        scope: RoleScope::Satker(satker_code.to_string()),
        permissions: vec![],
        managed_by: AdminLevel::AdminSatker(satker_code.to_string()),
    }
}

fn create_pegawai_claims(user: &User) -> serde_json::Value {
    json!({
        "sub": user.id,
        "nip": user.nip,
        "nama": user.nama,
        "satker_code": user.satker_code,
        "jabatan": user.jabatan,
        "roles": user.roles,
        "iat": chrono::Utc::now().timestamp(),
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    })
}

fn create_users_with_various_roles(count: usize) -> Vec<User> {
    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
        "KEJARI_YOGYA",
    ];

    let role_types = vec![
        "SecretonUser",
        "SecretonAdmin",
        "SecretonAuditor",
        "SecretonOperator",
    ];

    (0..count)
        .map(|i| {
            let satker_code = &satker_codes[i % satker_codes.len()];
            let role_type = &role_types[i % role_types.len()];

            let mut user = create_test_user(&format!("19800101200001{:04}", i), satker_code);
            user.roles = vec![Role {
                id: Uuid::new_v4(),
                name: role_type.to_string(),
                scope: RoleScope::Satker(satker_code.to_string()),
                permissions: vec![],
                managed_by: AdminLevel::AdminSatker(satker_code.to_string()),
            }];

            user
        })
        .collect()
}

fn create_admin_users_at_all_levels() -> Vec<User> {
    let mut admin_users = Vec::new();

    // AdminPusat
    let mut pusat_admin = create_test_user("198001012000010001", "KEJAGUNG");
    pusat_admin.roles = vec![Role {
        id: Uuid::new_v4(),
        name: "SecretonAdminPusat".to_string(),
        scope: RoleScope::Pusat,
        permissions: vec![],
        managed_by: AdminLevel::AdminPusat,
    }];
    admin_users.push(pusat_admin);

    // AdminEselonI
    let mut eselon_admin = create_test_user("198001012000010002", "KEJAGUNG");
    eselon_admin.roles = vec![Role {
        id: Uuid::new_v4(),
        name: "SecretonAdminEselonI".to_string(),
        scope: RoleScope::Pusat,
        permissions: vec![],
        managed_by: AdminLevel::AdminEselonI,
    }];
    admin_users.push(eselon_admin);

    // AdminWilayah
    let wilayah_codes = vec!["KEJATI_DKI", "KEJATI_JABAR", "KEJATI_JATENG"];
    for (i, wilayah) in wilayah_codes.iter().enumerate() {
        let mut wilayah_admin = create_test_user(&format!("19800101200001000{}", i + 3), wilayah);
        wilayah_admin.roles = vec![Role {
            id: Uuid::new_v4(),
            name: "SecretonAdminWilayah".to_string(),
            scope: RoleScope::Wilayah(wilayah.to_string()),
            permissions: vec![],
            managed_by: AdminLevel::AdminWilayah(wilayah.to_string()),
        }];
        admin_users.push(wilayah_admin);
    }

    // AdminSatker
    let satker_codes = vec![
        "KEJATI_DKI_JAKPUS",
        "KEJATI_DKI_JAKSEL",
        "KEJATI_JABAR_BANDUNG",
        "KEJARI_SOLO",
    ];
    for (i, satker) in satker_codes.iter().enumerate() {
        let mut satker_admin = create_test_user(&format!("19800101200001001{}", i), satker);
        satker_admin.roles = vec![Role {
            id: Uuid::new_v4(),
            name: "SecretonAdminSatker".to_string(),
            scope: RoleScope::Satker(satker.to_string()),
            permissions: vec![],
            managed_by: AdminLevel::AdminSatker(satker.to_string()),
        }];
        admin_users.push(satker_admin);
    }

    admin_users
}

fn create_test_session_data(size_bytes: usize) -> authenc::models::SessionData {
    let data = vec![0u8; size_bytes];
    authenc::models::SessionData {
        user_id: Uuid::new_v4(),
        session_id: Uuid::new_v4().to_string(),
        data,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
    }
}

fn create_audit_data_samples(count: usize) -> Vec<authenc::models::AuditData> {
    (0..count)
        .map(|i| authenc::models::AuditData {
            event_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_nip: format!("19800101200001{:04}", i),
            satker_code: "KEJATI_DKI_JAKPUS".to_string(),
            operation: format!("test_operation_{}", i),
            resource: format!("test_resource_{}", i),
            result: "success".to_string(),
            details: json!({
                "test_data": format!("benchmark_data_{}", i),
                "iteration": i,
            }),
        })
        .collect()
}

criterion_group!(
    benches,
    bench_pegawai_jwt_signing,
    bench_batch_nip_validation,
    bench_satker_secreton_integration,
    bench_role_based_access_control,
    bench_hierarchical_admin_operations,
    bench_post_quantum_operations,
    bench_session_data_encryption,
    bench_audit_signature_generation
);

criterion_main!(benches);
