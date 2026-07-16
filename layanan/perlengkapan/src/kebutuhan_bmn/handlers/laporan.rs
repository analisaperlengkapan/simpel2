//! Laporan Kebutuhan BMN — cross-campaign recap (E-5).
//!
//! Backs the `/kebutuhan-bmn/laporan` page. JSON and both export formats read
//! the *same* scoped repository query, so an export can never widen what the
//! on-screen table shows.
//!
//! No `require_role` gate: the recap is a read over data the caller can
//! already list, and `SatkerScope::from_claims` restricts every row to the
//! campaigns targeting them (fail-closed to `Denied` when no satker identity
//! is present). That mirrors `get_all_pengajuan`.

use super::params::RekapLaporanQueryParams;
use axum::{
    Json,
    extract::{Query, State},
};

use crate::kebutuhan_bmn::models::{RekapLaporanFilter, RekapLaporanRow};
use crate::kebutuhan_bmn::services::KebutuhanBmnService;
use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

fn filter_of(params: &RekapLaporanQueryParams) -> RekapLaporanFilter {
    RekapLaporanFilter {
        tahun: params.tahun,
        status_kode: params.status_kode,
    }
}

/// GET /kebutuhan-bmn/laporan/rekap
pub async fn get_rekap_laporan(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<RekapLaporanQueryParams>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<RekapLaporanRow>>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let rows = service
        .get_rekap_laporan(filter_of(&params), &scope)
        .await?;
    Ok(Json(ApiResponse::success(
        rows,
        "Rekap laporan kebutuhan BMN".to_string(),
    )))
}

/// GET /kebutuhan-bmn/laporan/rekap/export?format=xlsx|pdf
pub async fn export_rekap_laporan(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<RekapLaporanQueryParams>,
    claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;

    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let filter = filter_of(&params);
    let rows = service.get_rekap_laporan(filter.clone(), &scope).await?;

    let format = params
        .format
        .as_deref()
        .unwrap_or("xlsx")
        .to_ascii_lowercase();

    match format.as_str() {
        "xlsx" => {
            let bytes = crate::kebutuhan_bmn::rekap_export::render_rekap_xlsx(&rows, &filter)?;
            let headers = [
                (
                    header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Kebutuhan_BMN.xlsx\"",
                ),
            ];
            Ok((headers, bytes).into_response())
        }
        "pdf" => {
            let bytes = crate::kebutuhan_bmn::rekap_export::render_rekap_pdf(&rows, &filter)?;
            let headers = [
                (header::CONTENT_TYPE, "application/pdf"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Kebutuhan_BMN.pdf\"",
                ),
            ];
            Ok((headers, bytes).into_response())
        }
        _ => Err(bad_request(
            "Format tidak didukung. Gunakan format=xlsx atau format=pdf.",
        )),
    }
}
