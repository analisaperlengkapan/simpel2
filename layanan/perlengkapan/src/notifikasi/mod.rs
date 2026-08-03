//! Notifikasi module — email/SMS/WhatsApp/push/in-app delivery, templates,
//! preferences, queue processing, WebSocket fan-out. Cross-module callers
//! (workflow, bantuan) reach this module through the
//! [`crate::contracts::NotificationSender`] trait wired into
//! [`AppState`](crate::state::AppState).

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
// NOTE: no `scheduler` module. `NotificationScheduler` had zero references
// outside its own file — never constructed, never started — so its izin-expiry
// and SLA-breach loops never ran. Its SLA query also read
// `perlengkapan.kebutuhan_bmn` (a table that never existed) and decoded
// `EXTRACT(EPOCH ...)` as `f64`; since `EXTRACT` yields NUMERIC that would have
// panicked, and this crate builds with `panic = "abort"`, so had it ever been
// wired it would have killed the process. The live SLA path is
// `workflow::sla_scheduler`.
pub mod security;
pub mod service;
pub mod sms;
pub mod template;
pub mod websocket;
pub mod whatsapp;
