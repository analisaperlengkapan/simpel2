use crate::error::AppError;
use crate::models::{ArchiveCollection, ArchiveDocument};
use uuid::Uuid;
use sqlx::PgPool;

pub struct ArchiveService;

impl ArchiveService {
    pub async fn create_collection(pool: &PgPool, name: &str, description: Option<&str>, owner_id: Option<Uuid>) -> Result<ArchiveCollection, AppError> {
        let rec = sqlx::query_as!(ArchiveCollection,
            r#"INSERT INTO dokumen.archive_collections (id, name, description, created_at, owner_id)
            VALUES ($1, $2, $3, NOW(), $4) RETURNING *"#,
            Uuid::new_v4(), name, description, owner_id
        ).fetch_one(pool).await?;
        Ok(rec)
    }

    pub async fn add_document_to_collection(pool: &PgPool, collection_id: Uuid, document_id: Uuid) -> Result<ArchiveDocument, AppError> {
        let rec = sqlx::query_as!(ArchiveDocument,
            r#"INSERT INTO dokumen.archive_documents (id, collection_id, document_id, added_at)
            VALUES ($1, $2, $3, NOW()) RETURNING *"#,
            Uuid::new_v4(), collection_id, document_id
        ).fetch_one(pool).await?;
        Ok(rec)
    }

    pub async fn get_collections(pool: &PgPool, owner_id: Option<Uuid>) -> Result<Vec<ArchiveCollection>, AppError> {
        let collections = if let Some(uid) = owner_id {
            sqlx::query_as!(ArchiveCollection,
                r#"SELECT * FROM dokumen.archive_collections WHERE owner_id = $1 ORDER BY created_at DESC"#,
                uid
            ).fetch_all(pool).await?
        } else {
            sqlx::query_as!(ArchiveCollection,
                r#"SELECT * FROM dokumen.archive_collections ORDER BY created_at DESC"#,
            ).fetch_all(pool).await?
        };
        Ok(collections)
    }

    pub async fn search_archive(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<ArchiveDocument>, AppError> {
        let docs = sqlx::query_as!(ArchiveDocument,
            r#"SELECT * FROM dokumen.archive_documents ad
            JOIN dokumen.documents d ON ad.document_id = d.id
            WHERE d.filename ILIKE $1 OR d.metadata::text ILIKE $1
            ORDER BY ad.added_at DESC LIMIT $2"#,
            format!("%{}%", query), limit
        ).fetch_all(pool).await?;
        Ok(docs)
    }
} 