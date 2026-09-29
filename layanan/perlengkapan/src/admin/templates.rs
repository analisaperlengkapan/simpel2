//! Admin endpoints for the `/admin/templates` UI.
//!
//! Surfaces the existing [`dokumen::TemplateService`] CRUD as REST and adds
//! a `/preview` endpoint that drives the
//! [`crate::contracts::DocumentGenerator::preview`] port so the
//! frontend can render a live PDF/DOCX/HTML preview in an iframe without
//! persisting an artifact.
//!
//! Auth: cross-satker/admin only — mirrors the rest of `/admin/*`.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::contracts::{DocumentFormat, DocumentGenerator, DocumentRequest};

use crate::dokumen::TemplateService;
use crate::dokumen::template_models::{
    DocumentTemplate, ListTemplatesQuery, ListTemplatesResponse,
};
use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::Claims;
use crate::state::AppState;
use lib_perlengkapan::response::ApiResponse;

/// Template administration: `Capability::Administer`. Was
/// `is_cross_satker_role()` ("may read across satkers"), which let
/// `validator_pusat` read and *preview-render* every template.
fn require_admin(claims: &Claims) -> AppResult<()> {
    claims.require_capability(lib_core::authz::Capability::Administer)
}

fn template_service_error(e: crate::dokumen::error::AppError) -> AppError {
    use crate::dokumen::error::AppError as DokumenError;
    match e {
        DokumenError::NotFound(msg) => AppError::NotFound(msg),
        DokumenError::Validation(msg) => AppError::BadRequest(msg),
        DokumenError::BadRequest(msg) => AppError::BadRequest(msg),
        DokumenError::Unauthorized => {
            AppError::Authorization("Unauthorized for template operation".into())
        }
        other => AppError::Internal(format!("dokumen: {other}")),
    }
}

/// GET /admin/templates
pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListTemplatesQuery>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<ListTemplatesResponse>>> {
    require_admin(&claims)?;
    let svc = TemplateService::new();
    let result = svc
        .list_templates(&state.db_pool, query)
        .await
        .map_err(template_service_error)?;
    Ok(Json(ApiResponse::success(
        result,
        "Templates retrieved successfully".to_string(),
    )))
}

/// GET /admin/templates/{id}
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<DocumentTemplate>>> {
    require_admin(&claims)?;
    let svc = TemplateService::new();
    let template = svc
        .get_template(&state.db_pool, id)
        .await
        .map_err(template_service_error)?;
    Ok(Json(ApiResponse::success(
        template,
        "Template retrieved successfully".to_string(),
    )))
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    /// `pdf` (default), `docx`, `html`, `xlsx`.
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewBody {
    /// Free-form JSON the template engine renders against. When omitted the
    /// template's `sample_data` is used so the UI can show a default preview
    /// without operator input.
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

fn parse_format(spec: Option<&str>) -> AppResult<DocumentFormat> {
    Ok(match spec.map(str::to_ascii_lowercase).as_deref() {
        Some("pdf") | None => DocumentFormat::Pdf,
        Some("docx") => DocumentFormat::Docx,
        Some("xlsx") | Some("excel") => DocumentFormat::Excel,
        Some("html") => DocumentFormat::Html,
        Some("csv") => DocumentFormat::Csv,
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "format tidak dikenal: '{}' (pakai pdf|docx|xlsx|html|csv)",
                other
            )));
        }
    })
}

fn content_type_for(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Pdf => "application/pdf",
        DocumentFormat::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        DocumentFormat::Excel => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        DocumentFormat::Html => "text/html; charset=utf-8",
        DocumentFormat::Csv => "text/csv; charset=utf-8",
    }
}

fn ext_for(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Pdf => "pdf",
        DocumentFormat::Docx => "docx",
        DocumentFormat::Excel => "xlsx",
        DocumentFormat::Html => "html",
        DocumentFormat::Csv => "csv",
    }
}

/// POST /admin/templates/{id}/preview?format={pdf|docx|html|xlsx}
///
/// Body (optional):
/// ```json
/// { "data": { ... } }
/// ```
/// Returns the rendered bytes inline (no DB write) with the matching MIME so
/// the frontend can drop the response into a blob URL + `<iframe>`.
pub async fn preview_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<PreviewQuery>,
    claims: Claims,
    Json(body): Json<PreviewBody>,
) -> AppResult<Response> {
    require_admin(&claims)?;
    let format = parse_format(q.format.as_deref())?;

    // Fall back to the template's stored `sample_data` so the iframe shows a
    // sane default preview even when the editor hasn't supplied any data yet.
    let data = match body.data {
        Some(v) => v,
        None => {
            let svc = TemplateService::new();
            let template = svc
                .get_template(&state.db_pool, id)
                .await
                .map_err(template_service_error)?;
            template
                .sample_data
                .clone()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        }
    };

    let request = DocumentRequest {
        template_id: id.to_string(),
        format,
        data,
        locale: None,
        requested_by: Some(claims.user_id),
    };

    let docs: Arc<dyn DocumentGenerator> = state.docs.clone();
    let bytes = docs
        .preview(request)
        .await
        .map_err(|e| AppError::Internal(format!("preview render failed: {e}")))?;

    let filename = format!("template-{}-preview.{}", id, ext_for(format));
    let resp = (
        [
            (header::CONTENT_TYPE, content_type_for(format)),
            (
                header::CONTENT_DISPOSITION,
                &format!("inline; filename=\"{}\"", filename),
            ),
        ],
        axum::body::Body::from(bytes),
    )
        .into_response();
    Ok(resp)
}
