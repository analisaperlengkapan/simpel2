use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// Auto-expire permits that have passed their end date
    /// Requirements: REQ-P010
    pub async fn auto_expire_permits(&self) -> AppResult<usize> {
        let client = self.pool.client().await?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET status = 'EXPIRED',
                updated_at = NOW()
            WHERE status = 'ACTIVE'
            AND tanggal_selesai < CURRENT_DATE
        "#;

        let result = client
            .execute(query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get permits expiring soon (for notifications)
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(
        &self,
        days_threshold: i32,
    ) -> AppResult<Vec<IzinPemakaianBmn>> {
        let client = self.pool.client().await?;

        let query = r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
            AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + $1
            ORDER BY tanggal_selesai ASC
        "#;

        let rows = client
            .query(query, &[&days_threshold])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_permit(row))
            .collect())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================
}
