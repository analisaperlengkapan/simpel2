//! Badiklat-specific API client functions
//!
//! API calls specific to Education and Training (Badiklat) module

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResponse, API_BASE_URL};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Training {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingList {
    pub trainings: Vec<Training>,
    pub total: usize,
}

/// Get list of trainings
pub async fn get_trainings(page: usize, limit: usize) -> Result<TrainingList, ApiError> {
    let url = format!(
        "{}/badiklat/trainings?page={}&limit={}",
        API_BASE_URL, page, limit
    );

    let response = super::authenticated_request("GET", &url.replace(API_BASE_URL, ""), None)
        .await?
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    if !response.ok() {
        return Err(ApiError::ServerError(
            response.status(),
            response.status_text(),
        ));
    }

    let api_response: ApiResponse<TrainingList> = response
        .json()
        .await
        .map_err(|e| ApiError::SerializationError(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::ServerError(500, "No data in response".to_string()))
}

/// Get training by ID
pub async fn get_training(id: &str) -> Result<Training, ApiError> {
    let url = format!("/badiklat/trainings/{}", id);

    let response = super::authenticated_request("GET", &url, None)
        .await?
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    if !response.ok() {
        return Err(ApiError::ServerError(
            response.status(),
            response.status_text(),
        ));
    }

    let api_response: ApiResponse<Training> = response
        .json()
        .await
        .map_err(|e| ApiError::SerializationError(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::ServerError(500, "No data in response".to_string()))
}
