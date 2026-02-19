use serde_json::Value;
#[allow(unused_imports)]
use super::common::*;

use serde::{Deserialize, Serialize};

// ============================================================================
// PEMAKAIAN BMN (BMN Usage Permit) Types and API
// ============================================================================

/// Izin Pemakaian BMN (BMN Usage Permit)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IzinPemakaianBmn {
    pub id: String,
    pub nomor_izin: Option<String>,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_satker_id: String,
    pub pegawai_satker_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,
    pub jenis_bmn: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,
    pub serial_number: Option<String>,
    pub spesifikasi: Option<Value>,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub keperluan: String,
    pub lokasi_pemakaian: Option<String>,
    pub is_renewal: bool,
    pub previous_permit_id: Option<String>,
    pub file_pendukung: Option<Value>,
    // Document generation fields
    pub konsep_surat_url: Option<String>,
    pub konsep_surat_generated_at: Option<String>,
    pub signed_pdf_url: Option<String>,
    pub signed_pdf_uploaded_at: Option<String>,
    pub is_completed: Option<bool>,
    pub status: String,
    pub catatan_approval: Option<String>,
    pub catatan_revocation: Option<String>,
    pub approved_by: Option<String>,
    pub approved_by_nama: Option<String>,
    pub approved_at: Option<String>,
    pub revoked_by: Option<String>,
    pub revoked_by_nama: Option<String>,
    pub revoked_at: Option<String>,
    pub created_by: String,
    pub created_by_nama: String,
    pub updated_by: Option<String>,
    pub updated_by_nama: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Additional BMN items in this permit (multi-BMN per pegawai)
    #[serde(default)]
    pub bmn_items: Vec<PemakaianBmnItem>,
}

/// BMN item in a pemakaian permit (multi-BMN per pegawai)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PemakaianBmnItem {
    pub id: String,
    pub izin_pemakaian_id: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,
    pub detail_bmn: Option<Value>,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create a new BMN usage permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIzinPemakaianRequest {
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_satker_id: String,
    pub pegawai_satker_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,
    pub jenis_bmn: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,
    pub serial_number: Option<String>,
    pub spesifikasi: Option<Value>,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub keperluan: String,
    pub lokasi_pemakaian: Option<String>,
    pub file_pendukung: Option<Value>,
    pub is_renewal: Option<bool>,
    pub previous_permit_id: Option<String>,
    /// Additional BMN items (multi-BMN per pegawai)
    #[serde(default)]
    pub additional_bmn_items: Vec<CreateBmnItemRequest>,
}

/// Request to add a BMN item to a permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBmnItemRequest {
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,
    pub detail_bmn: Option<Value>,
    pub keterangan: Option<String>,
}

/// Request to upload signed PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSignedPdfRequest {
    pub signed_pdf_url: String,
}

/// Request to generate konsep surat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateKonsepSuratRequest {
    pub format: Option<String>,
}

/// BMN availability check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnAvailabilityResponse {
    pub bmn_nup: String,
    pub is_available: bool,
    pub active_permit_id: Option<String>,
    pub active_permit_holder: Option<String>,
    pub active_permit_expires: Option<String>,
}

/// Workflow transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PemakaianWorkflowTransitionInfo {
    pub status: String,
    pub label: String,
    pub requires_comment: bool,
}

/// Permit detail with allowed transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IzinPemakaianDetailResponse {
    #[serde(flatten)]
    pub izin: IzinPemakaianBmn,
    pub allowed_transitions: Vec<PemakaianWorkflowTransitionInfo>,
    pub days_until_expiry: Option<i64>,
    pub is_expiring_soon: bool,
    pub can_generate_konsep: Option<bool>,
    pub can_upload_signed_pdf: Option<bool>,
}

/// Request to transition workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PemakaianWorkflowTransitionRequest {
    pub target_status: String,
    pub catatan: Option<String>,
}

/// Request to revoke a permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokePermitRequest {
    pub alasan: String,
}

/// Request to renew a permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewPermitRequest {
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub keperluan: String,
}

/// Permit history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitHistoryEntry {
    pub id: String,
    pub nomor_izin: Option<String>,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub status: String,
    pub created_at: String,
}

/// Usage statistics for a BMN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmnUsageStats {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub total_days_used: i64,
    pub current_holder: Option<String>,
    pub permit_history: Vec<PermitHistoryEntry>,
}

/// Usage statistics for a pegawai
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiUsageStats {
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub permit_history: Vec<PermitHistoryEntry>,
}

const PEMAKAIAN_BMN_BASE: &str = "/api/pembinaan/perlengkapan/pemakaian-bmn";

