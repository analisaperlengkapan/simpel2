#[allow(unused_imports)]
use super::common::*;

#[cfg(target_arch = "wasm32")]
use crate::api::client::{auth_delete_json, auth_get_json, auth_post_json, auth_put_json};

// ============================================================================
// SK PENGHAPUSAN BMN WORKFLOW API FUNCTIONS
// ============================================================================

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

// --- Verifikasi Aset SIMAN (Fase 2.3) ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan_verifikasi_siman(
    id: &str,
) -> Result<ApiResponse<SimanAssetVerification>, crate::api::AppError> {
    auth_get_json(&format!("{}/{}/verifikasi-siman", PENGHAPUSAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_penghapusan_verifikasi_siman(
    _id: &str,
) -> Result<ApiResponse<SimanAssetVerification>, crate::api::AppError> {
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

// --- Update Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn update_penghapusan_bmn_workflow(
    id: &str,
    request: UpdatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_put_json(&format!("{}/{}", PENGHAPUSAN_BMN_BASE, id), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_penghapusan_bmn_workflow(
    _id: &str,
    _request: UpdatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Delete Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn delete_penghapusan_bmn_workflow(
    id: &str,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    auth_delete_json(&format!("{}/{}", PENGHAPUSAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_penghapusan_bmn_workflow(
    _id: &str,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Submit Penghapusan BMN to Validator Wilayah ---
#[cfg(target_arch = "wasm32")]
pub async fn submit_penghapusan_to_wilayah(
    id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/submit-wilayah", PENGHAPUSAN_BMN_BASE, id),
        &serde_json::json!({}),
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_penghapusan_to_wilayah(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Validator Wilayah Action for Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn penghapusan_validator_wilayah_action(
    id: &str,
    request: PenghapusanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/validator-wilayah", PENGHAPUSAN_BMN_BASE, id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn penghapusan_validator_wilayah_action(
    _id: &str,
    _request: PenghapusanValidatorWilayahActionRequest,
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

/// `GET /penghapusan-bmn/{id}/lampiran` — daftar lampiran yang sudah terupload.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan_lampiran(
    id: &str,
) -> Result<ApiResponse<Vec<PenghapusanBmnLampiran>>, crate::api::AppError> {
    auth_get_json(&format!("{}/{}/lampiran", PENGHAPUSAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_penghapusan_lampiran(
    _id: &str,
) -> Result<ApiResponse<Vec<PenghapusanBmnLampiran>>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

/// `POST /penghapusan-bmn/{id}/lampiran` — unggah Surat Usulan (1x, opsional)
/// dan/atau Lampiran pendukung (0..N) sebagai multipart. Minimal satu file.
#[cfg(target_arch = "wasm32")]
pub async fn upload_penghapusan_lampiran(
    id: &str,
    surat_usulan: Option<web_sys::File>,
    lampiran: Vec<web_sys::File>,
) -> Result<(), crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let form = web_sys::FormData::new()
        .map_err(|_| crate::api::AppError::network("Gagal membuat FormData".to_string()))?;
    if let Some(f) = surat_usulan.as_ref() {
        form.append_with_blob("surat_usulan", f).map_err(|_| {
            crate::api::AppError::network("Gagal melampirkan surat usulan".to_string())
        })?;
    }
    for f in &lampiran {
        form.append_with_blob("lampiran", f)
            .map_err(|_| crate::api::AppError::network("Gagal melampirkan file".to_string()))?;
    }

    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;
    let url = format!("{}/{}/lampiran", PENGHAPUSAN_BMN_BASE, id);
    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .body(form)
        .map_err(|e| crate::api::AppError::network(format!("Request error: {}", e)))?
        .send()
        .await
        .map_err(|e| crate::api::AppError::network(format!("Network error: {}", e)))?;

    if !resp.ok() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(crate::api::AppError::from_status(status, &body));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_penghapusan_lampiran(
    _id: &str,
    _surat_usulan: Option<web_sys::File>,
    _lampiran: Vec<web_sys::File>,
) -> Result<(), crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Legacy Workflow Transition for Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_penghapusan_bmn_status(
    id: &str,
    request: PenghapusanWorkflowTransitionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/transition", PENGHAPUSAN_BMN_BASE, id),
        &request,
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn transition_penghapusan_bmn_status(
    _id: &str,
    _request: PenghapusanWorkflowTransitionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}
