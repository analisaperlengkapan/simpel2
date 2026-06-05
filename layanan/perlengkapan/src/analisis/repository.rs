//! Persistence for Analisis Kebutuhan.
//!
//! The trait keeps the service layer testable (mocked in unit tests); the
//! concrete impl maps `perlengkapan.analisis_kebutuhan` rows to domain models
//! on the shared `Database` pool.

use async_trait::async_trait;
use uuid::Uuid;

use super::models::{AnalisisKebutuhan, CreateAnalisisRequest};
use crate::shared::db::Database;
use crate::shared::error::{AppError, AppResult, not_found};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AnalisisRepository: Send + Sync {
    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
    async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan>;
    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan>;
}

#[async_trait]
impl AnalisisRepository for Database {
    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let client =
            self.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by FROM perlengkapan.analisis_kebutuhan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let analisis: Vec<AnalisisKebutuhan> =
            rows.iter().map(AnalisisKebutuhan::from_row).collect();

        Ok((analisis, total))
    }

    async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan> {
        let client =
            self.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let row = client
            .query_opt("SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by FROM perlengkapan.analisis_kebutuhan WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| AnalisisKebutuhan::from_row(&r))
            .ok_or_else(|| not_found("Analisis Kebutuhan", &id.to_string()))
    }

    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        let client =
            self.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let id = Uuid::new_v4();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.analisis_kebutuhan
                (id, judul, kategori, deskripsi, prioritas, estimasi_biaya, justifikasi, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by
                "#,
                &[
                    &id,
                    &request.judul,
                    &request.kategori,
                    &request.deskripsi,
                    &request.prioritas,
                    &request.estimasi_biaya,
                    &request.justifikasi,
                    &user_id,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(AnalisisKebutuhan::from_row(&row))
    }
}
