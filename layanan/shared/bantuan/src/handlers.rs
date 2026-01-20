use crate::analytics::AnalyticsService;
use crate::chatbot::ChatbotService;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::export_import::ExportImportService;
use crate::faq::FaqService;
use crate::gdpr::GdprService;
use crate::knowledge::KnowledgeService;
use crate::ticket::TicketService;
use crate::webhook::WebhookService;
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub fn routes(app_config: AppConfig, pool: Pool) -> Router {
    let _faq = FaqService::new(pool.clone());
    let _ticket = TicketService::new(pool.clone());
    let _chatbot = ChatbotService::new(
        pool.clone(),
        app_config.ai_service_url.clone(),
        app_config.ai_service_api_key.clone(),
    );
    let _knowledge = KnowledgeService::new(pool.clone());
    let _analytics = AnalyticsService::new(pool.clone());
    let _webhook = WebhookService::new(pool.clone());
    let _export_import = ExportImportService::new(pool.clone());
    let _gdpr = GdprService::new(pool.clone());
    Router::new()
        // FAQ
        .route(
            "/faq/categories",
            get(list_faq_categories).post(create_faq_category),
        )
        .route(
            "/faq/categories/:id",
            get(get_faq_category)
                .put(update_faq_category)
                .delete(delete_faq_category),
        )
        .route(
            "/faq/categories/:id/articles",
            get(list_faq_articles_by_category),
        )
        .route(
            "/faq/articles",
            get(list_faq_articles).post(create_faq_article),
        )
        .route(
            "/faq/articles/:id",
            get(get_faq_article)
                .put(update_faq_article)
                .delete(delete_faq_article),
        )
        .route("/faq/search", get(search_faq))
        // Ticket
        .route("/tickets", get(list_tickets).post(create_ticket))
        .route(
            "/tickets/:id",
            get(get_ticket).put(update_ticket).delete(delete_ticket),
        )
        .route(
            "/tickets/:id/comments",
            get(list_ticket_comments).post(add_ticket_comment),
        )
        .route("/tickets/:id/status", put(update_ticket_status))
        .route("/tickets/search", get(search_tickets))
        // Chatbot
        .route("/chatbot/query", post(chatbot_query))
        .route("/chatbot/history", get(chatbot_history))
        .route("/chatbot/feedback", post(chatbot_feedback))
        .route("/chatbot/suggestions", get(chatbot_suggestions))
        // Knowledge
        .route(
            "/knowledge/articles",
            get(list_knowledge_articles).post(create_knowledge_article),
        )
        .route(
            "/knowledge/articles/:id",
            get(get_knowledge_article)
                .put(update_knowledge_article)
                .delete(delete_knowledge_article),
        )
        .route("/knowledge/search", get(search_knowledge))
        .route("/knowledge/related/:id", get(get_related_knowledge))
        .route("/knowledge/export", get(export_knowledge))
        .route("/knowledge/import", post(import_knowledge))
        // Analytics
        .route("/analytics/tickets", get(analytics_tickets))
        .route("/analytics/faq", get(analytics_faq))
        .route("/analytics/chatbot", get(analytics_chatbot))
        .route("/analytics/satisfaction", get(analytics_satisfaction))
        // Audit
        .route("/audit/logs", get(get_audit_logs))
        // Webhook
        .route(
            "/webhook/events",
            get(list_webhook_events).post(create_webhook_event),
        )
        .route("/webhook/events/:id/deliver", post(deliver_webhook_event))
        .route("/webhook/events/:id/retry", post(retry_webhook_event))
        // Export/Import
        .route("/export/:resource", get(export_resource))
        .route("/import/:resource", post(import_resource))
        // GDPR
        .route("/gdpr/request_delete", post(gdpr_request_delete))
        .route("/gdpr/request_download", post(gdpr_request_download))
        .route("/gdpr/status", get(gdpr_status))
        .route("/gdpr/process/:id", post(gdpr_process_request))
        // Health
        .route("/health", get(health))
        .with_state(pool)
}

// Handler stub (isi detail bertahap)
pub async fn list_faq_categories(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let cats = faq.list_categories().await?;
    Ok(Json(cats))
}

pub async fn create_faq_category(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = payload["name"]
        .as_str()
        .ok_or(AppError::Validation("name wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let parent_id = payload
        .get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let faq = FaqService::new(pool.clone());
    let cat = faq.create_category(name, description, parent_id).await?;
    // TODO: Audit log
    Ok(Json(cat))
}

pub async fn get_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let cat = faq.get_category(id).await?;
    Ok(Json(cat))
}

pub async fn update_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = payload["name"]
        .as_str()
        .ok_or(AppError::Validation("name wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let parent_id = payload
        .get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let faq = FaqService::new(pool.clone());
    let cat = faq
        .update_category(id, name, description, parent_id)
        .await?;
    // TODO: Audit log
    Ok(Json(cat))
}

pub async fn delete_faq_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool.clone());
    faq.delete_category(id).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "deleted", "id": id})))
}

