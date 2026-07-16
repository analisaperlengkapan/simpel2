//! Row types for the `bantuan.*` schema (see `V004__bantuan_tickets.sql`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

/// A helpdesk ticket raised from `/bantuan/helpdesk`.
///
/// `user_id` / `satker_code` are set from the reporter's JWT claims at create
/// and are never accepted from client input — see the RBAC note in V004.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub satker_code: Option<String>,
    pub subject: String,
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl From<&Row> for SupportTicket {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            satker_code: row.get("satker_code"),
            subject: row.get("subject"),
            description: row.get("description"),
            priority: row.get("priority"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            closed_at: row.get("closed_at"),
        }
    }
}

/// A threaded reply on a ticket, from either the reporter or helpdesk staff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketComment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for TicketComment {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ticket_id: row.get("ticket_id"),
            user_id: row.get("user_id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
        }
    }
}
