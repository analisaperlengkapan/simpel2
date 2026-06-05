//! # Pakaian Dinas Repository
//!
//! Database operations for the Pakaian Dinas module.
//! Uses tokio-postgres for async database access.

use deadpool_postgres::Pool;

/// Repository for Pakaian Dinas database operations
#[derive(Clone)]
pub struct PakaianDinasRepository {
    pool: Pool,
}

impl PakaianDinasRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // ============ Master: Jenis Pakaian Dinas ============
}

mod laporan;
mod master;
mod pegawai;
mod pengajuan;
mod satker;
mod workflow;

#[cfg(test)]
mod tests;