// --- List Permits ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pemakaian_bmn_list(
    page: i32,
    per_page: i32,
    status: Option<String>,
    jenis_bmn: Option<String>,
    pegawai_nip: Option<String>,
    satker_id: Option<String>,
    search: Option<String>,
) -> Result<PaginatedResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!("{}?page={}&per_page={}", PEMAKAIAN_BMN_BASE, page, per_page);
    if let Some(s) = status {
        url.push_str(&format!("&status={}", s));
    }
    if let Some(j) = jenis_bmn {
        url.push_str(&format!("&jenis_bmn={}", j));
    }
    if let Some(nip) = pegawai_nip {
        url.push_str(&format!("&pegawai_nip={}", nip));
    }
    if let Some(sid) = satker_id {
        url.push_str(&format!("&satker_id={}", sid));
    }
    if let Some(q) = search {
        url.push_str(&format!("&search={}", q));
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
pub async fn fetch_pemakaian_bmn_list(
    _page: i32,
    _per_page: i32,
    _status: Option<String>,
    _jenis_bmn: Option<String>,
    _pegawai_nip: Option<String>,
    _satker_id: Option<String>,
    _search: Option<String>,
) -> Result<PaginatedResponse<IzinPemakaianBmn>, String> {
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

// --- Get Permit Detail ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pemakaian_bmn_detail(
    id: &str,
) -> Result<ApiResponse<IzinPemakaianDetailResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/{}", PEMAKAIAN_BMN_BASE, id))
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
pub async fn fetch_pemakaian_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<IzinPemakaianDetailResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Create Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn create_pemakaian_bmn(
    request: CreateIzinPemakaianRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(PEMAKAIAN_BMN_BASE)
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
pub async fn create_pemakaian_bmn(
    _request: CreateIzinPemakaianRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- Check BMN Availability ---
#[cfg(target_arch = "wasm32")]
pub async fn check_bmn_availability(
    bmn_nup: &str,
) -> Result<ApiResponse<BmnAvailabilityResponse>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/bmn/{}/availability", PEMAKAIAN_BMN_BASE, bmn_nup))
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
pub async fn check_bmn_availability(
    _bmn_nup: &str,
) -> Result<ApiResponse<BmnAvailabilityResponse>, String> {
    Err("Server-side stub".to_string())
}

// --- Workflow Transition ---
#[cfg(target_arch = "wasm32")]
pub async fn transition_pemakaian_bmn_status(
    id: &str,
    request: PemakaianWorkflowTransitionRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/transition", PEMAKAIAN_BMN_BASE, id))
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
pub async fn transition_pemakaian_bmn_status(
    _id: &str,
    _request: PemakaianWorkflowTransitionRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- Activate Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn activate_pemakaian_bmn(
    id: &str,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/activate", PEMAKAIAN_BMN_BASE, id))
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
pub async fn activate_pemakaian_bmn(
    _id: &str,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- Revoke Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn revoke_pemakaian_bmn(
    id: &str,
    request: RevokePermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/revoke", PEMAKAIAN_BMN_BASE, id))
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
pub async fn revoke_pemakaian_bmn(
    _id: &str,
    _request: RevokePermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- Renew Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn renew_pemakaian_bmn(
    id: &str,
    request: RenewPermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!("{}/{}/renew", PEMAKAIAN_BMN_BASE, id))
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
pub async fn renew_pemakaian_bmn(
    _id: &str,
    _request: RenewPermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- BMN Usage History ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_bmn_usage_history(
    bmn_nup: &str,
) -> Result<ApiResponse<BmnUsageStats>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/bmn/{}/history", PEMAKAIAN_BMN_BASE, bmn_nup))
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
pub async fn fetch_bmn_usage_history(
    _bmn_nup: &str,
) -> Result<ApiResponse<BmnUsageStats>, String> {
    Err("Server-side stub".to_string())
}

// --- Pegawai Usage History ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pegawai_usage_history(
    pegawai_nip: &str,
) -> Result<ApiResponse<PegawaiUsageStats>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/pegawai/{}/history", PEMAKAIAN_BMN_BASE, pegawai_nip))
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
pub async fn fetch_pegawai_usage_history(
    _pegawai_nip: &str,
) -> Result<ApiResponse<PegawaiUsageStats>, String> {
    Err("Server-side stub".to_string())
}

// --- Get Expiring Permits ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_expiring_permits(
    days: i32,
) -> Result<ApiResponse<Vec<IzinPemakaianBmn>>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::get(&format!("{}/expiring?days={}", PEMAKAIAN_BMN_BASE, days))
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
pub async fn fetch_expiring_permits(
    _days: i32,
) -> Result<ApiResponse<Vec<IzinPemakaianBmn>>, String> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

// --- Pemakaian BMN: Generate Konsep Surat ---
#[cfg(target_arch = "wasm32")]
pub async fn generate_pemakaian_konsep_surat(
    id: &str,
    request: GenerateKonsepSuratRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/{}/generate-konsep-surat",
        PEMAKAIAN_BMN_BASE, id
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
pub async fn generate_pemakaian_konsep_surat(
    _id: &str,
    _request: GenerateKonsepSuratRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

// --- Pemakaian BMN: Upload Signed PDF ---
#[cfg(target_arch = "wasm32")]
pub async fn upload_pemakaian_signed_pdf(
    id: &str,
    request: UploadSignedPdfRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, gloo_net::Error> {
    use crate::components::auth::get_auth_token;
    use gloo_net::http::Request;

    let token = get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))?;

    let resp = Request::post(&format!(
        "{}/{}/upload-signed-pdf",
        PEMAKAIAN_BMN_BASE, id
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
pub async fn upload_pemakaian_signed_pdf(
    _id: &str,
    _request: UploadSignedPdfRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, String> {
    Err("Server-side stub".to_string())
}

