use super::error::AppError;
use super::models::{ArchiveCollection, ArchiveDocument, Document};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// Document retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Document type (e.g., "SK", "rekapitulasi")
    pub document_type: String,
    /// Retention period in years
    pub retention_years: i32,
}

impl RetentionPolicy {
    /// Get retention policy for document type
    pub fn for_document_type(document_type: &str) -> Self {
        match document_type {
            "SK" | "sk_penghapusan" | "sk_kebutuhan" => Self {
                document_type: document_type.to_string(),
                retention_years: 5,
            },
            "rekapitulasi" | "rekap_pakaian_dinas" => Self {
                document_type: document_type.to_string(),
                retention_years: 3,
            },
            _ => Self {
                document_type: document_type.to_string(),
                retention_years: 3, // Default 3 years
            },
        }
    }

    /// Calculate expiry date from creation date
    pub fn expiry_date(&self, created_at: DateTime<Utc>) -> DateTime<Utc> {
        created_at + Duration::days(365 * self.retention_years as i64)
    }

    /// Check if document should be deleted
    pub fn should_delete(&self, created_at: DateTime<Utc>) -> bool {
        Utc::now() > self.expiry_date(created_at)
    }
}

/// Archive search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSearchFilters {
    /// Date range start
    pub date_from: Option<DateTime<Utc>>,
    /// Date range end
    pub date_to: Option<DateTime<Utc>>,
    /// Document type filter
    pub document_type: Option<String>,
    /// Satker ID filter
    pub satker_id: Option<Uuid>,
    /// Search query (filename or metadata)
    pub query: Option<String>,
    /// Pagination limit
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Pagination offset
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub struct ArchiveService {
    /// Archive storage path (separate from main storage)
    archive_storage_path: PathBuf,
}

impl ArchiveService {
    pub fn new(archive_storage_path: PathBuf) -> Self {
        Self {
            archive_storage_path,
        }
    }

    /// Archive document after workflow completion
    /// Moves document to archive storage and updates status
    pub async fn archive_document_after_completion(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
        workflow_state: &str,
    ) -> Result<(), AppError> {
        // Only archive if workflow is COMPLETED
        if workflow_state != "COMPLETED" {
            return Ok(());
        }

        let client = pool.get().await?;

        // Get document details
        let row = client
            .query_opt(
                "SELECT * FROM dokumen.documents WHERE id = $1",
                &[&document_id],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        let document = Document::from(&row);

        // Skip if already archived
        if document.is_archived {
            return Ok(());
        }

        // Move document to archive storage
        let source_path = PathBuf::from(&document.storage_path);
        let archive_filename = format!("archive_{}", document.filename);
        let archive_path = self.archive_storage_path.join(&archive_filename);

        // Ensure archive directory exists
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Copy file to archive storage (keep original for safety)
        fs::copy(&source_path, &archive_path).await?;

        // Update document status to ARCHIVED
        client
            .execute(
                "UPDATE dokumen.documents SET is_archived = true, updated_at = NOW(),
                 metadata = jsonb_set(COALESCE(metadata, '{}'::jsonb), '{archive_path}', to_jsonb($2::text))
                 WHERE id = $1",
                &[&document_id, &archive_filename],
            )
            .await?;

        tracing::info!(
            "Document {} archived to {}",
            document_id,
            archive_path.display()
        );

        Ok(())
    }

    /// Get document from archive storage
    pub async fn get_archived_document(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
    ) -> Result<Vec<u8>, AppError> {
        let client = pool.get().await?;

        // Get document metadata
        let row = client
            .query_opt(
                "SELECT metadata FROM dokumen.documents WHERE id = $1 AND is_archived = true",
                &[&document_id],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Archived document not found".to_string()))?;

        let metadata: Option<serde_json::Value> = row.get("metadata");
        let archive_filename = metadata
            .as_ref()
            .and_then(|m| m.get("archive_path"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::NotFound("Archive path not found in metadata".to_string()))?;

        let archive_path = self.archive_storage_path.join(&archive_filename);

        // Read archived file
        let data = fs::read(&archive_path).await?;

        Ok(data)
    }

    /// Search archived documents with filters
    pub async fn search_archived_documents(
        &self,
        pool: &deadpool_postgres::Pool,
        filters: ArchiveSearchFilters,
    ) -> Result<Vec<Document>, AppError> {
        let client = pool.get().await?;

        let mut query = String::from("SELECT * FROM dokumen.documents WHERE is_archived = true");
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_count = 1;

        // Add date range filter
        if let Some(date_from) = filters.date_from {
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            params.push(Box::new(date_from));
            param_count += 1;
        }

        if let Some(date_to) = filters.date_to {
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            params.push(Box::new(date_to));
            param_count += 1;
        }

        // Add document type filter
        if let Some(doc_type) = filters.document_type {
            query.push_str(&format!(
                " AND metadata->>'document_type' = ${}",
                param_count
            ));
            params.push(Box::new(doc_type));
            param_count += 1;
        }

        // Add satker filter
        if let Some(satker_id) = filters.satker_id {
            query.push_str(&format!(" AND metadata->>'satker_id' = ${}", param_count));
            params.push(Box::new(satker_id.to_string()));
            param_count += 1;
        }

        // Add search query
        if let Some(search_query) = filters.query {
            let search_pattern = format!("%{}%", search_query);
            query.push_str(&format!(
                " AND (filename ILIKE ${} OR metadata::text ILIKE ${})",
                param_count, param_count
            ));
            params.push(Box::new(search_pattern));
            param_count += 1;
        }

        // Add ordering and pagination
        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_count,
            param_count + 1
        ));
        params.push(Box::new(filters.limit));
        params.push(Box::new(filters.offset));

        // Execute query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&query, &param_refs).await?;

        Ok(rows.iter().map(Document::from).collect())
    }

    /// Delete documents past retention period
    pub async fn delete_expired_documents(
        &self,
        pool: &deadpool_postgres::Pool,
    ) -> Result<usize, AppError> {
        let client = pool.get().await?;

        // Get all archived documents
        let rows = client
            .query(
                "SELECT id, created_at, metadata FROM dokumen.documents WHERE is_archived = true",
                &[],
            )
            .await?;

        let mut deleted_count = 0;

        for row in rows {
            let document_id: Uuid = row.get("id");
            let created_at: DateTime<Utc> = row.get("created_at");
            let metadata: Option<serde_json::Value> = row.get("metadata");

            // Get document type from metadata
            let document_type = metadata
                .as_ref()
                .and_then(|m| m.get("document_type"))
                .and_then(|t| t.as_str())
                .unwrap_or("default");

            let policy = RetentionPolicy::for_document_type(document_type);

            // Check if document should be deleted
            if policy.should_delete(created_at) {
                // Get archive filename
                let archive_filename = metadata
                    .as_ref()
                    .and_then(|m| m.get("archive_path"))
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string());

                // Delete physical file
                if let Some(filename) = archive_filename {
                    let archive_path = self.archive_storage_path.join(&filename);
                    if fs::metadata(&archive_path).await.is_ok() {
                        fs::remove_file(&archive_path).await?;
                        tracing::info!("Deleted expired archive file: {}", archive_path.display());
                    }
                }

                // Delete database record
                client
                    .execute(
                        "DELETE FROM dokumen.documents WHERE id = $1",
                        &[&document_id],
                    )
                    .await?;

                deleted_count += 1;
                tracing::info!(
                    "Deleted expired document {} (type: {}, age: {} years)",
                    document_id,
                    document_type,
                    (Utc::now() - created_at).num_days() / 365
                );
            }
        }

        Ok(deleted_count)
    }

