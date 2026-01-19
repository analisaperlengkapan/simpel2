use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub storage_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub algorithm: String,
    pub expiration: u64,
    pub refresh_expiration: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "default-secret".to_string(),
            issuer: "secreton".to_string(),
            audience: "secreton".to_string(),
            algorithm: "HS256".to_string(),
            expiration: 3600,
            refresh_expiration: 86400,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub token_ttl: u64,
    pub refresh_token_ttl: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt: JwtConfig::default(),
            token_ttl: 3600,
            refresh_token_ttl: 86400,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LeaseConfig {
    pub check_interval_secs: u64,
    pub notification_threshold_secs: i64,
    pub enable_notifications: bool,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HsmConfig {
    #[serde(default)]
    pub enabled: bool,
    pub library_path: Option<String>,
    pub slot_id: Option<u64>,
    pub pin: Option<String>,
    pub key_label: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub lease: LeaseConfig,
    #[serde(default)]
    pub hsm: HsmConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                log_level: "info".to_string(),
                storage_path: "./data".to_string(),
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "secreton".to_string(),
                username: "postgres".to_string(),
                password: "password".to_string(),
                max_connections: 5,
                connection_timeout: 30,
            },
            auth: AuthConfig {
                jwt: JwtConfig {
                    secret: "default-secret".to_string(),
                    issuer: "secreton".to_string(),
                    audience: "secreton".to_string(),
                    algorithm: "HS256".to_string(),
                    expiration: 3600,
                    refresh_expiration: 86400,
                },
                token_ttl: 3600,
                refresh_token_ttl: 86400,
            },
            lease: LeaseConfig::default(),
            hsm: HsmConfig::default(),
        }
    }
}
