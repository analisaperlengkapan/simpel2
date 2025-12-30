use crate::config::AppConfig;
use crate::email::EmailService;
use crate::error::AppError;
use crate::push::PushService;
use crate::security::RateLimitState;
use crate::template::TemplateService;
use crate::websocket::{self, WsState};
use crate::whatsapp::WhatsAppService;
use axum::{
    Router,
    extract::{Json, Path, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use deadpool_postgres::Pool;
use garde::Validate;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub email_service: Arc<EmailService>,
    pub whatsapp_service: Arc<WhatsAppService>,
    pub push_service: Arc<PushService>,
    pub template_service: Arc<TemplateService>,
    pub pool: Pool,
    pub ws_state: WsState,
}

pub fn routes(
    app_config: AppConfig,
    pool: Pool,
    _rate_limit_state: RateLimitState,
    ws_state: WsState,
) -> Router {
    let email_service = Arc::new(EmailService::new(app_config.clone(), pool.clone()));
    let whatsapp_service = Arc::new(WhatsAppService::new(app_config.clone(), pool.clone()));
    let push_service = Arc::new(PushService::new(app_config.clone(), pool.clone()));
    let template_service = Arc::new(TemplateService::new(pool.clone()));

    let app_state = AppState {
        email_service,
        whatsapp_service,
        push_service,
        template_service,
        pool: pool.clone(),
        ws_state: ws_state.clone(),
    };

    Router::new()
        // WebSocket
        .route("/notifications/ws", get(websocket::ws_handler))
        .with_state(ws_state)
        .merge(
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
                .with_state(app_state),
        )
}

#[derive(Deserialize, Validate)]
pub struct SendEmailRequest {
    #[garde(email)]
    pub recipient: String,
    #[garde(length(min = 1, max = 255))]
    pub subject: String,
    #[garde(length(min = 1))]
    pub body: String,
}

#[derive(Deserialize, Validate)]
pub struct SendBatchEmailRequestSimple {
    #[garde(length(min = 1, max = 100))]
    #[garde(inner(email))]
    pub recipients: Vec<String>,
    #[garde(length(min = 1, max = 255))]
    pub subject: String,
    #[garde(length(min = 1))]
    pub body: String,
}

#[derive(Deserialize, Validate)]
pub struct SendWhatsAppRequest {
    #[garde(length(min = 10, max = 20))]
    pub phone_number: String,
    #[garde(length(min = 1, max = 100))]
    pub template_name: String,
    #[garde(skip)]
    pub variables: Value,
}

#[derive(Deserialize, Validate)]
pub struct SendBatchWhatsAppRequest {
    #[garde(length(min = 1, max = 100))]
    #[garde(inner(length(min = 10, max = 20)))]
    pub recipients: Vec<String>,
    #[garde(length(min = 1, max = 100))]
    pub template_name: String,
    #[garde(skip)]
    pub variables: Value,
}

#[derive(Deserialize, Validate)]
pub struct SendPushRequest {
    #[garde(length(min = 1))]
    pub device_token: String,
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    #[garde(length(min = 1))]
    pub body: String,
    #[garde(skip)]
    pub data: Value,
}

#[derive(Deserialize, Validate)]
pub struct SendBatchPushRequest {
    #[garde(length(min = 1, max = 100))]
    #[garde(inner(length(min = 1)))]
    pub device_tokens: Vec<String>,
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    #[garde(length(min = 1))]
    pub body: String,
    #[garde(skip)]
    pub data: Value,
}

#[derive(Deserialize, Validate)]
pub struct CreateTemplateRequest {
    #[garde(length(min = 1, max = 100))]
    pub name: String,
    #[garde(length(min = 1))]
    pub content: String,
    #[garde(skip)]
    pub variables: Value,
}

#[derive(Deserialize, Validate)]
pub struct UpdateTemplateRequest {
    #[garde(length(min = 1))]
    pub content: String,
    #[garde(skip)]
    pub variables: Value,
}

pub async fn send_email(
    State(state): State<AppState>,
    Json(payload): Json<SendEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .email_service
        .send_email(&payload.recipient, &payload.subject, &payload.body)
        .await?;
    Ok(Json(result))
}

pub async fn send_batch_email(
    State(state): State<AppState>,
    Json(payload): Json<SendBatchEmailRequestSimple>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .email_service
        .send_batch_emails(payload.recipients, &payload.subject, &payload.body)
        .await?;
    Ok(Json(result))
}

pub async fn get_email_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.email_service.get_status(id).await?;
    Ok(Json(result))
}

pub async fn send_whatsapp(
    State(state): State<AppState>,
    Json(payload): Json<SendWhatsAppRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .whatsapp_service
        .send_whatsapp(&payload.phone_number, &payload.template_name, payload.variables)
        .await?;
    Ok(Json(result))
}

pub async fn send_batch_whatsapp(
    State(state): State<AppState>,
    Json(payload): Json<SendBatchWhatsAppRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .whatsapp_service
        .send_batch_whatsapp(
            payload.recipients,
            &payload.template_name,
            payload.variables,
        )
        .await?;
    Ok(Json(result))
}

pub async fn get_whatsapp_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.whatsapp_service.get_status(id).await?;
    Ok(Json(result))
}

pub async fn send_push(
    State(state): State<AppState>,
    Json(payload): Json<SendPushRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .push_service
        .send_push(
            &payload.device_token,
            &payload.title,
            &payload.body,
            payload.data,
        )
        .await?;
    Ok(Json(result))
}

pub async fn send_batch_push(
    State(state): State<AppState>,
    Json(payload): Json<SendBatchPushRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .push_service
        .send_batch_push(
            payload.device_tokens,
            &payload.title,
            &payload.body,
            payload.data,
        )
        .await?;
    Ok(Json(result))
}

pub async fn get_push_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.push_service.get_status(id).await?;
    Ok(Json(result))
}

pub async fn get_templates(State(_state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // TODO: Implement list templates if needed
    Ok(Json(json!({"message": "get_templates stub"})))
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .template_service
        .create_template(&payload.name, &payload.content, payload.variables)
        .await?;
    Ok(Json(result))
}

pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string().into()))?;

    let result = state
        .template_service
        .update_template(id, &payload.content, payload.variables)
        .await?;
    Ok(Json(result))
}

pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.template_service.delete_template(id).await?;
    Ok(Json(json!({"message": "Template deleted"})))
}

pub async fn get_audit_logs() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({"message": "get_audit_logs stub"})))
}

pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
