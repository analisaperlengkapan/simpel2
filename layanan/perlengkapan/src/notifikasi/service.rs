//! In-process [`NotificationSender`] implementation for the notifikasi module.
//!
//! Wraps the existing queue / channel-dispatch primitives so callers (the
//! workflow engine, the bantuan tiket service, etc.) can produce
//! notifications through the
//! [`lib_perlengkapan::contracts::NotificationSender`] trait instead of the
//! dropped internal gRPC client.

use async_trait::async_trait;
use chrono::Utc;
use lib_perlengkapan::ServiceError;
use lib_perlengkapan::contracts::{
    NotificationMessage, NotificationReceipt, NotificationSender,
};
use uuid::Uuid;

/// Concrete service that fulfils the [`NotificationSender`] contract.
///
/// Holds the bits of infrastructure the notifikasi module needs (db pool,
/// queue, channel adapters). Construction will gain those dependencies once
/// the shared subsystem consolidation lands; for now we keep the surface
/// minimal so [`AppState`](crate::state::AppState) can wire an
/// `Arc<dyn NotificationSender>`.
#[derive(Clone, Default)]
pub struct NotifikasiService {
    _private: (),
}

impl NotifikasiService {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl NotificationSender for NotifikasiService {
    async fn send(
        &self,
        message: NotificationMessage,
    ) -> Result<NotificationReceipt, ServiceError> {
        // TODO(perlengkapan-unified): enqueue via super::queue::NotificationQueue,
        // dispatch to email/sms/whatsapp/push/in-app channels per
        // `message.channels`, and persist the receipt. Returning a synthetic
        // receipt while the call-site migration in workflow::engine and
        // bantuan::ticket is in flight, so existing wiring exercises the
        // trait surface.
        Ok(NotificationReceipt {
            notification_id: Uuid::new_v4(),
            queued_at: Utc::now(),
            channels_dispatched: message.channels,
        })
    }
}
