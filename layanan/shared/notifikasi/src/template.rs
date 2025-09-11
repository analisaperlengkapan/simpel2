use crate::error::AppError;
use crate::models::NotificationTemplate;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TemplateService {
    pub pool: PgPool,
}

impl TemplateService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_template(
        &self,
        name: &str,
        content: &str,
        variables: Value,
    ) -> Result<NotificationTemplate, AppError> {
        let rec = sqlx::query_as!(NotificationTemplate,
            r#"INSERT INTO notifikasi.notification_templates (id, name, content, variables, version, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, 1, NOW(), NOW(), TRUE) RETURNING *"#,
            Uuid::new_v4(), name, content, variables
        ).fetch_one(&self.pool).await?;
        Ok(rec)
    }

    pub async fn get_template(&self, id: Uuid) -> Result<NotificationTemplate, AppError> {
        let rec = sqlx::query_as!(
            NotificationTemplate,
            r#"SELECT * FROM notifikasi.notification_templates WHERE id = $1"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    pub async fn update_template(
        &self,
        id: Uuid,
        content: &str,
        variables: Value,
    ) -> Result<NotificationTemplate, AppError> {
        let rec = sqlx::query_as!(NotificationTemplate,
            r#"UPDATE notifikasi.notification_templates SET content = $1, variables = $2, updated_at = NOW(), version = version + 1 WHERE id = $3 RETURNING *"#,
            content, variables, id
        ).fetch_one(&self.pool).await?;
        Ok(rec)
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM notifikasi.notification_templates WHERE id = $1"#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn render_template(&self, id: Uuid, variables: Value) -> Result<String, AppError> {
        let template = self.get_template(id).await?;
        let mut content = template.content;
        if let Some(vars) = variables.as_object() {
            for (k, v) in vars {
                let placeholder = format!("{{{{{}}}}}", k);
                let value = v.as_str().unwrap_or("");
                content = content.replace(&placeholder, value);
            }
        }
        Ok(content)
    }
}
