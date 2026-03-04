//! # SIMAN Integration for Kebutuhan BMN
//!
//! Integration service for fetching existing BMN/asset data from SIMAN
//! (Sistem Informasi Manajemen Aset Negara) to support feasibility analysis.
//!
//! ## Architecture
//! This module provides a clean interface between the kebutuhan_bmn service
//! and the layanan-integrasi SIMAN API client.
//!
//! ## Usage
//! ```rust,ignore
//! let siman = SimanIntegration::new(client);
//! let assets = siman.get_existing_assets("001.01.06", "Laptop").await?;
//! ```

use layanan_perlengkapan_integrasi::MonsaktiClient;
use layanan_perlengkapan_integrasi::{SimanAssetCategory, get_aset_by_category, get_row_count};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use crate::errors::{AppError, AppResult};

/// Information about an existing asset from SIMAN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimanAsset {
    /// Asset registration number
    pub no_aset: String,
    /// Asset name/description
    pub nama_aset: String,
    /// NUP (Nomor Urut Pendaftaran)
    pub nup: Option<String>,
    /// Asset condition (Baik, Rusak Ringan, Rusak Berat)
    pub kondisi: String,
    /// Acquisition year
    pub tahun_perolehan: Option<i32>,
    /// Acquisition value
    pub nilai_perolehan: Option<f64>,
    /// Current book value
    pub nilai_buku: Option<f64>,
    /// Location description
    pub lokasi: Option<String>,
    /// Asset category
    pub kategori: String,
    /// Satker ID (BA_KEY)
    pub satker_id: String,
    /// Additional metadata
    pub metadata: Option<Value>,
}

