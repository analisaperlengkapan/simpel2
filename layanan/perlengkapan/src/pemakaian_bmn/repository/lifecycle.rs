use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// The permit that replaced `id`, if one did.
    ///
    /// Read-only. The replacing itself is done by `trg_izin_pemakaian_supersede`
    /// (V014) in the same statement that activates the renewal, because the
    /// workflow engine commits on its own connection and no caller can make
    /// "activate" and "revoke the old one" a single transaction. This exists so
    /// the service can refuse a second renewal with a clear message before it
    /// reaches the trigger, which raises for the same condition when two
    /// activations race.
    pub async fn superseded_by(&self, id: uuid::Uuid) -> AppResult<Option<uuid::Uuid>> {
        let client = self.pool.client().await?;
        let row = client
            .query_opt(
                "SELECT superseded_by_id FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.and_then(|r| r.get::<_, Option<uuid::Uuid>>("superseded_by_id")))
    }

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

        // With their assets: the expiring-permits table names what is about
        // to lapse, and since V014 that lives only in the items.
        let mut permits: Vec<IzinPemakaianBmn> = rows
            .into_iter()
            .map(|row| self.row_to_permit(row))
            .collect();
        Self::attach_items(&**client, &mut permits).await?;
        Ok(permits)
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================
}
