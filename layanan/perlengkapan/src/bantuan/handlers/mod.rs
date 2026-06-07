pub mod analytics;
pub mod chatbot;
pub mod faq;
pub mod gdpr;
pub mod knowledge;
pub mod ticket;
pub mod webhook;

pub use analytics::*;
pub use chatbot::*;
pub use faq::*;
pub use gdpr::*;
pub use knowledge::*;
pub use ticket::*;
pub use webhook::*;

use crate::bantuan::analytics::AnalyticsService;
use crate::bantuan::chatbot::ChatbotService;
use crate::bantuan::config::AppConfig;
use crate::bantuan::export_import::ExportImportService;
use crate::bantuan::faq::FaqService;
use crate::bantuan::gdpr::GdprService;
use crate::bantuan::knowledge::KnowledgeService;
use crate::bantuan::ticket::TicketService;
use crate::bantuan::webhook::WebhookService;
use axum::{
    Router,
    routing::{get, post, put},
};
use deadpool_postgres::Pool;

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
