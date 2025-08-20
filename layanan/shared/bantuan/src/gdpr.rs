use crate::models::GdprRequest;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use chrono::Utc;

pub struct GdprService {
    pub pool: PgPool,
}

impl GdprService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn request_delete(&self, user_id: Uuid, details: Value) -> Result<GdprRequest, AppError> {
        let req = sqlx::query_as!(GdprRequest,
            r#"INSERT INTO bantuan.gdpr_requests (id, user_id, request_type, status, details, created_at)
            VALUES ($1, $2, 'delete', 'pending', $3, NOW()) RETURNING *"#,
            Uuid::new_v4(), user_id, details
        ).fetch_one(&self.pool).await?;
        Ok(req)
    }
    pub async fn request_download(&self, user_id: Uuid, details: Value) -> Result<GdprRequest, AppError> {
        let req = sqlx::query_as!(GdprRequest,
            r#"INSERT INTO bantuan.gdpr_requests (id, user_id, request_type, status, details, created_at)
            VALUES ($1, $2, 'download', 'pending', $3, NOW()) RETURNING *"#,
            Uuid::new_v4(), user_id, details
        ).fetch_one(&self.pool).await?;
        Ok(req)
    }
    pub async fn get_status(&self, user_id: Uuid) -> Result<Vec<GdprRequest>, AppError> {
        let reqs = sqlx::query_as!(GdprRequest,
            r#"SELECT * FROM bantuan.gdpr_requests WHERE user_id = $1 ORDER BY created_at DESC"#,
            user_id
        ).fetch_all(&self.pool).await?;
        Ok(reqs)
    }
    pub async fn process_request(&self, id: Uuid, status: &str, details: Value) -> Result<GdprRequest, AppError> {
        let req = sqlx::query_as!(GdprRequest,
            r#"UPDATE bantuan.gdpr_requests SET status = $1, details = $2 WHERE id = $3 RETURNING *"#,
            status, details, id
        ).fetch_one(&self.pool).await?;
        Ok(req)
    }
} 