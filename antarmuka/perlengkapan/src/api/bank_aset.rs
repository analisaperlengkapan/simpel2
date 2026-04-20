//! Frontend API for Bank Aset (unified SIMAN façade).

use serde::{Deserialize, Serialize};

use crate::api::client::{api_get, API_BASE};
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
    let per_page = if filter.per_page < 1 { 25 } else { filter.per_page };
    push_query(&mut url, "page", &page.to_string());
    push_query(&mut url, "per_page", &per_page.to_string());
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

pub async fn fetch_dashboard() -> AppResult<BankAsetDashboard> {
    let url = format!("{API_BASE}/bank-aset/dashboard");
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
