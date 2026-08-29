use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============ Existing Models ============

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AnalisisKebutuhan {
    pub id: String,
    pub judul: String,
    pub kategori: String,
    pub deskripsi: Option<String>,
    pub prioritas: String,
    pub status: String,
    pub estimasi_biaya: Option<f64>,
    pub justifikasi: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateAnalisisRequest {
    pub judul: String,
    pub kategori: String,
    pub deskripsi: Option<String>,
    pub prioritas: String,
    pub estimasi_biaya: Option<f64>,
    pub justifikasi: Option<String>,
}

// ============================================================================
// SK PENGHAPUSAN BMN WORKFLOW Types
// ============================================================================

/// Penghapusan BMN entity with full workflow fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PenghapusanBmnWorkflow {
    pub id: String,
    pub satker_id: String,
    pub asset_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub tanggal_penghapusan: String,
    pub alasan: String,
    pub metode_penghapusan: String,
    /// Nilai perolehan aset (harga pembelian). Sebelumnya field bernama
    /// `nilai_residu` — direname di V029 karena salah semantik.
    pub nilai_perolehan: Option<f64>,
    /// TRUE jika nilai_perolehan di-backfill dari kolom legacy nilai_residu;
    /// UI dapat menampilkan banner "perlu diverifikasi" untuk data lama.
    #[serde(default)]
    pub nilai_perolehan_dari_backfill: bool,
    pub status: String,
    /// Label status dari backend (`PenghapusanBmnStatus::label`). Dirender apa
    /// adanya — frontend tidak lagi menyimpan kosakata statusnya sendiri.
    #[serde(default)]
    pub status_label: String,
    /// Nada semantik badge: `neutral`/`info`/`success`/`warning`/`danger`.
    #[serde(default)]
    pub status_tone: String,
    /// Satu kalimat "apa berikutnya" untuk fase ini
    /// (`PenghapusanBmnStatus::hint`).
    #[serde(default)]
    pub status_hint: String,
    pub status_kode: i32,
    // Lampiran
    pub lampiran_persyaratan: Option<String>,
    pub lampiran_pendukung: Option<Value>,
    pub catatan_operator: Option<String>,
    // Validator Wilayah
    pub catatan_validator_wilayah: Option<String>,
    pub validator_wilayah_id: Option<String>,
    pub tanggal_submit_wilayah: Option<String>,
    pub tanggal_verifikasi_wilayah: Option<String>,
    // Validator Pusat
    pub catatan_validator_pusat: Option<String>,
    pub validator_pusat_id: Option<String>,
    pub tanggal_submit_pusat: Option<String>,
    pub tanggal_verifikasi_pusat: Option<String>,
    // SK Document — DOCX (editable) + PDF (final) produced side-by-side.
    pub konsep_sk_url: Option<String>,
    pub konsep_sk_generated_at: Option<String>,
    pub konsep_sk_pdf_url: Option<String>,
    pub konsep_sk_pdf_generated_at: Option<String>,
    pub signed_sk_pdf_url: Option<String>,
    pub signed_sk_pdf_uploaded_at: Option<String>,
    pub is_completed: bool,
    pub document_id: Option<String>,
    pub document_url: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Create SK Penghapusan BMN request (workflow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePenghapusanBmnWorkflowRequest {
    pub satker_id: String,
    pub asset_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub tanggal_penghapusan: String,
    pub alasan: String,
    pub metode_penghapusan: String,
    pub nilai_perolehan: Option<f64>,
    pub lampiran_persyaratan: String,
    pub catatan_operator: Option<String>,
    /// V036 (Fase 2.8): item BMN tambahan. Item pertama tetap dikirim via
    /// kolom tunggal di atas (backward compat); `items` memuat seluruh item
    /// (termasuk yg pertama) bila usulan multi-item.
    #[serde(default)]
    pub items: Vec<CreatePenghapusanBmnItemRequest>,
}

/// Penghapusan BMN list filters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PenghapusanBmnFilters {
    pub satker_id: Option<String>,
    pub status: Option<String>,
    pub status_kode: Option<i32>,
    pub metode_penghapusan: Option<String>,
    pub tahun: Option<i32>,
}

/// Upload signed SK PDF request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSignedSKRequest {
    pub signed_sk_pdf_url: String,
}

