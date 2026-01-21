use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CommonError>;

/// Standard API Error response structure
#[derive(Debug, serde::Serialize)]
#[cfg(feature = "axum")]
pub struct ApiError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(serde_json::json!({
            "error": self.message,
            "code": self.code
        }));
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
