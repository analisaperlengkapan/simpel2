use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub database_pool_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let database_pool_size = env::var("DATABASE_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        Ok(Config {
            server_port,
            database_url,
            database_pool_size,
        })
    }
}

// Minimal error placeholder since we aren't using the config crate fully yet
#[derive(Debug, thiserror::Error)]
#[error("Config error")]
pub struct ConfigError;
