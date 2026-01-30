use serde::{Deserialize, Serialize};

/// Common server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind the server to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on (HTTP/REST)
    #[serde(default = "default_port")]
    pub port: u16,

    /// gRPC server port
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,

    /// Enable gRPC server
    #[serde(default = "default_grpc_enabled")]
    pub grpc_enabled: bool,

    /// Number of worker threads to use (defaults to number of CPU cores)
    pub workers: Option<usize>,

    /// Keep-alive timeout in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,

    /// Client timeout in seconds
    #[serde(default = "default_client_timeout")]
    pub client_timeout: u64,

    /// Client disconnect timeout in seconds
    #[serde(default = "default_client_disconnect_timeout")]
    pub client_disconnect_timeout: u64,

    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Public endpoint prefix
    #[serde(default = "default_public_prefix")]
    pub public_prefix: String,

    /// Admin endpoint prefix
    #[serde(default = "default_admin_prefix")]
    pub admin_prefix: String,

    /// Internal endpoint prefix
    #[serde(default = "default_internal_prefix")]
    pub internal_prefix: String,

    /// Enable TLS
    #[serde(default)]
    pub tls_enabled: bool,

    /// Path to TLS certificate file
    pub tls_cert_path: Option<String>,

    /// Path to TLS private key file
    pub tls_key_path: Option<String>,

    /// List of allowed CORS origins
    #[serde(default = "default_cors_origins")]
    pub cors_allowed_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            grpc_port: default_grpc_port(),
            grpc_enabled: default_grpc_enabled(),
            workers: None,
            keep_alive: default_keep_alive(),
            client_timeout: default_client_timeout(),
            client_disconnect_timeout: default_client_disconnect_timeout(),
            max_connections: default_max_connections(),
            public_prefix: default_public_prefix(),
            admin_prefix: default_admin_prefix(),
            internal_prefix: default_internal_prefix(),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            cors_allowed_origins: default_cors_origins(),
        }
    }
}

// Default values functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    3000
}
fn default_grpc_port() -> u16 {
    9088
}
fn default_grpc_enabled() -> bool {
    true
}
fn default_keep_alive() -> u64 {
    75
}
fn default_client_timeout() -> u64 {
    120  // Increased from 30 to 120 for complex operations like CAPTCHA generation
}
fn default_client_disconnect_timeout() -> u64 {
    5
}
fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
}
fn default_max_connections() -> u32 {
    100
}
fn default_public_prefix() -> String {
    "/api/v1".to_string()
}
fn default_admin_prefix() -> String {
    "/admin".to_string()
}
fn default_internal_prefix() -> String {
    "/internal".to_string()
}

/// Common database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database server hostname or IP address
    pub host: String,
    /// Database server port
    pub port: u16,
    /// Database username
    pub username: String,
    /// Database password
    pub password: String,
    /// Database name
    pub database: String,
    /// Maximum number of database connections
    pub max_connections: u32,
    /// Minimum number of idle connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Idle connection timeout in seconds (how long before an idle connection is closed)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    /// Maximum connection lifetime in seconds (how long a connection can live)
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
    /// Optional audit log database URL
    pub audit_log_url: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "simpel".to_string(),
            max_connections: 10,
            min_connections: default_min_connections(),
            connection_timeout: 30,
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
            audit_log_url: None,
        }
    }
}

fn default_min_connections() -> u32 {
    10
}
fn default_idle_timeout() -> u64 {
    600
}
fn default_max_lifetime() -> u64 {
    1800
}

impl DatabaseConfig {
    /// Get the connection string (URL) for the database
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// Log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, compact, full)
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}

/// Base configuration for simple backend services
/// Loads common fields from environment variables with sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseServiceConfig {
    /// Database connection URL
    pub database_url: String,
    /// Database connection pool size
    #[serde(default = "default_pool_size")]
    pub database_pool_size: usize,
    /// Server port
    #[serde(default = "default_service_port")]
    pub server_port: u16,
    /// Server host
    #[serde(default = "default_service_host")]
    pub server_host: String,
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_pool_size() -> usize {
    10
}
fn default_service_port() -> u16 {
    3000
}
fn default_service_host() -> String {
    "0.0.0.0".to_string()
}

impl Default for BaseServiceConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            database_pool_size: default_pool_size(),
            server_port: default_service_port(),
            server_host: default_service_host(),
            log_level: default_log_level(),
        }
    }
}

impl BaseServiceConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        // Load .env file if present
        let _ = dotenvy::dotenv();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/simpelv2".to_string());

        let database_pool_size = std::env::var("DATABASE_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_pool_size());

        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_service_port());

        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| default_service_host());

        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| default_log_level());

        Self {
            database_url,
            database_pool_size,
            server_port,
            server_host,
            log_level,
        }
    }

    /// Get socket address for binding
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([0, 0, 0, 0], self.server_port))
    }
}
