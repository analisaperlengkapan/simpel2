use super::error::AppError;
use super::models::HelpAnalytics;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;

pub struct AnalyticsService {
    pub pool: Pool,
}

impl AnalyticsService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn get_ticket_analytics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<HelpAnalytics>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'ticket' AND recorded_at BETWEEN $1 AND $2 ORDER BY recorded_at DESC"#,
            &[&start, &end]
        ).await?;
        let analytics = rows
            .into_iter()
            .map(|row| HelpAnalytics::from(&row))
            .collect();
        Ok(analytics)
    }
    pub async fn get_faq_analytics(
        &self,
        category_id: Option<uuid::Uuid>,
        period: &str,
    ) -> Result<Vec<HelpAnalytics>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(cat_id) = category_id {
            client.query(
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'faq' AND details->>'category_id' = $1 AND details->>'period' = $2 ORDER BY recorded_at DESC"#,
                &[&cat_id.to_string(), &period]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'faq' AND details->>'period' = $1 ORDER BY recorded_at DESC"#,
                &[&period]
            ).await?
        };
        let analytics = rows
            .into_iter()
            .map(|row| HelpAnalytics::from(&row))
            .collect();
        Ok(analytics)
    }
    pub async fn get_chatbot_analytics(
        &self,
        user_id: Option<uuid::Uuid>,
        period: &str,
    ) -> Result<Vec<HelpAnalytics>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(uid) = user_id {
            client.query(
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'chatbot' AND details->>'user_id' = $1 AND details->>'period' = $2 ORDER BY recorded_at DESC"#,
                &[&uid.to_string(), &period]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'chatbot' AND details->>'period' = $1 ORDER BY recorded_at DESC"#,
                &[&period]
            ).await?
        };
        let analytics = rows
            .into_iter()
            .map(|row| HelpAnalytics::from(&row))
            .collect();
        Ok(analytics)
    }
    pub async fn get_satisfaction(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<HelpAnalytics>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.help_analytics WHERE metric = 'satisfaction' AND recorded_at BETWEEN $1 AND $2 ORDER BY recorded_at DESC"#,
            &[&start, &end]
        ).await?;
        let analytics = rows
            .into_iter()
            .map(|row| HelpAnalytics::from(&row))
            .collect();
        Ok(analytics)
    }
}
