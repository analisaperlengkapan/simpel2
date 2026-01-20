use serde::Deserialize;
use std::env;

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[allow(dead_code)]
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("SERVER_PORT must be a number");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        Self {
            server_host,
            server_port,
            database_url,
        }
    }
}
