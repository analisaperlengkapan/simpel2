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
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs, scope_and};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AnalisisRepository: Send + Sync {
    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
        scope: &SatkerScope,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
    /// One analysis, within `scope`; out of scope is a 404.
    async fn get_analisis_by_id(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<AnalisisKebutuhan>;
    /// `satker_code` is the creator's, from their claims — never the request's.
    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
        satker_code: Option<String>,
    ) -> AppResult<AnalisisKebutuhan>;
}

#[async_trait]
impl AnalisisRepository for Database {
    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
        scope: &SatkerScope,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let client =
            self.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let offset = (page - 1) * per_page;

        // The same scope predicate on the count and on the page, so `total`
        // describes what the caller can actually page through.
        let mut count_params: Vec<BoxedParam> = Vec::new();
        let count_scope = scope_and(scope, "satker_code", &mut count_params);
        let total: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan WHERE TRUE{count_scope}"
                ),
                &as_refs(&count_params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let mut params: Vec<BoxedParam> = Vec::new();
        let page_scope = scope_and(scope, "satker_code", &mut params);
        params.push(Box::new(per_page as i64));
        let limit_idx = params.len();
        params.push(Box::new(offset as i64));
        let offset_idx = params.len();
        let rows = client
            .query(
                &format!(
                    "SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by \
                     FROM perlengkapan.analisis_kebutuhan WHERE TRUE{page_scope} \
                     ORDER BY created_at DESC LIMIT ${limit_idx} OFFSET ${offset_idx}"
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let analisis: Vec<AnalisisKebutuhan> =
            rows.iter().map(AnalisisKebutuhan::from_row).collect();

        Ok((analisis, total))
    }

    async fn get_analisis_by_id(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<AnalisisKebutuhan> {
        let client =
            self.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let mut params: Vec<BoxedParam> = vec![Box::new(id)];
        let scope_sql = scope_and(scope, "satker_code", &mut params);
        let row = client
            .query_opt(
                &format!(
                    "SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by \
                     FROM perlengkapan.analisis_kebutuhan WHERE id = $1{scope_sql}"
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| AnalisisKebutuhan::from_row(&r))
            .ok_or_else(|| not_found("Analisis Kebutuhan", &id.to_string()))
    }

    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
        satker_code: Option<String>,
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
                (id, judul, kategori, deskripsi, prioritas, estimasi_biaya, justifikasi, created_by, updated_by, satker_code)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
                    &satker_code,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(AnalisisKebutuhan::from_row(&row))
    }
}
