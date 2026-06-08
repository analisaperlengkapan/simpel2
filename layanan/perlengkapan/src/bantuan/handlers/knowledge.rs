use crate::bantuan::error::AppError;
use crate::bantuan::knowledge::KnowledgeService;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub async fn list_knowledge_articles(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = params
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.list_articles(category_id).await?;
    Ok(Json(arts))
}
pub async fn create_knowledge_article(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let knowledge = KnowledgeService::new(pool.clone());
    let art = knowledge
        .create_article(title, content, category_id, tags)
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(art))
}
pub async fn get_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool);
    let art = knowledge.get_article(id).await?;
    Ok(Json(art))
}
pub async fn update_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let knowledge = KnowledgeService::new(pool.clone());
    let art = knowledge
        .update_article(id, title, content, category_id, tags)
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(art))
}
pub async fn delete_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool.clone());
    knowledge.delete_article(id).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "deleted", "id": id})))
}
pub async fn search_knowledge(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.search(query, max_results).await?;
    Ok(Json(arts))
}
pub async fn get_related_knowledge(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params["limit"].as_u64().unwrap_or(5) as u32;
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.get_related(id, limit).await?;
    Ok(Json(arts))
}
pub async fn export_knowledge(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.export_articles().await?;
    Ok(Json(arts))
}
pub async fn import_knowledge(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool.clone());
    let articles = serde_json::from_value(payload["articles"].clone())
        .map_err(|e| AppError::Validation(e.to_string()))?;
    knowledge.import_articles(articles).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "imported"})))
}
