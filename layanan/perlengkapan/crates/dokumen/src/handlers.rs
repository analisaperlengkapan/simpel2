use crate::archive::ArchiveService;
use crate::audit::insert_audit_log;
use crate::classify::ClassifyService;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::*;
use crate::ocr::OcrService;
use crate::security::{Role, permission_middleware};
use crate::storage::StorageService;
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::http::{StatusCode, header};
use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    response::{Body, IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde_json::json;
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

pub fn routes(app_config: AppConfig, pool: PgPool) -> Router {
    let storage = StorageService::new(app_config.clone());
    let ocr = OcrService::new(app_config.clone());
    let classify = ClassifyService::new(app_config.clone());
    let archive = ArchiveService;
    Router::new()
        .route("/documents/upload", post(upload_document))
        .route("/documents/:id", get(get_document).delete(delete_document))
        .route("/documents/:id/download", get(download_document))
        .route("/documents/:id/preview", get(preview_document))
        .route("/documents/:id/ocr", post(request_ocr))
        .route("/documents/:id/ocr/status", get(get_ocr_status))
        .route("/documents/:id/ocr/text", get(get_ocr_text))
        .route("/documents/:id/classify", post(classify_document))
        .route("/documents/:id/tags", get(get_tags).put(update_tags))
        .route("/documents/:id/archive", post(archive_document))
        .route("/archive/collections", get(get_collections))
        .route("/archive/search", get(search_archive))
        .route("/audit/logs", get(get_audit_logs))
        .route("/health", get(health))
        .route("/health/storage", get(health_storage))
        .route("/health/ai", get(health_ai))
        .with_state(pool)
}

// Handler stub (isi detail bertahap)
pub async fn upload_document(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let storage = StorageService::new(app_config.clone());
    let mut filename = None;
    let mut file_bytes = None;
    let mut metadata = None;
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");
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
    // Simpan file terenkripsi
    storage.save_encrypted(&filename, &file_bytes).await?;
    // Insert DB
    let doc = sqlx::query_as!(Document,
        r#"INSERT INTO dokumen.documents (id, filename, content_type, size, storage_path, owner_id, created_at, updated_at, is_archived, checksum, encrypted, current_version, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), FALSE, $7, TRUE, 1, $8)
        RETURNING *"#,
        Uuid::new_v4(),
        filename,
        "application/octet-stream",
        file_bytes.len() as i64,
        filename, // storage_path = filename (bisa diubah ke UUID/struktur folder)
        Uuid::nil(), // owner_id, ganti dengan user_id dari auth
        None::<String>,
        metadata
    ).fetch_one(&pool).await?;
    // Audit log
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "upload",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(doc))
}
pub async fn get_document(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    // TODO: Ambil user_id dari auth, cek permission
    // if !is_viewer(&pool, user_id, id).await? { return Err(AppError::Forbidden); }
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "get_metadata",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(doc))
}
pub async fn delete_document(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Ambil metadata dokumen
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    // TODO: Ambil user_id dari auth, cek permission
    // if !is_owner(&pool, user_id, id).await? { return Err(AppError::Forbidden); }
    let storage = StorageService::new(app_config.clone());
    storage.delete(&doc.filename).await?;
    sqlx::query!(r#"DELETE FROM dokumen.documents WHERE id = $1"#, id)
        .execute(&pool)
        .await?;
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "delete",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"status": "deleted", "id": id})))
}
pub async fn download_document(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    // Ambil metadata dokumen
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    // TODO: Ambil user_id dari auth, cek permission
    // if !is_viewer(&pool, user_id, id).await? { return Err(AppError::Forbidden); }
    let storage = StorageService::new(app_config.clone());
    let data = storage.load_decrypted(&doc.filename).await?;
    // Audit log
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "download",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, doc.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", doc.filename),
        )
        .body(Body::from(data))
        .unwrap();
    Ok(resp)
}
pub async fn preview_document(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    // Ambil metadata dokumen
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    // TODO: Ambil user_id dari auth, cek permission
    // if !is_viewer(&pool, user_id, id).await? { return Err(AppError::Forbidden); }
    let storage = StorageService::new(app_config.clone());
    let data = storage.load_decrypted(&doc.filename).await?;
    // TODO: Generate preview (PDF/Word/image). Untuk sekarang, return file as-is.
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(doc.id),
        "preview",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, doc.content_type)
        .body(Body::from(data))
        .unwrap();
    Ok(resp)
}
pub async fn request_ocr(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    let storage = StorageService::new(app_config.clone());
    let file_bytes = storage.load_decrypted(&doc.filename).await?;
    let ocr_service = OcrService::new(app_config.clone());
    let ocr_result = ocr_service.request_ocr(id, file_bytes).await?;
    // Simpan hasil ke DB
    sqlx::query!(
        r#"INSERT INTO dokumen.ocr_results (id, document_id, status, text, accuracy, processed_at, error_message)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (document_id) DO UPDATE SET status = $3, text = $4, accuracy = $5, processed_at = $6, error_message = $7"#,
        ocr_result.id, id, ocr_result.status, ocr_result.text, ocr_result.accuracy, ocr_result.processed_at, ocr_result.error_message
    ).execute(&pool).await?;
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(id),
        "ocr_request",
        &serde_json::json!({"filename": doc.filename}),
        None,
        None,
    )
    .await?;
    Ok(Json(ocr_result))
}

