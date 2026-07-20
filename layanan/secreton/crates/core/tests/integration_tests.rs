// Integration tests for Secreton Core
//
// NOTE: This test file references outdated API structures (EngineRegistry, MemorySecretEngine)
// The current secrets module uses SecretEngine trait with different implementations.
// TODO: Rewrite tests to use secreton_core::services::secrets module

// DISABLED: Outdated API - needs complete rewrite for current module structure
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;

    use secreton_core::services::secrets::{EngineRegistry, SecretEngineMetrics};

    #[tokio::test]
    async fn test_registry_multiple_engines() {
        let mut registry = EngineRegistry::new();

        // Register multiple engines
        let memory1 = MemorySecretEngine::new();
        let memory2 = MemorySecretEngine::new();

        assert!(registry.register(memory1).is_ok());
        // Second memory engine should fail (duplicate type)
        assert!(registry.register(memory2).is_err());
    }

    #[tokio::test]
    async fn test_registry_get_engine() {
        let mut registry = EngineRegistry::new();
        let engine = MemorySecretEngine::new();

        registry.register(engine).unwrap();

        let retrieved = registry.get_engine("memory");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_registry_get_nonexistent_engine() {
        let registry = EngineRegistry::new();
        let retrieved = registry.get_engine("nonexistent");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_registry_list_engines() {
        let mut registry = EngineRegistry::new();
        let engine = MemorySecretEngine::new();

        registry.register(engine).unwrap();

        let engines = registry.list_engines();
        assert_eq!(engines.len(), 1);
        assert_eq!(engines[0], "memory");
    }

    #[tokio::test]
    async fn test_registry_list_empty() {
        let registry = EngineRegistry::new();
        let engines = registry.list_engines();
        assert_eq!(engines.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_metrics_collection() {
        let mut registry = EngineRegistry::new();
        let engine = MemorySecretEngine::new();

        registry.register(engine).unwrap();

        let metrics: CoreResult<std::collections::HashMap<String, SecretEngineMetrics>> =
            registry.collect_metrics().await;
        assert!(metrics.is_ok());
        assert!(metrics.unwrap().contains_key("memory"));
    }

    #[tokio::test]
    async fn test_registry_metrics_empty() {
        let registry = EngineRegistry::new();
        let metrics: CoreResult<std::collections::HashMap<String, SecretEngineMetrics>> =
            registry.collect_metrics().await;
        assert!(metrics.is_ok());
        assert_eq!(metrics.unwrap().len(), 0);
    }
}

#[cfg(test)]
mod secret_lifecycle_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_secret() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});
        let result: CoreResult<Secret> = engine.create_secret("test/path", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_read_update_delete() {
        let engine = MemorySecretEngine::new();

        // Create
        let data = json!({"username": "admin", "password": "secret"});
        let secret = engine
            .create_secret("app/config", data, None)
            .await
            .unwrap();
        assert_eq!(secret.path, "app/config");

        // Read
        let read_secret: Secret = engine.read_secret("app/config").await.unwrap();
        assert_eq!(read_secret.path, "app/config");

        // Update
        let new_data = json!({"username": "admin", "password": "newsecret"});
        let updated = engine
            .update_secret("app/config", new_data, None)
            .await
            .unwrap();
        assert_eq!(updated.version, 2);

        // Delete
        let _: CoreResult<()> = engine.delete_secret("app/config").await;
        assert!(engine.read_secret("app/config").await.is_err());
    }

    #[tokio::test]
    async fn test_create_multiple_secrets() {
        let engine = MemorySecretEngine::new();

        for i in 0..10 {
            let data = json!({"index": i});
            let result: CoreResult<Secret> = engine
                .create_secret(&format!("test/{}", i), data, None)
                .await;
            assert!(result.is_ok());
        }

        let secrets: Vec<String> = engine.list_secrets("test/").await.unwrap();
        assert!(secrets.len() >= 10);
    }

    #[tokio::test]
    async fn test_update_increments_version() {
        let engine = MemorySecretEngine::new();

        let data = json!({"key": "value1"});
        let secret = engine
            .create_secret("test/versioning", data, None)
            .await
            .unwrap();
        assert_eq!(secret.version, 1);

        let data2 = json!({"key": "value2"});
        let updated = engine
            .update_secret("test/versioning", data2, None)
            .await
            .unwrap();
        assert_eq!(updated.version, 2);

        let data3 = json!({"key": "value3"});
        let updated2 = engine
            .update_secret("test/versioning", data3, None)
            .await
            .unwrap();
        assert_eq!(updated2.version, 3);
    }

    #[tokio::test]
    async fn test_list_secrets_with_prefix() {
        let engine = MemorySecretEngine::new();

        // Create multiple secrets
        let data = json!({"key": "value"});
        engine
            .create_secret("app/db/password", data.clone(), None)
            .await
            .unwrap();
        engine
            .create_secret("app/api/key", data.clone(), None)
            .await
            .unwrap();
        engine
            .create_secret("service/token", data, None)
            .await
            .unwrap();

        // List with prefix
        let app_secrets: Vec<String> = engine.list_secrets("app/").await.unwrap();
        assert!(app_secrets.len() >= 2);

        let all_secrets: Vec<String> = engine.list_secrets("").await.unwrap();
        assert!(all_secrets.len() >= 3);
    }

    #[tokio::test]
    async fn test_list_empty_prefix() {
        let engine = MemorySecretEngine::new();
        let secrets: Vec<String> = engine.list_secrets("nonexistent/").await.unwrap();
        assert_eq!(secrets.len(), 0);
    }

    #[tokio::test]
    async fn test_secret_metadata() {
        let engine = MemorySecretEngine::new();

        let data = json!({"key": "value"});
        let secret: Secret = engine.create_secret("test/path", data, None).await.unwrap();

        assert_eq!(secret.version, 1);
        assert!(secret.created_at <= chrono::Utc::now());
        assert!(secret.updated_at <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_secret_with_complex_data() {
        let engine = MemorySecretEngine::new();

        let data = json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "username": "admin",
                "password": "secret123"
            },
            "api_keys": ["key1", "key2", "key3"],
            "enabled": true
        });

        let secret = engine
            .create_secret("app/complex", data, None)
            .await
            .unwrap();
        assert_eq!(secret.path, "app/complex");

        let read: Secret = engine.read_secret("app/complex").await.unwrap();
        assert!(read.data.data.get("database").is_some());
        assert!(read.data.data.get("api_keys").is_some());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;

    #[tokio::test]
    async fn test_read_nonexistent_secret() {
        let engine = MemorySecretEngine::new();
        let result: CoreResult<Secret> = engine.read_secret("nonexistent/path").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_secret() {
        let engine = MemorySecretEngine::new();
        let result: CoreResult<()> = engine.delete_secret("nonexistent/path").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_nonexistent_secret() {
        let engine = MemorySecretEngine::new();
        let data = serde_json::json!({"key": "value"});
        let result: CoreResult<Secret> = engine.update_secret("nonexistent/path", data, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_double_delete() {
        let engine = MemorySecretEngine::new();

        let data = serde_json::json!({"key": "value"});
        engine
            .create_secret("test/delete", data, None)
            .await
            .unwrap();

        // First delete should succeed
        let _: CoreResult<()> = engine.delete_secret("test/delete").await;

        // Second delete should fail
        assert!(engine.delete_secret("test/delete").await.is_err());
    }

    #[tokio::test]
    async fn test_read_after_delete() {
        let engine = MemorySecretEngine::new();

        let data = serde_json::json!({"key": "value"});
        engine
            .create_secret("test/deleted", data, None)
            .await
            .unwrap();
        let _: CoreResult<()> = engine.delete_secret("test/deleted").await;

        let result: CoreResult<Secret> = engine.read_secret("test/deleted").await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod performance_tests {

    use secreton_core::engines::MemorySecretEngine;
    use std::time::Instant;

    #[tokio::test]
    async fn test_bulk_create_performance() {
        let engine = MemorySecretEngine::new();
        let start = Instant::now();

        // Create 100 secrets
        for i in 0..100 {
            let data = serde_json::json!({"index": i});
            engine
                .create_secret(&format!("perf/test_{}", i), data, None)
                .await
                .unwrap();
        }

        let duration = start.elapsed();

        // Should complete in reasonable time (< 1 second for memory engine)
        assert!(duration.as_secs() < 1);
    }

    #[tokio::test]
    async fn test_bulk_read_performance() {
        let engine = MemorySecretEngine::new();

        // Create secrets first
        for i in 0..100 {
            let data = serde_json::json!({"index": i});
            engine
                .create_secret(&format!("perf/test_{}", i), data, None)
                .await
                .unwrap();
        }

        let start = Instant::now();

        // Read 100 secrets
        for i in 0..100 {
            engine
                .read_secret(&format!("perf/test_{}", i))
                .await
                .unwrap();
        }

        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_secs() < 1);
    }

    #[tokio::test]
    async fn test_bulk_update_performance() {
        let engine = MemorySecretEngine::new();

        // Create secrets first
        for i in 0..50 {
            let data = serde_json::json!({"index": i, "version": 1});
            engine
                .create_secret(&format!("perf/update_{}", i), data, None)
                .await
                .unwrap();
        }

        let start = Instant::now();

        // Update 50 secrets
        for i in 0..50 {
            let data = serde_json::json!({"index": i, "version": 2});
            engine
                .update_secret(&format!("perf/update_{}", i), data, None)
                .await
                .unwrap();
        }

        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 500);
    }

    #[tokio::test]
    async fn test_bulk_delete_performance() {
        let engine = MemorySecretEngine::new();

        // Create secrets first
        for i in 0..100 {
            let data = serde_json::json!({"index": i});
            engine
                .create_secret(&format!("perf/delete_{}", i), data, None)
                .await
                .unwrap();
        }

        let start = Instant::now();

        // Delete 100 secrets
        for i in 0..100 {
            engine
                .delete_secret(&format!("perf/delete_{}", i))
                .await
                .unwrap();
        }

        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_secs() < 1);
    }

    #[tokio::test]
    async fn test_large_secret_performance() {
        let engine = MemorySecretEngine::new();

        // Create a large secret (1000 key-value pairs)
        let mut large_data = serde_json::Map::new();
        for i in 0..1000 {
            large_data.insert(
                format!("key_{}", i),
                serde_json::json!(format!("value_{}", i)),
            );
        }

        let start = Instant::now();
        let result = engine
            .create_secret("perf/large", serde_json::Value::Object(large_data), None)
            .await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 100);
    }
}

#[cfg(test)]
mod secret_path_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_path() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});
        let result: CoreResult<Secret> = engine.create_secret("simple", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_nested_path() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});
        let result = engine
            .create_secret("level1/level2/level3/secret", data, None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_path_with_special_chars() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});
        let result = engine
            .create_secret("app/db-config/prod_env", data, None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_similar_paths() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});

        engine
            .create_secret("app/config", data.clone(), None)
            .await
            .unwrap();
        let _: Secret = engine
            .create_secret("app/config/db", data.clone(), None)
            .await
            .unwrap();
        let _: Secret = engine
            .create_secret("app/config/api", data, None)
            .await
            .unwrap();

        let secrets: Vec<String> = engine.list_secrets("app/config").await.unwrap();
        assert!(secrets.len() >= 3);
    }
}

