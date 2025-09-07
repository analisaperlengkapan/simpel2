use axum::{Json, http::StatusCode, extract::{Multipart, Path}};
use serde_json::json;
use axum::extract::State;
use crate::models::AppState;
use uuid::Uuid;
use validator::Validate;
use crate::models::{ModelRegistry, ModelMetadata};
use chrono::Utc;
use reqwest::Client;

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTextRequest {
    #[validate(length(min = 3))]
    pub prompt: String,
    pub ab_group: Option<String>,
}

fn sanitize_input(input: &str) -> String {
    let s = input.replace(|c: char| c.is_control(), "");
    htmlescape::encode_minimal(&s)
}
fn sanitize_output(output: &str) -> String {
    htmlescape::encode_minimal(output)
}

fn problem_json(status: StatusCode, title: &str, detail: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": detail
        }))
    )
}

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // Cek DB
    let db_ok = match state.pool.get().await {
        Ok(client) => client.query("SELECT 1", &[]).await.is_ok(),
        Err(_) => false,
    };
    // Cek Qdrant
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let qdrant_ok = Client::new().get(format!("{}/collections", qdrant_url)).send().await.map(|r| r.status().is_success()).unwrap_or(false);
    // Cek model ready (dummy: assume always ready)
    let model_ok = true;
    let all_ok = db_ok && qdrant_ok && model_ok;
    if all_ok {
        (StatusCode::OK, Json(json!({"status": "ok", "db": db_ok, "qdrant": qdrant_ok, "model": model_ok})))
    } else {
        let mut detail = vec![];
        if !db_ok { detail.push("db"); }
        if !qdrant_ok { detail.push("qdrant"); }
        if !model_ok { detail.push("model"); }
        problem_json(StatusCode::SERVICE_UNAVAILABLE, "Healthcheck Failed", &format!("Component(s) failed: {}", detail.join(", ")))
    }
}

