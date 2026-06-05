use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use uuid::Uuid;

impl PemakaianBmnRepository {
    /// Generate permit number
    /// Requirements: REQ-P005
    ///
    /// Format: `IP/YYYY/MM/NNNN` — sequential within each YYYY/MM bucket.
    ///
    /// Tanpa serialisasi, dua aktivasi paralel di bulan yg sama dpt membaca
    /// `MAX(num)` yg sama (snapshot CTE) dan menghasilkan dua `nomor_izin`
    /// identik → UNIQUE conflict atau nomor lompat. Mitigasi:
    /// `pg_advisory_xact_lock` dgn kunci per-bulan agar generator berurutan.
    pub async fn generate_permit_number(&self, id: Uuid) -> AppResult<String> {
        let mut client = self.pool.client().await?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Kunci per-bulan: hashtext('izin_pemakaian_bmn:nomor:YYYY-MM').
        // xact_lock dilepas otomatis di akhir transaksi (commit/rollback).
        tx.execute(
            "SELECT pg_advisory_xact_lock(hashtext('izin_pemakaian_bmn:nomor:' || TO_CHAR(NOW(), 'YYYY-MM')))",
            &[],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            WITH next_number AS (
                SELECT COALESCE(MAX(
                    CAST(SUBSTRING(nomor_izin FROM 'IP/\d{4}/\d{2}/(\d{4})') AS INTEGER)
                ), 0) + 1 AS num
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE nomor_izin LIKE 'IP/' || TO_CHAR(NOW(), 'YYYY/MM') || '/%'
            )
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET nomor_izin = 'IP/' || TO_CHAR(NOW(), 'YYYY/MM/') || LPAD((SELECT num FROM next_number)::TEXT, 4, '0'),
                updated_at = NOW()
            WHERE id = $1
            RETURNING nomor_izin
        "#;

        let row = tx
            .query_one(query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let nomor: String = row.get("nomor_izin");

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(nomor)
    }

    /// Update document fields after document generation
    /// Requirements: REQ-P006, REQ-D002
    pub async fn update_document_fields(
        &self,
        id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET document_id = $1,
                document_url = $2,
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
        "#;

        let row = client
            .query_one(query, &[&document_id, &document_url, &id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    // ========================================================================
    // Fase 1.11: Cek pegawai + Cek BMN dgn period
    // ========================================================================
}
