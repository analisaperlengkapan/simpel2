use crate::bantuan::error::AppError;
use crate::bantuan::faq::FaqService;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

// Handler stub (isi detail bertahap)
pub async fn list_faq_categories(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let cats = faq.list_categories().await?;
    Ok(Json(cats))
}
pub async fn create_faq_category(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = payload["name"]
        .as_str()
        .ok_or(AppError::Validation("name wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let parent_id = payload
        .get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let faq = FaqService::new(pool.clone());
    let cat = faq.create_category(name, description, parent_id).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(cat))
}
pub async fn get_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let cat = faq.get_category(id).await?;
    Ok(Json(cat))
}
pub async fn update_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = payload["name"]
        .as_str()
        .ok_or(AppError::Validation("name wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let parent_id = payload
        .get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let faq = FaqService::new(pool.clone());
    let cat = faq
        .update_category(id, name, description, parent_id)
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(cat))
}
pub async fn delete_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool.clone());
    faq.delete_category(id).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "deleted", "id": id})))
}
pub async fn list_faq_articles_by_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let arts = faq.list_articles(Some(id)).await?;
    Ok(Json(arts))
}
pub async fn list_faq_articles(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let arts = faq.list_articles(None).await?;
    Ok(Json(arts))
}
pub async fn create_faq_article(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = payload["category_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("category_id wajib".to_string()))?;
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let faq = FaqService::new(pool.clone());
    let art = faq
        .create_article(category_id, title, content, tags)
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(art))
}
pub async fn get_faq_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let art = faq.get_article(id).await?;
    Ok(Json(art))
}
pub async fn update_faq_article(
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
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let faq = FaqService::new(pool.clone());
    let art = faq.update_article(id, title, content, tags).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(art))
}
pub async fn delete_faq_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool.clone());
    faq.delete_article(id).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "deleted", "id": id})))
}
pub async fn search_faq(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let faq = FaqService::new(pool);
    let arts = faq.search_faqs(query, max_results).await?;
    Ok(Json(arts))
}
