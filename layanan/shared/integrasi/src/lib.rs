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
pub mod siman;

// Batch processing (unified)
pub mod batch;

// Re-exports for convenience
pub use audit::{
    ApiCallLog, BatchProcessingLog, DataSyncLog, TokenResetLog, get_api_stats_by_module,
    get_recent_failed_calls, get_token_health,
};
pub use client::MonsaktiClient;
pub use config::Config;
pub use error::MonsaktiError;
pub use response::MonsaktiResponse;
pub use storage::{StorageStrategy, storage_from_env};

// Batch processing exports
pub use batch::{
    KL_KEJAKSAAN, fetch_all_data, fetch_all_data_with_mysimkari, fetch_all_satker,
    fetch_all_satker_with_modules, fetch_all_satker_with_modules_from_db, fetch_global_references,
    fetch_mysimkari, fetch_satker_complete, fetch_satker_parallel, fetch_satker_with_modules,
    get_satker_list, get_satker_list_from_db,
};

// Database utilities
pub use db::{bulk_insert_postgres, save_to_database};

// API module exports
pub use monsakti::{adm, ang, ast, ben, glp, kom, pem, per};
pub use mysimkari::mysimkari::{get_satker, pegawai_satker};

// SIMAN module exports
pub use siman::{
    SimanAssetCategory, SimanDataRequest, SimanResponse, SimanTokenResponse,
    fetch_all_aset_paginated, get_aset_alat_besar, get_aset_alat_persenjataan,
    get_aset_angkutan_bermotor, get_aset_bangunan_air, get_aset_by_category,
    get_aset_gedung_bangunan, get_aset_instalasi_jaringan, get_aset_jalan_jembatan, get_aset_kdp,
    get_aset_khusus_tik, get_aset_non_tik, get_aset_rumah, get_aset_tak_berwujud, get_aset_tanah,
    get_aset_tetap_lainnya, get_aset_tetap_renovasi, get_row_count,
};
