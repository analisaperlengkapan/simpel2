use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// Revoke the permit a newly-activated renewal replaces.
    ///
    /// A signed SK is never edited. A renewal is signed on a different date,
    /// so it is issued as its own SK and the one it replaces is revoked as
    /// superseded — the newest is what is in force, the older is withdrawn
    /// because it has been replaced.
    ///
    /// The `status IN ('ACTIVE','EXPIRED')` predicate is what makes this safe
    /// to call more than once: a permit already revoked (by hand, or by an
    /// earlier run of this) is left exactly as it was, with its own reason
    /// intact. Returns whether a row actually moved, so the caller can log the
    /// difference between "superseded" and "there was nothing to supersede".
    pub async fn mark_superseded(
        &self,
        old_id: uuid::Uuid,
        new_id: uuid::Uuid,
        new_label: &str,
        revoked_by: uuid::Uuid,
    ) -> AppResult<bool> {
        let client = self.pool.client().await?;

        let n = client
            .execute(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status             = 'REVOKED',
                    superseded_by_id   = $2,
                    superseded_at      = NOW(),
                    revoked_by         = $4,
                    revoked_by_nama    = 'system',
                    revoked_at         = NOW(),
                    -- `catatan_revocation`, not `revocation_reason`. The table
                    -- has both; only this one is written by the normal revoke
                    -- path and read by the entity and the frontend.
                    -- `revocation_reason` has zero readers and zero writers in
                    -- the repository, so a reason written there would be
                    -- invisible everywhere (the first draft did exactly that,
                    -- and only the integration test caught it).
                    catatan_revocation = 'Digantikan oleh izin ' || $3,
                    updated_at         = NOW()
                WHERE id = $1
                  AND status IN ('ACTIVE', 'EXPIRED')
                "#,
                &[&old_id, &new_id, &new_label, &revoked_by],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(n > 0)
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

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_permit(row))
            .collect())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================
}
