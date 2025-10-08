use crate::config::AppConfig;
use crate::email::EmailService;
use crate::error::AppError;
use crate::push::PushService;
use crate::security::RateLimitState;
use crate::template::TemplateService;
use crate::whatsapp::WhatsAppService;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub fn routes(app_config: AppConfig, pool: Pool, _rate_limit_state: RateLimitState) -> Router {
    let _email = EmailService::new(app_config.clone(), pool.clone());
    let _whatsapp = WhatsAppService::new(app_config.clone(), pool.clone());
    let _push = PushService::new(app_config.clone(), pool.clone());
    let _template = TemplateService::new(pool.clone());
    Router::new()
        // Email
        .route("/notifications/email/send", post(send_email))
        .route("/notifications/email/batch", post(send_batch_email))
        .route("/notifications/email/status/:id", get(get_email_status))
        // WhatsApp
        .route("/notifications/whatsapp/send", post(send_whatsapp))
        .route("/notifications/whatsapp/batch", post(send_batch_whatsapp))
        .route(
            "/notifications/whatsapp/status/:id",
            get(get_whatsapp_status),
        )
        // Push
        .route("/notifications/push/send", post(send_push))
        .route("/notifications/push/batch", post(send_batch_push))
        .route("/notifications/push/status/:id", get(get_push_status))
        // Template
        .route("/templates", get(get_templates).post(create_template))
        .route(
            "/templates/:id",
            put(update_template).delete(delete_template),
        )
        // Audit
        .route("/audit/logs", get(get_audit_logs))
        // Health
        .route("/health", get(health))
        .with_state(pool)
}

// Handler stub (isi detail bertahap)
pub async fn send_email() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_email stub"})))
}
pub async fn send_batch_email() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_batch_email stub"})))
}
pub async fn get_email_status(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "get_email_status stub", "id": id})))
}
pub async fn send_whatsapp() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_whatsapp stub"})))
}
pub async fn send_batch_whatsapp() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_batch_whatsapp stub"})))
}
pub async fn get_whatsapp_status(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(
        json!({"message": "get_whatsapp_status stub", "id": id}),
    ))
}
pub async fn send_push() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_push stub"})))
}
pub async fn send_batch_push() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "send_batch_push stub"})))
}
pub async fn get_push_status(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "get_push_status stub", "id": id})))
}
pub async fn get_templates() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "get_templates stub"})))
}
pub async fn create_template() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "create_template stub"})))
}
pub async fn update_template(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "update_template stub", "id": id})))
}
pub async fn delete_template(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "delete_template stub", "id": id})))
}
pub async fn get_audit_logs() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "get_audit_logs stub"})))
}
pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
