use super::params::*;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

// ============================================================================
// Analisis Kelayakan Handlers
// ============================================================================

/// GET /kebutuhan-bmn/satker/:id/analisis
/// Get feasibility analysis for a satker
pub async fn get_analisis_kelayakan(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<AnalisisKelayakanResponse>>, AppError> {
    let response = service.get_analisis_kelayakan(satker_id).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Analisis kelayakan retrieved successfully".to_string(),
    )))
}
/// GET /kebutuhan-bmn/satker/:id/laporan/preview?format=pdf
///
/// Stream Laporan Hasil Analisis Kebutuhan BMN sbg `application/pdf` dgn
/// `Content-Disposition: inline` — FE dapat me-render di `<iframe>` tanpa
/// memicu unduhan. Hanya format `pdf` yg didukung untuk preview saat ini.
pub async fn preview_laporan_analisis(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(query): Query<LaporanFormatQuery>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    let format = query
        .format
        .as_deref()
        .unwrap_or("pdf")
        .to_ascii_lowercase();
    if format != "pdf" {
        return Err(bad_request(
            "Hanya format=pdf yg didukung utk preview inline. Gunakan endpoint download utk format lain.",
        ));
    }
    let bytes =
        crate::kebutuhan_bmn::pdf_laporan::generate_laporan_analisis_pdf(&service, satker_id)
            .await?;
    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "inline; filename=\"Laporan_Analisis_Kebutuhan_BMN.pdf\"",
        ),
    ];
    Ok((headers, bytes).into_response())
}
/// GET /kebutuhan-bmn/satker/:id/laporan/download?format=pdf|docx
///
/// Stream Laporan Hasil Analisis Kebutuhan BMN sbg attachment (force
/// download). Format `pdf` saat ini didukung; `docx` ditandai sebagai
/// follow-up (memerlukan template DOCX yg masih dirancang dgn Biro Hukum).
pub async fn download_laporan_analisis(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(query): Query<LaporanFormatQuery>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    let format = query
        .format
        .as_deref()
        .unwrap_or("pdf")
        .to_ascii_lowercase();
    match format.as_str() {
        "pdf" => {
            let bytes = crate::kebutuhan_bmn::pdf_laporan::generate_laporan_analisis_pdf(
                &service, satker_id,
            )
            .await?;
            let headers = [
                (header::CONTENT_TYPE, "application/pdf"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Analisis_Kebutuhan_BMN.pdf\"",
                ),
            ];
            Ok((headers, bytes).into_response())
        }
        "docx" => Err(bad_request(
            "Export DOCX masih dlm perancangan template (Biro Hukum). Gunakan format=pdf utk sementara.",
        )),
        _ => Err(bad_request(
            "Format tidak didukung. Gunakan format=pdf (atau format=docx setelah template selesai).",
        )),
    }
}
