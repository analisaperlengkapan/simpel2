//! Dashboard API — the recap export.
//!
//! `fetch_dashboard_stats` used to live here, against `GET /dashboard/stats`.
//! That endpoint was a second aggregate over the same SIMAN table whose payload
//! was a strict subset of `GET /bank-aset/dashboard` — and unlike that one it
//! was never scoped, so it answered every operator with all 624 533 national
//! assets while the bank-aset card beside it correctly answered their own
//! 1 681. The home page now reads the scoped superset via
//! [`crate::api::bank_aset::fetch_dashboard`]; there is one definition again.

use super::client::api_get;
use serde::Deserialize;
use std::collections::HashMap;

/// Download the perlengkapan dashboard recap as `format` ("excel" | "pdf").
///
/// `tahun_anggaran` is REQUIRED by the endpoint (`DashboardParams.tahun_anggaran`
/// is a bare `i32`), so omitting it is a legitimate 400 — the caller always
/// passes one.
///
/// These two endpoints had no FE caller at all (#97), which is exactly why the
/// 500 they returned went unnoticed until an e2e probed them directly (#116).
#[cfg(target_arch = "wasm32")]
pub async fn export_dashboard(
    format: &str,
    tahun_anggaran: i32,
) -> Result<Vec<u8>, crate::api::AppError> {
    super::client::auth_get_binary(&format!(
        "{}/dashboard/perlengkapan/export/{format}?tahun_anggaran={tahun_anggaran}",
        super::client::API_BASE
    ))
    .await
}

// ============================================================================
// GET /dashboard/perlengkapan
// ============================================================================
//
// Mirrors `layanan/perlengkapan/src/dashboard/models.rs` FIELD FOR FIELD. That
// is not politeness: a DTO here that names a field the backend never sends
// makes serde reject the whole body, and the request still carried HTTP 200, so
// the failure has no status code to catch (#820 — the pakaian-dinas period
// dropdown died this exact way for weeks). If the backend struct changes, this
// one changes with it.
//
// Two shapes worth stating out loud:
//   * this endpoint answers BARE — no `{success, data}` envelope, unlike every
//     other endpoint in the service. An integration test pins that.
//   * `total_by_tahun` is a `HashMap<i32, i64>` server-side, and JSON object
//     keys are strings. It is mirrored as `HashMap<String, i64>` and parsed at
//     the point of use rather than trusting a numeric-key coercion.

#[derive(Clone, Debug, Deserialize)]
pub struct SatkerCount {
    pub satker_id: String,
    pub satker_nama: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KebutuhanMetrics {
    #[serde(default)]
    pub total_by_status: HashMap<String, i64>,
    #[serde(default)]
    pub total_by_satker: Vec<SatkerCount>,
    #[serde(default)]
    pub total_by_tahun: HashMap<String, i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PakaianDinasMetrics {
    #[serde(default)]
    pub total_by_jenis: HashMap<String, i64>,
    #[serde(default)]
    pub total_by_ukuran: HashMap<String, i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub average_time_hours: f64,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkflowMetrics {
    pub average_processing_time_hours: f64,
    #[serde(default)]
    pub bottlenecks: Vec<BottleneckInfo>,
    pub sla_breaches_today: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModuleStatusMetrics {
    #[serde(default)]
    pub total_by_status: HashMap<String, i64>,
    pub total: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PerlengkapanDashboardMetrics {
    pub kebutuhan_metrics: KebutuhanMetrics,
    #[serde(default)]
    pub gap_analysis: Vec<GapAnalysisResult>,
    pub pakaian_dinas_metrics: PakaianDinasMetrics,
    pub workflow_metrics: WorkflowMetrics,
    // `asset_utilization` is deliberately NOT mirrored. It is a third count of
    // "how many SIMAN assets, how many in good condition" — the same question
    // `/bank-aset/dashboard` answers for the condition cards above. Rendering
    // both would put two independently-computed answers to one question on one
    // screen, which is the fault this page was consolidated to remove. serde
    // ignores unmodelled fields, so the payload stays valid.
    pub pemakaian_metrics: ModuleStatusMetrics,
    pub penghapusan_metrics: ModuleStatusMetrics,
}

/// Fetch the scoped perlengkapan metrics for one budget year.
///
/// Scoped server-side per tier — pusat sees the country, a wilayah validator
/// their region, a satker user their own satker — so nothing here needs
/// filtering client-side, and nothing here may be assumed nationwide.
pub async fn fetch_perlengkapan_metrics(
    tahun_anggaran: i32,
) -> Result<PerlengkapanDashboardMetrics, crate::api::AppError> {
    api_get(&format!(
        "/dashboard/perlengkapan?tahun_anggaran={tahun_anggaran}"
    ))
    .await
}
