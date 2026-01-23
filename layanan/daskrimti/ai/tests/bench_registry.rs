use layanan_ai::models::{ModelMetadata, ModelRegistry};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use tokio::time::Instant;
use uuid::Uuid;

// Replicating the pattern in handlers.rs (Optimized)
static REGISTRY: Lazy<Arc<Mutex<ModelRegistry>>> =
    Lazy::new(|| Arc::new(Mutex::new(ModelRegistry::new())));

#[tokio::test]
async fn bench_registry_optimized() {
    let start = Instant::now();
    let tasks = 100;
    let writes_per_task = 100;
    let reads_per_task = 1000;

    let handles = (0..tasks)
        .map(|_| {
            tokio::spawn(async move {
                let id = Uuid::new_v4().to_string();

                // Write: Add models
                for i in 0..writes_per_task {
                    let meta = ModelMetadata {
                        id: format!("{}_{}", id, i),
                        name: "test".to_string(),
                        version: "1.0".to_string(),
                        path: "path".to_string(),
                        status: "draft".to_string(),
                        created_at: chrono::Utc::now(),
                        approved_by: None,
                    };
                    REGISTRY.lock().unwrap().add_model(meta);
                }

                // Read: Get models
                for i in 0..reads_per_task {
                    let _ = REGISTRY.lock().unwrap().get_model(&format!(
                        "{}_{}",
                        id,
                        i % writes_per_task
                    ));
                }

                // Write: Approve models (Update)
                for i in 0..writes_per_task {
                    REGISTRY
                        .lock()
                        .unwrap()
                        .approve_model(&format!("{}_{}", id, i), "admin");
                }
            })
        })
        .collect::<Vec<_>>();

    for h in handles {
        h.await.unwrap();
    }
    let duration = start.elapsed();
    println!("BENCH_RESULT_OPTIMIZED: {:?}", duration);
}
