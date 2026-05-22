use super::archive::ArchiveService;
use super::audit::insert_audit_log;
use super::classify::ClassifyService;
use super::config::AppConfig;
use super::error::AppError;
use super::models::*;
use super::ocr::OcrService;
use super::storage::StorageService;
use axum::extract::Multipart;
use axum::http::{StatusCode, header};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde_json::json;
use uuid::Uuid;

/// Shared application state for handlers
#[derive(Clone)]
pub struct HandlerState {
    pub pool: deadpool_postgres::Pool,
    pub config: AppConfig,
}

pub fn routes(state: HandlerState) -> Router {
    Router::new()
        .route("/documents/upload", post(upload_document))
        .route("/documents/{id}", get(get_document).delete(delete_document))
        .route("/documents/{id}/download", get(download_document))
        .route("/documents/{id}/preview", get(preview_document))
        .route("/documents/{id}/ocr", post(request_ocr))
        .route("/documents/{id}/ocr/status", get(get_ocr_status))
        .route("/documents/{id}/ocr/text", get(get_ocr_text))
        .route("/documents/{id}/classify", post(classify_document))
        .route("/documents/{id}/tags", get(get_tags).put(update_tags))
        .route("/documents/{id}/archive", post(archive_document))
        .route("/archive/collections", get(get_collections))
        .route("/archive/search", get(search_archive))
        .route("/archive/documents/search", get(search_archived_documents))
        .route("/archive/documents/{id}", get(get_archived_document))
        .route(
            "/archive/documents/expired",
            delete(delete_expired_documents),
        )
        .route("/audit/logs", get(get_audit_logs))
        .route("/health", get(health))
        .route("/health/storage", get(health_storage))
        .route("/health/ai", get(health_ai))
        .with_state(state)
}

pub async fn upload_document(
    State(state): State<HandlerState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let storage = StorageService::new(state.config.clone());
    let mut filename = None;
    let mut file_bytes = None;
    let mut metadata: Option<serde_json::Value> = None;
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            let data = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest("Gagal membaca file".to_string()))?;
            file_bytes = Some(data);
        } else if name == "metadata" {
            let data = field.text().await.unwrap_or_default();
            metadata = serde_json::from_str(&data).ok();
        }
    }
    let filename = filename.ok_or(AppError::BadRequest("File wajib diupload".to_string()))?;
    let file_bytes = file_bytes.ok_or(AppError::BadRequest("File wajib diupload".to_string()))?;
    storage.validate_extension(&filename)?;
    storage.validate_size(file_bytes.len() as u64)?;
    storage.save_encrypted(&filename, &file_bytes).await?;

    let client = state.pool.get().await?;
    let doc_id = Uuid::new_v4();
    // TODO(dokumen-handlers-unified-state): these standalone dokumen
    // handlers use a private `HandlerState { pool, config }` instead of
    // the unified `crate::state::AppState` (which carries Claims via the
    // axum middleware). Migrate them to the unified router so this
    // owner_id projection drops out — `Uuid::nil()` is a sentinel for
    // unit tests / dev only.
    let owner_id = Uuid::nil();
    let size = file_bytes.len() as i64;
    let row = client
        .query_one(
            "INSERT INTO dokumen.documents (id, filename, content_type, size, storage_path, owner_id, created_at, updated_at, is_archived, checksum, encrypted, current_version, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), FALSE, $7, TRUE, 1, $8)
             RETURNING *",
            &[&doc_id, &filename, &"application/octet-stream", &size, &filename, &owner_id, &None::<String>, &metadata],
        )
        .await?;

    let doc = Document::from(&row);

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "upload",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(doc))
}

pub async fn get_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "get_metadata",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(doc))
}

pub async fn delete_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    let storage = StorageService::new(state.config.clone());
    storage.delete(&doc.filename).await?;
    client
        .execute("DELETE FROM dokumen.documents WHERE id = $1", &[&id])
        .await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "delete",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"status": "deleted", "id": id})))
}

pub async fn download_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    let storage = StorageService::new(state.config.clone());
    let data = storage.load_decrypted(&doc.filename).await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "download",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &doc.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", doc.filename),
        )
        .body(axum::body::Body::from(data))
        .unwrap();
    Ok(resp)
}

pub async fn preview_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    let storage = StorageService::new(state.config.clone());
    let data = storage.load_decrypted(&doc.filename).await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "preview",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &doc.content_type)
        .body(axum::body::Body::from(data))
        .unwrap();
    Ok(resp)
}

pub async fn request_ocr(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    let storage = StorageService::new(state.config.clone());
    let file_bytes = storage.load_decrypted(&doc.filename).await?;
    let ocr_service = OcrService::new(state.config.clone());
    let ocr_result = ocr_service.request_ocr(id, file_bytes).await?;

    client
        .execute(
            "INSERT INTO dokumen.ocr_results (id, document_id, status, text, accuracy, processed_at, error_message)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (document_id) DO UPDATE SET status = $3, text = $4, accuracy = $5, processed_at = $6, error_message = $7",
            &[
                &ocr_result.id, &id, &ocr_result.status, &ocr_result.text,
                &ocr_result.accuracy, &ocr_result.processed_at, &ocr_result.error_message,
            ],
        )
        .await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(id),
        "ocr_request",
        &json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(ocr_result))
}

