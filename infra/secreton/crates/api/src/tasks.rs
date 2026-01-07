//! Background tasks for Secreton API
//!
//! Handles periodic maintenance tasks like garbage collection.

use crate::ApiState;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::{error, info};

/// Start background tasks
pub fn start_background_tasks(state: ApiState) {
    let state = Arc::new(state);

    // Spawn garbage collection task
    tokio::spawn(run_periodic_garbage_collection(state.clone()));
}

async fn run_periodic_garbage_collection(state: Arc<ApiState>) {
    // Run every hour
    let mut interval = time::interval(Duration::from_secs(3600));

    // Skip the immediate first tick
    interval.tick().await;

    info!("Starting periodic garbage collection task (interval: 1 hour)");

    loop {
        interval.tick().await;

        info!("Running scheduled garbage collection...");

        match state.services.admin.run_garbage_collection().await {
            Ok(result) => {
                info!(
                    "Garbage collection completed successfully. Details: {:?}",
                    result.details
                );
            },
            Err(e) => {
                error!("Garbage collection failed: {}", e);
            }
        }
    }
}
