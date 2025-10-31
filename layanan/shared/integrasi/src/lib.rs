// Core modules
pub mod audit;
pub mod client;
pub mod config;
pub mod db;
pub mod error;
pub mod response;
pub mod storage;

// API modules
pub mod monsakti;
pub mod mysimkari;

// Batch processing (unified)
pub mod batch;

// Re-exports for convenience
pub use audit::{
    get_api_stats_by_module, get_recent_failed_calls, get_token_health, ApiCallLog,
    BatchProcessingLog, DataSyncLog, TokenResetLog,
};
pub use client::MonsaktiClient;
pub use config::Config;
pub use error::MonsaktiError;
pub use response::MonsaktiResponse;
pub use storage::{storage_from_env, StorageStrategy};

// Batch processing exports
pub use batch::{
    fetch_all_data, fetch_all_satker, fetch_global_references, fetch_mysimkari,
    fetch_satker_complete, fetch_satker_parallel, get_satker_list, KL_KEJAKSAAN,
};

// Database utilities
pub use db::{bulk_insert_postgres, save_to_database};

// API module exports
pub use monsakti::{adm, ang, ast, ben, glp, kom, pem, per};
pub use mysimkari::mysimkari::{get_satker, pegawai_satker};
