//! SPI Federation HTTP handlers

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

/// SPI federation configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SpiFederationConfig {
    pub id: Uuid,
    pub name: String,
    pub provider_id: Uuid,
    pub spi_plugin_id: Uuid,
    pub config: serde_json::Value,
}

/// GET /api/v1/iam/spi/federation - List SPI federation configurations
pub async fn list_spi_federation_configs(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<SpiFederationConfig>>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI federation not yet implemented. Requires spi_federation_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/spi/federation - Create SPI federation configuration
pub async fn create_spi_federation_config(
    State(_state): State<Arc<IamApiState>>,
    Json(_config): Json<SpiFederationConfig>,
) -> ApiResult<(StatusCode, Json<SpiFederationConfig>)> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI federation not yet implemented. Requires spi_federation_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/spi/federation/{id} - Get SPI federation configuration
pub async fn get_spi_federation_config(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<SpiFederationConfig>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI federation not yet implemented. Requires spi_federation_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/spi/federation/{id} - Update SPI federation configuration
pub async fn update_spi_federation_config(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_config): Json<SpiFederationConfig>,
) -> ApiResult<Json<SpiFederationConfig>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI federation not yet implemented. Requires spi_federation_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/spi/federation/{id} - Delete SPI federation configuration
pub async fn delete_spi_federation_config(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "SPI federation not yet implemented. Requires spi_federation_service in IamApiState.",
    )))
}
