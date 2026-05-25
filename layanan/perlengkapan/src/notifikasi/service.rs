//! In-process [`NotificationSender`] implementation for the notifikasi module.
//!
//! Wraps the postgres-backed `notifikasi.in_app_notifications` table so
//! callers (the workflow engine, the bantuan tiket service, etc.) can
//! produce notifications through the
//! [`lib_perlengkapan::contracts::NotificationSender`] trait instead of the
//! dropped internal gRPC client.
//!
//! Channel coverage:
//! - `InApp`   — writes a row into `notifikasi.in_app_notifications` (real).
//! - `Email`   — relayed through [`super::email::EmailService`] when one is
//!   attached via [`NotifikasiService::with_email`]. Falls back to a log-only
//!   skip if email isn't wired (dev profiles without SMTP/Secreton).
//! - `Sms` / `Whatsapp` / `Push` — still logged-and-skipped pending their own
//!   wiring (next iteration).

use super::email::EmailService;
use async_trait::async_trait;
use chrono::Utc;
use deadpool_postgres::Pool;
use lib_perlengkapan::ServiceError;
use lib_perlengkapan::contracts::{
    NotificationChannel, NotificationMessage, NotificationPriority, NotificationReceipt,
    NotificationSender,
};
use std::sync::Arc;
use uuid::Uuid;

/// Concrete service that fulfils the [`NotificationSender`] contract.
#[derive(Clone)]
pub struct NotifikasiService {
    pool: Pool,
    email: Option<Arc<EmailService>>,
}

impl NotifikasiService {
    pub fn new(pool: Pool) -> Self {
        Self { pool, email: None }
    }

    /// Attach an [`EmailService`] so SMTP-channel messages actually leave the
    /// service. Without this, requests for `NotificationChannel::Email` are
    /// dropped with a warning (matches the previous behaviour).
    pub fn with_email(mut self, email: Arc<EmailService>) -> Self {
        self.email = Some(email);
        self
    }

    fn priority_as_str(p: NotificationPriority) -> &'static str {
        match p {
            NotificationPriority::Low => "low",
            NotificationPriority::Medium => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Critical => "urgent",
        }
    }

    async fn dispatch_in_app(&self, msg: &NotificationMessage) -> Result<Uuid, ServiceError> {
        let notification_id = Uuid::new_v4();
        let priority = Self::priority_as_str(msg.priority);
        let metadata = msg.variables.clone().unwrap_or(serde_json::Value::Null);

        let query = r#"
            INSERT INTO notifikasi.in_app_notifications
                (id, user_id, notification_type, title, message, priority,
                 category, action_url, metadata, read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, NOW())
        "#;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| ServiceError::DatabasePool(format!("pool: {}", e)))?;

        let category: &str = "info";
        client
            .execute(
                query,
                &[
                    &notification_id,
                    &msg.recipient_user_id,
                    &msg.event,
                    &msg.title,
                    &msg.body,
                    &priority,
                    &category,
                    &msg.deeplink,
                    &metadata,
                ],
            )
            .await
            .map_err(|e| ServiceError::database(format!("INSERT in_app_notifications: {}", e)))?;

        Ok(notification_id)
    }
}

#[async_trait]
impl NotificationSender for NotifikasiService {
    async fn send(
        &self,
        message: NotificationMessage,
    ) -> Result<NotificationReceipt, ServiceError> {
        let mut dispatched = Vec::with_capacity(message.channels.len());
        let mut last_err: Option<ServiceError> = None;
        let mut id: Option<Uuid> = None;

        // Dispatch only to channels listed on the message. Failures on one
        // channel must not abort the others — best-effort fan-out matches the
        // previous gRPC client behaviour.
        for channel in &message.channels {
            match channel {
                NotificationChannel::InApp => match self.dispatch_in_app(&message).await {
                    Ok(notification_id) => {
                        if id.is_none() {
                            id = Some(notification_id);
                        }
                        dispatched.push(*channel);
                    }
                    Err(e) => {
                        tracing::warn!(
                            recipient = %message.recipient_user_id,
                            error = %e,
                            "in_app dispatch failed"
                        );
                        last_err = Some(e);
                    }
                },
                NotificationChannel::Email => match (&self.email, &message.recipient_email) {
                    (Some(email), Some(to)) => {
                        match email.send_email(to, &message.title, &message.body).await {
                            Ok(_) => dispatched.push(*channel),
                            Err(e) => {
                                tracing::warn!(
                                    recipient = %message.recipient_user_id,
                                    to = %to,
                                    error = %e,
                                    "email dispatch failed"
                                );
                                last_err = Some(ServiceError::ExternalService(format!(
                                    "email send: {}",
                                    e
                                )));
                            }
                        }
                    }
                    (Some(_), None) => {
                        tracing::info!(
                            recipient = %message.recipient_user_id,
                            event = %message.event,
                            "email channel requested but message lacks recipient_email"
                        );
                    }
                    (None, _) => {
                        tracing::info!(
                            recipient = %message.recipient_user_id,
                            event = %message.event,
                            "email channel not wired (no EmailService attached); skipped"
                        );
                    }
                },
                NotificationChannel::Sms
                | NotificationChannel::Whatsapp
                | NotificationChannel::Push => {
                    tracing::info!(
                        recipient = %message.recipient_user_id,
                        channel = ?channel,
                        event = %message.event,
                        "notification channel not yet wired; skipped"
                    );
                }
            }
        }

        if dispatched.is_empty() {
            // If we attempted only un-wired channels (or in_app failed), report
            // the most recent failure so the caller knows nothing landed.
            if let Some(err) = last_err {
                return Err(err);
            }
        }

        Ok(NotificationReceipt {
            notification_id: id.unwrap_or_else(Uuid::new_v4),
            queued_at: Utc::now(),
            channels_dispatched: dispatched,
        })
    }
}
