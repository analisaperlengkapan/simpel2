use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub approved_by: Option<String>,
}

// Current Implementation (Vec)
pub struct ModelRegistryVec {
    pub models: Vec<ModelMetadata>,
}

impl ModelRegistryVec {
    pub fn new() -> Self {
        Self { models: vec![] }
    }
    pub fn add_model(&mut self, meta: ModelMetadata) {
        self.models.push(meta);
    }
    pub fn get_model(&self, id: &str) -> Option<&ModelMetadata> {
        self.models.iter().find(|m| m.id == id)
    }
    pub fn approve_model(&mut self, id: &str, user: &str) -> bool {
        if let Some(m) = self.models.iter_mut().find(|m| m.id == id) {
            m.status = "approved".to_string();
            m.approved_by = Some(user.to_string());
            true
        } else {
            false
        }
    }
}

// Optimized Implementation (HashMap)
pub struct ModelRegistryMap {
    pub models: HashMap<String, ModelMetadata>,
}

impl ModelRegistryMap {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }
    pub fn add_model(&mut self, meta: ModelMetadata) {
        self.models.insert(meta.id.clone(), meta);
    }
    pub fn get_model(&self, id: &str) -> Option<&ModelMetadata> {
        self.models.get(id)
    }
    pub fn approve_model(&mut self, id: &str, user: &str) -> bool {
        if let Some(m) = self.models.get_mut(id) {
            m.status = "approved".to_string();
            m.approved_by = Some(user.to_string());
            true
        } else {
            false
        }
    }
}

#[tokio::test]
async fn bench_performance() {
    let num_models = 2_000; // Reduced to avoid test timeout if Vec is too slow
    let num_ops = 10_000;

    // Generate data
    let mut models = vec![];
    for i in 0..num_models {
        models.push(ModelMetadata {
            id: format!("model-{}", i),
            name: format!("Model {}", i),
            version: "1.0".to_string(),
            path: "/tmp".to_string(),
            status: "draft".to_string(),
            created_at: Utc::now(),
            approved_by: None,
        });
    }

    // 1. Mutex + Vec (Baseline / "Bad" Code)
    let registry_mutex_vec = Arc::new(Mutex::new(ModelRegistryVec::new()));
    for m in &models {
        registry_mutex_vec.lock().unwrap().add_model(m.clone());
    }

    let start = Instant::now();
    for i in 0..num_ops {
        let id = format!("model-{}", i % num_models);
        let _ = registry_mutex_vec.lock().unwrap().get_model(&id).map(|m| m.id.clone());
    }
    let duration_mutex_vec = start.elapsed();
    println!("Mutex + Vec (O(N)): {:?}", duration_mutex_vec);

    // 2. RwLock + Vec (Current Code)
    let registry_rw_vec = Arc::new(RwLock::new(ModelRegistryVec::new()));
    for m in &models {
        registry_rw_vec.write().await.add_model(m.clone());
    }

    let start = Instant::now();
    for i in 0..num_ops {
        let id = format!("model-{}", i % num_models);
        let guard = registry_rw_vec.read().await;
        let _ = guard.get_model(&id).map(|m| m.id.clone());
    }
    let duration_rw_vec = start.elapsed();
    println!("RwLock + Vec (O(N)): {:?}", duration_rw_vec);

    // 3. RwLock + HashMap (Optimized Code)
    let registry_rw_map = Arc::new(RwLock::new(ModelRegistryMap::new()));
    for m in &models {
        registry_rw_map.write().await.add_model(m.clone());
    }

    let start = Instant::now();
    for i in 0..num_ops {
        let id = format!("model-{}", i % num_models);
        let guard = registry_rw_map.read().await;
        let _ = guard.get_model(&id).map(|m| m.id.clone());
    }
    let duration_rw_map = start.elapsed();
    println!("RwLock + HashMap (O(1)): {:?}", duration_rw_map);

    // Verify improvement
    assert!(duration_rw_map < duration_rw_vec, "HashMap should be faster than Vec");
}
