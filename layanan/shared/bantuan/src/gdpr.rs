use crate::models::GdprRequest;
use crate::error::AppError;
use deadpool_postgres::Pool;
use uuid::Uuid;
use serde_json::Value;

pub struct GdprService {
    pub pool: Pool,
}

impl GdprService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn request_delete(&self, user_id: Uuid, details: Value) -> Result<GdprRequest, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"INSERT INTO bantuan.gdpr_requests (id, user_id, request_type, status, details, created_at)
            VALUES ($1, $2, 'delete', 'pending', $3, NOW()) RETURNING *"#,
            &[&Uuid::new_v4(), &user_id, &details]
        ).await?;
        Ok(GdprRequest::from(&row))
    }
    pub async fn request_download(&self, user_id: Uuid, details: Value) -> Result<GdprRequest, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"INSERT INTO bantuan.gdpr_requests (id, user_id, request_type, status, details, created_at)
            VALUES ($1, $2, 'download', 'pending', $3, NOW()) RETURNING *"#,
            &[&Uuid::new_v4(), &user_id, &details]
        ).await?;
        Ok(GdprRequest::from(&row))
    }
    pub async fn get_status(&self, user_id: Uuid) -> Result<Vec<GdprRequest>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.gdpr_requests WHERE user_id = $1 ORDER BY created_at DESC"#,
            &[&user_id]
        ).await?;
        let reqs = rows.into_iter().map(|row| GdprRequest::from(&row)).collect();
        Ok(reqs)
    }
    pub async fn process_request(&self, id: Uuid, status: &str, details: Value) -> Result<GdprRequest, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.gdpr_requests SET status = $1, details = $2 WHERE id = $3 RETURNING *"#,
            &[&status, &details, &id]
        ).await?;
        Ok(GdprRequest::from(&row))
    }
}
