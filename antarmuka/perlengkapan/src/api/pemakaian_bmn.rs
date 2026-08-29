#[allow(unused_imports)]
use super::common::*;

#[cfg(target_arch = "wasm32")]
use crate::api::client::{auth_get_json, auth_post_json};
use serde_json::Value;

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
    // Document generation fields — both DOCX (editable) and PDF (final) are
    // produced side-by-side; URLs point at the streaming routes
    // `…/{id}/konsep-surat.docx` and `.pdf`.
    pub konsep_surat_url: Option<String>,
    pub konsep_surat_generated_at: Option<String>,
    pub konsep_surat_pdf_url: Option<String>,
    pub konsep_surat_pdf_generated_at: Option<String>,
    pub signed_pdf_url: Option<String>,
    pub signed_pdf_uploaded_at: Option<String>,
    pub is_completed: Option<bool>,
    pub status: String,
    /// Label status dari backend (`PemakaianBmnStatus::label`). Dirender apa
    /// adanya — frontend tidak lagi menyimpan kosakata statusnya sendiri.
    #[serde(default)]
    pub status_label: String,
    /// Nada semantik badge: `neutral`/`info`/`success`/`warning`/`danger`.
    #[serde(default)]
    pub status_tone: String,
    // Satker-internal approval chain (V035). These were missing entirely, so
    // the FE could not even display who validated or approved a permit — let
    // alone drive the chain. Nullability mirrors the backend entity field for
    // field: everything here is unset until that step happens.
    pub validator_satker_id: Option<String>,
    pub validator_satker_nama: Option<String>,
    pub tanggal_validasi_satker: Option<String>,
    pub catatan_validator_satker: Option<String>,
    pub approver_satker_id: Option<String>,
    pub approver_satker_nama: Option<String>,
    pub tanggal_approval_satker: Option<String>,
    pub catatan_approver_satker: Option<String>,
    /// Optimistic-concurrency token. The satker action endpoints require
    /// `expected_version` and reject a stale one, so the FE must echo back the
    /// value it rendered rather than inventing one. Non-Option because the
    /// backend always sends it.
    pub version: i32,
    pub catatan_approval: Option<String>,
    pub catatan_revocation: Option<String>,
    pub approved_by: Option<String>,
    pub approved_by_nama: Option<String>,
    pub approved_at: Option<String>,
    pub revoked_by: Option<String>,
    pub revoked_by_nama: Option<String>,
    pub revoked_at: Option<String>,
    // Nullable in the schema, and the backend models them as `Option` (see
    // `pemakaian_bmn/models/entities.rs`). Declaring them required here made
    // serde fail on the WHOLE `PaginatedResponse`, so one null audit column
    // blanked the entire list page while the API was returning the rows fine.
    // Every sibling audit field (approved_by/revoked_by/updated_by) was already
    // optional; these two were the outliers.
    pub created_by: Option<String>,
    pub created_by_nama: Option<String>,
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
    /// Imperative label for the button, authored by the backend
    /// (`PemakaianBmnAction::action_label`). Rendered verbatim: the list
    /// arrives already narrowed to what THIS caller may do, so the FE keeps no
    /// RBAC rules of its own.
    pub action_label: String,
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
    /// Label status dari backend (`PemakaianBmnStatus::label`). Dirender apa
    /// adanya — frontend tidak lagi menyimpan kosakata statusnya sendiri.
    #[serde(default)]
    pub status_label: String,
    /// Nada semantik badge: `neutral`/`info`/`success`/`warning`/`danger`.
    #[serde(default)]
    pub status_tone: String,
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

/// Three headline monitoring cards (read-only Validator Wilayah/Pusat dashboard).
/// All three are scoped to the caller's role; `tidak_dipakai` is best-effort
/// from SIMAN and `None` when SIMAN is unreachable or a `jenis_bmn` filter is
/// applied (that filter has no SIMAN-side counterpart).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringSummaryCards {
    pub sedang_dipakai: i64,
    pub akan_expired_30d: i64,
    pub tidak_dipakai: Option<i64>,
}

