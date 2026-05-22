pub mod admin;
pub mod bank_aset;
pub mod bantuan;
pub mod cache_strategy;
pub mod connection_config;
pub mod dashboard;
pub mod database;
pub mod database_optimization;
pub mod dokumen;
pub mod errors;
pub mod grpc_clients;
pub mod handlers;
pub mod health;
pub mod kebutuhan_bmn;
pub mod logging;
pub mod mapping_kodefikasi;
pub mod metrics;
pub mod middleware;
pub mod migrations;
pub mod models;
pub mod notifikasi;
pub mod pakaian_dinas;
pub mod pemakaian_bmn;
pub mod penghapusan_bmn;
pub mod rate_limiting;
pub mod repository;
pub mod roadmap_sarpras;
pub mod routes;
pub mod services;
pub mod shared;
pub mod state;
pub mod workflow;

pub use state::AppState;

#[cfg(test)]
pub mod tests;
