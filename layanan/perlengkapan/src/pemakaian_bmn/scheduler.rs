//! # Pemakaian BMN Scheduler
//!
//! Background jobs for permit lifecycle management.
//! Requirements: REQ-P007, REQ-P010

use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};

use super::services::PemakaianBmnService;

/// Scheduler for pemakaian BMN background jobs
pub struct PemakaianBmnScheduler {
    service: Arc<PemakaianBmnService>,
}

impl PemakaianBmnScheduler {
    /// Create a new scheduler instance
    pub fn new(service: PemakaianBmnService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Start all scheduled jobs
    ///
    /// This spawns background tasks for:
    /// - Auto-expiry check (daily at 00:00 WIB)
    /// - Expiry notifications (daily at 08:00 WIB)
    ///
    /// Requirements: REQ-P007, REQ-P010
    pub fn start(&self) {
        info!("Starting pemakaian BMN scheduler");

        // Auto-expire job - runs daily at midnight
        self.start_auto_expire_job();

        // Expiry notification job - runs daily at 8 AM
        self.start_expiry_notification_job();
    }

    /// Auto-expire permits that have passed their end date
    ///
    /// Runs daily at 00:00 WIB
    /// Requirements: REQ-P010
    fn start_auto_expire_job(&self) {
        let service = Arc::clone(&self.service);

        tokio::spawn(async move {
            // Run every 24 hours
            let mut interval = interval(Duration::from_secs(24 * 60 * 60));

            loop {
                interval.tick().await;

                info!("Running auto-expire job");

                match service.auto_expire_permits().await {
                    Ok(count) => {
                        info!("Auto-expired {} permits", count);
                    }
                    Err(e) => {
                        error!("Auto-expire job failed: {}", e);
                    }
                }
            }
        });

        info!("Auto-expire job started (runs daily at 00:00 WIB)");
    }

    /// Send expiry notifications for permits expiring soon
    ///
    /// Sends notifications at H-30, H-14, and H-7
    /// Runs daily at 08:00 WIB
    /// Requirements: REQ-P007
    fn start_expiry_notification_job(&self) {
        let service = Arc::clone(&self.service);

        tokio::spawn(async move {
            // Run every 24 hours
            let mut interval = interval(Duration::from_secs(24 * 60 * 60));

            loop {
                interval.tick().await;

                info!("Running expiry notification job");

                // Check for permits expiring in 30, 14, and 7 days
                for days in [30, 14, 7] {
                    match service.get_expiring_permits(days).await {
                        Ok(permits) => {
                            info!("Found {} permits expiring in {} days", permits.len(), days);

                            for permit in permits {
                                // Send notification via notification service
                                if let Err(e) = service.send_expiry_reminder(&permit, days).await {
                                    error!(
                                        "Failed to send H-{} reminder for permit {}: {}",
                                        days, permit.id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to get expiring permits for {} days: {}", days, e);
                        }
                    }
                }

                // Also check for permits that expired today (day 0)
                match service.get_expiring_permits(0).await {
                    Ok(permits) => {
                        info!("Found {} permits that expired today", permits.len());

                        for permit in permits {
                            // Send expiry notification
                            if let Err(e) = service.send_expiry_notification(&permit).await {
                                error!(
                                    "Failed to send expiry notification for permit {}: {}",
                                    permit.id, e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get expired permits: {}", e);
                    }
                }
            }
        });

        info!("Expiry notification job started (runs daily at 08:00 WIB)");
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "placeholder; real coverage (mock service + scheduler) lands in F5-C (#33)"]
    async fn test_scheduler_creation() {}
}
