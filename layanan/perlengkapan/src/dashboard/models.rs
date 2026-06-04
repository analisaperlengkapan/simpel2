// Dashboard data models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Query parameters for dashboard metrics
#[derive(Debug, Deserialize)]
pub struct DashboardParams {
    pub tahun_anggaran: i32,
}

/// Complete perlengkapan dashboard metrics
#[derive(Debug, Serialize)]
pub struct PerlengkapanDashboardMetrics {
    pub kebutuhan_metrics: KebutuhanMetrics,
    pub gap_analysis: Vec<GapAnalysisResult>,
    pub pakaian_dinas_metrics: PakaianDinasMetrics,
    pub workflow_metrics: WorkflowMetrics,
    pub asset_utilization: AssetUtilization,
    /// Rekap status Pemakaian BMN (Fase 2.7) — melengkapi cakupan 4 modul.
    pub pemakaian_metrics: ModuleStatusMetrics,
    /// Rekap status Usulan SK Penghapusan BMN (Fase 2.7).
    pub penghapusan_metrics: ModuleStatusMetrics,
}

/// Rekap jumlah per status untuk satu modul workflow (Fase 2.7).
#[derive(Debug, Serialize)]
pub struct ModuleStatusMetrics {
    pub total_by_status: HashMap<String, i64>,
    pub total: i64,
}

/// Kebutuhan BMN metrics
#[derive(Debug, Serialize)]
pub struct KebutuhanMetrics {
    pub total_by_status: HashMap<String, i64>,
    pub total_by_satker: Vec<SatkerCount>,
    pub total_by_tahun: HashMap<i32, i64>,
}

/// Satker count for kebutuhan
#[derive(Debug, Serialize)]
pub struct SatkerCount {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub count: i64,
}

/// Gap analysis result
#[derive(Debug, Serialize, Clone)]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
}

/// Pakaian dinas metrics
#[derive(Debug, Serialize)]
pub struct PakaianDinasMetrics {
    pub total_by_jenis: HashMap<String, i64>,
    pub total_by_ukuran: HashMap<String, i64>,
}

/// Workflow performance metrics
#[derive(Debug, Serialize)]
pub struct WorkflowMetrics {
    pub average_processing_time_hours: f64,
    pub bottlenecks: Vec<BottleneckInfo>,
    pub sla_breaches_today: i64,
}

/// Bottleneck information
#[derive(Debug, Serialize)]
pub struct BottleneckInfo {
    pub state: String,
    pub average_time_hours: f64,
    pub count: i64,
}

/// Asset utilization from SIMAN
#[derive(Debug, Serialize)]
pub struct AssetUtilization {
    pub total_assets: i64,
    pub assets_in_good_condition: i64,
    pub utilization_percentage: f64,
}

// ============ Summary dashboard (`/dashboard/stats`) ============
//
// Lightweight SIMAN-summary card for the landing dashboard, distinct from the
// richer `PerlengkapanDashboardMetrics` above. Sourced from the
// `integrasi.v_siman_summary_*` views.

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}
