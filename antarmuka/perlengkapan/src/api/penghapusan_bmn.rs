#[allow(unused_imports)]
use super::common::*;

use serde::{Deserialize, Serialize};

// ============================================================================
// SK PENGHAPUSAN BMN WORKFLOW API FUNCTIONS
// ============================================================================

const PENGHAPUSAN_BMN_BASE: &str = "/api/pembinaan/perlengkapan/penghapusan-bmn";

// --- List Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_penghapusan_bmn_list(
    page: i32,
    per_page: i32,
    filters: PenghapusanBmnFilters,
) -> Result<PaginatedResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!("{}?page={}&per_page={}", PENGHAPUSAN_BMN_BASE, page, per_page);
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
pub async fn fetch_penghapusan_bmn_list(
    _page: i32,
    _per_page: i32,
    _filters: PenghapusanBmnFilters,
) -> Result<PaginatedResponse<PenghapusanBmnWorkflow>, String> {
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
) -> Result<ApiResponse<PenghapusanBmnDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/{}/detail", PENGHAPUSAN_BMN_BASE, id))
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
pub async fn fetch_penghapusan_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Create Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn create_penghapusan_bmn_workflow(
    request: CreatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(PENGHAPUSAN_BMN_BASE)
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
pub async fn create_penghapusan_bmn_workflow(
    _request: CreatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Update Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn update_penghapusan_bmn_workflow(
    id: &str,
    request: UpdatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::put(&format!("{}/{}", PENGHAPUSAN_BMN_BASE, id))
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
pub async fn update_penghapusan_bmn_workflow(
    _id: &str,
    _request: UpdatePenghapusanBmnWorkflowRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Delete Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn delete_penghapusan_bmn_workflow(
    id: &str,
) -> Result<ApiResponse<()>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::delete(&format!("{}/{}", PENGHAPUSAN_BMN_BASE, id))
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
pub async fn delete_penghapusan_bmn_workflow(
    _id: &str,
) -> Result<ApiResponse<()>, String> {
    Err("Server-side stub".to_string())
}

// --- Submit Penghapusan BMN to Validator Wilayah ---
#[cfg(target_arch = "wasm32")]
pub async fn submit_penghapusan_to_wilayah(
    id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/submit-wilayah", PENGHAPUSAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
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
pub async fn submit_penghapusan_to_wilayah(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Validator Wilayah Action for Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn penghapusan_validator_wilayah_action(
    id: &str,
    request: PenghapusanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/{}/validator-wilayah",
        PENGHAPUSAN_BMN_BASE, id
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
pub async fn penghapusan_validator_wilayah_action(
    _id: &str,
    _request: PenghapusanValidatorWilayahActionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Generate Konsep SK Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn generate_penghapusan_konsep_sk(
    id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/generate-sk", PENGHAPUSAN_BMN_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
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
pub async fn generate_penghapusan_konsep_sk(
    _id: &str,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Upload Signed SK Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn upload_penghapusan_signed_sk(
    id: &str,
    request: UploadSignedSKRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/{}/upload-signed-sk",
        PENGHAPUSAN_BMN_BASE, id
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
pub async fn upload_penghapusan_signed_sk(
    _id: &str,
    _request: UploadSignedSKRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}

// --- Legacy Workflow Transition for Penghapusan BMN ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_penghapusan_bmn_status(
    id: &str,
    request: PenghapusanWorkflowTransitionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/{}/transition",
        PENGHAPUSAN_BMN_BASE, id
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
pub async fn transition_penghapusan_bmn_status(
    _id: &str,
    _request: PenghapusanWorkflowTransitionRequest,
) -> Result<ApiResponse<PenghapusanBmnWorkflow>, String> {
    Err("Server-side stub".to_string())
}