pub async fn list_faq_articles_by_category(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let arts = faq.list_articles(Some(id)).await?;
    Ok(Json(arts))
}

pub async fn list_faq_articles(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let arts = faq.list_articles(None).await?;
    Ok(Json(arts))
}

pub async fn create_faq_article(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = payload["category_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("category_id wajib".to_string()))?;
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let faq = FaqService::new(pool.clone());
    let art = faq
        .create_article(category_id, title, content, tags)
        .await?;
    // TODO: Audit log
    Ok(Json(art))
}

pub async fn get_faq_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool);
    let art = faq.get_article(id).await?;
    Ok(Json(art))
}

pub async fn update_faq_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let faq = FaqService::new(pool.clone());
    let art = faq.update_article(id, title, content, tags).await?;
    // TODO: Audit log
    Ok(Json(art))
}

pub async fn delete_faq_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let faq = FaqService::new(pool.clone());
    faq.delete_article(id).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "deleted", "id": id})))
}

pub async fn search_faq(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let faq = FaqService::new(pool);
    let arts = faq.search_faqs(query, max_results).await?;
    Ok(Json(arts))
}

pub async fn list_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let status = params.get("status").and_then(|v| v.as_str());
    let ticket = TicketService::new(pool);
    let tickets = ticket.list_tickets(user_id, status).await?;
    Ok(Json(tickets))
}

pub async fn create_ticket(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let subject = payload["subject"]
        .as_str()
        .ok_or(AppError::Validation("subject wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let priority = payload.get("priority").and_then(|v| v.as_str());
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let spam_score = payload
        .get("spam_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let ticket_service = TicketService::new(pool.clone());
    let ticket = ticket_service
        .create_ticket(
            user_id,
            subject,
            description,
            priority,
            category_id,
            spam_score,
        )
        .await?;
    // TODO: Audit log
    Ok(Json(ticket))
}

pub async fn get_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool);
    let t = ticket.get_ticket(id).await?;
    Ok(Json(t))
}

pub async fn update_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let subject = payload["subject"]
        .as_str()
        .ok_or(AppError::Validation("subject wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let priority = payload.get("priority").and_then(|v| v.as_str());
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let status = payload["status"].as_str().unwrap_or("open");
    let ticket_service = TicketService::new(pool.clone());
    let t = ticket_service
        .update_ticket(id, subject, description, priority, category_id, status)
        .await?;
    // TODO: Audit log
    Ok(Json(t))
}

pub async fn delete_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool.clone());
    ticket.delete_ticket(id).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "deleted", "id": id})))
}

pub async fn list_ticket_comments(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool);
    let comments = ticket.list_comments(id).await?;
    Ok(Json(comments))
}

pub async fn add_ticket_comment(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let ticket = TicketService::new(pool.clone());
    let comment = ticket.add_comment(id, user_id, content).await?;
    // TODO: Audit log
    Ok(Json(comment))
}

pub async fn update_ticket_status(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let status = payload["status"]
        .as_str()
        .ok_or(AppError::Validation("status wajib".to_string()))?;
    let ticket = TicketService::new(pool.clone());
    let t = ticket.update_status(id, status).await?;
    // TODO: Audit log
    Ok(Json(t))
}

pub async fn search_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let ticket = TicketService::new(pool);
    let tickets = ticket.search_tickets(query, max_results).await?;
    Ok(Json(tickets))
}
pub async fn chatbot_query(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let message = payload["message"]
        .as_str()
        .ok_or(AppError::Validation("message wajib".to_string()))?;
    let context = payload.get("context").cloned();
    let chatbot = ChatbotService::new(pool.clone(), "http://localhost:3002".to_string(), None); // TODO: ambil dari config
    let reply = chatbot.process_query(user_id, message, context).await?;
    // TODO: Audit log
    Ok(Json(json!({"reply": reply})))
}
pub async fn chatbot_history(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let conversation_id = params["conversation_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("conversation_id wajib".to_string()))?;
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    let history = chatbot.get_history(conversation_id).await?;
    Ok(Json(history))
}

pub async fn chatbot_feedback(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let message_id = payload["message_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("message_id wajib".to_string()))?;
    let feedback = payload["feedback"]
        .as_i64()
        .ok_or(AppError::Validation("feedback wajib".to_string()))? as i32;
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    chatbot.record_feedback(message_id, feedback).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "ok"})))
}

pub async fn chatbot_suggestions(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let context = params.get("context").cloned();
    let chatbot = ChatbotService::new(pool, "http://localhost:3002".to_string(), None);
    let suggestions = chatbot.get_suggestions(user_id, context).await?;
    Ok(Json(suggestions))
}
pub async fn list_knowledge_articles(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = params
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.list_articles(category_id).await?;
    Ok(Json(arts))
}

pub async fn create_knowledge_article(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let knowledge = KnowledgeService::new(pool.clone());
    let art = knowledge
        .create_article(title, content, category_id, tags)
        .await?;
    // TODO: Audit log
    Ok(Json(art))
}

