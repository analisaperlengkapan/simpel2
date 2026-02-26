use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use secreton_storage::{
    KvBackendAdapter, MemoryBackend, QueryParams, SecretEntry, SecurityLevel, StorageBackend,
    backends::{FileBackend, FileConfig},
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Helper to create test engine entry
fn create_test_entry(path: &str, data_size: usize) -> SecretEntry {
    SecretEntry {
        id: Uuid::new_v4(),
        path: path.to_string(),
        encrypted_data: vec![0u8; data_size],
        encryption_metadata: serde_json::json!({"algorithm": "aes-256-gcm"}),
        security_level: SecurityLevel::Internal,
        metadata: serde_json::json!({"test": "benchmark"}),
        tags: vec!["benchmark".to_string()],
        version: 1,
        owner_id: "bench-user".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: None,
    }
}

/// Benchmark memory backend operations
fn bench_memory_backend(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_memory");

    let data_sizes = vec![(128, "128B"), (1024, "1KB"), (10240, "10KB")];

    // Benchmark writes
    for (size, size_name) in &data_sizes {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size_name), size, |b, s| {
            b.iter(|| {
                let backend = MemoryBackend::new();
                let entry = create_test_entry("test/bench", *s);
                rt.block_on(async {
                    backend.store(black_box(&entry)).await.unwrap();
                });
            })
        });
    }

    // Benchmark reads
    for (size, size_name) in &data_sizes {
        let backend = Arc::new(MemoryBackend::new());
        let entry = create_test_entry("test/bench", *size);

        rt.block_on(async {
            backend.store(&entry).await.unwrap();
        });

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("read", size_name),
            &backend,
            |b, backend| {
                b.iter(|| {
                    rt.block_on(async {
                        let fetched = backend
                            .get_by_path(black_box("test/bench"))
                            .await
                            .unwrap()
                            .unwrap();
                        black_box(fetched);
                    });
                })
            },
        );
    }

    group.finish();
}

/// Benchmark file backend operations
fn bench_file_backend(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_file");

    let temp_dir = tempfile::tempdir().unwrap();
    let data_sizes = vec![(128, "128B"), (1024, "1KB"), (10240, "10KB")];

    // Benchmark writes
    for (size, size_name) in &data_sizes {
        let config = FileConfig {
            path: temp_dir
                .path()
                .join(format!("bench_{}", size_name))
                .to_path_buf(),
            sync_writes: false, // Async for better performance
            ..Default::default()
        };

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size_name), &config, |b, cfg| {
            b.iter(|| {
                rt.block_on(async {
                    let file_backend = FileBackend::new(cfg.clone()).await.unwrap();
                    let backend = KvBackendAdapter::new(file_backend);
                    let entry = create_test_entry(
                        &format!("test/bench_{}", cfg.path.to_str().unwrap(),),
                        *size,
                    );
                    backend.store(black_box(&entry)).await.unwrap();
                });
            })
        });
    }

    // Benchmark reads
    for (size, size_name) in &data_sizes {
        let config = FileConfig {
            path: temp_dir
                .path()
                .join(format!("bench_read_{}", size_name))
                .to_path_buf(),
            sync_writes: false,
            ..Default::default()
        };

        let entry = create_test_entry("test/bench/read", *size);

        rt.block_on(async {
            let file_backend = FileBackend::new(config.clone()).await.unwrap();
            let backend = KvBackendAdapter::new(file_backend);
            backend.store(&entry).await.unwrap();
        });

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("read", size_name), &config, |b, cfg| {
            b.iter(|| {
                rt.block_on(async {
                    let file_backend = FileBackend::new(cfg.clone()).await.unwrap();
                    let backend = KvBackendAdapter::new(file_backend);
                    let fetched = backend
                        .get_by_path(black_box("test/bench/read"))
                        .await
                        .unwrap()
                        .unwrap();
                    black_box(fetched);
                });
            })
        });
    }

    group.finish();
}

/// Benchmark list operations
fn bench_storage_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_list");

    // Test different entry counts
    let entry_counts = vec![10, 100, 1000];

    for count in &entry_counts {
        let backend = Arc::new(MemoryBackend::new());

        // Populate with test entries
        rt.block_on(async {
            for i in 0..*count {
                let entry = create_test_entry(&format!("test/bench/item_{}", i), 128);
                backend.store(&entry).await.unwrap();
            }
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &backend,
            |b, backend| {
                b.iter(|| {
                    rt.block_on(async {
                        let params = QueryParams::new()
                            .with_path_prefix(black_box("test/bench").to_string());
                        let entries = backend.list(&params).await.unwrap();
                        black_box(entries);
                    });
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent operations
fn bench_storage_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("storage_concurrent");

    let concurrency_levels = vec![1, 5, 10, 20];

    for concurrency in &concurrency_levels {
        let backend = Arc::new(MemoryBackend::new());

        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, c| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = Vec::new();

                        for i in 0..*c {
                            let backend_clone = Arc::clone(&backend);
                            let handle = tokio::spawn(async move {
                                let entry =
                                    create_test_entry(&format!("concurrent/item_{}", i), 512);
                                backend_clone.store(&entry).await.unwrap();
                                let fetched = backend_clone
                                    .get_by_path(&format!("concurrent/item_{}", i))
                                    .await
                                    .unwrap()
                                    .unwrap();
                                black_box(fetched);
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    storage_benches,
    bench_memory_backend,
    bench_file_backend,
    bench_storage_list,
    bench_storage_concurrent
);

criterion_main!(storage_benches);
