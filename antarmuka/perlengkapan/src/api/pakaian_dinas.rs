#[allow(unused_imports)]
use super::common::*;

use serde::{Deserialize, Serialize};

// ============ PAKAIAN DINAS (Official Uniform) Models ============

/// Jenis Pakaian Dinas (Type of official uniform)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JenisPakaianDinas {
    pub id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateJenisPakaianDinasRequest {
    pub nama: String,
    pub keterangan: Option<String>,
}

/// Spesifikasi Pakaian Dinas (Uniform specification)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpesifikasiPakaianDinas {
    pub id: String,
    pub jenis_pakaian_dinas_id: String,
    pub nama: String,
    pub keterangan: Option<String>,
    pub foto: Option<String>,
    pub jenis_pakaian_nama: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Master Ukuran (Size master data)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ukuran {
    pub id: String,
    pub group: String,
    pub size: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Pengajuan Pakaian Dinas (Uniform request/application)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanPakaianDinas {
    pub id: String,
    pub nama: String,
    pub tahun: i32,
    pub is_open: bool,
    pub tgl_open: Option<String>,
    pub tgl_close: Option<String>,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create request — mirrors backend `pakaian_dinas::models::CreatePengajuanRequest`
/// exactly (#19). Dates are `YYYY-MM-DD` strings (serde → NaiveDate); IDs are
/// UUID strings (serde → Uuid). `pilihan_satker` ∈ {all, sebagian, wilayah};
/// `wilayah_id` wajib saat `wilayah`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePengajuanPakaianDinasRequest {
    pub nama: String,
    #[serde(default)]
    pub deskripsi: Option<String>,
    #[serde(default)]
    pub tgl_mulai: Option<String>,
    #[serde(default)]
    pub tgl_selesai: Option<String>,
    pub is_reguler: bool,
    #[serde(default)]
    pub tahun: Option<i32>,
    pub pilihan_satker: String,
    #[serde(default)]
    pub dengan_unit_kerja: bool,
    #[serde(default)]
    pub jenis_pakaian_dinas_id: Option<String>,
    pub spesifikasi_ids: Vec<String>,
    #[serde(default)]
    pub satker_ids: Option<Vec<String>>,
    #[serde(default)]
    pub wilayah_id: Option<String>,
}

/// Pengajuan Satker (Work unit submission). Mirrors the backend DTO
/// `pakaian_dinas::models::PengajuanSatker` exactly — `aktivitas_id` is the
/// numeric workflow status code (see [`aktivitas_label`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanSatker {
    pub id: String,
    pub pengajuan_id: String,
    pub satker_id: String,
    #[serde(default)]
    pub satker_pusat_id: Option<String>,
    #[serde(default)]
    pub id_kejati: Option<String>,
    #[serde(default)]
    pub id_kejari: Option<String>,
    #[serde(default)]
    pub id_cabjari: Option<String>,
    pub aktivitas_id: i32,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub satker_nama: Option<String>,
    #[serde(default)]
    pub satker_kode: Option<String>,
    #[serde(default)]
    pub aktivitas_label: Option<String>,
    #[serde(default)]
    pub total_pegawai: Option<i64>,
}

/// One per-satker workflow activity row. Mirrors the backend
/// `PengajuanSatkerAktivitas`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PengajuanSatkerAktivitas {
    pub id: String,
    pub pengajuan_satker_id: String,
    pub aktivitas_id: i32,
    #[serde(default)]
    pub komentar: Option<String>,
    #[serde(default)]
    pub nip: Option<String>,
    #[serde(default)]
    pub nama: Option<String>,
    #[serde(default)]
    pub pangkat: Option<String>,
    #[serde(default)]
    pub jabatan: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    pub created_at: String,
}

