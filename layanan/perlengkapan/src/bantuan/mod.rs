//! Bantuan module — FAQ, ticketing, knowledge base, chatbot, analytics,
//! rate limiting, captcha, GDPR export/import.
//!
//! Previously a separate crate (`layanan-perlengkapan-bantuan`); folded into
//! the unified service. The original standalone `main.rs` is gone; the
//! unified `src/main.rs` wires up bantuan handlers via the shared AppState
//! once the trait integration with notifikasi lands.

#![allow(dead_code)]

pub mod analytics;
pub mod audit;
pub mod captcha;
pub mod chatbot;
pub mod config;
pub mod error;
pub mod export_import;
pub mod faq;
pub mod gdpr;
pub mod handlers;
pub mod knowledge;
pub mod models;
pub mod rate_limit;
pub mod rbac;
pub mod ticket;
pub mod webhook;

/// Local state used by the bantuan handlers/middleware.
///
/// Kept self-contained for now so the module compiles in isolation; once the
/// unified `shared::state` consolidation lands, this will be replaced by a
/// projection from the global `AppState`.
#[derive(Clone)]
pub struct AppState {
    pub db: deadpool_postgres::Pool,
    pub redis: redis::Client,
    pub config: config::AppConfig,
    pub metrics_registry: prometheus::Registry,
    pub rate_limit: rate_limit::RateLimitState,
}

