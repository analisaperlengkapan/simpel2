//! Error types for gRPC service

use tonic::Status;

/// gRPC service errors
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<GrpcError> for Status {
    fn from(err: GrpcError) -> Self {
        match err {
            GrpcError::AuthenticationFailed(msg) => {
                Status::unauthenticated(msg)
            }
            GrpcError::InvalidToken(msg) => {
                Status::unauthenticated(msg)
            }
            GrpcError::UserNotFound(msg) => {
                Status::not_found(msg)
            }
            GrpcError::PermissionDenied(msg) => {
                Status::permission_denied(msg)
            }
            GrpcError::InvalidRequest(msg) => {
                Status::invalid_argument(msg)
            }
            GrpcError::Internal(msg) => {
                Status::internal(msg)
            }
        }
    }
}

/// Result type for gRPC operations
pub type GrpcResult<T> = Result<T, GrpcError>;
