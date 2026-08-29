//! Frontend API for Bank Aset (unified SIMAN façade).

use serde::{Deserialize, Serialize};

use crate::api::client::{API_BASE, api_get};
use crate::api::common::PaginatedResponse;
use crate::api::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAsetItem {
    pub id: String,
    pub kategori_aset: String,
    pub no_aset: String,
    pub nama_aset: Option<String>,
    pub kode_barang: Option<String>,
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>,
    pub lokasi: Option<String>,
    pub satker: Option<String>,
    pub kode_satker: Option<String>,
    pub nup: Option<String>,
    pub nilai_perolehan: Option<f64>,
    pub tgl_perolehan: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiwayatEntry {
    pub id: String,
    pub ref_no: Option<String>,
    pub status: Option<String>,
    pub deskripsi: Option<String>,
    pub tanggal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAsetDetail {
    #[serde(flatten)]
    pub item: BankAsetItem,
    #[serde(default)]
    pub riwayat_pemakaian: Vec<RiwayatEntry>,
    #[serde(default)]
    pub riwayat_penghapusan: Vec<RiwayatEntry>,
    #[serde(default)]
    pub riwayat_kebutuhan: Vec<RiwayatEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KondisiStat {
    pub kondisi: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KategoriStat {
    pub kategori: String,
    pub count: i64,
    pub nilai: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SatkerStat {
    pub satker: String,
    pub count: i64,
    pub nilai: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TahunStat {
    pub tahun: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAsetDashboard {
    pub total_aset: i64,
    pub total_nilai_perolehan: f64,
    pub total_satker: i64,
    pub total_kategori: i64,
    #[serde(default)]
    pub kondisi_breakdown: Vec<KondisiStat>,
    #[serde(default)]
    pub kategori_breakdown: Vec<KategoriStat>,
    #[serde(default)]
    pub top_satker: Vec<SatkerStat>,
    #[serde(default)]
    pub per_tahun: Vec<TahunStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SebaranSatker {
    pub kode_satker: Option<String>,
    pub nama_satker: String,
    pub total_aset: i64,
    pub nilai_perolehan: f64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAsetSebaran {
    #[serde(default)]
    pub satker: Vec<SebaranSatker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LastSyncInfo {
    pub last_sync_at: Option<String>,
    pub total_aset: i64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponseWrap<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

#[derive(Default, Debug, Clone)]
pub struct ListFilter {
    pub page: i32,
    pub per_page: i32,
    pub jenis: Option<String>,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

fn push_query(buf: &mut String, key: &str, value: &str) {
    if buf.contains('?') {
        buf.push('&');
    } else {
        buf.push('?');
    }
    buf.push_str(key);
    buf.push('=');
    buf.push_str(&urlencoding::encode(value));
}

pub async fn fetch_list(filter: &ListFilter) -> AppResult<PaginatedResponse<BankAsetItem>> {
    let mut url = format!("{API_BASE}/bank-aset");
    let page = if filter.page < 1 { 1 } else { filter.page };
    let per_page = if filter.per_page < 1 {
        25
    } else {
        filter.per_page
    };
    push_query(&mut url, "page", &page.to_string());
    push_query(&mut url, "per_page", &per_page.to_string());
    if let Some(v) = &filter.jenis {
        if !v.is_empty() {
            push_query(&mut url, "jenis", v);
        }
    }
    if let Some(v) = &filter.kategori {
        if !v.is_empty() {
            push_query(&mut url, "kategori", v);
        }
    }
    if let Some(v) = &filter.kondisi {
        if !v.is_empty() {
            push_query(&mut url, "kondisi", v);
        }
    }
    if let Some(v) = &filter.satker {
        if !v.is_empty() {
            push_query(&mut url, "satker", v);
        }
    }
    if let Some(v) = &filter.search {
        if !v.is_empty() {
            push_query(&mut url, "search", v);
        }
    }
    if let Some(v) = &filter.sort {
        if !v.is_empty() {
            push_query(&mut url, "sort", v);
        }
    }
    api_get::<PaginatedResponse<BankAsetItem>>(&url).await
}

pub async fn fetch_detail(id: &str) -> AppResult<BankAsetDetail> {
    let url = format!("{API_BASE}/bank-aset/{id}");
    let resp: ApiResponseWrap<BankAsetDetail> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// Slim BMN lookup by NUP — the subset of the `GET /bank-aset/lookup?nup=`
/// payload the pemakaian-bmn form auto-fills (serde ignores the rest).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BankAsetLookup {
    pub kode_barang: Option<String>,
    pub nama_barang: Option<String>,
    pub merk: Option<String>,
    pub tahun_perolehan: Option<String>,
}

/// Returns `Ok(Some(_))` on a hit, `Ok(None)` on a 404 (NUP not in
/// `integrasi.siman_aset`), and `Err(_)` on a transport / server error so
/// callers can surface the right UX.
pub async fn lookup_by_nup(nup: &str) -> AppResult<Option<BankAsetLookup>> {
    let encoded = urlencoding_simple(nup);
    let url = format!("{API_BASE}/bank-aset/lookup?nup={}", encoded);
    let resp: ApiResponseWrap<BankAsetLookup> = match api_get(&url).await {
        Ok(r) => r,
        Err(AppError::NotFound(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(Some(resp.data))
}

/// Minimal URL-encoder for the NUP query parameter — covers the characters
/// present in a real NUP (digits, dashes) plus the small set of safe
/// fallbacks. Avoids pulling in a heavyweight URL crate.
fn urlencoding_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            for byte in ch.to_string().as_bytes() {
                out.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    out
}

/// The dashboard drill-down, as the frontend holds it.
///
/// Every field NARROWS. None of them can widen: the server pushes the caller's
/// scope before these are applied, so naming a satker outside the caller's tier
/// answers with zeros rather than with that satker. That guarantee lives on the
/// server — this struct is only how the request says what it wants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct AsetFilterQuery {
    pub jenis: Option<String>,
    /// `kdsatker_keu`, not the name: two satkers can share a name.
    pub satker_kode: Option<String>,
    /// SIMAN `wilayah_kode` (digits 6-9 of `kdsatker_keu`).
    pub wilayah: Option<String>,
    pub tgl_from: Option<String>,
    pub tgl_to: Option<String>,
}

impl AsetFilterQuery {
    /// `""` when nothing is set, else a leading-`?` query string.
    pub fn to_query(&self) -> String {
        let mut buf = String::new();
        for (k, v) in [
            ("jenis", &self.jenis),
            ("satker_kode", &self.satker_kode),
            ("wilayah", &self.wilayah),
            ("tgl_from", &self.tgl_from),
            ("tgl_to", &self.tgl_to),
        ] {
            if let Some(v) = v.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                push_query(&mut buf, k, v);
            }
        }
        buf
    }
}

pub async fn fetch_dashboard(filter: &AsetFilterQuery) -> AppResult<BankAsetDashboard> {
    let url = format!("{API_BASE}/bank-aset/dashboard{}", filter.to_query());
    let resp: ApiResponseWrap<BankAsetDashboard> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn fetch_sebaran() -> AppResult<BankAsetSebaran> {
    let url = format!("{API_BASE}/bank-aset/sebaran");
    let resp: ApiResponseWrap<BankAsetSebaran> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn fetch_last_sync() -> AppResult<LastSyncInfo> {
    let url = format!("{API_BASE}/bank-aset/last-sync");
    let resp: ApiResponseWrap<LastSyncInfo> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FilterOption {
    pub value: String,
    pub count: i64,
    /// Human label when `value` is a code (satker_kode, wilayah). Absent when
    /// the value IS the label, so this mirrors the backend's
    /// `skip_serializing_if`.
    #[serde(default)]
    pub label: Option<String>,
}

impl FilterOption {
    /// What to put in the dropdown: the label when there is one, else the value.
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.value)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BankAsetFilterOptions {
    pub jenis: Vec<FilterOption>,
    pub kategori: Vec<FilterOption>,
    pub kondisi: Vec<FilterOption>,
    /// Satker by NAME — what the Bank Aset list filters on. Two satkers can
    /// share a name, so the dashboard drill-down uses `satker_kode` instead.
    pub satker: Vec<FilterOption>,
    /// Satker by CODE (`kdsatker_keu`), with the name as label.
    #[serde(default)]
    pub satker_kode: Vec<FilterOption>,
    /// Region by SIMAN `wilayah_kode`, with the Kejati name as label.
    #[serde(default)]
    pub wilayah: Vec<FilterOption>,
}

/// Distinct filter values (jenis BMN, kategori, kondisi, satker) for populating
/// the filter dropdowns dynamically from real SIMAN data.
pub async fn fetch_filter_options(filter: &AsetFilterQuery) -> AppResult<BankAsetFilterOptions> {
    // Narrowed by the same filter the data is: selecting a region should leave
    // the satker list holding that region's satkers, not all 554.
    let url = format!("{API_BASE}/bank-aset/filter-options{}", filter.to_query());
    let resp: ApiResponseWrap<BankAsetFilterOptions> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}