macro_rules! not_implemented {
    () => {
        async { (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Not implemented"}))) }
    };
}

pub async fn generate_text(State(state): State<AppState>, Json(payload): Json<GenerateTextRequest>) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Validation error: {}", e)})));
    }
    let prompt = sanitize_input(&payload.prompt);
    // A/B testing logic
    let ab_group = payload.ab_group.as_deref().unwrap_or("A");
    let (result, model_used) = if ab_group == "B" {
        (format!("[B] AI: {}", prompt), "model_B")
    } else {
        (format!("[A] AI: {}", prompt), "model_A")
    };
    let result = sanitize_output(&result);
    (StatusCode::OK, Json(json!({"result": result, "model_used": model_used})))
}
pub async fn summarize_text(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn extract_text(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn rag_query(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn rag_ingest(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn classify_document(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn generate_recommendation(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn train_model(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn predict(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn rlhf_train(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn active_learning_query(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn transfer_learning(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }
pub async fn hitl_annotate(_: State<AppState>, _: Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) { not_implemented!().await }

pub async fn upload_model(_: State<AppState>, _: Multipart) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Not implemented"})))
}

pub async fn download_model(_: State<AppState>, Path(_): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Not implemented"})))
}

pub async fn approve_model(State(_): State<AppState>, Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) {
    let user = payload["user"].as_str().unwrap_or("admin");
    use once_cell::sync::Lazy;
    static REGISTRY: Lazy<Mutex<ModelRegistry>> = Lazy::new(|| Mutex::new(ModelRegistry::new()));
    if REGISTRY.lock().unwrap().approve_model(&id, user) {
        (StatusCode::OK, Json(json!({"model_id": id, "status": "approved", "approved_by": user})))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Model not found"})))
    }
}

pub async fn rollback_model(_: State<AppState>, Path(_): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Not implemented"})))
}

pub async fn enqueue_job(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) {
    // Resource limit: max 1024MB mem, 2 core
    let mem = payload["requested_memory_mb"].as_u64().unwrap_or(0);
    let cpu = payload["requested_cpu"].as_f64().unwrap_or(0.0);
    if mem > 1024 {
        return problem_json(StatusCode::BAD_REQUEST, "Resource Limit Exceeded", "Requested memory exceeds 1024MB");
    }
    if cpu > 2.0 {
        return problem_json(StatusCode::BAD_REQUEST, "Resource Limit Exceeded", "Requested CPU exceeds 2 core");
    }
    let id = Uuid::new_v4();
    let job_type = payload["job_type"].as_str().unwrap_or("unknown");
    let status = "queued";
    let now = chrono::Utc::now();
    let payload_json = serde_json::to_value(&payload).unwrap_or(json!({}));

    let client = state.pool.get().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database connection failed"}))))?;
    let res = client
        .execute(
            "INSERT INTO ai_jobs (id, job_type, payload, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&id, &job_type, &payload_json, &status, &now, &now]
        )
        .await;
    )
    .execute(&state.pool)
    .await;
    match res {
        Ok(_) => (StatusCode::OK, Json(json!({"job_id": id, "status": status}))),
        Err(e) => problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", &format!("DB error: {}", e)),
    }
}

pub async fn job_status(State(state): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let uuid = match uuid::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return problem_json(StatusCode::BAD_REQUEST, "Invalid Job ID", "Job id is not a valid UUID"),
    };

    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(_) => return problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", "Failed to get database connection"),
    };

    let rows = client
        .query("SELECT status, result, error FROM ai_jobs WHERE id = $1", &[&uuid])
        .await;

    match rows {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                (StatusCode::OK, Json(json!({
                    "job_id": id,
                    "status": row.get::<_, String>("status"),
                    "result": row.get::<_, Option<serde_json::Value>>("result"),
                    "error": row.get::<_, Option<String>>("error")
                })))
            } else {
                problem_json(StatusCode::NOT_FOUND, "Job Not Found", "Job not found")
            }
        }
        },
        Ok(None) => problem_json(StatusCode::NOT_FOUND, "Job Not Found", "No job found with the given id"),
        Err(e) => problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", &format!("DB error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::extract::State;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use crate::models::{AppState, JobQueue};

    #[tokio::test]
    async fn test_health() {
        let status = health().await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_enqueue_and_status() {
        let job_queue = Arc::new(Mutex::new(JobQueue::new()));
        let pool_config = deadpool_postgres::Config {
            user: Some("user".to_string()),
            password: Some("pass".to_string()),
            host: Some("localhost".to_string()),
            dbname: Some("test".to_string()),
            ..Default::default()
        };
        let pool = pool_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap();
        let state = AppState {
            pool,
            config: crate::config::Config::default(),
            llm_service: crate::llm::LlmService,
            ocr_service: crate::ocr::OcrService,
            rag_service: crate::rag::RagService,
            job_queue: job_queue.clone(),
        };
        let payload = json!({"job_type": "test", "data": "abc"});
        let (status, resp) = enqueue_job(State(state.clone()), axum::Json(payload)).await;
        assert_eq!(status, StatusCode::OK);
        let job_id = resp.0["job_id"].as_str().unwrap();
        let (status2, resp2) = job_status(State(state), axum::extract::Path(job_id.to_string())).await;
        assert_eq!(status2, StatusCode::OK);
        assert_eq!(resp2.0["status"], "queued");
    }
}

pub async fn register_model(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) {
    let id = Uuid::new_v4().to_string();
    let meta = ModelMetadata {
        id: id.clone(),
        name: payload["name"].as_str().unwrap_or("").to_string(),
        version: payload["version"].as_str().unwrap_or("1.0").to_string(),
        path: payload["path"].as_str().unwrap_or("").to_string(),
        status: "draft".to_string(),
        created_at: Utc::now(),
        approved_by: None,
    };
    // Sementara: registry in-memory global static
    use once_cell::sync::Lazy;
    static REGISTRY: Lazy<Mutex<ModelRegistry>> = Lazy::new(|| Mutex::new(ModelRegistry::new()));
    REGISTRY.lock().unwrap().add_model(meta);
    (StatusCode::OK, Json(json!({"model_id": id, "status": "draft"})))
}

pub async fn get_model(State(_): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    use once_cell::sync::Lazy;
    static REGISTRY: Lazy<Mutex<ModelRegistry>> = Lazy::new(|| Mutex::new(ModelRegistry::new()));
    if let Some(meta) = REGISTRY.lock().unwrap().get_model(&id) {
        (StatusCode::OK, Json(json!({"id": meta.id, "name": meta.name, "version": meta.version, "status": meta.status, "approved_by": meta.approved_by})))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Model not found"})))
    }
}

pub async fn cancel_job(State(state): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let uuid = match uuid::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return problem_json(StatusCode::BAD_REQUEST, "Invalid Job ID", "Job id is not a valid UUID"),
    };
    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(e) => return problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Connection Error", &format!("Pool error: {}", e)),
    };
    let res = client
        .execute("UPDATE ai_jobs SET status = 'cancelled', updated_at = NOW() WHERE id = $1 AND status IN ('queued','running')", &[&uuid])
        .await;
    match res {
        Ok(rows_affected) if rows_affected > 0 => (StatusCode::OK, Json(json!({"job_id": id, "status": "cancelled"}))),
        Ok(_) => problem_json(StatusCode::NOT_FOUND, "Job Not Found or Not Cancellable", "No cancellable job found with the given id"),
        Err(e) => problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", &format!("DB error: {}", e)),
    }
}

pub async fn retry_job(State(state): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let uuid = match uuid::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return problem_json(StatusCode::BAD_REQUEST, "Invalid Job ID", "Job id is not a valid UUID"),
    };
    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(e) => return problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Connection Error", &format!("Pool error: {}", e)),
    };
    let res = client
        .execute("UPDATE ai_jobs SET status = 'queued', updated_at = NOW() WHERE id = $1 AND status IN ('failed','cancelled')", &[&uuid])
        .await;
    match res {
        Ok(rows_affected) if rows_affected > 0 => (StatusCode::OK, Json(json!({"job_id": id, "status": "queued"}))),
        Ok(_) => problem_json(StatusCode::NOT_FOUND, "Job Not Found or Not Retryable", "No retryable job found with the given id"),
        Err(e) => problem_json(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", &format!("DB error: {}", e)),
    }
}

pub async fn explain(State(_): State<AppState>, Json(payload): Json<serde_json::Value>) -> (StatusCode, Json<serde_json::Value>) {
    let input = payload["input"].as_str().unwrap_or("");
    // Dummy explain: random feature importance
    let importance = vec![
        ("feature1", 0.7),
        ("feature2", 0.2),
        ("feature3", 0.1),
    ];
    (StatusCode::OK, Json(json!({
        "input": input,
        "reason": "Prediction didominasi oleh feature1",
        "feature_importance": importance
    })))
}

pub async fn model_monitor(State(_): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // Dummy metrics
    let metrics = json!({
        "accuracy": 0.92,
        "drift": 0.03,
        "latency_ms": 120,
        "last_update": chrono::Utc::now(),
    });
    (StatusCode::OK, Json(metrics))
}
