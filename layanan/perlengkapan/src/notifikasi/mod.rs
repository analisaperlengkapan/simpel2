//! Notifikasi module — email/SMS/WhatsApp/push/in-app delivery, templates,
//! preferences, queue processing, WebSocket fan-out. Cross-module callers
//! (workflow, bantuan) reach this module through the
//! [`crate::contracts::NotificationSender`] trait wired into
//! [`AppState`](crate::state::AppState).

// Many services and queue processors contain placeholder/stub fields with
// unused variables and dead code that will be cleaned up incrementally.
// Suppress these module-wide for now.
#![allow(dead_code)]
#![allow(async_fn_in_trait)]

pub mod api;
pub mod audit;
pub mod config;
pub mod email;
pub mod error;
pub mod handlers;
pub mod in_app;
pub mod models;
pub mod preferences;
pub mod push;
pub mod queue;
pub mod queue_processor;
pub mod scheduler;
pub mod security;
pub mod service;
pub mod sms;
pub mod template;
pub mod websocket;
pub mod whatsapp;
