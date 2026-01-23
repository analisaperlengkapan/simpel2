//! Dashboard handlers

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub period: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct OverviewResponse {
    pub total_users: i64,
    pub active_sessions: i64,
    pub total_documents: i64,
    pub pending_tasks: i64,
}

#[derive(Debug, Serialize)]
pub struct WidgetResponse {
    pub id: Uuid,
    pub name: String,
    pub widget_type: String,
    pub config: serde_json::Value,
}

pub async fn get_overview(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<DashboardQuery>,
) -> Result<Json<OverviewResponse>> {
    // TODO: Implement actual data aggregation
    Ok(Json(OverviewResponse {
        total_users: 1000,
        active_sessions: 150,
        total_documents: 5000,
        pending_tasks: 25,
    }))
}

pub async fn get_widgets(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<DashboardQuery>,
) -> Result<Json<Vec<WidgetResponse>>> {
    // TODO: Fetch from database
    Ok(Json(vec![]))
}

pub async fn get_widget(
    State(_state): State<Arc<AppState>>,
    Path(widget_id): Path<Uuid>,
) -> Result<Json<WidgetResponse>> {
    Ok(Json(WidgetResponse {
        id: widget_id,
        name: "Sample Widget".to_string(),
        widget_type: "chart".to_string(),
        config: serde_json::json!({}),
    }))
}
