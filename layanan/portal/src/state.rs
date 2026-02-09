//! Application state

use crate::config::AppConfig;
use crate::services::{AuthencClient, SecretonClient};
use deadpool_postgres::{Config, Pool, Runtime};
use std::sync::Arc;

pub struct AppState {
    pub config: AppConfig,
    pub db: Pool,
    pub authenc: Arc<AuthencClient>,
    pub secreton: Arc<SecretonClient>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        // Initialize database pool
        let mut pg_config = Config::new();
        pg_config.url = Some(config.database.url.clone());
        let db = pg_config.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        // Initialize gRPC clients
        let authenc = Arc::new(AuthencClient::new(&config.authenc.grpc_url).await?);
        let secreton = Arc::new(SecretonClient::new(&config.secreton.grpc_url).await?);

        Ok(Self {
            config,
            db,
            authenc,
            secreton,
        })
    }
}