/// Human label for a workflow status code (mirrors backend
/// `AktivitasStatus::label`).
pub fn aktivitas_label(code: i32) -> &'static str {
    match code {
        1000 => "Penyiapan / Input",
        1001 | 1012 => "Diajukan ke Validator Wilayah",
        1003 => "Revisi Pelaksana",
        1004 | 1010 => "Diajukan ke Pusat",
        1007 => "Revisi Wilayah",
        1008 => "Selesai",
        _ => "Dalam Proses",
    }
}

/// True for terminal-rejection / revision states (rendered red in the timeline).
pub fn aktivitas_is_revisi(code: i32) -> bool {
    matches!(code, 1003 | 1007)
}

/// Workflow action request — body for `POST /pakaian-dinas/validator-action`.
/// Field names match the backend `ValidatorActionRequest` (`aksi`/`komentar`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorActionRequest {
    pub pengajuan_satker_id: String,
    pub aksi: String,
    pub komentar: Option<String>,
}

/// Pegawai Pakaian Dinas (Employee uniform sizes)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PegawaiPakaianDinas {
    pub id: String,
    pub pegawai_id: String,
    pub pegawai_nama: Option<String>,
    pub pegawai_nip: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpsertPegawaiUkuranRequest {
    pub pegawai_id: String,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
}

/// Report: Rekap Ukuran (Size summary)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LaporanRekapUkuran {
    pub pakaian_nama: String,
    pub ukuran_group: String,
    pub ukuran: String,
    pub jumlah_laki: i64,
    pub jumlah_perempuan: i64,
    pub jumlah_total: i64,
}

/// Report: Daftar Pegawai (Employee list)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LaporanDaftarPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_nama: String,
    pub jabatan: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>,
    pub eselon: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub with_hijab: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LaporanQuery {
    pub pengajuan_id: Option<String>,
    pub satker_id: Option<String>,
    pub jenis_pakaian_id: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}

// ============ PAKAIAN DINAS API Functions ============

