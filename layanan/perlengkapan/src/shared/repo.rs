// ============================================================================
// shared/repo.rs — Repository helpers (Fase 1.3)
// ============================================================================
//
// Pattern berulang yg dihilangkan:
//
//   let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
//
// muncul 70x di pemakaian_bmn + 11x bank_aset + 17x penghapusan_bmn +
// 31x kebutuhan_bmn — 5-baris boilerplate per call. Trait extension
// `PoolExt::client()` mengubahnya jadi:
//
//   let client = self.pool.client().await?;
//
// Tidak ubah signature publik repository → murni internal refactor.
// ============================================================================

use crate::shared::error::AppError;
use async_trait::async_trait;
use deadpool_postgres::{Object as PgObject, Pool};

/// Extension trait untuk `deadpool_postgres::Pool` agar pengambilan client
/// jadi satu baris dgn error mapping seragam ke `AppError::Database`.
#[async_trait]
pub trait PoolExt {
    /// Ambil satu client dari pool. Map pool error → `AppError::Database`.
    async fn client(&self) -> Result<PgObject, AppError>;
}

#[async_trait]
impl PoolExt for Pool {
    async fn client(&self) -> Result<PgObject, AppError> {
        self.get()
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    // PoolExt::client() butuh DB nyata → integration test, di-skip di unit
    // test. Coverage di-validasi via call site yg compile bersih.
}