    pub async fn create_collection(
        pool: &deadpool_postgres::Pool,
        name: &str,
        description: Option<&str>,
        owner_id: Option<Uuid>,
    ) -> Result<ArchiveCollection, AppError> {
        let client = pool.get().await?;
        let id = Uuid::new_v4();
        let row = client
            .query_one(
                "INSERT INTO dokumen.archive_collections (id, name, description, created_at, owner_id)
                 VALUES ($1, $2, $3, NOW(), $4) RETURNING *",
                &[&id, &name, &description, &owner_id],
            )
            .await?;
        Ok(ArchiveCollection::from(&row))
    }

    pub async fn add_document_to_collection(
        &self,
        pool: &deadpool_postgres::Pool,
        collection_id: Uuid,
        document_id: Uuid,
    ) -> Result<ArchiveDocument, AppError> {
        let client = pool.get().await?;
        let id = Uuid::new_v4();
        let row = client
            .query_one(
                "INSERT INTO dokumen.archive_documents (id, collection_id, document_id, added_at)
                 VALUES ($1, $2, $3, NOW()) RETURNING *",
                &[&id, &collection_id, &document_id],
            )
            .await?;
        Ok(ArchiveDocument::from(&row))
    }

    pub async fn get_collections(
        &self,
        pool: &deadpool_postgres::Pool,
        owner_id: Option<Uuid>,
    ) -> Result<Vec<ArchiveCollection>, AppError> {
        let client = pool.get().await?;
        let rows = if let Some(uid) = owner_id {
            client
                .query(
                    "SELECT * FROM dokumen.archive_collections WHERE owner_id = $1 ORDER BY created_at DESC",
                    &[&uid],
                )
                .await?
        } else {
            client
                .query(
                    "SELECT * FROM dokumen.archive_collections ORDER BY created_at DESC",
                    &[],
                )
                .await?
        };
        Ok(rows.iter().map(ArchiveCollection::from).collect())
    }

    pub async fn search_archive(
        &self,
        pool: &deadpool_postgres::Pool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ArchiveDocument>, AppError> {
        let client = pool.get().await?;
        let search_pattern = format!("%{}%", query);
        let rows = client
            .query(
                "SELECT ad.* FROM dokumen.archive_documents ad
                 JOIN dokumen.documents d ON ad.document_id = d.id
                 WHERE d.filename ILIKE $1 OR d.metadata::text ILIKE $1
                 ORDER BY ad.added_at DESC LIMIT $2",
                &[&search_pattern, &limit],
            )
            .await?;
        Ok(rows.iter().map(ArchiveDocument::from).collect())
    }
}
