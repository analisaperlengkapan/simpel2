use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Database error: {0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Timeout error")]
    Timeout,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Pool config error: {0}")]
    PoolConfig(String),
    #[error("Prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl From<deadpool_postgres::CreatePoolError> for IntegrationError {
    fn from(err: deadpool_postgres::CreatePoolError) -> Self {
        IntegrationError::PoolConfig(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
