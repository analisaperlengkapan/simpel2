use anyhow::Result;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::env;
use tokio_postgres::NoTls;

pub async fn create_pool() -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.host = Some(env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
    cfg.user = Some(env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()));
    cfg.password = Some(env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()));
    cfg.dbname = Some(env::var("DB_NAME").unwrap_or_else(|_| "simpel_pengawasan".to_string()));
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    Ok(pool)
}

pub async fn migrate(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;
    let migration_sql = include_str!("../migrations/init.sql");
    client.batch_execute(migration_sql).await?;
    Ok(())
}