#[cfg(test)]
mod data_validation_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn test_empty_data() {
        let engine = MemorySecretEngine::new();
        let data = json!({});
        let result: CoreResult<Secret> = engine.create_secret("test/empty", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_values() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": null});
        let result: CoreResult<Secret> = engine.create_secret("test/null", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_array_data() {
        let engine = MemorySecretEngine::new();
        let data = json!({"items": [1, 2, 3, 4, 5]});
        let result: CoreResult<Secret> = engine.create_secret("test/array", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_nested_objects() {
        let engine = MemorySecretEngine::new();
        let data = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "key": "deep_value"
                    }
                }
            }
        });
        let result: CoreResult<Secret> = engine.create_secret("test/nested", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unicode_data() {
        let engine = MemorySecretEngine::new();
        let data = json!({
            "chinese": "你好世界",
            "japanese": "こんにちは",
            "emoji": "🔐🎉✅"
        });
        let result: CoreResult<Secret> = engine.create_secret("test/unicode", data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_large_string_value() {
        let engine = MemorySecretEngine::new();
        let large_string = "x".repeat(10000);
        let data = json!({"large": large_string});
        let result: CoreResult<Secret> =
            engine.create_secret("test/large_string", data, None).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod metrics_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use secreton_core::services::secrets::SecretEngineMetrics;
    use serde_json::json;

    #[tokio::test]
    async fn test_metrics_after_create() {
        let engine = MemorySecretEngine::new();

        for i in 0..5 {
            let data = json!({"index": i});
            let _: CoreResult<Secret> = engine
                .create_secret(&format!("test/{}", i), data, None)
                .await;
        }

        let metrics: SecretEngineMetrics = engine.collect_metrics().await.unwrap();
        assert_eq!(metrics.engine_type, "memory");
        assert!(metrics.active_secrets >= 5);
    }

    #[tokio::test]
    async fn test_metrics_after_delete() {
        let engine = MemorySecretEngine::new();

        // Create 10 secrets
        for i in 0..10 {
            let data = json!({"index": i});
            let _: CoreResult<Secret> = engine
                .create_secret(&format!("test/{}", i), data, None)
                .await;
        }

        // Delete 5 secrets
        for i in 0..5 {
            let _: CoreResult<()> = engine.delete_secret(&format!("test/{}", i)).await;
        }

        let metrics: SecretEngineMetrics = engine.collect_metrics().await.unwrap();
        assert!(metrics.active_secrets >= 5);
    }

    #[tokio::test]
    async fn test_metrics_empty_engine() {
        let engine = MemorySecretEngine::new();
        let metrics: SecretEngineMetrics = engine.collect_metrics().await.unwrap();
        assert_eq!(metrics.engine_type, "memory");
        assert_eq!(metrics.active_secrets, 0);
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::task;

    #[tokio::test]
    async fn test_concurrent_creates() {
        let engine = Arc::new(MemorySecretEngine::new());
        let mut handles = vec![];

        for i in 0..20 {
            let engine_clone: Arc<MemorySecretEngine> = Arc::clone(&engine);
            let handle: tokio::task::JoinHandle<CoreResult<Secret>> = task::spawn(async move {
                let data = json!({"index": i});
                engine_clone
                    .create_secret(&format!("concurrent/{}", i), data, None)
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            let res: secreton_core::CoreResult<secreton_core::models::secret::Secret> =
                handle.await.unwrap();
            assert!(res.is_ok());
        }

        let secrets: Vec<String> = engine.list_secrets("concurrent/").await.unwrap();
        assert!(secrets.len() >= 20);
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let engine = Arc::new(MemorySecretEngine::new());

        // Create a secret first
        let data = json!({"key": "value"});
        let _: CoreResult<Secret> = engine.create_secret("shared/secret", data, None).await;

        let mut handles = vec![];
        for _ in 0..50 {
            let engine_clone: Arc<MemorySecretEngine> = Arc::clone(&engine);
            let handle: tokio::task::JoinHandle<CoreResult<Secret>> =
                task::spawn(async move { engine_clone.read_secret("shared/secret").await });
            handles.push(handle);
        }

        for handle in handles {
            let res: secreton_core::CoreResult<secreton_core::models::secret::Secret> =
                handle.await.unwrap();
            assert!(res.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_updates() {
        let engine = Arc::new(MemorySecretEngine::new());

        // Create initial secret
        let data = json!({"counter": 0});
        let _: CoreResult<Secret> = engine.create_secret("shared/counter", data, None).await;

        let mut handles = vec![];
        for i in 0..10 {
            let engine_clone: Arc<MemorySecretEngine> = Arc::clone(&engine);
            let handle: tokio::task::JoinHandle<CoreResult<Secret>> = task::spawn(async move {
                let data = json!({"counter": i});
                engine_clone
                    .update_secret("shared/counter", data, None)
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            let res: secreton_core::CoreResult<secreton_core::models::secret::Secret> =
                handle.await.unwrap();
            assert!(res.is_ok());
        }

        let secret_res: CoreResult<Secret> = engine.read_secret("shared/counter").await;
        let secret = secret_res.unwrap();
        assert!(secret.version > 1);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn test_very_long_path() {
        let engine = MemorySecretEngine::new();
        let long_path = "a/".repeat(50) + "secret";
        let data = json!({"key": "value"});
        let result: CoreResult<Secret> = engine.create_secret(&long_path, data, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_path_with_dots() {
        let engine = MemorySecretEngine::new();
        let data = json!({"key": "value"});
        let result: CoreResult<Secret> = engine
            .create_secret("app/../config/secret", data, None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rapid_create_delete() {
        let engine = MemorySecretEngine::new();

        for i in 0..100 {
            let data = json!({"index": i});
            let _: CoreResult<Secret> = engine.create_secret("rapid/test", data, None).await;
            let _: CoreResult<()> = engine.delete_secret("rapid/test").await;
        }

        let result: CoreResult<Secret> = engine.read_secret("rapid/test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_immediately_after_create() {
        let engine = MemorySecretEngine::new();

        let data1 = json!({"version": 1});
        let _: CoreResult<Secret> = engine.create_secret("test/immediate", data1, None).await;

        let data2 = json!({"version": 2});
        let updated: Secret = engine
            .update_secret("test/immediate", data2, None)
            .await
            .unwrap();

        assert_eq!(updated.version, 2);
    }

    #[tokio::test]
    async fn test_special_characters_in_values() {
        let engine = MemorySecretEngine::new();
        let data = json!({
            "password": "p@$$w0rd!#%&*()[]{}",
            "url": "https://example.com:8080/path?query=value&other=123",
            "json": "{\"nested\": \"value\"}"
        });
        let result: CoreResult<Secret> = engine.create_secret("test/special", data, None).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod stress_tests {
    use secreton_core::CoreResult;
    use secreton_core::engines::MemorySecretEngine;
    use secreton_core::models::secret::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn test_many_secrets() {
        let engine = MemorySecretEngine::new();

        // Create 500 secrets
        for i in 0..500 {
            let data = json!({"index": i});
            let _: CoreResult<Secret> = engine
                .create_secret(&format!("stress/test_{}", i), data, None)
                .await;
        }

        let secrets_res: CoreResult<Vec<String>> = engine.list_secrets("stress/").await;
        let secrets = secrets_res.unwrap();
        assert!(secrets.len() >= 500);
    }

    #[tokio::test]
    async fn test_deep_nesting() {
        let engine = MemorySecretEngine::new();

        let mut nested = json!({"value": "deep"});
        for _ in 0..50 {
            nested = json!({"nested": nested});
        }

        let result: CoreResult<Secret> = engine.create_secret("test/deep_nest", nested, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_many_keys() {
        let engine = MemorySecretEngine::new();

        let mut data = serde_json::Map::new();
        for i in 0..1000 {
            data.insert(format!("key_{}", i), json!(format!("value_{}", i)));
        }

        let result: CoreResult<Secret> = engine
            .create_secret("test/many_keys", json!(data), None)
            .await;
        assert!(result.is_ok());
    }
}
