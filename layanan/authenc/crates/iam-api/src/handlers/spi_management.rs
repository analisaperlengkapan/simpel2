//! SPI (Service Provider Interface) Management HTTP handlers

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// SPI plugin configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SpiPluginConfig {
    pub id: Uuid,
    pub name: String,
    pub plugin_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

/// GET /api/v1/iam/spi/plugins - List SPI plugins
pub async fn list_spi_plugins(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<SpiPluginConfig>>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/spi/plugins - Register SPI plugin
pub async fn register_spi_plugin(
    State(_state): State<Arc<IamApiState>>,
    Json(_config): Json<SpiPluginConfig>,
) -> ApiResult<(StatusCode, Json<SpiPluginConfig>)> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/spi/plugins/{id} - Get SPI plugin
pub async fn get_spi_plugin(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<SpiPluginConfig>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/spi/plugins/{id} - Update SPI plugin
pub async fn update_spi_plugin(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_config): Json<SpiPluginConfig>,
) -> ApiResult<Json<SpiPluginConfig>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/spi/plugins/{id} - Unregister SPI plugin
pub async fn unregister_spi_plugin(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/spi/plugins/{id}/enable - Enable SPI plugin
pub async fn enable_plugin(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/spi/plugins/{id}/disable - Disable SPI plugin
pub async fn disable_plugin(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI management not yet implemented. Requires spi_service in IamApiState.",
    )))
}
