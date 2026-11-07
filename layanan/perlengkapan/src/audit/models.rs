//! # Cross-module audit trail — API models
//!
//! Types for the BPK-ready audit reader at `GET /audit`. This reads the
//! canonical cross-module sink `perlengkapan.audit_log` (written by every
//! module via [`crate::contracts::AuditSink`]).

use serde::{Deserialize, Serialize};

/// One row from `perlengkapan.audit_log`, flattened for the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    pub id: String,
    pub occurred_at: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub actor_ip: Option<String>,
    /// Stable action verb (create/approve/reject/submit/export/custom/…).
    pub action: String,
    /// Free-form name when `action == custom` (e.g. "workflow.transition").
    #[serde(default)]
    pub action_name: Option<String>,
    /// Resource kind ("kebutuhan_bmn", "penghapusan_bmn", "tiket", …).
    pub resource_type: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    pub module: String,
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Statutory retention horizon (V037).
    #[serde(default)]
    pub retention_until: Option<String>,
}

/// Query parameters for `GET /audit`. Every field is optional; combine them
/// to narrow the trail. `entity` aliases `resource_type` to match the DoD's
/// `GET /audit?entity=...` shape.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditTrailQuery {
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
    /// Resource type filter (DoD spelling).
    #[serde(default)]
    pub entity: Option<String>,
    /// Pin to a single resource instance — the per-entity full trail.
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    /// Actor: UUID (exact) or name fragment (ILIKE).
    #[serde(default)]
    pub actor: Option<String>,
    /// RFC3339 lower bound (inclusive) on occurred_at.
    #[serde(default)]
    pub from: Option<String>,
    /// RFC3339 upper bound (inclusive) on occurred_at.
    #[serde(default)]
    pub to: Option<String>,
    /// Free-text search over message / action_name / actor_username.
    #[serde(default)]
    pub q: Option<String>,
}

/// The values that actually occur in the trail, for filter dropdowns
/// (`GET /audit/facets`).
///
/// The admin page used to ship these lists hand-written, and they described a
/// system that was never audited (`bank_aset`, `master_data`, `login`, ...)
/// while omitting the ones that were (`tiket`, `pengajuan_satker`). Deriving the
/// options from the table means a dropdown can only offer a value that matches
/// at least one row, and a newly audited module appears without a frontend
/// change.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuditFacets {
    /// `module` values, sorted.
    pub modules: Vec<String>,
    /// `resource_type` values, sorted (the `entity` filter of `GET /audit`).
    pub entities: Vec<String>,
    /// `action` verbs, sorted.
    pub actions: Vec<String>,
}