/// Satu item BMN dalam usulan multi-item (Fase 2.8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PenghapusanBmnItem {
    pub id: String,
    pub penghapusan_id: String,
    pub asset_id: Option<String>,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub nilai_perolehan: Option<f64>,
    #[serde(default)]
    pub nilai_perolehan_dari_backfill: bool,
    pub kondisi: Option<String>,
    #[serde(default)]
    pub urutan: i32,
}

/// Item input untuk create multi-item (Fase 2.8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CreatePenghapusanBmnItemRequest {
    pub asset_id: Option<String>,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub nilai_perolehan: Option<f64>,
    pub kondisi: Option<String>,
}

/// Penghapusan BMN detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanBmnDetailResponse {
    #[serde(flatten)]
    pub penghapusan: PenghapusanBmnWorkflow,
    pub allowed_transitions: Vec<PenghapusanTransitionInfo>,
    pub can_generate_sk: bool,
    pub can_upload_signed_sk: bool,
    /// V036 (Fase 2.8): daftar item BMN dalam usulan.
    #[serde(default)]
    pub items: Vec<PenghapusanBmnItem>,
}

/// Transition info for penghapusan BMN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanTransitionInfo {
    pub status_kode: i32,
    pub status_nama: String,
    /// Imperative label for the button that performs this transition, authored
    /// by the backend (`PenghapusanBmnStatus::action_label`). Rendered verbatim
    /// so the workflow vocabulary has exactly one home.
    pub action_label: String,
    /// Canonical state name for the generic `/transition` endpoint.
    pub to_state: String,
    pub requires_comment: bool,
}

/// Penghapusan workflow state codes, as sent in
/// [`PenghapusanTransitionInfo::status_kode`].
///
/// Mirrors `PenghapusanBmnStatus` in `layanan/perlengkapan`. The FE needs these
/// for exactly one reason: choosing WHICH endpoint performs a transition, since
/// several moves have purpose-built endpoints rather than the generic one. It
/// does NOT use them to decide what a user may do — that is the backend's
/// answer, delivered pre-filtered in `allowed_transitions`.
pub struct PenghapusanBmnStatusKode;

impl PenghapusanBmnStatusKode {
    pub const SUBMIT_WILAYAH: i32 = 4001;
    pub const RETURNED_TO_OPERATOR: i32 = 4002;
    pub const SUBMIT_PUSAT: i32 = 4003;
    pub const VERIFIKASI_PUSAT: i32 = 4004;
    pub const KONSEP_SK_GENERATED: i32 = 4005;
    pub const SK_SIGNED: i32 = 4006;
    pub const KONSEP_SK_WILAYAH_GENERATED: i32 = 4010;
    pub const SK_SIGNED_WILAYAH: i32 = 4011;

    /// States reached by producing or uploading a document, not by a
    /// comment-and-confirm action. These have dedicated controls on the SK
    /// panel, so the generic action list leaves them out.
    pub fn is_document_flow(status_kode: i32) -> bool {
        matches!(
            status_kode,
            Self::KONSEP_SK_GENERATED
                | Self::SK_SIGNED
                | Self::KONSEP_SK_WILAYAH_GENERATED
                | Self::SK_SIGNED_WILAYAH
        )
    }
}

// ============ Response Wrappers ============
//
// `ApiResponse<T>` + `PaginatedResponse<T>` live in `lib_perlengkapan::response`
// so the backend (axum) and the frontend (Leptos WASM) share one definition
// and a single on-wire shape. Re-export here so existing call sites that
// reference `crate::api::common::ApiResponse` keep compiling.

pub use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

// `ApiResponse` is re-exported from `lib_perlengkapan::response` above
// (see "Response Wrappers" section), so this previously-local definition
// is gone — single source of truth shared with the backend.

// ============ API Client Functions ============

#[cfg(target_arch = "wasm32")]
pub async fn fetch_analisis(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<AnalisisKebutuhan>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/v1/perlengkapan/analisis?page={}&per_page={}",
        page, per_page
    );

    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(crate::api::AppError::network(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: PaginatedResponse<AnalisisKebutuhan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_analisis(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<AnalisisKebutuhan>, crate::api::AppError> {
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
pub async fn create_analisis(
    request: CreateAnalisisRequest,
) -> Result<ApiResponse<AnalisisKebutuhan>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/analisis";
    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)?
        .send()
        .await?;

    if !resp.ok() {
        return Err(crate::api::AppError::network(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<AnalisisKebutuhan> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_analisis(
    _request: CreateAnalisisRequest,
) -> Result<ApiResponse<AnalisisKebutuhan>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}
