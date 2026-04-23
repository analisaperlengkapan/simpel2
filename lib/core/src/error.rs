use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Validation errors: {0}")]
    ValidationErrors(#[from] crate::validation::ValidationErrors),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CommonError>;
