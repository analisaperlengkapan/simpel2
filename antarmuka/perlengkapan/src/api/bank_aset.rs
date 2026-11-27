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
    /// Tempat satker menurut MySIMKARI; kosong bila satker belum punya pasangan atau koordinat.
    #[serde(default)]
    pub lintang: Option<f64>,
    #[serde(default)]
    pub bujur: Option<f64>,
    #[serde(default)]
    pub provinsi: Option<String>,
    /// Wilayah (Kejati) satker ini: digit 6-9 kode SIMAN, dan nama Kejati-nya bila ada.
    #[serde(default)]
    pub wilayah_kode: Option<String>,
    #[serde(default)]
    pub wilayah_nama: Option<String>,
    /// Berapa aset satker ini yang membawa koordinat GPS dari SIMAN.
    #[serde(default)]
    pub aset_bergps: i64,
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
    /// SIMAN `kdsatker_keu`, exact. Comes from the Sebaran map's link, not from the dropdown.
    pub kode_siman: Option<String>,
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
    if let Some(v) = &filter.kode_siman {
        if !v.is_empty() {
            push_query(&mut url, "kode_siman", v);
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

/// Satu entri kodefikasi barang — cermin `KodefikasiBarang` di
/// `layanan/perlengkapan/src/bank_aset/models.rs`.
///
/// Field-nya sengaja tidak `Option`: keempatnya `NOT NULL` di
/// `integrasi.mv_kodefikasi_barang` (baris ber-kode atau ber-nama kosong
/// disaring saat snapshot dibentuk).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct KodefikasiBarang {
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah_aset: i64,
    pub jumlah_satker: i64,
}

/// `GET /bank-aset/kodefikasi?q=&limit=` — kodefikasi barang untuk pemilih.
///
/// Tidak ber-scope satker, dan itu disengaja: permohonan kebutuhan justru
/// paling sering menyebut barang yang satker-nya BELUM punya.
pub async fn fetch_kodefikasi(q: &str, limit: u32) -> AppResult<Vec<KodefikasiBarang>> {
    let mut url = format!("{API_BASE}/bank-aset/kodefikasi");
    let q = q.trim();
    if !q.is_empty() {
        push_query(&mut url, "q", q);
    }
    push_query(&mut url, "limit", &limit.to_string());
    let resp: ApiResponseWrap<Vec<KodefikasiBarang>> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn fetch_detail(id: &str) -> AppResult<BankAsetDetail> {
    let url = format!("{API_BASE}/bank-aset/{id}");
    let resp: ApiResponseWrap<BankAsetDetail> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

// `GET /bank-aset/lookup?nup=` tidak lagi dipanggil dari sini.
//
// Ia mencari aset dengan NUP saja, dan NUP saja bukan identitas: ia nomor urut
// DI DALAM satu satker untuk satu kode barang, jadi berulang — 24.715 aset
// ber-NUP "2" pada snapshot staging. Formulir pemakaian kini memakai pemilih
// yang mengembalikan baris asetnya utuh (kode satker + kode barang + NUP), jadi
// pembungkus ini beserta encoder kecilnya dibuang alih-alih dibungkam
// `#[allow(dead_code)]`. Endpoint backend-nya kini tanpa pemanggil frontend.

/// The dashboard drill-down, as the frontend holds it.
///
/// Every field NARROWS. None of them can widen: the server pushes the caller's
/// scope before these are applied, so naming a satker outside the caller's tier
/// answers with zeros rather than with that satker. That guarantee lives on the
/// server — this struct is only how the request says what it wants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct AsetFilterQuery {
    pub jenis: Option<String>,
    /// The satker's MySIMKARI `kode_satker` (not the SIMAN `kdsatker_keu`, and not the name: two
    /// satkers can share a name). The server maps it to SIMAN codes through the code map, and
    /// answers zeros for a code it does not know.
    pub satker_kode: Option<String>,
    /// The satker's SIMAN code (`kdsatker_keu`), exact: the one filter that names a single satker
    /// when two of them share a name (554 names for 556 codes on staging).
    pub kode_siman: Option<String>,
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
            ("kode_siman", &self.kode_siman),
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
    /// For a satker option: the region it sits in (a `wilayah` option's `value`). Lets the
    /// satker list be narrowed to a chosen region on the client, without another request.
    #[serde(default)]
    pub induk: Option<String>,
    /// Every region the satker holds assets in (`induk` is the smallest). One MySIMKARI satker can
    /// hold several SIMAN codes spread over regions, and the region filter selects by asset.
    #[serde(default)]
    pub induk_semua: Vec<String>,
}

impl FilterOption {
    /// Does this satker hold assets in `wilayah`?
    pub fn di_wilayah(&self, wilayah: &str) -> bool {
        self.induk.as_deref() == Some(wilayah) || self.induk_semua.iter().any(|w| w == wilayah)
    }

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

/// The three lists the dashboard's drill-down needs (jenis, satker by code with its region, and
/// wilayah), asked for ONCE and unfiltered: the satker list is narrowed to a chosen region on the
/// client through `induk`. The server computes each requested list with a scan of the asset table
/// (2-3 s for pusat), so asking only for these and asking once is what keeps a drill-down instant.
pub async fn fetch_filter_options_dasbor() -> AppResult<BankAsetFilterOptions> {
    let url = format!("{API_BASE}/bank-aset/filter-options?facet=jenis,satker_kode,wilayah");
    let resp: ApiResponseWrap<BankAsetFilterOptions> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drill_down_filter_carries_the_siman_code() {
        let by_code = AsetFilterQuery {
            satker_kode: Some("02.28".into()),
            ..Default::default()
        };
        assert_eq!(by_code.to_query(), "?satker_kode=02.28");
        // The SIMAN code names exactly one satker, even when two of them share a name.
        let exact = AsetFilterQuery {
            kode_siman: Some("006010500691160000KD".into()),
            ..Default::default()
        };
        assert_eq!(exact.to_query(), "?kode_siman=006010500691160000KD");
        assert_eq!(AsetFilterQuery::default().to_query(), "");
    }

    #[test]
    fn a_row_from_a_server_that_predates_the_map_still_decodes() {
        // rc52 answers without the place, the wilayah or the GPS count; the page must not break.
        let old = r#"{"kode_satker":"K","nama_satker":"S","total_aset":1,"nilai_perolehan":2.0,
                      "aset_baik":1,"aset_rusak":0}"#;
        let s: SebaranSatker = serde_json::from_str(old).expect("old shape");
        assert!(s.lintang.is_none() && s.wilayah_kode.is_none());
        assert_eq!(s.aset_bergps, 0);
    }
}
