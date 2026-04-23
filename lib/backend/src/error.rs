//! ApiError with Axum IntoResponse support

use lib_core::error::CommonError;

/// Standard API Error response structure
#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<lib_core::validation::ValidationError>>,
}

impl ApiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            errors: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_validation_errors(
        mut self,
        errors: Vec<lib_core::validation::ValidationError>,
    ) -> Self {
        self.errors = Some(errors);
        self
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = if self.errors.is_some() {
            axum::http::StatusCode::BAD_REQUEST
        } else {
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        };

        let body = axum::Json(serde_json::json!({
            "error": self.message,
            "code": self.code,
            "errors": self.errors
        }));

        (status, body).into_response()
    }
}

impl From<CommonError> for ApiError {
    fn from(error: CommonError) -> Self {
        match error {
            CommonError::ValidationErrors(errors) => ApiError::new("Validation failed")
                .with_code("validation_error")
                .with_validation_errors(errors.errors),
            CommonError::Validation { message } => {
                ApiError::new(message).with_code("validation_error")
            }
            _ => ApiError::new(error.to_string()),
        }
    }
}
