#[allow(unused_imports)]
use super::common::*;

#[cfg(target_arch = "wasm32")]
use crate::api::client::{auth_get_json, auth_post_json};

// ============================================================================
// SK PENGHAPUSAN BMN WORKFLOW API FUNCTIONS
// ============================================================================

#[cfg(target_arch = "wasm32")]
const PENGHAPUSAN_BMN_BASE: &str = "/api/v1/perlengkapan/penghapusan-bmn";

// --- List Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan_bmn_list(
    page: i32,
    per_page: i32,
    filters: PenghapusanBmnFilters,
) -> Result<PaginatedResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    let mut url = format!(
        "{}?page={}&per_page={}",
        PENGHAPUSAN_BMN_BASE, page, per_page
    );
    if let Some(ref satker_id) = filters.satker_id {
        url.push_str(&format!("&satker_id={}", satker_id));
    }
    if let Some(ref status) = filters.status {
        url.push_str(&format!("&status={}", status));
    }
    if let Some(status_kode) = filters.status_kode {
        url.push_str(&format!("&status_kode={}", status_kode));
    }
    if let Some(ref metode) = filters.metode_penghapusan {
        url.push_str(&format!("&metode_penghapusan={}", metode));
    }
    if let Some(tahun) = filters.tahun {
        url.push_str(&format!("&tahun={}", tahun));
    }

    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_penghapusan_bmn_list(
    _page: i32,
    _per_page: i32,
    _filters: PenghapusanBmnFilters,
) -> Result<PaginatedResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Ok(PaginatedResponse {
        success: true,
        data: vec![],
        total: 0,
        page: 1,
        per_page: 20,
        total_pages: 0,
        message: "Server-side stub".to_string(),
    })
}

// --- Get Penghapusan BMN Detail ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan_bmn_detail(
    id: &str,
) -> Result<ApiResponse<PenghapusanBmnDetailResponse>, crate::api::AppError> {
    auth_get_json(&format!("{}/{}/detail", PENGHAPUSAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_penghapusan_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Create Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn create_penghapusan_bmn_workflow(
    request: CreatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(PENGHAPUSAN_BMN_BASE, &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_penghapusan_bmn_workflow(
    _request: CreatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Generate Konsep SK Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn generate_penghapusan_konsep_sk(
    id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/generate-sk", PENGHAPUSAN_BMN_BASE, id),
        &serde_json::json!({}),
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn generate_penghapusan_konsep_sk(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Upload Signed SK Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn upload_penghapusan_signed_sk(
    id: &str,
    request: UploadSignedSKRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/upload-signed-sk", PENGHAPUSAN_BMN_BASE, id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_penghapusan_signed_sk(
    _id: &str,
    _request: UploadSignedSKRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Lampiran Pendukung (Surat Usulan + Lampiran[]) — Fase 0.6 / #15 ---
