use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::AppResult;
use tracing::{info, warn};

impl PemakaianBmnService {
    /// Get permits expiring soon (for notifications)
    ///
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(
        &self,
        days_threshold: i32,
    ) -> AppResult<Vec<IzinPemakaianBmn>> {
        self.repository.get_expiring_permits(days_threshold).await
    }

    /// Auto-expire permits that have passed their end date
    ///
    /// This should be called by a scheduled job
    /// Requirements: REQ-P010
    pub async fn auto_expire_permits(&self) -> AppResult<usize> {
        info!("Running auto-expire job for permits");
        let count = self.repository.auto_expire_permits().await?;
        info!("Auto-expired {} permits", count);
        Ok(count)
    }

    /// Send expiry reminder notification for a permit
    ///
    /// Requirements: REQ-P007, REQ-N008
    pub async fn send_expiry_reminder(
        &self,
        permit: &IzinPemakaianBmn,
        days_remaining: i32,
    ) -> AppResult<()> {
        if let Some(notifier) = &self.notifier {
            // Create notification data
            let notification_type = crate::workflow::WorkflowNotificationType::WorkflowTransition {
                entity_type: "pemakaian_bmn".to_string(),
                entity_id: permit.id.to_string(),
                from_state: "ACTIVE".to_string(),
                to_state: format!("EXPIRING_IN_{}_DAYS", days_remaining),
                transition_by: "system".to_string(),
                catatan: Some(format!(
                    "Izin pemakaian BMN {} untuk {} akan berakhir dalam {} hari (tanggal: {}). Nomor izin: {}. Silakan perpanjang jika masih diperlukan.",
                    permit.bmn_nama_barang,
                    permit.pegawai_nama,
                    days_remaining,
                    permit.tanggal_selesai,
                    permit.nomor_izin.as_deref().unwrap_or("N/A")
                )),
            };

            let msg = crate::workflow::to_notification_message(
                permit.created_by,
                &notification_type,
                crate::workflow::NotificationPriority::High,
            );
            match notifier.send(msg).await {
                Ok(_) => {
                    info!(
                        "Sent H-{} expiry reminder for permit {} to user {}",
                        days_remaining, permit.id, permit.created_by
                    );
                    let days_label = days_remaining.to_string();
                    crate::shared::metrics::permit_expiry_reminders_sent_total()
                        .with_label_values(&[days_label.as_str(), "success"])
                        .inc();
                }
                Err(e) => {
                    warn!(
                        "Failed to send expiry reminder for permit {}: {}",
                        permit.id, e
                    );
                    let days_label = days_remaining.to_string();
                    crate::shared::metrics::permit_expiry_reminders_sent_total()
                        .with_label_values(&[days_label.as_str(), "error"])
                        .inc();
                    crate::shared::metrics::permit_expiry_reminder_errors_total()
                        .with_label_values(&["notification_failed"])
                        .inc();
                }
            }
        } else {
            warn!("Notification sender not configured, skipping expiry reminder");
        }

        Ok(())
    }

    /// Send expiry notification for an expired permit
    ///
    /// Requirements: REQ-P007, REQ-N008
    pub async fn send_expiry_notification(&self, permit: &IzinPemakaianBmn) -> AppResult<()> {
        if let Some(notifier) = &self.notifier {
            // Create notification data
            let notification_type = crate::workflow::WorkflowNotificationType::WorkflowTransition {
                entity_type: "pemakaian_bmn".to_string(),
                entity_id: permit.id.to_string(),
                from_state: "ACTIVE".to_string(),
                to_state: "EXPIRED".to_string(),
                transition_by: "system".to_string(),
                catatan: Some(format!(
                    "Izin pemakaian BMN {} untuk {} telah berakhir pada tanggal {}. Nomor izin: {}. BMN harus segera dikembalikan atau izin diperpanjang.",
                    permit.bmn_nama_barang,
                    permit.pegawai_nama,
                    permit.tanggal_selesai,
                    permit.nomor_izin.as_deref().unwrap_or("N/A")
                )),
            };

            let msg = crate::workflow::to_notification_message(
                permit.created_by,
                &notification_type,
                crate::workflow::NotificationPriority::Urgent,
            );
            match notifier.send(msg).await {
                Ok(_) => {
                    info!(
                        "Sent expiry notification for permit {} to user {}",
                        permit.id, permit.created_by
                    );
                    crate::shared::metrics::permit_expiry_notifications_sent_total()
                        .with_label_values(&["success"])
                        .inc();
                }
                Err(e) => {
                    warn!(
                        "Failed to send expiry notification for permit {}: {}",
                        permit.id, e
                    );
                    crate::shared::metrics::permit_expiry_notifications_sent_total()
                        .with_label_values(&["error"])
                        .inc();
                    crate::shared::metrics::permit_expiry_reminder_errors_total()
                        .with_label_values(&["notification_failed"])
                        .inc();
                }
            }
        } else {
            warn!("Notification sender not configured, skipping expiry notification");
        }

        Ok(())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================
}
