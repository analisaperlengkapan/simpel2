// ============================================================================
// Penghapusan BMN Repository
// Description: Database operations for SK Penghapusan BMN workflow
// Requirements: REQ-W001
// ============================================================================

use super::models::*;
use crate::errors::{AppError, AppResult};
use deadpool_postgres::Pool;
use uuid::Uuid;

pub struct PenghapusanBmnRepository {
    pool: Pool,
}

impl PenghapusanBmnRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a new penghapusan BMN record
    pub async fn create(
        &self,
        request: CreatePenghapusanBmnRequest,
        created_by: Uuid,
    ) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        let query = r#"
            INSERT INTO perlengkapan.penghapusan_bmn (
                id, satker_id, asset_id, kode_barang, nama_barang, nup,
                tanggal_penghapusan, alasan, metode_penghapusan, nilai_residu,
                status, status_kode, lampiran_persyaratan, lampiran_pendukung,
                catatan_operator, is_completed,
                created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, false,
                $16, NOW(), NOW()
            )
            RETURNING *
        "#;

        let id = Uuid::new_v4();
        let status = PenghapusanBmnStatus::Draft;
        let lampiran_pendukung: Option<serde_json::Value> = None;

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &request.satker_id,
                    &request.asset_id,
                    &request.kode_barang,
                    &request.nama_barang,
                    &request.nup,
                    &request.tanggal_penghapusan,
                    &request.alasan,
                    &request.metode_penghapusan,
                    &request.nilai_residu,
                    &status.to_state_name(),
                    &status.to_code(),
                    &request.lampiran_persyaratan,
                    &lampiran_pendukung,
                    &request.catatan_operator,
                    &created_by,
                ],
            )
            .await?;

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// Get penghapusan BMN by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        let query = r#"
            SELECT * FROM perlengkapan.penghapusan_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&id])
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Penghapusan BMN not found: {}", id)))?;

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// List penghapusan BMN with filters and pagination
    pub async fn list(
        &self,
        filters: PenghapusanBmnFilters,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PenghapusanBmn>, i64)> {
        let client = self.pool.get().await?;

        let mut where_clauses = vec!["1=1".to_string()];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        let mut param_count = 1;

        if let Some(ref satker_id) = filters.satker_id {
            where_clauses.push(format!("satker_id = ${}", param_count));
            params.push(Box::new(*satker_id));
            param_count += 1;
        }

        if let Some(ref status) = filters.status {
            where_clauses.push(format!("status = ${}", param_count));
            params.push(Box::new(status.clone()));
            param_count += 1;
        }

        if let Some(ref metode) = filters.metode_penghapusan {
            where_clauses.push(format!("metode_penghapusan = ${}", param_count));
            params.push(Box::new(metode.clone()));
            param_count += 1;
        }

        if let Some(ref tahun) = filters.tahun {
            where_clauses.push(format!(
                "EXTRACT(YEAR FROM tanggal_penghapusan) = ${}",
                param_count
            ));
            params.push(Box::new(*tahun));
            param_count += 1;
        }

        let where_clause = where_clauses.join(" AND ");

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.penghapusan_bmn WHERE {}",
            where_clause
        );

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client.query_one(&count_query, &param_refs).await?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT * FROM perlengkapan.penghapusan_bmn WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_count,
            param_count + 1
        );

        params.push(Box::new(per_page));
        params.push(Box::new(offset));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&data_query, &param_refs).await?;
        let penghapusan: Vec<PenghapusanBmn> = rows.iter().map(PenghapusanBmn::from_row).collect();

        Ok((penghapusan, total))
    }

    /// Update penghapusan BMN (Draft/ReturnedToOperator only)
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
    ) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET tanggal_penghapusan = COALESCE($1, tanggal_penghapusan),
                alasan = COALESCE($2, alasan),
                metode_penghapusan = COALESCE($3, metode_penghapusan),
                nilai_residu = COALESCE($4, nilai_residu),
                lampiran_persyaratan = COALESCE($5, lampiran_persyaratan),
                catatan_operator = COALESCE($6, catatan_operator),
                updated_at = NOW()
            WHERE id = $7
            RETURNING *
        "#;

        let row = client
            .query_opt(
                query,
                &[
                    &request.tanggal_penghapusan,
                    &request.alasan,
                    &request.metode_penghapusan,
                    &request.nilai_residu,
                    &request.lampiran_persyaratan,
                    &request.catatan_operator,
                    &id,
                ],
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Penghapusan BMN not found: {}", id)))?;

        Ok(PenghapusanBmn::from_row(&row))
    }

    /// Delete penghapusan BMN (soft delete by setting status to CANCELLED)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE id = $1
        "#;

        let rows_affected = client.execute(query, &[&id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// Update status (used by workflow engine)
    pub async fn update_status(&self, id: Uuid, status: &str) -> AppResult<()> {
        let client = self.pool.get().await?;

        let status_kode = PenghapusanBmnStatus::from_state_name(status)
            .map(|s| s.to_code())
            .unwrap_or(0);

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET status = $1, status_kode = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        let rows_affected = client.execute(query, &[&status, &status_kode, &id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// Update validator wilayah info
    pub async fn update_validator_wilayah(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET validator_wilayah_id = $1,
                catatan_validator_wilayah = COALESCE($2, catatan_validator_wilayah),
                tanggal_submit_wilayah = NOW(),
                updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(query, &[&validator_id, &catatan, &id])
            .await?;
        Ok(())
    }

    /// Update validator pusat info
    pub async fn update_validator_pusat(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET validator_pusat_id = $1,
                catatan_validator_pusat = COALESCE($2, catatan_validator_pusat),
                tanggal_submit_pusat = NOW(),
                updated_at = NOW()
            WHERE id = $3
        "#;

        client
            .execute(query, &[&validator_id, &catatan, &id])
            .await?;
        Ok(())
    }

    /// Update konsep SK URL (after DOCX generation)
    pub async fn update_konsep_sk(&self, id: Uuid, konsep_sk_url: &str) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET konsep_sk_url = $1,
                konsep_sk_generated_at = NOW(),
                updated_at = NOW()
            WHERE id = $2
        "#;

        client.execute(query, &[&konsep_sk_url, &id]).await?;
        Ok(())
    }

    /// Update signed SK PDF URL
    pub async fn update_signed_sk(&self, id: Uuid, signed_sk_pdf_url: &str) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET signed_sk_pdf_url = $1,
                signed_sk_uploaded_at = NOW(),
                is_completed = true,
                updated_at = NOW()
            WHERE id = $2
        "#;

        client.execute(query, &[&signed_sk_pdf_url, &id]).await?;
        Ok(())
    }

    /// Update document metadata (legacy, kept for backward compat)
    pub async fn update_document_metadata(
        &self,
        id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET document_id = $1, document_url = $2, updated_at = NOW()
            WHERE id = $3
        "#;

        let rows_affected = client
            .execute(query, &[&document_id, &document_url, &id])
            .await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }
}
