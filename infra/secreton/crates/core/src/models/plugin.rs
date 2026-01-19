use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `PluginCatalogEntry`.
pub struct PluginCatalogEntry {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub artifact_path: String,
    pub pinned: bool,
    pub metadata: Option<serde_json::Value>,
}
