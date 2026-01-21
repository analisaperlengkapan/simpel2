//! Configuration handlers

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: serde_json::Value,
}

pub async fn list_configs(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<ConfigItem>>> {
    // TODO: Fetch from database
    Ok(Json(vec![]))
}

pub async fn get_config(
    State(_state): State<Arc<AppState>>,
    Path(config_id): Path<Uuid>,
) -> Result<Json<ConfigItem>> {
    Ok(Json(ConfigItem {
        id: config_id,
        key: "sample_config".to_string(),
        value: serde_json::json!({}),
        category: "general".to_string(),
    }))
}

pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Path(config_id): Path<Uuid>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigItem>> {
    Ok(Json(ConfigItem {
        id: config_id,
        key: "sample_config".to_string(),
        value: request.value,
        category: "general".to_string(),
    }))
}
