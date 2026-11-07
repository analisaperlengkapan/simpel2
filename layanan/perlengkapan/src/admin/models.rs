//! # Admin API Models
//!
//! Types for the master data hub. Mirrors the contract expected by
//! `antarmuka/perlengkapan/src/api/admin.rs`. (The audit log viewer reads the
//! cross-module trail at `/audit`; see `crate::audit`.)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterSource {
    pub key: String,
    pub label: String,
    pub description: String,
    pub icon: Option<String>,
    pub record_count: i64,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterRecord {
    pub id: String,
    pub code: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub extra: serde_json::Value,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterUpsertRequest {
    #[serde(default)]
    pub code: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MasterListQuery {
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
    #[serde(default)]
    pub q: Option<String>,
}