/// One row of "siapa memakai BMN apa".
///
/// Field names and types mirror `PemakaianBmnMonitoringRow` in
/// `layanan/perlengkapan/src/pemakaian_bmn/models/responses.rs`. Naming a field
/// the backend does not send makes serde reject the whole 200 response — the
/// failure has no HTTP status of its own, so it looks like an empty screen
/// rather than an error (#820).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PemakaianBmnMonitoringRow {
    pub id: String,
    pub nomor_izin: Option<String>,
    pub satker_code: Option<String>,
    pub satker_nama: Option<String>,
    pub kode_barang: String,
    /// Nama standar barang menurut kode barangnya.
    pub nama_barang: String,
    /// NUP — Nomor Urut Pendaftaran.
    pub nup: String,
    /// Merk/tipe: penamaan bebas operator SIMAN, bukan nama resmi barang.
    pub merk_tipe: Option<String>,
    pub jenis_bmn: String,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub durasi_hari: i64,
    pub sisa_hari: i64,
    pub status: String,
    /// Label status dari backend (`PemakaianBmnStatus::label`). Dirender apa
    /// adanya — frontend tidak lagi menyimpan kosakata statusnya sendiri.
    #[serde(default)]
    pub status_label: String,
    /// Nada semantik badge: `neutral`/`info`/`success`/`warning`/`danger`.
    #[serde(default)]
    pub status_tone: String,
}

/// A page of monitoring rows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PemakaianBmnMonitoringPage {
    pub data: Vec<PemakaianBmnMonitoringRow>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[cfg(target_arch = "wasm32")]
