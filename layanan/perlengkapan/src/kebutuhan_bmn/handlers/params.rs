use serde::Deserialize;

use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::Claims;

use crate::kebutuhan_bmn::repository::UserInfo;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}
#[derive(Debug, Deserialize)]
pub struct PengajuanQueryParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub satker_id: Option<String>,
    pub search: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct BarangQueryParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub kode_barang: Option<String>,
    pub prioritas_min: Option<i32>,
    pub search: Option<String>,
}
fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    20
}
impl PaginationQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(bad_request("Page must be greater than 0"));
        }
        if self.page > 100_000 {
            return Err(bad_request("Page must be less than or equal to 100,000"));
        }
        if self.per_page < 1 {
            return Err(bad_request("Per page must be greater than 0"));
        }
        if self.per_page > 1000 {
            return Err(bad_request("Per page must be less than or equal to 1000"));
        }
        Ok(())
    }
}
// ============================================================================
// Helper Functions
// ============================================================================

/// Roles that take part in the Kebutuhan BMN workflow. The coarse gate for the
/// transition endpoints; the engine still enforces the per-state role.
///
/// `admin` is deliberately absent: the engine no longer lets an administrator
/// complete a state on behalf of the business role it names, so listing admin
/// here only produced a gate that passed and an engine that then refused.
pub(crate) const KEBUTUHAN_WORKFLOW_ROLES: &[&str] =
    &["operator_satker", "validator_wilayah", "validator_pusat"];

/// The role `claims` should act as for a move to status `target_code`: the
/// caller's own role that the workflow names for that state, else the primary.
///
/// A user holding several roles must be checked as the one that authorizes the
/// move — see [`Claims::acting_role`].
pub(crate) fn acting_role_for_target(claims: &Claims, target_code: i32) -> String {
    use crate::kebutuhan_bmn::models::KebutuhanBmnStatus;
    use crate::workflow::config::WorkflowConfig;
    match KebutuhanBmnStatus::from_code(target_code) {
        Some(status) => claims.role_for_transition(
            &WorkflowConfig::default_kebutuhan_bmn(),
            status.to_state_name(),
        ),
        None => claims.role.clone(),
    }
}

pub(crate) fn extract_user_info(claims: &Claims) -> UserInfo {
    UserInfo {
        nip: claims.nip.clone(),
        nama: claims.nama.clone(),
        pangkat: None,
        jabatan: claims.jabatan.clone(),
        role: Some(claims.role.clone()),
    }
}
/// POST /kebutuhan-bmn/pengajuan/:id/satker
/// Add a satker to pengajuan
#[derive(Debug, Deserialize)]
pub struct AddSatkerRequest {
    pub satker_id: String,
    pub satker_name: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct LaporanFormatQuery {
    pub format: Option<String>,
}
// ============================================================================
// SIMAN Integration Handlers
// ============================================================================

/// Query parameters for SIMAN asset search
#[derive(Debug, Deserialize)]
pub struct SimanSearchQuery {
    pub search: String,
    pub kategori: Option<String>,
    #[serde(default = "default_siman_limit")]
    pub limit: usize,
}
fn default_siman_limit() -> usize {
    20
}
// ============================================================================
// Search Handlers
// ============================================================================

/// Query parameters for advanced search
#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    /// Search term (required)
    pub q: String,

    /// Pagination
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,

    /// Filters
    /// MySIMKARI `kode_satker` (TEXT), not a surrogate id (V006/#94).
    pub satker_id: Option<String>,
    pub tahun_anggaran: Option<i32>,
    pub status: Option<String>, // Comma-separated list
    pub kode_barang: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,

    /// Sort options
    #[serde(default = "default_sort_field")]
    pub sort_by: String,
    #[serde(default = "default_sort_direction")]
    pub sort_dir: String,
}
fn default_sort_field() -> String {
    "relevance".to_string()
}
fn default_sort_direction() -> String {
    "desc".to_string()
}
/// GET /kebutuhan-bmn/search/suggestions
/// Get search suggestions based on partial query
#[derive(Debug, Deserialize)]
pub struct SuggestionsQuery {
    pub q: String,
    #[serde(default = "default_suggestions_limit")]
    pub limit: i32,
}
fn default_suggestions_limit() -> i32 {
    10
}

/// Query params for the Laporan Kebutuhan BMN recap (E-5).
/// `format` is only read by the export endpoint.
#[derive(Debug, Deserialize)]
pub struct RekapLaporanQueryParams {
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub format: Option<String>,
}
