#[allow(unused_imports)]
use super::common::*;
use super::kebutuhan_bmn_types::*;

use serde::{Deserialize, Serialize};

// ============================================================================
// KEBUTUHAN BMN API FUNCTIONS
// ============================================================================

const KEBUTUHAN_BMN_BASE: &str = "/api/pembinaan/perlengkapan/kebutuhan-bmn";

// --- Dashboard ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/dashboard", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_dashboard()
-> Result<ApiResponse<KebutuhanBmnDashboardStats>, String> {
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

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

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_list(
    _query: KebutuhanBmnQuery,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, String> {
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_kebutuhan_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn(
    request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/pengajuan", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn(
    _request: CreateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn(
    id: &str,
    request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn(
    _id: &str,
    _request: UpdateKebutuhanBmnRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn(id: &str) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&format!("{}/pengajuan/{}", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn(_id: &str) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Workflow Transition ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_kebutuhan_bmn_status(
    id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/pengajuan/{}/transition",
        KEBUTUHAN_BMN_BASE, id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_kebutuhan_bmn_status(
    _id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Satker Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_satkers(
    pengajuan_id: &str,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let url = format!(
        "{}/pengajuan/{}/satker?page={}&per_page={}",
        KEBUTUHAN_BMN_BASE, pengajuan_id, page, per_page
    );

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_satkers(
    _pengajuan_id: &str,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PengajuanKebutuhanBmnSatker>, String> {
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/satker/{}", KEBUTUHAN_BMN_BASE, satker_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_with_barang(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerWithBarangResponse>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn transition_satker_status(
    satker_id: &str,
    request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/transition",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_satker_status(
    _satker_id: &str,
    _request: WorkflowTransitionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_satker_aktivitas(
    satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/satker/{}/aktivitas",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_aktivitas(
    _satker_id: &str,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>, String> {
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/satker/{}/analisis",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_satker_analisis(
    _satker_id: &str,
) -> Result<ApiResponse<AnalisisKelayakanResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Barang Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn create_kebutuhan_bmn_barang(
    satker_id: &str,
    request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/barang",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_kebutuhan_bmn_barang(
    _satker_id: &str,
    _request: CreateKebutuhanBmnBarangRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn update_kebutuhan_bmn_barang(
    barang_id: &str,
    request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_kebutuhan_bmn_barang(
    _barang_id: &str,
    _request: UpdateBarangApprovalRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnBarang>, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_kebutuhan_bmn_barang(
    barang_id: &str,
) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&format!("{}/barang/{}", KEBUTUHAN_BMN_BASE, barang_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_kebutuhan_bmn_barang(_barang_id: &str) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Priority Operations ---
#[cfg(target_arch = "wasm32")]
pub async fn set_kebutuhan_bmn_prioritas(
    request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/prioritas", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn set_kebutuhan_bmn_prioritas(
    _request: SetPrioritasRequest,
) -> Result<ApiResponse<Vec<PengajuanKebutuhanBmnBarang>>, String> {
    Err("Server-side stub".to_string())
}

// --- Export ---
#[cfg(target_arch = "wasm32")]
pub async fn export_kebutuhan_bmn(id: &str) -> Result<Vec<u8>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/pengajuan/{}/export", KEBUTUHAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.binary().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_kebutuhan_bmn(_id: &str) -> Result<Vec<u8>, String> {
    Err("Server-side stub".to_string())
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;
    use serde_json::json;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let body = json!({
        "kebutuhan_ids": ids,
        "komentar": komentar,
    });

    let resp = Request::post(&format!("{}/batch/approve", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    let api_response: ApiResponse<BatchOperationResponse> = resp.json().await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_approve_kebutuhan(
    _ids: Vec<uuid::Uuid>,
    _komentar: Option<String>,
) -> Result<BatchOperationResponse, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn batch_reject_kebutuhan(
    ids: Vec<uuid::Uuid>,
    komentar: String,
) -> Result<BatchOperationResponse, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;
    use serde_json::json;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let body = json!({
        "kebutuhan_ids": ids,
        "komentar": komentar,
    });

    let resp = Request::post(&format!("{}/batch/reject", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    let api_response: ApiResponse<BatchOperationResponse> = resp.json().await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_reject_kebutuhan(
    _ids: Vec<uuid::Uuid>,
    _komentar: String,
) -> Result<BatchOperationResponse, String> {
    Err("Server-side stub".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn batch_update_status(
    ids: Vec<uuid::Uuid>,
    target_status: i32,
    komentar: Option<String>,
) -> Result<BatchOperationResponse, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;
    use serde_json::json;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let body = json!({
        "kebutuhan_ids": ids,
        "target_status": target_status,
        "komentar": komentar,
    });

    let resp = Request::post(&format!("{}/batch/update-status", KEBUTUHAN_BMN_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)?
        .send()
        .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    let api_response: ApiResponse<BatchOperationResponse> = resp.json().await?;
    Ok(api_response.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn batch_update_status(
    _ids: Vec<uuid::Uuid>,
    _target_status: i32,
    _komentar: Option<String>,
) -> Result<BatchOperationResponse, String> {
    Err("Server-side stub".to_string())
}

// --- Kebutuhan BMN Workflow: Submit Satker to Wilayah ---
#[cfg(target_arch = "wasm32")]
pub async fn submit_kebutuhan_satker_to_wilayah(
    satker_id: &str,
    request: SubmitKebutuhanSatkerRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/submit-wilayah",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_kebutuhan_satker_to_wilayah(
    _satker_id: &str,
    _request: SubmitKebutuhanSatkerRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, String> {
    Err("Server-side stub".to_string())
}

// --- Kebutuhan BMN Workflow: Validator Wilayah Action ---
#[cfg(target_arch = "wasm32")]
pub async fn kebutuhan_validator_wilayah_action(
    satker_id: &str,
    request: KebutuhanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/validator-wilayah",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn kebutuhan_validator_wilayah_action(
    _satker_id: &str,
    _request: KebutuhanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, String> {
    Err("Server-side stub".to_string())
}

// --- Kebutuhan BMN Workflow: Validator Pusat Decision ---
#[cfg(target_arch = "wasm32")]
pub async fn kebutuhan_validator_pusat_keputusan(
    satker_id: &str,
    request: ValidatorPusatKeputusanRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, gloo_net::Error> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/satker/{}/keputusan-pusat",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .header("Content-Type", "application/json")
    .json(&request)?
    .send()
    .await?;

    if !resp.ok() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            resp.status(),
            err_text
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn kebutuhan_validator_pusat_keputusan(
    _satker_id: &str,
    _request: ValidatorPusatKeputusanRequest,
) -> Result<ApiResponse<PengajuanKebutuhanBmnSatker>, String> {
    Err("Server-side stub".to_string())
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let mut url = format!("{}/siman/search?search={}", KEBUTUHAN_BMN_BASE, search);
    if let Some(kat) = kategori {
        url.push_str(&format!("&kategori={}", kat));
    }
    if let Some(lim) = limit {
        url.push_str(&format!("&limit={}", lim));
    }

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn search_siman_assets(
    _search: &str,
    _kategori: Option<&str>,
    _limit: Option<usize>,
) -> Result<ApiResponse<Vec<SimanAsset>>, String> {
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
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!(
        "{}/siman/summary/{}",
        KEBUTUHAN_BMN_BASE, satker_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_siman_satker_summary(
    _satker_id: &str,
) -> Result<ApiResponse<SatkerAssetSummary>, String> {
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
