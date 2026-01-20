use crate::{llm::LlmService, ocr::OcrService, rag::RagService};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub config: crate::config::Config,
    pub llm_service: LlmService,
    pub ocr_service: OcrService,
    pub rag_service: RagService,
    pub job_queue: Arc<Mutex<JobQueue>>,
    pub http_client: reqwest::Client,
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub payload: String,
    pub status: String,
}

pub struct JobQueue {
    pub queue: VecDeque<Job>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn enqueue(&mut self, job: Job) {
        self.queue.push_back(job);
    }
    pub fn dequeue(&mut self) -> Option<Job> {
        self.queue.pop_front()
    }
    pub fn status(&self, id: &str) -> Option<Job> {
        self.queue.iter().find(|j| j.id == id).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub status: String, // draft, approved, deprecated
    pub created_at: DateTime<Utc>,
    pub approved_by: Option<String>,
}

pub struct ModelRegistry {
    pub models: DashMap<String, ModelMetadata>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self { models: DashMap::new() }
    }
    pub fn add_model(&self, meta: ModelMetadata) {
        self.models.insert(meta.id.clone(), meta);
    }
    pub fn get_model(&self, id: &str) -> Option<ModelMetadata> {
        self.models.get(id).map(|m| m.clone())
    }
    pub fn approve_model(&self, id: &str, user: &str) -> bool {
        if let Some(mut m) = self.models.get_mut(id) {
            m.status = "approved".to_string();
            m.approved_by = Some(user.to_string());
            true
        } else {
            false
        }
    }
    // dst, bisa dikembangkan
}
