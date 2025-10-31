/// Tipe error untuk operasi MonSAKTI
#[derive(Debug, thiserror::Error)]
pub enum MonsaktiError {
    /// Request HTTP gagal
    #[error("Request HTTP gagal: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    /// Token telah kadaluarsa
    #[error("Token kadaluarsa")]
    TokenExpired,
    
    /// Error dari API
    #[error("Error API: {0}")]
    ApiError(String),
    
    /// Error konfigurasi
    #[error("Error konfigurasi: {0}")]
    ConfigError(String),
    
    /// Error I/O
    #[error("Error I/O: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Error JSON
    #[error("Error JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Error database
    #[error("Error database: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    
    /// Error CSV
    #[error("Error CSV: {0}")]
    CsvError(#[from] csv::Error),
}
