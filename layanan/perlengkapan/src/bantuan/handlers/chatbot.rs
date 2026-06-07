use crate::bantuan::chatbot::ChatbotService;
use crate::bantuan::error::AppError;
use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub async fn chatbot_query(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let message = payload["message"]
        .as_str()
        .ok_or(AppError::Validation("message wajib".to_string()))?;
    let context = payload.get("context").cloned();
    // TODO(bantuan-chatbot-config): build ChatbotService from the bantuan
    // `AppConfig` (already loaded in `routes()`) instead of hard-coded
    // `http://localhost:3002`. Drops out automatically once the bantuan
    // router gets mounted into the unified app and `State<Pool>` is
    // replaced with `AppState` that already exposes bantuan config.
    let chatbot = ChatbotService::new(pool.clone(), "http://localhost:3002".to_string(), None);
    let reply = chatbot.process_query(user_id, message, context).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"reply": reply})))
}
pub async fn chatbot_history(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let conversation_id = params["conversation_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("conversation_id wajib".to_string()))?;
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    let history = chatbot.get_history(conversation_id).await?;
    Ok(Json(history))
}
pub async fn chatbot_feedback(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let message_id = payload["message_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("message_id wajib".to_string()))?;
    let feedback = payload["feedback"]
        .as_i64()
        .ok_or(AppError::Validation("feedback wajib".to_string()))? as i32;
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    chatbot.record_feedback(message_id, feedback).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "ok"})))
}
pub async fn chatbot_suggestions(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let context = params.get("context").cloned();
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    let suggestions = chatbot.get_suggestions(user_id, context).await?;
    Ok(Json(suggestions))
}
