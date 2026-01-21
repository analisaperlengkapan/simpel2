use deadpool_postgres::{Pool, Runtime, Config};
use tokio_postgres::NoTls;
use anyhow::Result;

/// Standard database configuration shared across services
pub struct DbConfig {
    pub url: String,
    pub max_size: usize,
}

/// Create a standardized PostgreSQL connection pool
pub fn create_postgres_pool(config: DbConfig) -> Result<Pool> {
    let mut pg_config = Config::new();
    pg_config.url = Some(config.url);

    let pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    Ok(pool)
}

/// Standarized health check for database
pub async fn check_db_health(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;
    Ok(())
}
