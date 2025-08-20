use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
