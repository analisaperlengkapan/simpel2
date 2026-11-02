use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use uuid::Uuid;

impl PemakaianBmnRepository {
    /// Get BMN items for a permit
    pub async fn get_bmn_items(&self, izin_pemakaian_id: Uuid) -> AppResult<Vec<PemakaianBmnItem>> {
        let client = self.pool.client().await?;

        let rows = client
            .query(
                // ORDER BY urutan, not created_at: `urutan` is what the SK prints,
                // and two items written in the same transaction share a
                // created_at to the microsecond.
                "SELECT * FROM perlengkapan.pemakaian_bmn_items \
                 WHERE izin_pemakaian_id = $1 ORDER BY urutan",
                &[&izin_pemakaian_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(PemakaianBmnItem::from_row).collect())
    }

    /// Update both DOCX and PDF konsep surat URLs and the matching on-disk
    /// paths in a single statement. This is the only konsep-surat write
    /// path — DOCX + PDF are always produced together.
    pub async fn update_konsep_surat(
        &self,
        id: Uuid,
        docx_url: &str,
        docx_path: &str,
        pdf_url: &str,
        pdf_path: &str,
    ) -> AppResult<()> {
        let client = self.pool.client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET konsep_surat_url = $1,
                    konsep_surat_docx_path = $2,
                    konsep_surat_generated_at = NOW(),
                    konsep_surat_pdf_url = $3,
                    konsep_surat_pdf_path = $4,
                    konsep_surat_pdf_generated_at = NOW(),
                    updated_at = NOW()
                WHERE id = $5
                "#,
                &[&docx_url, &docx_path, &pdf_url, &pdf_path, &id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Fetch the on-disk path the route handler should stream from.
    pub async fn konsep_surat_path(&self, id: Uuid, format: &str) -> AppResult<Option<String>> {
        let column = match format {
            "docx" => "konsep_surat_docx_path",
            "pdf" => "konsep_surat_pdf_path",
            _ => return Ok(None),
        };
        let client = self.pool.client().await?;
        let query = format!(
            "SELECT {} FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
            column
        );
        let row = client
            .query_opt(&query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.and_then(|r| r.try_get::<_, Option<String>>(0).ok().flatten()))
    }

    /// Update signed PDF URL and mark as completed
    pub async fn update_signed_pdf(&self, id: Uuid, signed_pdf_url: &str) -> AppResult<()> {
        let client = self.pool.client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET signed_pdf_url = $1,
                    signed_pdf_uploaded_at = NOW(),
                    is_completed = true,
                    updated_at = NOW()
                WHERE id = $2
                "#,
                &[&signed_pdf_url, &id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