const PEMAKAIAN_BMN_BASE: &str = "/api/v1/perlengkapan/pemakaian-bmn";

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
) -> Result<PaginatedResponse<IzinPemakaianBmn>, crate::api::AppError> {
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

    auth_get_json(&url).await
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
) -> Result<PaginatedResponse<IzinPemakaianBmn>, crate::api::AppError> {
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
) -> Result<ApiResponse<IzinPemakaianDetailResponse>, crate::api::AppError> {
    auth_get_json(&format!("{}/{}", PEMAKAIAN_BMN_BASE, id)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pemakaian_bmn_detail(
    _id: &str,
) -> Result<ApiResponse<IzinPemakaianDetailResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Create Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn create_pemakaian_bmn(
    request: CreateIzinPemakaianRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(PEMAKAIAN_BMN_BASE, &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pemakaian_bmn(
    _request: CreateIzinPemakaianRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Check BMN Availability ---
//
// `kode_barang` wajib dikirim: sebuah aset dikenali oleh kode satker + kode
// barang + NUP, dan NUP sendirian tidak mengidentifikasi apa pun (44.017 aset
// SIMAN memakai NUP `1`). Backend menolak permintaan tanpa parameter ini.
#[cfg(target_arch = "wasm32")]
pub async fn check_bmn_availability(
    bmn_nup: &str,
    bmn_kode_barang: &str,
) -> Result<ApiResponse<BmnAvailabilityResponse>, crate::api::AppError> {
    auth_get_json(&format!(
        "{}/bmn/{}/availability?kode_barang={}",
        PEMAKAIAN_BMN_BASE,
        urlencoding::encode(bmn_nup),
        urlencoding::encode(bmn_kode_barang)
    ))
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn check_bmn_availability(
    _bmn_nup: &str,
    _bmn_kode_barang: &str,
) -> Result<ApiResponse<BmnAvailabilityResponse>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Satker-internal approval chain (#96) ---
//
// The backend has enforced this 3-step chain since V035
// (Submitted -> SubmittedApproverSatker -> Approved) via `PemakaianBmnPolicy`,
// but nothing in the FE ever called it, and the `validator_satker` /
// `approver_satker` roles it requires were absent from the IAM seed — so the
// chain was unreachable twice over.
//
// Both endpoints take an internally-tagged body (`action` discriminator) and
// `expected_version` for optimistic concurrency; a stale version is rejected
// rather than silently overwriting a concurrent decision.

/// Validator Satker forwards to Approver Satker, or returns for revision.
/// `action` is "forward" or "return"; the backend requires a catatan of at
/// least 10 characters when returning.
#[cfg(target_arch = "wasm32")]
pub async fn validator_satker_action(
    id: &str,
    action: &str,
    expected_version: i32,
    catatan: Option<String>,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/validator-satker-action", PEMAKAIAN_BMN_BASE, id),
        &serde_json::json!({
            "action": action,
            "expected_version": expected_version,
            "catatan": catatan,
        }),
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn validator_satker_action(
    _id: &str,
    _action: &str,
    _expected_version: i32,
    _catatan: Option<String>,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

/// Approver Satker (Pengguna Barang Satker) approves, or returns for revision.
/// `action` is "approve" or "return".
#[cfg(target_arch = "wasm32")]
pub async fn approver_satker_action(
    id: &str,
    action: &str,
    expected_version: i32,
    catatan: Option<String>,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/approver-satker-action", PEMAKAIAN_BMN_BASE, id),
        &serde_json::json!({
            "action": action,
            "expected_version": expected_version,
            "catatan": catatan,
        }),
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn approver_satker_action(
    _id: &str,
    _action: &str,
    _expected_version: i32,
    _catatan: Option<String>,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

/// Operator re-submits after revision. No catatan: the data changes are
/// already recorded by the preceding permit update.
#[cfg(target_arch = "wasm32")]
pub async fn resubmit_pemakaian_bmn(
    id: &str,
    expected_version: i32,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(
        &format!("{}/{}/resubmit", PEMAKAIAN_BMN_BASE, id),
        &serde_json::json!({ "expected_version": expected_version }),
    )
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn resubmit_pemakaian_bmn(
    _id: &str,
    _expected_version: i32,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Revoke Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn revoke_pemakaian_bmn(
    id: &str,
    request: RevokePermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(&format!("{}/{}/revoke", PEMAKAIAN_BMN_BASE, id), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn revoke_pemakaian_bmn(
    _id: &str,
    _request: RevokePermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Renew Permit ---
#[cfg(target_arch = "wasm32")]
pub async fn renew_pemakaian_bmn(
    id: &str,
    request: RenewPermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    auth_post_json(&format!("{}/{}/renew", PEMAKAIAN_BMN_BASE, id), &request).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn renew_pemakaian_bmn(
    _id: &str,
    _request: RenewPermitRequest,
) -> Result<ApiResponse<IzinPemakaianBmn>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- BMN Usage History ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_bmn_usage_history(
    bmn_nup: &str,
) -> Result<ApiResponse<BmnUsageStats>, crate::api::AppError> {
    auth_get_json(&format!("{}/bmn/{}/history", PEMAKAIAN_BMN_BASE, bmn_nup)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_bmn_usage_history(
    _bmn_nup: &str,
) -> Result<ApiResponse<BmnUsageStats>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Pegawai Usage History ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pegawai_usage_history(
    pegawai_nip: &str,
) -> Result<ApiResponse<PegawaiUsageStats>, crate::api::AppError> {
    auth_get_json(&format!(
        "{}/pegawai/{}/history",
        PEMAKAIAN_BMN_BASE, pegawai_nip
    ))
    .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pegawai_usage_history(
    _pegawai_nip: &str,
) -> Result<ApiResponse<PegawaiUsageStats>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Get Expiring Permits ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_expiring_permits(
    days: i32,
) -> Result<ApiResponse<Vec<IzinPemakaianBmn>>, crate::api::AppError> {
    auth_get_json(&format!("{}/expiring?days={}", PEMAKAIAN_BMN_BASE, days)).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_expiring_permits(
    _days: i32,
) -> Result<ApiResponse<Vec<IzinPemakaianBmn>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

// --- Monitoring Summary (3 headline cards) ---
/// `satker_code` is the MySIMKARI kode_satker and can only NARROW what the
/// caller's role already permits — the backend ANDs the caller's scope on top.
/// It replaces the old `satker_id`, which the backend parsed as a `Uuid` while
/// this function sent a `String`: any non-UUID value produced a 400 from the
/// query extractor before the handler ever ran.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_monitoring_summary(
    satker_code: Option<String>,
    jenis_bmn: Option<String>,
) -> Result<ApiResponse<MonitoringSummaryCards>, crate::api::AppError> {
    let mut url = format!("{}/monitoring/summary", PEMAKAIAN_BMN_BASE);
    let mut sep = '?';
    if let Some(code) = satker_code {
        url.push_str(&format!("{}satker_code={}", sep, code));
        sep = '&';
    }
    if let Some(j) = jenis_bmn {
        url.push_str(&format!("{}jenis_bmn={}", sep, j));
    }
    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_monitoring_summary(
    _satker_code: Option<String>,
    _jenis_bmn: Option<String>,
) -> Result<ApiResponse<MonitoringSummaryCards>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: MonitoringSummaryCards {
            sedang_dipakai: 0,
            akan_expired_30d: 0,
            tidak_dipakai: None,
        },
        message: "Server-side stub".to_string(),
    })
}

// --- Monitoring: daftar pemakaian BMN (ter-scope per-role) ---
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pemakaian_monitoring(
    page: i64,
    per_page: i64,
    search: Option<String>,
) -> Result<ApiResponse<PemakaianBmnMonitoringPage>, crate::api::AppError> {
    let mut url = format!(
        "{}/monitoring/pemakaian?page={}&per_page={}",
        PEMAKAIAN_BMN_BASE, page, per_page
    );
    if let Some(q) = search.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        url.push_str(&format!("&search={}", q));
    }
    auth_get_json(&url).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pemakaian_monitoring(
    _page: i64,
    _per_page: i64,
    _search: Option<String>,
) -> Result<ApiResponse<PemakaianBmnMonitoringPage>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: PemakaianBmnMonitoringPage {
            data: vec![],
            total: 0,
            page: 1,
            per_page: 20,
            total_pages: 0,
        },
        message: "Server-side stub".to_string(),
    })
}
