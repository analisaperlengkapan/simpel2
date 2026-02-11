// ============================================================================
// Penghapusan BMN Repository
// Description: Database operations for BMN disposal
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
                status, created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW()
            )
            RETURNING *
        "#;

        let id = Uuid::new_v4();
        let status = "DRAFT";

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
                    &status,
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

        let mut where_clauses = vec!["1=1"];
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let mut param_count = 1;

        // Build dynamic WHERE clause
        let satker_id_param;
        if let Some(ref satker_id) = filters.satker_id {
            where_clauses.push(&format!("satker_id = ${}", param_count));
            satker_id_param = satker_id;
            params.push(satker_id_param);
            param_count += 1;
        }

        let status_param;
        if let Some(ref status) = filters.status {
            where_clauses.push(&format!("status = ${}", param_count));
            status_param = status;
            params.push(status_param);
            param_count += 1;
        }

        let metode_param;
        if let Some(ref metode) = filters.metode_penghapusan {
            where_clauses.push(&format!("metode_penghapusan = ${}", param_count));
            metode_param = metode;
            params.push(metode_param);
            param_count += 1;
        }

        let tahun_param;
        if let Some(ref tahun) = filters.tahun {
            where_clauses.push(&format!("EXTRACT(YEAR FROM tanggal_penghapusan) = ${}", param_count));
            tahun_param = tahun;
            params.push(tahun_param);
            param_count += 1;
        }

        let where_clause = where_clauses.join(" AND ");

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.penghapusan_bmn WHERE {}",
            where_clause
        );

        let count_row = client.query_one(&count_query, &params).await?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT * FROM perlengkapan.penghapusan_bmn WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_count, param_count + 1
        );

        params.push(&per_page);
        params.push(&offset);

        let rows = client.query(&data_query, &params).await?;
        let penghapusan: Vec<PenghapusanBmn> = rows.iter().map(PenghapusanBmn::from_row).collect();

        Ok((penghapusan, total))
    }

    /// Update penghapusan BMN
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
    ) -> AppResult<PenghapusanBmn> {
        let client = self.pool.get().await?;

        let mut set_clauses = vec!["updated_at = NOW()"];
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let mut param_count = 1;

        let tanggal_param;
        if let Some(ref tanggal) = request.tanggal_penghapusan {
            set_clauses.push(&format!("tanggal_penghapusan = ${}", param_count));
            tanggal_param = tanggal;
            params.push(tanggal_param);
            param_count += 1;
        }

        let alasan_param;
        if let Some(ref alasan) = request.alasan {
            set_clauses.push(&format!("alasan = ${}", param_count));
            alasan_param = alasan;
            params.push(alasan_param);
            param_count += 1;
        }

        let metode_param;
        if let Some(ref metode) = request.metode_penghapusan {
            set_clauses.push(&format!("metode_penghapusan = ${}", param_count));
            metode_param = metode;
            params.push(metode_param);
            param_count += 1;
        }

        let nilai_param;
        if let Some(ref nilai) = request.nilai_residu {
            set_clauses.push(&format!("nilai_residu = ${}", param_count));
            nilai_param = nilai;
            params.push(nilai_param);
            param_count += 1;
        }

        let set_clause = set_clauses.join(", ");
        let query = format!(
            "UPDATE perlengkapan.penghapusan_bmn SET {} WHERE id = ${} RETURNING *",
            set_clause, param_count
        );

        params.push(&id);

        let row = client
            .query_opt(&query, &params)
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

        let query = r#"
            UPDATE perlengkapan.penghapusan_bmn
            SET status = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        let rows_affected = client.execute(query, &[&status, &id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }

    /// Update document metadata (after document generation)
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

        let rows_affected = client.execute(query, &[&document_id, &document_url, &id]).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Penghapusan BMN not found: {}",
                id
            )));
        }

        Ok(())
    }
}
