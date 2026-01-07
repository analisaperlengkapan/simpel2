use gloo_net::http::Request;
use crate::features::auth::UserSession;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<serde_json::Value>,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error ({0}): {1}")]
    Server(u16, String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Unauthorized")]
    Unauthorized,
}

pub async fn fetch_api<T, R>(
    method: &str,
    path: &str,
    body: Option<&T>,
    session: Option<&UserSession>,
) -> Result<R, ApiError>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let url = format!("/api{}", path); // Proxy rewrites /api to backend
    let mut req = match method {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PUT" => Request::put(&url),
        "DELETE" => Request::delete(&url),
        _ => return Err(ApiError::Network(format!("Unsupported method: {}", method))),
    };

    if let Some(user_session) = session {
        req = req.header("Authorization", &format!("Bearer {}", user_session.token));
    }

    let resp = if let Some(data) = body {
        req.json(data).map_err(|e| ApiError::Serialization(e.to_string()))?
            .send().await.map_err(|e| ApiError::Network(e.to_string()))?
    } else {
        req.send().await.map_err(|e| ApiError::Network(e.to_string()))?
    };

    if resp.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(ApiError::Server(resp.status(), text));
    }

    resp.json::<R>().await.map_err(|e| ApiError::Serialization(e.to_string()))
}
