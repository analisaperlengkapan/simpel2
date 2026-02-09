//! Application configuration

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub authenc: AuthencConfig,
    pub secreton: SecretonConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthencConfig {
    pub grpc_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecretonConfig {
    pub grpc_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3010".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://localhost/simpelv2".to_string()),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            authenc: AuthencConfig {
                grpc_url: std::env::var("AUTHENC_GRPC_URL")
                    .unwrap_or_else(|_| "http://localhost:50051".to_string()),
            },
            secreton: SecretonConfig {
                grpc_url: std::env::var("SECRETON_GRPC_URL")
                    .unwrap_or_else(|_| "http://localhost:50052".to_string()),
            },
        })
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3010,
        }
    }
}
