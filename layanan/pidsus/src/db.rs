use crate::config::Config;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

pub type DbPool = Pool;

pub fn create_pool(config: &Config) -> DbPool {
    let pg_config = config
        .database_url
        .parse::<tokio_postgres::Config>()
        .expect("Invalid DB URL");

    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

    Pool::builder(mgr)
        .max_size(config.database_pool_size)
        .build()
        .expect("Failed to create DB pool")
}