pub async fn get_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool);
    let art = knowledge.get_article(id).await?;
    Ok(Json(art))
}

pub async fn update_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let title = payload["title"]
        .as_str()
        .ok_or(AppError::Validation("title wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let tags = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    let knowledge = KnowledgeService::new(pool.clone());
    let art = knowledge
        .update_article(id, title, content, category_id, tags)
        .await?;
    // TODO: Audit log
    Ok(Json(art))
}

pub async fn delete_knowledge_article(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool.clone());
    knowledge.delete_article(id).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "deleted", "id": id})))
}

pub async fn search_knowledge(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.search(query, max_results).await?;
    Ok(Json(arts))
}

pub async fn get_related_knowledge(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params["limit"].as_u64().unwrap_or(5) as u32;
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.get_related(id, limit).await?;
    Ok(Json(arts))
}

pub async fn export_knowledge(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool);
    let arts = knowledge.export_articles().await?;
    Ok(Json(arts))
}

pub async fn import_knowledge(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let knowledge = KnowledgeService::new(pool.clone());
    let articles = serde_json::from_value(payload["articles"].clone())
        .map_err(|e| AppError::Validation(e.to_string()))?;
    knowledge.import_articles(articles).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "imported"})))
}
pub async fn analytics_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    use chrono::{DateTime, Utc};
    let start = params["start"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = params["end"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_ticket_analytics(start, end).await?;
    Ok(Json(data))
}

pub async fn analytics_faq(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let category_id = params
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let period = params["period"].as_str().unwrap_or("month");
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_faq_analytics(category_id, period).await?;
    Ok(Json(data))
}

pub async fn analytics_chatbot(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let period = params["period"].as_str().unwrap_or("month");
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_chatbot_analytics(user_id, period).await?;
    Ok(Json(data))
}

pub async fn analytics_satisfaction(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    use chrono::{DateTime, Utc};
    let start = params["start"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = params["end"]
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let analytics = AnalyticsService::new(pool);
    let data = analytics.get_satisfaction(start, end).await?;
    Ok(Json(data))
}
pub async fn get_audit_logs(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let action = params.get("action").and_then(|v| v.as_str());
    let resource = params.get("resource").and_then(|v| v.as_str());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as i64;
    let logs = crate::audit::query_audit_logs(&pool, user_id, action, resource, limit).await?;
    Ok(Json(logs))
}
pub async fn list_webhook_events(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let event_type = params.get("event_type").and_then(|v| v.as_str());
    let delivered = params.get("delivered").and_then(|v| v.as_bool());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as i64;
    let webhook = WebhookService::new(pool);
    let events = webhook.list_events(event_type, delivered, limit).await?;
    Ok(Json(events))
}

pub async fn create_webhook_event(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let event_type = payload["event_type"]
        .as_str()
        .ok_or(AppError::Validation("event_type wajib".to_string()))?;
    let payload_data = payload.get("payload").cloned().unwrap_or(json!({}));
    let webhook = WebhookService::new(pool.clone());
    let event = webhook.create_event(event_type, &payload_data).await?;
    // TODO: Audit log
    Ok(Json(event))
}

pub async fn deliver_webhook_event(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let url = payload["url"]
        .as_str()
        .ok_or(AppError::Validation("url wajib".to_string()))?;
    let webhook = WebhookService::new(pool.clone());
    webhook.deliver_event(id, url).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "delivered", "id": id})))
}

pub async fn retry_webhook_event(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let url = payload["url"]
        .as_str()
        .ok_or(AppError::Validation("url wajib".to_string()))?;
    let webhook = WebhookService::new(pool.clone());
    webhook.retry_event(id, url).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "retried", "id": id})))
}
pub async fn export_resource(
    State(pool): State<Pool>,
    Path(resource): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let export_import = ExportImportService::new(pool);
    let data = export_import.export_resource(&resource).await?;
    Ok(Json(data))
}
pub async fn import_resource(
    State(pool): State<Pool>,
    Path(resource): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let export_import = ExportImportService::new(pool.clone());
    export_import.import_resource(&resource, payload).await?;
    // TODO: Audit log
    Ok(Json(json!({"status": "imported", "resource": resource})))
}
pub async fn gdpr_request_delete(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.request_delete(user_id, details).await?;
    // TODO: Audit log
    Ok(Json(req))
}

pub async fn gdpr_request_download(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.request_download(user_id, details).await?;
    // TODO: Audit log
    Ok(Json(req))
}

pub async fn gdpr_status(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let gdpr = GdprService::new(pool);
    let reqs = gdpr.get_status(user_id).await?;
    Ok(Json(reqs))
}

pub async fn gdpr_process_request(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let status = payload["status"]
        .as_str()
        .ok_or(AppError::Validation("status wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.process_request(id, status, details).await?;
    // TODO: Audit log
    Ok(Json(req))
}
pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
