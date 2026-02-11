//! SIMAN Data Transformation Layer
//!
//! This module handles transformation of SIMAN API responses to internal format.
//! It provides:
//! - Data validation
//! - Format mapping from SIMAN to internal schema
//! - Raw data preservation for debugging
//! - Error handling for malformed data

use crate::error::MonsaktiError;
use crate::siman::models::SimanAssetCategory;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

/// Internal asset representation after transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedAsset {
    /// Internal UUID (generated)
    pub id: Uuid,
    /// Asset category
    pub kategori_aset: String,
    /// NUP (Nomor Urut Pendaftaran)
    pub nup: Option<String>,
    /// Kode barang
    pub kode_barang: Option<String>,
    /// Nama barang
    pub nama_barang: Option<String>,
    /// Kondisi (BAIK, RUSAK RINGAN, RUSAK BERAT)
    pub kondisi: Option<String>,
    /// Tahun perolehan
    pub tahun_perolehan: Option<i32>,
    /// Nilai perolehan
    pub nilai_perolehan: Option<f64>,
    /// Satker code
    pub satker_code: Option<String>,
    /// Raw data from SIMAN API (for debugging)
    pub raw_data: Value,
    /// Validation errors (if any)
    pub validation_errors: Vec<String>,
}

impl TransformedAsset {
    /// Create a new transformed asset
    pub fn new(category: SimanAssetCategory, raw_data: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            kategori_aset: category.description().to_string(),
            nup: None,
            kode_barang: None,
            nama_barang: None,
            kondisi: None,
            tahun_perolehan: None,
            nilai_perolehan: None,
            satker_code: None,
            raw_data,
            validation_errors: Vec::new(),
        }
    }

    /// Add a validation error
    pub fn add_validation_error(&mut self, error: String) {
        self.validation_errors.push(error);
    }

    /// Check if the asset has validation errors
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Check if the asset is valid (no critical errors)
    pub fn is_valid(&self) -> bool {
        // Asset is valid if it has at least NUP or kode_barang
        self.nup.is_some() || self.kode_barang.is_some()
    }
}

/// Data transformer for SIMAN assets
pub struct SimanTransformer;

impl SimanTransformer {
    /// Transform SIMAN API response to internal format
    ///
    /// # Arguments
    /// * `category` - Asset category
    /// * `raw_data` - Raw JSON data from SIMAN API
    ///
    /// # Returns
    /// Transformed asset with validation
    pub fn transform(
        category: SimanAssetCategory,
        raw_data: Value,
    ) -> Result<TransformedAsset, MonsaktiError> {
        let mut asset = TransformedAsset::new(category, raw_data.clone());

        // Extract fields based on common SIMAN response structure
        if let Some(obj) = raw_data.as_object() {
            // NUP - various possible field names
            asset.nup = Self::extract_string(obj, &["NUP", "nup", "nomor_urut_pendaftaran"]);

            // Kode barang
            asset.kode_barang = Self::extract_string(obj, &["KODE_BARANG", "kode_barang", "kode"]);

            // Nama barang
            asset.nama_barang = Self::extract_string(obj, &["NAMA_BARANG", "nama_barang", "nama"]);

            // Kondisi
            asset.kondisi = Self::extract_string(obj, &["KONDISI", "kondisi", "kondisi_barang"]);

            // Tahun perolehan
            asset.tahun_perolehan = Self::extract_i32(obj, &["TAHUN_PEROLEHAN", "tahun_perolehan", "tahun"]);

            // Nilai perolehan
            asset.nilai_perolehan = Self::extract_f64(obj, &["NILAI_PEROLEHAN", "nilai_perolehan", "nilai"]);

            // Satker code
            asset.satker_code = Self::extract_string(obj, &["KODE_SATKER", "kode_satker", "satker"]);
        } else {
            asset.add_validation_error("Raw data is not a JSON object".to_string());
        }

        // Validate required fields
        Self::validate(&mut asset);

        Ok(asset)
    }

