use serde::{Deserialize, Serialize};

/// Document service configuration extending BaseServiceConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Base service config fields
    pub database_url: String,
    #[serde(default = "default_pool_size")]
    pub database_pool_size: usize,
    #[serde(default = "default_port")]
    pub server_port: u16,
    #[serde(default = "default_host")]
    pub server_host: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Encryption key for document storage (32 bytes for AES-256)
    #[serde(default = "default_encryption_key")]
    pub encryption_key: String,
    /// Base path for file storage
    #[serde(default = "default_storage_path")]
    pub storage_path: String,
    /// Archive storage path (separate from main storage)
    #[serde(default = "default_archive_storage_path")]
    pub archive_storage_path: String,
    /// Allowed file extensions
    #[serde(default = "default_allowed_extensions")]
    pub allowed_extensions: Vec<String>,
    /// Maximum file size in bytes
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    /// AI service URL for OCR/classification
    #[serde(default = "default_ai_service_url")]
    pub ai_service_url: String,
    /// AI service API key
    pub ai_service_api_key: Option<String>,
}

fn default_pool_size() -> usize {
    10
}

fn default_port() -> u16 {
    3006
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_encryption_key() -> String {
    // 32-byte key for AES-256 (in production, load from Secreton)
    "01234567890123456789012345678901".to_string()
}

fn default_storage_path() -> String {
    "/var/data/dokumen".to_string()
}

fn default_archive_storage_path() -> String {
    "/var/data/dokumen/archive".to_string()
}

fn default_allowed_extensions() -> Vec<String> {
    vec![
        "pdf".to_string(),
        "doc".to_string(),
        "docx".to_string(),
        "xls".to_string(),
        "xlsx".to_string(),
        "png".to_string(),
        "jpg".to_string(),
        "jpeg".to_string(),
    ]
}

fn default_max_file_size() -> u64 {
    50 * 1024 * 1024 // 50MB
}

fn default_ai_service_url() -> String {
    "http://localhost:8090".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            database_pool_size: default_pool_size(),
            server_port: default_port(),
            server_host: default_host(),
            log_level: default_log_level(),
            encryption_key: default_encryption_key(),
            storage_path: default_storage_path(),
            archive_storage_path: default_archive_storage_path(),
            allowed_extensions: default_allowed_extensions(),
            max_file_size: default_max_file_size(),
            ai_service_url: default_ai_service_url(),
            ai_service_api_key: None,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/perlengkapan".to_string()
            }),
            database_pool_size: std::env::var("DATABASE_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_pool_size),
            server_port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_port),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| default_host()),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| default_log_level()),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| default_encryption_key()),
            storage_path: std::env::var("STORAGE_PATH")
                .unwrap_or_else(|_| default_storage_path()),
            archive_storage_path: std::env::var("ARCHIVE_STORAGE_PATH")
                .unwrap_or_else(|_| default_archive_storage_path()),
            allowed_extensions: std::env::var("ALLOWED_EXTENSIONS")
                .map(|s| s.split(',').map(|e| e.trim().to_string()).collect())
                .unwrap_or_else(|_| default_allowed_extensions()),
            max_file_size: std::env::var("MAX_FILE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_file_size),
            ai_service_url: std::env::var("AI_SERVICE_URL")
                .unwrap_or_else(|_| default_ai_service_url()),
            ai_service_api_key: std::env::var("AI_SERVICE_API_KEY").ok(),
        }
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([0, 0, 0, 0], self.server_port))
    }
}
