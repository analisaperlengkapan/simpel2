use crate::models::HelpAnalytics;
use crate::error::AppError;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

pub struct AnalyticsService {
    pub pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_ticket_analytics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<HelpAnalytics>, AppError> {
        let rows = sqlx::query_as!(HelpAnalytics,
            r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'ticket' AND recorded_at BETWEEN $1 AND $2 ORDER BY recorded_at DESC"#,
            start, end
        ).fetch_all(&self.pool).await?;
        Ok(rows)
    }
    pub async fn get_faq_analytics(&self, category_id: Option<uuid::Uuid>, period: &str) -> Result<Vec<HelpAnalytics>, AppError> {
        let rows = if let Some(cat_id) = category_id {
            sqlx::query_as!(HelpAnalytics,
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'faq' AND details->>'category_id' = $1 AND details->>'period' = $2 ORDER BY recorded_at DESC"#,
                cat_id.to_string(), period
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(HelpAnalytics,
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'faq' AND details->>'period' = $1 ORDER BY recorded_at DESC"#,
                period
            ).fetch_all(&self.pool).await?
        };
        Ok(rows)
    }
    pub async fn get_chatbot_analytics(&self, user_id: Option<uuid::Uuid>, period: &str) -> Result<Vec<HelpAnalytics>, AppError> {
        let rows = if let Some(uid) = user_id {
            sqlx::query_as!(HelpAnalytics,
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'chatbot' AND details->>'user_id' = $1 AND details->>'period' = $2 ORDER BY recorded_at DESC"#,
                uid.to_string(), period
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(HelpAnalytics,
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'chatbot' AND details->>'period' = $1 ORDER BY recorded_at DESC"#,
                period
            ).fetch_all(&self.pool).await?
        };
        Ok(rows)
    }
    pub async fn get_satisfaction(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<HelpAnalytics>, AppError> {
        let rows = sqlx::query_as!(HelpAnalytics,
            r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'satisfaction' AND recorded_at BETWEEN $1 AND $2 ORDER BY recorded_at DESC"#,
            start, end
        ).fetch_all(&self.pool).await?;
        Ok(rows)
    }
} 