pub async fn get_ocr_status(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt(
            "SELECT * FROM dokumen.ocr_results WHERE document_id = $1",
            &[&id],
        )
        .await?;
    let status = row
        .map(|r| OcrResult::from(&r).status)
        .unwrap_or_else(|| "not_requested".to_string());
    Ok(Json(json!({"status": status})))
}

pub async fn get_ocr_text(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt(
            "SELECT * FROM dokumen.ocr_results WHERE document_id = $1",
            &[&id],
        )
        .await?;
    let text = row
        .map(|r| OcrResult::from(&r))
        .and_then(|o| o.text)
        .unwrap_or_default();
    Ok(Json(json!({"text": text})))
}

pub async fn classify_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.pool.get().await?;
    let row = client
        .query_opt("SELECT * FROM dokumen.documents WHERE id = $1", &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    let doc = Document::from(&row);

    let storage = StorageService::new(state.config.clone());
    let file_bytes = storage.load_decrypted(&doc.filename).await?;
    let classify_service = ClassifyService::new(state.config.clone());
    let tags = classify_service.classify_document(id, file_bytes).await?;
    classify_service
        .add_tags(&state.pool, id, tags.clone())
        .await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(id),
        "classify",
        &json!({"tags": tags}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"tags": tags})))
}

pub async fn get_tags(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let classify_service = ClassifyService::new(state.config.clone());
    let tags = classify_service.get_tags(&state.pool, id).await?;
    Ok(Json(tags))
}

pub async fn update_tags(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let tags = payload["tags"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let classify_service = ClassifyService::new(state.config.clone());
    classify_service
        .update_tags(&state.pool, id, tags.clone())
        .await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(id),
        "update_tags",
        &json!({"tags": tags}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"tags": tags})))
}

pub async fn archive_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let collection_id = payload["collection_id"]
        .as_str()
        .ok_or(AppError::BadRequest("collection_id wajib".to_string()))?;
    let collection_id = Uuid::parse_str(collection_id)
        .map_err(|_| AppError::BadRequest("collection_id tidak valid".to_string()))?;
    let archive_service =
        ArchiveService::new(std::path::PathBuf::from(&state.config.archive_storage_path));
    let archive_doc = archive_service
        .add_document_to_collection(&state.pool, collection_id, id)
        .await?;

    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(id),
        "archive",
        &json!({"collection_id": collection_id}),
        None,
        None,
    )
    .await?;
    Ok(Json(archive_doc))
}

pub async fn get_collections(
    State(state): State<HandlerState>,
) -> Result<impl IntoResponse, AppError> {
    let archive_service =
        ArchiveService::new(std::path::PathBuf::from(&state.config.archive_storage_path));
    let collections = archive_service.get_collections(&state.pool, None).await?;
    Ok(Json(collections))
}

pub async fn search_archive(
    State(state): State<HandlerState>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let limit = params["limit"].as_i64().unwrap_or(20);
    let archive_service =
        ArchiveService::new(std::path::PathBuf::from(&state.config.archive_storage_path));
    let docs = archive_service
        .search_archive(&state.pool, query, limit)
        .await?;
    Ok(Json(docs))
}

pub async fn get_audit_logs(
    State(state): State<HandlerState>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let document_id = params["document_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());
    let user_id = params["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());
    let limit = params["limit"].as_i64().unwrap_or(20);
    let logs = super::audit::query_audit_logs(&state.pool, document_id, user_id, limit).await?;
    Ok(Json(logs))
}

/// Get archived document by ID
pub async fn get_archived_document(
    State(state): State<HandlerState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    use std::path::PathBuf;
    let archive_service = ArchiveService::new(PathBuf::from(&state.config.archive_storage_path));

    let data = archive_service
        .get_archived_document(&state.pool, id)
        .await?;

    // Get document metadata for filename
    let client = state.pool.get().await?;
    let row = client
        .query_opt(
            "SELECT filename, content_type FROM dokumen.documents WHERE id = $1",
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    let filename: String = row.get("filename");
    let content_type: String = row.get("content_type");

    // Log audit
    insert_audit_log(
        &state.pool,
        Some(Uuid::nil()),
        Some(id),
        "retrieve_archive",
        &json!({"filename": filename}),
        None,
        None,
    )
    .await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        data,
    ))
}

/// Search archived documents with filters
pub async fn search_archived_documents(
    State(state): State<HandlerState>,
    Query(filters): Query<super::archive::ArchiveSearchFilters>,
) -> Result<impl IntoResponse, AppError> {
    use std::path::PathBuf;
    let archive_service = ArchiveService::new(PathBuf::from(&state.config.archive_storage_path));

    let documents = archive_service
        .search_archived_documents(&state.pool, filters)
        .await?;

    Ok(Json(documents))
}

/// Delete expired documents (admin only)
pub async fn delete_expired_documents(
    State(state): State<HandlerState>,
) -> Result<impl IntoResponse, AppError> {
    use std::path::PathBuf;
    let archive_service = ArchiveService::new(PathBuf::from(&state.config.archive_storage_path));

    let deleted_count = archive_service
        .delete_expired_documents(&state.pool)
        .await?;

    Ok(Json(json!({
        "deleted_count": deleted_count,
        "message": format!("Deleted {} expired documents", deleted_count)
    })))
}

pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

pub async fn health_storage() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

pub async fn health_ai() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