    /// Transform multiple assets
    pub fn transform_batch(
        category: SimanAssetCategory,
        raw_data_array: Vec<Value>,
    ) -> Result<Vec<TransformedAsset>, MonsaktiError> {
        let mut transformed = Vec::new();
        let mut error_count = 0;
        let total_count = raw_data_array.len();

        for raw_data in raw_data_array {
            match Self::transform(category, raw_data) {
                Ok(asset) => {
                    if asset.has_errors() {
                        warn!(
                            "Asset has validation errors: {:?}",
                            asset.validation_errors
                        );
                        error_count += 1;
                    }
                    transformed.push(asset);
                }
                Err(e) => {
                    warn!("Failed to transform asset: {}", e);
                    error_count += 1;
                }
            }
        }

        if error_count > 0 {
            warn!(
                "Transformation completed with {} errors out of {} records",
                error_count,
                total_count
            );
        }

        Ok(transformed)
    }

    /// Extract string value from multiple possible field names
    fn extract_string(obj: &serde_json::Map<String, Value>, field_names: &[&str]) -> Option<String> {
        for field_name in field_names {
            if let Some(value) = obj.get(*field_name) {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract i32 value from multiple possible field names
    fn extract_i32(obj: &serde_json::Map<String, Value>, field_names: &[&str]) -> Option<i32> {
        for field_name in field_names {
            if let Some(value) = obj.get(*field_name) {
                // Try as number
                if let Some(n) = value.as_i64() {
                    return Some(n as i32);
                }
                // Try as string
                if let Some(s) = value.as_str() {
                    if let Ok(n) = s.parse::<i32>() {
                        return Some(n);
                    }
                }
            }
        }
        None
    }

    /// Extract f64 value from multiple possible field names
    fn extract_f64(obj: &serde_json::Map<String, Value>, field_names: &[&str]) -> Option<f64> {
        for field_name in field_names {
            if let Some(value) = obj.get(*field_name) {
                // Try as number
                if let Some(n) = value.as_f64() {
                    return Some(n);
                }
                // Try as string
                if let Some(s) = value.as_str() {
                    if let Ok(n) = s.parse::<f64>() {
                        return Some(n);
                    }
                }
            }
        }
        None
    }

    /// Validate transformed asset
    fn validate(asset: &mut TransformedAsset) {
        // Check required fields
        if asset.nup.is_none() && asset.kode_barang.is_none() {
            asset.add_validation_error(
                "Missing required field: NUP or kode_barang must be present".to_string(),
            );
        }

        if asset.nama_barang.is_none() {
            asset.add_validation_error("Missing field: nama_barang".to_string());
        }

        // Validate kondisi values
        if let Some(ref kondisi) = asset.kondisi {
            let valid_kondisi = ["BAIK", "RUSAK RINGAN", "RUSAK BERAT", "HILANG"];
            if !valid_kondisi.contains(&kondisi.as_str()) {
                asset.add_validation_error(format!(
                    "Invalid kondisi value: {}. Expected one of: {:?}",
                    kondisi, valid_kondisi
                ));
            }
        }

        // Validate tahun_perolehan
        if let Some(tahun) = asset.tahun_perolehan {
            let current_year = chrono::Utc::now().year();
            if tahun < 1900 || tahun > current_year {
                asset.add_validation_error(format!(
                    "Invalid tahun_perolehan: {}. Must be between 1900 and {}",
                    tahun, current_year
                ));
            }
        }

        // Validate nilai_perolehan
        if let Some(nilai) = asset.nilai_perolehan {
            if nilai < 0.0 {
                asset.add_validation_error(format!(
                    "Invalid nilai_perolehan: {}. Must be non-negative",
                    nilai
                ));
            }
        }

        debug!(
            "Validated asset: {} errors",
            asset.validation_errors.len()
        );
    }

    /// Convert transformed asset to database-ready JSON
    pub fn to_database_json(asset: &TransformedAsset) -> Value {
        serde_json::json!({
            "id": asset.id,
            "kategori_aset": asset.kategori_aset,
            "nup": asset.nup,
            "kode_barang": asset.kode_barang,
            "nama_barang": asset.nama_barang,
            "kondisi": asset.kondisi,
            "tahun_perolehan": asset.tahun_perolehan,
            "nilai_perolehan": asset.nilai_perolehan,
            "satker_code": asset.satker_code,
            "raw_data": asset.raw_data,
            "validation_errors": asset.validation_errors,
        })
    }

    /// Convert batch of transformed assets to database-ready JSON array
    pub fn to_database_json_batch(assets: &[TransformedAsset]) -> Value {
        let json_array: Vec<Value> = assets
            .iter()
            .map(|asset| Self::to_database_json(asset))
            .collect();

        Value::Array(json_array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform_valid_asset() {
        let raw_data = json!({
            "NUP": "12345678",
            "KODE_BARANG": "1.3.2.05.01.01.0001",
            "NAMA_BARANG": "Tanah Kantor",
            "KONDISI": "BAIK",
            "TAHUN_PEROLEHAN": 2020,
            "NILAI_PEROLEHAN": 1000000000.0,
            "KODE_SATKER": "123456"
        });

        let result = SimanTransformer::transform(SimanAssetCategory::Tanah, raw_data);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.nup, Some("12345678".to_string()));
        assert_eq!(asset.kode_barang, Some("1.3.2.05.01.01.0001".to_string()));
        assert_eq!(asset.nama_barang, Some("Tanah Kantor".to_string()));
        assert_eq!(asset.kondisi, Some("BAIK".to_string()));
        assert_eq!(asset.tahun_perolehan, Some(2020));
        assert_eq!(asset.nilai_perolehan, Some(1000000000.0));
        assert!(!asset.has_errors());
        assert!(asset.is_valid());
    }

    #[test]
    fn test_transform_missing_required_fields() {
        let raw_data = json!({
            "NAMA_BARANG": "Tanah Kantor",
            "KONDISI": "BAIK"
        });

        let result = SimanTransformer::transform(SimanAssetCategory::Tanah, raw_data);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert!(asset.has_errors());
        assert!(!asset.is_valid());
        assert!(asset.validation_errors.iter().any(|e| e.contains("NUP or kode_barang")));
    }

    #[test]
    fn test_transform_invalid_kondisi() {
        let raw_data = json!({
            "NUP": "12345678",
            "NAMA_BARANG": "Tanah Kantor",
            "KONDISI": "INVALID_VALUE"
        });

        let result = SimanTransformer::transform(SimanAssetCategory::Tanah, raw_data);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert!(asset.has_errors());
        assert!(asset.validation_errors.iter().any(|e| e.contains("Invalid kondisi")));
    }

    #[test]
    fn test_transform_batch() {
        let raw_data_array = vec![
            json!({
                "NUP": "12345678",
                "KODE_BARANG": "1.3.2.05.01.01.0001",
                "NAMA_BARANG": "Tanah Kantor",
                "KONDISI": "BAIK"
            }),
            json!({
                "NUP": "87654321",
                "KODE_BARANG": "1.3.2.05.01.01.0002",
                "NAMA_BARANG": "Gedung Kantor",
                "KONDISI": "RUSAK RINGAN"
            }),
        ];

        let result = SimanTransformer::transform_batch(SimanAssetCategory::Tanah, raw_data_array);
        assert!(result.is_ok());

        let assets = result.unwrap();
        assert_eq!(assets.len(), 2);
        assert!(assets.iter().all(|a| a.is_valid()));
    }

    #[test]
    fn test_to_database_json() {
        let raw_data = json!({
            "NUP": "12345678",
            "NAMA_BARANG": "Tanah Kantor"
        });

        let asset = SimanTransformer::transform(SimanAssetCategory::Tanah, raw_data).unwrap();
        let json = SimanTransformer::to_database_json(&asset);

        assert!(json.is_object());
        assert!(json.get("id").is_some());
        assert_eq!(json.get("kategori_aset").and_then(|v| v.as_str()), Some("Tanah"));
        assert!(json.get("raw_data").is_some());
    }
}