pub async fn get_ocr_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ocr = sqlx::query_as!(
        OcrResult,
        r#"SELECT * FROM dokumen.ocr_results WHERE document_id = $1"#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    let status = ocr.map(|o| o.status).unwrap_or("not_requested".to_string());
    Ok(Json(json!({"status": status})))
}

pub async fn get_ocr_text(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ocr = sqlx::query_as!(
        OcrResult,
        r#"SELECT * FROM dokumen.ocr_results WHERE document_id = $1"#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    let text = ocr.and_then(|o| o.text).unwrap_or_default();
    Ok(Json(json!({"text": text})))
}
pub async fn classify_document(
    State(pool): State<PgPool>,
    State(app_config): State<AppConfig>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT * FROM dokumen.documents WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await?;
    let storage = StorageService::new(app_config.clone());
    let file_bytes = storage.load_decrypted(&doc.filename).await?;
    let classify_service = ClassifyService::new(app_config.clone());
    let tags = classify_service.classify_document(id, file_bytes).await?;
    classify_service.add_tags(&pool, id, tags.clone()).await?;
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(id),
        "classify",
        &serde_json::json!({"tags": tags}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"tags": tags})))
}

pub async fn get_tags(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let classify_service = ClassifyService::new(AppConfig::from_env());
    let tags = classify_service.get_tags(&pool, id).await?;
    Ok(Json(tags))
}

pub async fn update_tags(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let tags = payload["tags"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let classify_service = ClassifyService::new(AppConfig::from_env());
    classify_service
        .update_tags(&pool, id, tags.clone())
        .await?;
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(id),
        "update_tags",
        &serde_json::json!({"tags": tags}),
        None,
        None,
    )
    .await?;
    Ok(Json(json!({"tags": tags})))
}
pub async fn archive_document(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let collection_id = payload["collection_id"]
        .as_str()
        .ok_or(AppError::BadRequest("collection_id wajib".to_string()))?;
    let collection_id = Uuid::parse_str(collection_id)
        .map_err(|_| AppError::BadRequest("collection_id tidak valid".to_string()))?;
    let archive_service = ArchiveService;
    let archive_doc = archive_service
        .add_document_to_collection(&pool, collection_id, id)
        .await?;
    insert_audit_log(
        &pool,
        Some(Uuid::nil()),
        Some(id),
        "archive",
        &serde_json::json!({"collection_id": collection_id}),
        None,
        None,
    )
    .await?;
    Ok(Json(archive_doc))
}

pub async fn get_collections(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let archive_service = ArchiveService;
    let collections = archive_service.get_collections(&pool, None).await?;
    Ok(Json(collections))
}

pub async fn search_archive(
    State(pool): State<PgPool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let limit = params["limit"].as_i64().unwrap_or(20);
    let archive_service = ArchiveService;
    let docs = archive_service.search_archive(&pool, query, limit).await?;
    Ok(Json(docs))
}
pub async fn get_audit_logs(
    State(pool): State<PgPool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let document_id = params["document_id"]
        .as_str()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let user_id = params["user_id"]
        .as_str()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let limit = params["limit"].as_i64().unwrap_or(20);
    let logs = crate::audit::query_audit_logs(&pool, document_id, user_id, limit).await?;
    Ok(Json(logs))
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
