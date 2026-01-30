// ! Authentication API client
//!
//! This module provides authentication-related API calls

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{API_BASE_URL, ApiError, ApiResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Login user
pub async fn login(username: String, password: String) -> Result<LoginResponse, ApiError> {
    let request_body = LoginRequest { username, password };

    let response = Request::post(&format!("{}/auth/login", API_BASE_URL))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| ApiError::SerializationError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    if !response.ok() {
        return Err(ApiError::ServerError(
            response.status(),
            response.status_text(),
        ));
    }

    let api_response: ApiResponse<LoginResponse> = response
        .json()
        .await
        .map_err(|e| ApiError::SerializationError(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::ServerError(500, "No data in response".to_string()))
}

/// Logout user
pub async fn logout() -> Result<(), ApiError> {
    let token = super::get_auth_token().ok_or(ApiError::Unauthorized)?;

    let response = Request::post(&format!("{}/auth/logout", API_BASE_URL))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    if !response.ok() {
        return Err(ApiError::ServerError(
            response.status(),
            response.status_text(),
        ));
    }

    Ok(())
}

/// Refresh authentication token
pub async fn refresh_token(refresh_token: String) -> Result<LoginResponse, ApiError> {
    let request_body = RefreshTokenRequest { refresh_token };

    let response = Request::post(&format!("{}/auth/refresh", API_BASE_URL))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| ApiError::SerializationError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    if !response.ok() {
        return Err(ApiError::ServerError(
            response.status(),
            response.status_text(),
        ));
    }

    let api_response: ApiResponse<LoginResponse> = response
        .json()
        .await
        .map_err(|e| ApiError::SerializationError(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::ServerError(500, "No data in response".to_string()))
}