// --- Jenis Pakaian Dinas ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_jenis_pakaian_dinas(
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<JenisPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/jenis?page={}&per_page={}",
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

    let result: PaginatedResponse<JenisPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_jenis_pakaian_dinas(
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<JenisPakaianDinas>, crate::api::AppError> {
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

/// `GET /kebutuhan-bmn/wilayah` — daftar wilayah Kejaksaan Tinggi (#19).
/// Dipakai bersama dgn Kebutuhan BMN; sumber `integrasi.mysimkari_satker`.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_wilayah_kejati() -> Result<Vec<String>, crate::api::AppError> {
    use crate::api::client::auth_get_json;
    let resp: ApiResponse<Vec<String>> =
        auth_get_json("/api/v1/perlengkapan/kebutuhan-bmn/wilayah").await?;
    Ok(resp.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_wilayah_kejati() -> Result<Vec<String>, crate::api::AppError> {
    Ok(vec![])
}

#[cfg(target_arch = "wasm32")]
pub async fn create_jenis_pakaian_dinas(
    request: CreateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/pakaian-dinas/jenis";
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

    let result: ApiResponse<JenisPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_jenis_pakaian_dinas(
    _request: CreateJenisPakaianDinasRequest,
) -> Result<ApiResponse<JenisPakaianDinas>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_jenis_pakaian_dinas(
    id: String,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/v1/perlengkapan/pakaian-dinas/jenis/{}", id);
    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(crate::api::AppError::network(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<()> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_jenis_pakaian_dinas(
    _id: String,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Spesifikasi ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_spesifikasi_pakaian(
    page: i32,
    per_page: i32,
    jenis_id: Option<String>,
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/spesifikasi?page={}&per_page={}",
        page, per_page
    );
    if let Some(jid) = jenis_id {
        url.push_str(&format!("&jenis_pakaian_dinas_id={}", jid));
    }

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

    let result: PaginatedResponse<SpesifikasiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_spesifikasi_pakaian(
    _page: i32,
    _per_page: i32,
    _jenis_id: Option<String>,
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, crate::api::AppError> {
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

// --- Sub-Spesifikasi ---

// --- Master Ukuran ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_master_ukuran(
    group: Option<String>,
) -> Result<ApiResponse<Vec<Ukuran>>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let mut url = "/api/v1/perlengkapan/pakaian-dinas/ukuran".to_string();
    if let Some(g) = group {
        url.push_str(&format!("?group={}", g));
    }

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

    let result: ApiResponse<Vec<Ukuran>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_master_ukuran(
    _group: Option<String>,
) -> Result<ApiResponse<Vec<Ukuran>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

// --- Pengajuan Pakaian Dinas ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_pakaian_dinas(
    page: i32,
    per_page: i32,
    tahun: Option<i32>,
) -> Result<PaginatedResponse<PengajuanPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/pengajuan?page={}&per_page={}",
        page, per_page
    );
    if let Some(t) = tahun {
        url.push_str(&format!("&tahun={}", t));
    }

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

    let result: PaginatedResponse<PengajuanPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_pakaian_dinas(
    _page: i32,
    _per_page: i32,
    _tahun: Option<i32>,
) -> Result<PaginatedResponse<PengajuanPakaianDinas>, crate::api::AppError> {
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
pub async fn create_pengajuan_pakaian_dinas(
    request: CreatePengajuanPakaianDinasRequest,
) -> Result<ApiResponse<PengajuanPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/pakaian-dinas/pengajuan";
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

    let result: ApiResponse<PengajuanPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_pengajuan_pakaian_dinas(
    _request: CreatePengajuanPakaianDinasRequest,
) -> Result<ApiResponse<PengajuanPakaianDinas>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_pengajuan_pakaian_dinas(
    id: String,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!("/api/v1/perlengkapan/pakaian-dinas/pengajuan/{}", id);
    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(crate::api::AppError::network(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<()> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_pengajuan_pakaian_dinas(
    _id: String,
) -> Result<ApiResponse<()>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Pengajuan Satker ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pengajuan_satker(
    pengajuan_id: String,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<PengajuanSatker>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/pengajuan/{}/satker?page={}&per_page={}",
        pengajuan_id, page, per_page
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

    let result: PaginatedResponse<PengajuanSatker> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pengajuan_satker(
    _pengajuan_id: String,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<PengajuanSatker>, crate::api::AppError> {
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

/// `GET /pakaian-dinas/satker/{id}/aktivitas` — per-satker workflow history (#40).
#[cfg(target_arch = "wasm32")]
pub async fn fetch_pakaian_satker_aktivitas(
    satker_id: String,
) -> Result<Vec<PengajuanSatkerAktivitas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/satker/{}/aktivitas",
        satker_id
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

    let result: ApiResponse<Vec<PengajuanSatkerAktivitas>> = resp.json().await?;
    Ok(result.data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pakaian_satker_aktivitas(
    _satker_id: String,
) -> Result<Vec<PengajuanSatkerAktivitas>, crate::api::AppError> {
    Ok(vec![])
}

#[cfg(target_arch = "wasm32")]
pub async fn process_validator_action(
    request: ValidatorActionRequest,
) -> Result<ApiResponse<PengajuanSatker>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/pakaian-dinas/validator-action";
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

    let result: ApiResponse<PengajuanSatker> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn process_validator_action(
    _request: ValidatorActionRequest,
) -> Result<ApiResponse<PengajuanSatker>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Pegawai Ukuran ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pegawai_ukuran(
    // Kept for QueryClient cache-keying. The backend resolves the pegawai
    // from the NIP claim inside the JWT, so the value is not sent on the
    // wire — but caching `()` would collide across users on shared devices.
    _pegawai_id: String,
) -> Result<ApiResponse<Option<PegawaiPakaianDinas>>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/pakaian-dinas/ukuran-pakaian-pegawai";

    let token = get_auth_token().ok_or_else(|| {
        crate::api::AppError::network("No authentication token found".to_string())
    })?;
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(crate::api::AppError::network(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    let result: ApiResponse<Option<PegawaiPakaianDinas>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_pegawai_ukuran(
    _pegawai_id: String,
) -> Result<ApiResponse<Option<PegawaiPakaianDinas>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: None,
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn upsert_pegawai_ukuran(
    request: UpsertPegawaiUkuranRequest,
) -> Result<ApiResponse<PegawaiPakaianDinas>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let url = "/api/v1/perlengkapan/pakaian-dinas/ukuran-pakaian-pegawai";
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

    let result: ApiResponse<PegawaiPakaianDinas> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_pegawai_ukuran(
    _request: UpsertPegawaiUkuranRequest,
) -> Result<ApiResponse<PegawaiPakaianDinas>, crate::api::AppError> {
    Err(crate::api::AppError::Unknown(
        "Server-side stub".to_string(),
    ))
}

// --- Pegawai by Satker (from MySIMKARI) ---

// --- Laporan (Reports) ---

#[cfg(target_arch = "wasm32")]
pub async fn fetch_laporan_rekap_ukuran(
    query: LaporanQuery,
) -> Result<ApiResponse<Vec<LaporanRekapUkuran>>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let mut url = "/api/v1/perlengkapan/pakaian-dinas/laporan/rekap-ukuran".to_string();
    let mut params = vec![];
    if let Some(ref pid) = query.pengajuan_id {
        params.push(format!("pengajuan_id={}", pid));
    }
    if let Some(ref sid) = query.satker_id {
        params.push(format!("satker_id={}", sid));
    }
    if let Some(ref jid) = query.jenis_pakaian_id {
        params.push(format!("jenis_pakaian_id={}", jid));
    }
    if let Some(ref jk) = query.jenis_kelamin {
        params.push(format!("jenis_kelamin={}", jk));
    }
    if let Some(ref e) = query.eselon {
        params.push(format!("eselon={}", e));
    }
    if let Some(ref j) = query.jenis {
        params.push(format!("jenis={}", j));
    }
    if !params.is_empty() {
        url.push_str(&format!("?{}", params.join("&")));
    }

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

    let result: ApiResponse<Vec<LaporanRekapUkuran>> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_laporan_rekap_ukuran(
    _query: LaporanQuery,
) -> Result<ApiResponse<Vec<LaporanRekapUkuran>>, crate::api::AppError> {
    Ok(ApiResponse {
        success: true,
        data: vec![],
        message: "Server-side stub".to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_laporan_daftar_pegawai(
    query: LaporanQuery,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResponse<LaporanDaftarPegawai>, crate::api::AppError> {
    use crate::api::client::get_auth_token;
    use gloo_net::http::Request;

    let mut url = format!(
        "/api/v1/perlengkapan/pakaian-dinas/laporan/daftar-pegawai?page={}&per_page={}",
        page, per_page
    );
    if let Some(ref pid) = query.pengajuan_id {
        url.push_str(&format!("&pengajuan_id={}", pid));
    }
    if let Some(ref sid) = query.satker_id {
        url.push_str(&format!("&satker_id={}", sid));
    }
    if let Some(ref jid) = query.jenis_pakaian_id {
        url.push_str(&format!("&jenis_pakaian_id={}", jid));
    }
    if let Some(ref jk) = query.jenis_kelamin {
        url.push_str(&format!("&jenis_kelamin={}", jk));
    }
    if let Some(ref e) = query.eselon {
        url.push_str(&format!("&eselon={}", e));
    }
    if let Some(ref j) = query.jenis {
        url.push_str(&format!("&jenis={}", j));
    }

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

    let result: PaginatedResponse<LaporanDaftarPegawai> = resp.json().await?;
    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_laporan_daftar_pegawai(
    _query: LaporanQuery,
    _page: i32,
    _per_page: i32,
) -> Result<PaginatedResponse<LaporanDaftarPegawai>, crate::api::AppError> {
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
