use crate::config::AppConfig;
use crate::email::EmailService;
use crate::error::AppError;
use crate::in_app::{InAppNotificationChannel, NotificationPriority, NotificationType};
use crate::preferences::{NotificationPreferencesService, UpdatePreferencesRequest};
use crate::push::PushService;
use crate::security::RateLimitState;
use crate::template::TemplateService;
use crate::websocket::{self, WsState};
use crate::whatsapp::WhatsAppService;
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn routes(
    app_config: AppConfig,
    pool: Pool,
    _rate_limit_state: RateLimitState,
    ws_state: WsState,
) -> Router {
    let _ = EmailService::new(app_config.clone(), pool.clone());
    let _ = WhatsAppService::new(app_config.clone(), pool.clone());
    let _ = PushService::new(app_config.clone(), pool.clone());
    let _ = TemplateService::new(pool.clone());
    Router::new()
        // WebSocket
        .route("/notifications/ws", get(websocket::ws_handler))
        .with_state(ws_state)
        .merge(
            Router::new()
                // In-app notifications
                .route("/notifications/in-app", post(send_in_app_notification))
                .route("/notifications/in-app/list", get(list_in_app_notifications))
                .route("/notifications/in-app/unread-count", get(get_unread_count))
                .route("/notifications/in-app/:id/read", put(mark_notification_as_read))
                .route("/notifications/in-app/mark-all-read", put(mark_all_notifications_as_read))
                // Notification preferences
                .route("/notifications/preferences/:user_id", get(get_user_preferences))
                .route("/notifications/preferences/:user_id", put(update_user_preferences))
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
                .with_state(pool),
        )
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

// In-app notification handlers

#[derive(Debug, Deserialize)]
pub struct SendInAppNotificationRequest {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendInAppNotificationResponse {
    pub notification_id: Uuid,
    pub message: String,
}

pub async fn send_in_app_notification(
    State(pool): State<Pool>,
    Json(req): Json<SendInAppNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let channel = InAppNotificationChannel::new(pool);

    let priority = match req.priority.as_deref() {
        Some("low") => NotificationPriority::Low,
        Some("high") => NotificationPriority::High,
        Some("urgent") => NotificationPriority::Urgent,
        _ => NotificationPriority::Normal,
    };

    let notification_id = channel
        .send(req.user_id, req.notification_type, priority)
        .await?;

    Ok(Json(SendInAppNotificationResponse {
        notification_id,
        message: "Notification sent successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub user_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub unread_only: bool,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_in_app_notifications(
    State(pool): State<Pool>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let channel = InAppNotificationChannel::new(pool);

    let notifications = channel
        .get_notifications(query.user_id, query.limit, query.offset, query.unread_only)
        .await?;

    Ok(Json(json!({
        "notifications": notifications,
        "count": notifications.len(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct UnreadCountQuery {
    pub user_id: Uuid,
}

pub async fn get_unread_count(
    State(pool): State<Pool>,
    Query(query): Query<UnreadCountQuery>,
) -> Result<impl IntoResponse, AppError> {
    let channel = InAppNotificationChannel::new(pool);

    let count = channel.get_unread_count(query.user_id).await?;

    Ok(Json(json!({
        "unread_count": count,
    })))
}

#[derive(Debug, Deserialize)]
pub struct MarkAsReadRequest {
    pub user_id: Uuid,
}

pub async fn mark_notification_as_read(
    State(pool): State<Pool>,
    Path(notification_id): Path<Uuid>,
    Json(req): Json<MarkAsReadRequest>,
) -> Result<impl IntoResponse, AppError> {
    let channel = InAppNotificationChannel::new(pool);

    channel.mark_as_read(notification_id, req.user_id).await?;

    Ok(Json(json!({
        "message": "Notification marked as read",
    })))
}

#[derive(Debug, Deserialize)]
pub struct MarkAllAsReadRequest {
    pub user_id: Uuid,
}

pub async fn mark_all_notifications_as_read(
    State(pool): State<Pool>,
    Json(req): Json<MarkAllAsReadRequest>,
) -> Result<impl IntoResponse, AppError> {
    let channel = InAppNotificationChannel::new(pool);

    let count = channel.mark_all_as_read(req.user_id).await?;

    Ok(Json(json!({
        "message": format!("Marked {} notifications as read", count),
        "count": count,
    })))
}


// Notification preferences handlers

pub async fn get_user_preferences(
    State(pool): State<Pool>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let service = NotificationPreferencesService::new(pool);

    let preferences = service.get_preferences(user_id).await?;

    Ok(Json(preferences))
}

pub async fn update_user_preferences(
    State(pool): State<Pool>,
    Path(user_id): Path<Uuid>,
    Json(updates): Json<UpdatePreferencesRequest>,
) -> Result<impl IntoResponse, AppError> {
    let service = NotificationPreferencesService::new(pool);

    let preferences = service.update_preferences(user_id, updates).await?;

    Ok(Json(json!({
        "message": "Preferences updated successfully",
        "preferences": preferences,
    })))
}
