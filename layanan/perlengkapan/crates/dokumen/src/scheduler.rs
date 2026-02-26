use crate::archive::ArchiveService;
use crate::error::AppError;
use chrono::Timelike;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{Duration, interval};

/// Scheduler for automatic document archival and deletion
pub struct DocumentScheduler {
    pool: deadpool_postgres::Pool,
    archive_service: Arc<ArchiveService>,
}

impl DocumentScheduler {
    pub fn new(pool: deadpool_postgres::Pool, archive_storage_path: PathBuf) -> Self {
        Self {
            pool,
            archive_service: Arc::new(ArchiveService::new(archive_storage_path)),
        }
    }

    /// Start the scheduler (runs in background)
    pub async fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            self.run_scheduler().await;
        });
    }

    /// Main scheduler loop
    async fn run_scheduler(&self) {
        // Run daily at 02:00 AM (check every hour)
        let mut interval = interval(Duration::from_secs(3600)); // 1 hour

        loop {
            interval.tick().await;

            // Check if it's 02:00 AM
            let now = chrono::Local::now();
            if now.hour() == 2 && now.minute() < 60 {
                tracing::info!("Running scheduled document archival and cleanup");

                // Archive completed workflow documents
                if let Err(e) = self.archive_completed_documents().await {
                    tracing::error!("Failed to archive completed documents: {}", e);
                }

                // Delete expired documents
                if let Err(e) = self.delete_expired_documents().await {
                    tracing::error!("Failed to delete expired documents: {}", e);
                }
            }
        }
    }

    /// Archive documents from completed workflows
    async fn archive_completed_documents(&self) -> Result<(), AppError> {
        let client = self.pool.get().await?;

        // Find documents from completed workflows that are not yet archived
        let rows = client
            .query(
                "SELECT DISTINCT d.id, d.metadata->>'workflow_state' as workflow_state
                 FROM dokumen.documents d
                 WHERE d.is_archived = false
                 AND d.metadata->>'workflow_state' = 'COMPLETED'",
                &[],
            )
            .await?;

        let mut archived_count = 0;

        for row in rows {
            let document_id: uuid::Uuid = row.get("id");
            let workflow_state: Option<String> = row.get("workflow_state");

            if let Some(state) = workflow_state {
                match self
                    .archive_service
                    .archive_document_after_completion(&self.pool, document_id, &state)
                    .await
                {
                    Ok(_) => {
                        archived_count += 1;
                        tracing::info!("Archived document {}", document_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to archive document {}: {}", document_id, e);
                    }
                }
            }
        }

        tracing::info!("Archived {} documents", archived_count);
        Ok(())
    }

    /// Delete documents past retention period
    async fn delete_expired_documents(&self) -> Result<(), AppError> {
        let deleted_count = self
            .archive_service
            .delete_expired_documents(&self.pool)
            .await?;

        tracing::info!("Deleted {} expired documents", deleted_count);
        Ok(())
    }
}
