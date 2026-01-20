use crate::error::AppError;
use crate::models::ChatbotMessage;
use deadpool_postgres::Pool;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub struct ChatbotService {
    pub pool: Pool,
    pub ai_url: String,
    pub ai_api_key: Option<String>,
    pub client: Client,
}

impl ChatbotService {
    pub fn new(pool: Pool, ai_url: String, ai_api_key: Option<String>) -> Self {
        Self {
            pool,
            ai_url,
            ai_api_key,
            client: Client::new(),
        }
    }

    pub async fn process_query(
        &self,
        user_id: Option<Uuid>,
        message: &str,
        context: Option<serde_json::Value>,
    ) -> Result<String, AppError> {
        let client = self.pool.get().await?;
        // Simpan conversation jika belum ada
        let conv_id = Uuid::new_v4();
        client.execute(
            r#"INSERT INTO bantuan.chatbot_conversations (id, user_id, context, created_at) VALUES ($1, $2, $3, NOW())"#,
            &[&conv_id, &user_id, &context]
        ).await?;
        // Simpan pesan user
        client.execute(
            r#"INSERT INTO bantuan.chatbot_messages (id, conversation_id, sender, message, created_at) VALUES ($1, $2, 'user', $3, NOW())"#,
            &[&Uuid::new_v4(), &conv_id, &message]
        ).await?;
        // Kirim ke AI service
        let mut req = self
            .client
            .post(format!("{}/chatbot/query", self.ai_url))
            .json(&json!({"user_id": user_id, "message": message, "context": context}));
        if let Some(ref key) = self.ai_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req.send().await.map_err(|e| AppError::Ai(e.to_string()))?;
        let ai_resp: serde_json::Value =
            resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        let reply = ai_resp["reply"].as_str().unwrap_or("").to_string();
        // Simpan pesan bot
        client.execute(
            r#"INSERT INTO bantuan.chatbot_messages (id, conversation_id, sender, message, created_at) VALUES ($1, $2, 'bot', $3, NOW())"#,
            &[&Uuid::new_v4(), &conv_id, &reply]
        ).await?;
        Ok(reply)
    }
    pub async fn get_history(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<ChatbotMessage>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.chatbot_messages WHERE conversation_id = $1 ORDER BY created_at ASC"#,
            &[&conversation_id]
        ).await?;
        Ok(rows
            .into_iter()
            .map(|row| ChatbotMessage::from(&row))
            .collect())
    }
    pub async fn record_feedback(&self, message_id: Uuid, feedback: i32) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client
            .execute(
                r#"UPDATE bantuan.chatbot_messages SET feedback = $1 WHERE id = $2"#,
                &[&feedback, &message_id],
            )
            .await?;
        Ok(())
    }
    pub async fn get_suggestions(
        &self,
        user_id: Option<Uuid>,
        _context: Option<serde_json::Value>,
    ) -> Result<Vec<String>, AppError> {
        let mut req = self
            .client
            .get(format!("{}/chatbot/suggestions", self.ai_url))
            .query(&[
                "user_id",
                &user_id.map(|u| u.to_string()).unwrap_or_default(),
            ]);
        if let Some(ref key) = self.ai_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req.send().await.map_err(|e| AppError::Ai(e.to_string()))?;
        let ai_resp: serde_json::Value =
            resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        let suggestions = ai_resp["suggestions"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(suggestions)
    }
}
