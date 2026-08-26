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

    /// Get permits expiring soon, restricted to `scope`.
    ///
    /// Two callers with opposite needs, hence the explicit scope argument
    /// rather than an implicit one: the expiry scheduler passes
    /// [`SatkerScope::All`] because it must notify every satker, while the
    /// monitoring endpoint passes the caller's own scope. Making the scheduler
    /// state its intent is what keeps the endpoint from quietly inheriting the
    /// national view — which is exactly how it was leaking.
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(
        &self,
        days_threshold: i32,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<Vec<IzinPemakaianBmn>> {
        let client = self.pool.client().await?;

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(days_threshold)];
        let scope_sql = match scope.push_condition("satker_code", &mut params) {
            Some(cond) => format!(" AND {cond}"),
            None => String::new(),
        };
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!(
            r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
            AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + $1::int{scope_sql}
            ORDER BY tanggal_selesai ASC, id ASC
        "#
        );

        let rows = client
            .query(&query, &refs)
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
