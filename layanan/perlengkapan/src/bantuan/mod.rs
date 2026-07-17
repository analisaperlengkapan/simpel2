//! Bantuan — helpdesk support tickets.
//!
//! Backs the `/bantuan/helpdesk` page: a user files a ticket, helpdesk staff
//! drive it to resolution. Ticket events fan out through the same ports the
//! workflow engine uses — [`NotificationSender`](crate::contracts::NotificationSender)
//! (so the /notifikasi centre surfaces them uniformly) and
//! [`AuditSink`](crate::contracts::AuditSink) (so `perlengkapan.audit_log`
//! records who-did-what-when).
//!
//! # Scope
//!
//! Tickets and their comments — nothing else. This module previously also
//! carried a chatbot, knowledge base, FAQ CRUD, analytics, webhooks, GDPR
//! erasure and export/import: ~2 000 lines that were never mounted, queried a
//! `bantuan.*` schema no migration created, and — in the chatbot's case —
//! required an `AI_SERVICE_URL` present in no compose file and no Helm values
//! (its config loader `expect()`ed the var, so constructing it would panic).
//! They were removed rather than wired blind (see `V004__bantuan_tickets.sql`,
//! #98). `/bantuan/faq` and `/bantuan/panduan` are static pages by design and
//! need no backend.

pub mod handlers;
pub mod models;
pub mod ticket;

pub use models::{SupportTicket, TicketComment};
pub use ticket::TicketService;