/// Summary of existing assets for a satker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerAssetSummary {
    pub satker_id: String,
    pub satker_name: Option<String>,
    pub total_assets: i64,
    pub total_value: f64,
    pub by_category: Vec<CategoryAssetCount>,
    pub by_condition: Vec<ConditionAssetCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAssetCount {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionAssetCount {
    pub condition: String,
    pub count: i64,
}

/// SIMAN integration service
#[derive(Clone)]
pub struct SimanIntegration {
    client: Arc<RwLock<MonsaktiClient>>,
}

impl SimanIntegration {
    /// Create a new SIMAN integration service
    pub fn new(client: MonsaktiClient) -> Self {
        Self {
            client: Arc::new(RwLock::new(client)),
        }
    }

    /// Get row count for a specific asset category
    pub async fn get_asset_count(&self, category: SimanAssetCategory) -> AppResult<i64> {
        let mut client = self.client.write().await;
        get_row_count(&mut client, category, None)
            .await
            .map_err(|e| AppError::Internal(format!("SIMAN error: {}", e)))
    }

    /// Get existing assets for a satker by category with pagination
    pub async fn get_assets_by_category(
        &self,
        category: SimanAssetCategory,
        start_id: u32,
        end_id: u32,
    ) -> AppResult<Vec<SimanAsset>> {
        let mut client = self.client.write().await;
        let data = get_aset_by_category(&mut client, category, start_id, end_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("SIMAN error: {}", e)))?;

        // Parse JSON values to SimanAsset
        let assets = data
            .into_iter()
            .filter_map(|v| self.parse_asset_value(v, category))
            .collect();

        Ok(assets)
    }

    /// Search for similar assets by name/keyword within a category
    pub async fn search_similar_assets(
        &self,
        category: SimanAssetCategory,
        search_term: &str,
        limit: usize,
    ) -> AppResult<Vec<SimanAsset>> {
        // First get total count
        let total = self.get_asset_count(category).await?;
        if total == 0 {
            return Ok(vec![]);
        }

        // Fetch in batches and filter
        let batch_size = 500u32;
        let mut found_assets = Vec::new();
        let search_lower = search_term.to_lowercase();

        let mut start = 1u32;
        while start <= total as u32 && found_assets.len() < limit {
            let end = (start + batch_size - 1).min(total as u32);

            let assets = self.get_assets_by_category(category, start, end).await?;

            for asset in assets {
                if asset.nama_aset.to_lowercase().contains(&search_lower) {
                    found_assets.push(asset);
                    if found_assets.len() >= limit {
                        break;
                    }
                }
            }

            start = end + 1;
        }

        Ok(found_assets)
    }

    /// Get asset summary for a satker across all categories
    pub async fn get_satker_asset_summary(
        &self,
        _satker_id: &str,
    ) -> AppResult<SatkerAssetSummary> {
        // Note: SIMAN API uses BA_KEY from config, not per-satker query
        // This is a limitation of the current SIMAN API structure
        // For now, we return aggregate counts for the configured BA_KEY

        let mut by_category = Vec::new();
        let mut total_assets = 0i64;

        for category in SimanAssetCategory::all() {
            match self.get_asset_count(category).await {
                Ok(count) => {
                    total_assets += count;
                    by_category.push(CategoryAssetCount {
                        category: category.display_name().to_string(),
                        count,
                        value: 0.0, // Would need to sum from actual data
                    });
                }
                Err(e) => {
                    warn!(
                        "Failed to get count for category {}: {}",
                        category.display_name(),
                        e
                    );
                }
            }
        }

        Ok(SatkerAssetSummary {
            satker_id: _satker_id.to_string(),
            satker_name: None,
            total_assets,
            total_value: 0.0,
            by_category,
            by_condition: vec![],
        })
    }

    /// Get existing inventory count for a specific barang type
    /// Used for feasibility analysis
    pub async fn get_existing_inventory_count(
        &self,
        barang_name: &str,
        kategori_aset: Option<&str>,
    ) -> AppResult<i64> {
        // Determine category from kategori_aset or infer from name
        let category = kategori_aset
            .and_then(SimanAssetCategory::from_str)
            .unwrap_or_else(|| self.infer_category(barang_name));

        // Search for similar assets
        let similar = self
            .search_similar_assets(category, barang_name, 1000)
            .await?;

        Ok(similar.len() as i64)
    }

    /// Get detailed existing assets matching a barang name
    /// Returns assets that can be used for gap analysis
    pub async fn get_matching_assets(
        &self,
        barang_name: &str,
        kategori_aset: Option<&str>,
        limit: usize,
    ) -> AppResult<Vec<SimanAsset>> {
        let category = kategori_aset
            .and_then(SimanAssetCategory::from_str)
            .unwrap_or_else(|| self.infer_category(barang_name));

        self.search_similar_assets(category, barang_name, limit)
            .await
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

    /// Parse JSON value from SIMAN API to SimanAsset struct
    fn parse_asset_value(&self, value: Value, category: SimanAssetCategory) -> Option<SimanAsset> {
        let obj = value.as_object()?;

        // Common field mappings (SIMAN uses various field names)
        let no_aset = self
            .extract_string(obj, &["NO_ASET", "NO_REG", "KODE_BARANG", "no_aset"])
            .unwrap_or_else(|| "N/A".to_string());

        let nama_aset = self
            .extract_string(obj, &["NAMA_ASET", "URAI_BARANG", "nama_aset", "NAMA"])
            .unwrap_or_else(|| "Unknown".to_string());

        let nup = self.extract_string(obj, &["NUP", "nup"]);

        let kondisi = self
            .extract_string(obj, &["KONDISI", "kondisi"])
            .unwrap_or_else(|| "Tidak Diketahui".to_string());

        let tahun_perolehan = self
            .extract_number(
                obj,
                &["TAHUN_PEROLEHAN", "TGL_PEROLEHAN", "tahun_perolehan"],
            )
            .map(|n| n as i32);

        let nilai_perolehan =
            self.extract_float(obj, &["NILAI_PEROLEHAN", "HARGA", "nilai_perolehan"]);

        let nilai_buku = self.extract_float(obj, &["NILAI_BUKU", "nilai_buku"]);

        let lokasi = self.extract_string(obj, &["LOKASI", "ALAMAT", "lokasi"]);

        let satker_id = self
            .extract_string(obj, &["BA_KEY", "SATKER_ID", "ba_key"])
            .unwrap_or_default();

        Some(SimanAsset {
            no_aset,
            nama_aset,
            nup,
            kondisi,
            tahun_perolehan,
            nilai_perolehan,
            nilai_buku,
            lokasi,
            kategori: category.display_name().to_string(),
            satker_id,
            metadata: Some(value),
        })
    }

    /// Extract string from object with multiple possible field names
    fn extract_string(
        &self,
        obj: &serde_json::Map<String, Value>,
        keys: &[&str],
    ) -> Option<String> {
        for key in keys {
            if let Some(Value::String(s)) = obj.get(*key) {
                return Some(s.clone());
            }
        }
        None
    }

    /// Extract number from object with multiple possible field names
    fn extract_number(&self, obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<i64> {
        for key in keys {
            if let Some(v) = obj.get(*key) {
                if let Some(n) = v.as_i64() {
                    return Some(n);
                }
                if let Some(n) = v.as_f64() {
                    return Some(n as i64);
                }
                if let Some(s) = v.as_str()
                    && let Ok(n) = s.parse::<i64>() {
                        return Some(n);
                    }
            }
        }
        None
    }

    /// Extract float from object with multiple possible field names
    fn extract_float(&self, obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<f64> {
        for key in keys {
            if let Some(v) = obj.get(*key) {
                if let Some(n) = v.as_f64() {
                    return Some(n);
                }
                if let Some(n) = v.as_i64() {
                    return Some(n as f64);
                }
                if let Some(s) = v.as_str()
                    && let Ok(n) = s.parse::<f64>() {
                        return Some(n);
                    }
            }
        }
        None
    }

    /// Infer asset category from barang name
    fn infer_category(&self, barang_name: &str) -> SimanAssetCategory {
        let name_lower = barang_name.to_lowercase();

        // TIK-related
        if name_lower.contains("laptop")
            || name_lower.contains("komputer")
            || name_lower.contains("printer")
            || name_lower.contains("server")
            || name_lower.contains("monitor")
            || name_lower.contains("cpu")
            || name_lower.contains("keyboard")
            || name_lower.contains("mouse")
            || name_lower.contains("scanner")
            || name_lower.contains("projector")
            || name_lower.contains("ups")
            || name_lower.contains("router")
            || name_lower.contains("switch")
        {
            return SimanAssetCategory::KhususTIK;
        }

        // Vehicles
        if name_lower.contains("mobil")
            || name_lower.contains("motor")
            || name_lower.contains("kendaraan")
            || name_lower.contains("sepeda")
            || name_lower.contains("truk")
            || name_lower.contains("bus")
        {
            return SimanAssetCategory::AngkutanBermotor;
        }

        // Furniture / Non-TIK equipment
        if name_lower.contains("meja")
            || name_lower.contains("kursi")
            || name_lower.contains("lemari")
            || name_lower.contains("filing")
            || name_lower.contains("sofa")
            || name_lower.contains("rak")
            || name_lower.contains("ac")
            || name_lower.contains("kipas")
            || name_lower.contains("dispenser")
        {
            return SimanAssetCategory::NonTIK;
        }

        // Buildings
        if name_lower.contains("gedung")
            || name_lower.contains("bangunan")
            || name_lower.contains("kantor")
            || name_lower.contains("ruko")
        {
            return SimanAssetCategory::GedungBangunan;
        }

        // Land
        if name_lower.contains("tanah") || name_lower.contains("lahan") {
            return SimanAssetCategory::Tanah;
        }

        // Default to Non-TIK for general equipment
        SimanAssetCategory::NonTIK
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infer_category_tik() {
        let client = layanan_perlengkapan_integrasi::MonsaktiClient::new(
            layanan_perlengkapan_integrasi::Config::default(),
        )
        .await
        .unwrap();
        let integration = SimanIntegration::new(client);

        assert_eq!(
            integration.infer_category("Laptop Dell Latitude"),
            SimanAssetCategory::KhususTIK
        );
        assert_eq!(
            integration.infer_category("Komputer Desktop HP"),
            SimanAssetCategory::KhususTIK
        );
        assert_eq!(
            integration.infer_category("Printer Canon LBP"),
            SimanAssetCategory::KhususTIK
        );
    }

    #[tokio::test]
    async fn test_infer_category_vehicle() {
        let client = layanan_perlengkapan_integrasi::MonsaktiClient::new(
            layanan_perlengkapan_integrasi::Config::default(),
        )
        .await
        .unwrap();
        let integration = SimanIntegration::new(client);

        assert_eq!(
            integration.infer_category("Mobil Toyota Avanza"),
            SimanAssetCategory::AngkutanBermotor
        );
        assert_eq!(
            integration.infer_category("Sepeda Motor Honda"),
            SimanAssetCategory::AngkutanBermotor
        );
    }

    #[tokio::test]
    async fn test_infer_category_non_tik() {
        let client = layanan_perlengkapan_integrasi::MonsaktiClient::new(
            layanan_perlengkapan_integrasi::Config::default(),
        )
        .await
        .unwrap();
        let integration = SimanIntegration::new(client);

        assert_eq!(
            integration.infer_category("Meja Kerja"),
            SimanAssetCategory::NonTIK
        );
        assert_eq!(
            integration.infer_category("Kursi Kantor"),
            SimanAssetCategory::NonTIK
        );
        assert_eq!(
            integration.infer_category("AC Split Daikin"),
            SimanAssetCategory::NonTIK
        );
    }
}
