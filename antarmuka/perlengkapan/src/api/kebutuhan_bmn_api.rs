#[allow(unused_imports)]
use super::common::*;
use super::kebutuhan_bmn_types::*;

#[cfg(target_arch = "wasm32")]
use crate::api::client::{
    auth_delete_json, auth_get_binary, auth_get_json, auth_post_json, auth_put_json,
};

use serde::{Deserialize, Serialize};

// ============================================================================
// KEBUTUHAN BMN API FUNCTIONS
// ============================================================================

const KEBUTUHAN_BMN_BASE: &str = "/api/pembinaan/perlengkapan/kebutuhan-bmn";

// --- Dashboard ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, gloo_net::Error> {
    auth_get_json(&format!("{}/dashboard", KEBUTUHAN_BMN_BASE)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: KebutuhanBmnDashboardStats {
            total_pengajuan: 0,
            pengajuan_draft: 0,
            pengajuan_in_progress: 0,
            pengajuan_completed: 0,
            total_satker_terlibat: 0,
            total_barang_diminta: 0,
            total_barang_disetujui: 0,
            by_tahun: vec![],
            by_status: vec![],
        },
        message: "Server-side stub".to_string(),
    })
}

// --- Pengajuan CRUD ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_list(
    query: KebutuhanBmnQuery,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, gloo_net::Error> {
    let mut url = format!(
        "{}/pengajuan?page={}&per_page={}",
        KEBUTUHAN_BMN_BASE, page, per_page
    );
    if let Some(tahun) = query.tahun {
        url.push_str(&format!("&tahun={}", tahun));
    }
    if let Some(status_kode) = query.status_kode {
        url.push_str(&format!("&status_kode={}", status_kode));
    }
    if let Some(ref satker_id) = query.satker_id {
        url.push_str(&format!("&satker_id={}", satker_id));
    }
    if let Some(ref search) = query.search {
        url.push_str(&format!("&search={}", search));
    }

    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_list(
    _query: KebutuhanBmnQuery,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, crate::api::AppError> {
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

#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_detail(
    id: &str,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    auth_get_json(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<PengajuanDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn(
    request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    auth_post_json(&format!("{}/pengajuan", KEBUTUHAN_BMN_BASE), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn(
    _request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn(
    id: &str,
    request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    auth_put_json(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn(
    _id: &str,
    _request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn(id: &str) -> Result<ApiResponse<()>, gloo_net::Error> {
    auth_delete_json(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn(_id: &str) -> Result<ApiResponse<()>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Workflow Transition ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_kebutuhan_bmn_status(
    id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/pengajuan/{}/transition", KEBUTUHAN_BMN_BASE, id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_kebutuhan_bmn_status(
    _id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Satker Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_satkers(
    pengajuan_id: &str,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    let url = format!(
        "{}/pengajuan/{}/satker?page={}&per_page={}",
        KEBUTUHAN_BMN_BASE, pengajuan_id, page, per_page
    );

    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_satkers(
    _pengajuan_id: &str,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, crate::api::AppError> {
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

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_with_barang(
    satker_id: &str,
) -> Result<ApiResponse<SatkerWithBarangResponse>, gloo_net::Error> {
    auth_get_json(&format!("{}/satker/{}", KEBUTUHAN_BMN_BASE, satker_id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_with_barang(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerWithBarangResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn transition_satker_status(
    satker_id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/satker/{}/transition", KEBUTUHAN_BMN_BASE, satker_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_satker_status(
    _satker_id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_aktivitas(
    satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, gloo_net::Error> {
    auth_get_json(&format!("{}/satker/{}/aktivitas", KEBUTUHAN_BMN_BASE, satker_id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_aktivitas(
    _satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_analisis(
    satker_id: &str,
) -> Result<ApiResponse<AnalisisKelayakanResponse>, gloo_net::Error> {
    auth_get_json(&format!("{}/satker/{}/analisis", KEBUTUHAN_BMN_BASE, satker_id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_analisis(
    _satker_id: &str,
) -> Result<ApiResponse<AnalisisKelayakanResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Barang Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn_barang(
    satker_id: &str,
    request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/satker/{}/barang", KEBUTUHAN_BMN_BASE, satker_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn_barang(
    _satker_id: &str,
    _request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn_barang(
    barang_id: &str,
    request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    auth_put_json(
        &format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn_barang(
    _barang_id: &str,
    _request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn_barang(
    barang_id: &str,
) -> Result<ApiResponse<()>, gloo_net::Error> {
    auth_delete_json(&format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn_barang(_barang_id: &str) -> Result<ApiResponse<()>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Priority Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn set_kebutuhan_bmn_prioritas(
    request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, gloo_net::Error> {
    auth_post_json(&format!("{}/prioritas", KEBUTUHAN_BMN_BASE), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn set_kebutuhan_bmn_prioritas(
    _request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Export ---
#[cfg(target_arch = "wasm32")]
pub async fn export_kebutuhan_bmn(id: &str) -> Result<Vec<u8>, gloo_net::Error> {
    auth_get_binary(&format!("{}/pengajuan/{}/export", KEBUTUHAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_kebutuhan_bmn(_id: &str) -> Result<Vec<u8>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Batch Operations ---

/// Batch operation item result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperationItemResult {
    pub kebutuhan_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperationResponse {
    pub batch_id: String,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub results: Vec<BatchOperationItemResult>,
    pub operation_type: String,
    pub executed_at: String,
    pub executed_by: Option<String>,
}

#[cfg(target_arch = "wasm32")]
pub async fn batch_approve_kebutuhan(
    ids: Vec<uuid::Uuid>,
    komentar: Option<String>,
) -> Result<BatchOperationResponse, gloo_net::Error> {
    use serde_json::json;

    let body = json!({
        "kebutuhan_ids": ids,
        "komentar": komentar,
    });

    let api_response: ApiResponse<BatchOperationResponse> =
        auth_post_json(&format!("{}/batch/approve", KEBUTUHAN_BMN_BASE), &body).await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_approve_kebutuhan(
    _ids: Vec<uuid::Uuid>,
    _komentar: Option<String>,
) -> Result<BatchOperationResponse, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn batch_reject_kebutuhan(
    ids: Vec<uuid::Uuid>,
    komentar: String,
) -> Result<BatchOperationResponse, gloo_net::Error> {
    use serde_json::json;

    let body = json!({
        "kebutuhan_ids": ids,
        "komentar": komentar,
    });

    let api_response: ApiResponse<BatchOperationResponse> =
        auth_post_json(&format!("{}/batch/reject", KEBUTUHAN_BMN_BASE), &body).await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_reject_kebutuhan(
    _ids: Vec<uuid::Uuid>,
    _komentar: String,
) -> Result<BatchOperationResponse, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn batch_update_status(
    ids: Vec<uuid::Uuid>,
    target_status: i32,
    komentar: Option<String>,
) -> Result<BatchOperationResponse, gloo_net::Error> {
    use serde_json::json;

    let body = json!({
        "kebutuhan_ids": ids,
        "target_status": target_status,
        "komentar": komentar,
    });

    let api_response: ApiResponse<BatchOperationResponse> =
        auth_post_json(&format!("{}/batch/update-status", KEBUTUHAN_BMN_BASE), &body).await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_update_status(
    _ids: Vec<uuid::Uuid>,
    _target_status: i32,
    _komentar: Option<String>,
) -> Result<BatchOperationResponse, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Kebutuhan BMN Workflow: Submit Satker to Wilayah ---
#[cfg(target_arch = "wasm32")]
pub async fn submit_kebutuhan_satker_to_wilayah(
    satker_id: &str,
    request: SubmitKebutuhanSatkerRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/satker/{}/submit-wilayah", KEBUTUHAN_BMN_BASE, satker_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_kebutuhan_satker_to_wilayah(
    _satker_id: &str,
    _request: SubmitKebutuhanSatkerRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Kebutuhan BMN Workflow: Validator Wilayah Action ---
#[cfg(target_arch = "wasm32")]
pub async fn kebutuhan_validator_wilayah_action(
    satker_id: &str,
    request: KebutuhanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/satker/{}/validator-wilayah", KEBUTUHAN_BMN_BASE, satker_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn kebutuhan_validator_wilayah_action(
    _satker_id: &str,
    _request: KebutuhanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// --- Kebutuhan BMN Workflow: Validator Pusat Decision ---
#[cfg(target_arch = "wasm32")]
pub async fn kebutuhan_validator_pusat_keputusan(
    satker_id: &str,
    request: ValidatorPusatKeputusanRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    auth_post_json(
        &format!("{}/satker/{}/keputusan-pusat", KEBUTUHAN_BMN_BASE, satker_id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn kebutuhan_validator_pusat_keputusan(
    _satker_id: &str,
    _request: ValidatorPusatKeputusanRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown("Server-side stub".to_string()))
}

// ============================================================================
// SIMAN Integration API
// ============================================================================

/// Search for existing assets from SIMAN
#[cfg(target_arch = "wasm32")]
pub async fn search_siman_assets(
    search: &str,
    kategori: Option<&str>,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<SimanAsset>>, gloo_net::Error> {
    let mut url = format!("{}/siman/search?search={}", KEBUTUHAN_BMN_BASE, search);
    if let Some(kat) = kategori {
        url.push_str(&format!("&kategori={}", kat));
    }
    if let Some(lim) = limit {
        url.push_str(&format!("&limit={}", lim));
    }

    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn search_siman_assets(
    _search: &str,
    _kategori: Option<&str>,
    _limit: Option<usize>,
) -> Result<ApiResponse<Vec<SimanAsset>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

/// Get SIMAN asset summary for a satker
#[cfg(target_arch = "wasm32")]
pub async fn fetch_siman_satker_summary(
    satker_id: &str,
) -> Result<ApiResponse<SatkerAssetSummary>, gloo_net::Error> {
    auth_get_json(&format!("{}/siman/summary/{}", KEBUTUHAN_BMN_BASE, satker_id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_siman_satker_summary(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerAssetSummary>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: SatkerAssetSummary {
            satker_id: String::new(),
            satker_name: None,
            total_assets: 0,
            total_value: 0.0,
            by_category: vec![],
            by_condition: vec![],
        },
        message: "Server-side stub".to_string(),
    })
}
