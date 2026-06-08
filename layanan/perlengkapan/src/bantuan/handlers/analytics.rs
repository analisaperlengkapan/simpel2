use crate::bantuan::analytics::AnalyticsService;
use crate::bantuan::error::AppError;
use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use uuid::Uuid;

pub async fn analytics_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    use chrono::{DateTime, Utc};
    let start = params["start"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = params["end"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_ticket_analytics(start, end).await?;
    Ok(Json(data))
}
pub async fn analytics_faq(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = params
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let period = params["period"].as_str().unwrap_or("month");
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_faq_analytics(category_id, period).await?;
    Ok(Json(data))
}
pub async fn analytics_chatbot(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let period = params["period"].as_str().unwrap_or("month");
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_chatbot_analytics(user_id, period).await?;
    Ok(Json(data))
}
pub async fn analytics_satisfaction(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    use chrono::{DateTime, Utc};
    let start = params["start"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = params["end"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_satisfaction(start, end).await?;
    Ok(Json(data))
}
pub async fn get_audit_logs(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let action = params.get("action").and_then(|v| v.as_str());
    let resource = params.get("resource").and_then(|v| v.as_str());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as i64;
    let logs =
        crate::bantuan::audit::query_audit_logs(&pool, user_id, action, resource, limit).await?;
    Ok(Json(logs))
}
