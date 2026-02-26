use crate::error::MonsaktiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, warn};

/// Satker code mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerCodeMapping {
    /// MySIMKARI satker code
    pub mysimkari_code: String,
    /// Internal SIMPEL satker code
    pub simpel_code: String,
    /// Mapping notes
    pub notes: Option<String>,
}

/// Satker code mapper
pub struct SatkerCodeMapper {
    mappings: HashMap<String, String>,
}

impl SatkerCodeMapper {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Load mappings from configuration
    pub fn load_mappings(&mut self, mappings: Vec<SatkerCodeMapping>) {
        for mapping in mappings {
            self.mappings
                .insert(mapping.mysimkari_code, mapping.simpel_code);
        }
    }

    /// Map MySIMKARI satker code to SIMPEL satker code
    pub fn map_satker_code(&self, mysimkari_code: &str) -> Option<String> {
        self.mappings.get(mysimkari_code).cloned()
    }

    /// Add a single mapping
    pub fn add_mapping(&mut self, mysimkari_code: String, simpel_code: String) {
        self.mappings.insert(mysimkari_code, simpel_code);
    }
}

impl Default for SatkerCodeMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Transformed satker data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedSatker {
    pub id: String,
    pub kode_satker: String,
    pub nama_satker: String,
    pub wilayah: Option<String>,
    pub tipe_satker: Option<String>,
    pub simpel_satker_code: Option<String>,
    pub raw_data: Value,
    pub transformed_at: DateTime<Utc>,
}

/// Transformed pegawai data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_id: Option<String>,
    pub satker_code: Option<String>,
    pub simpel_satker_code: Option<String>,
    pub jabatan: Option<String>,
    pub golongan: Option<String>,
    pub status_pegawai: Option<String>,
    pub is_active: bool,
    pub raw_data: Value,
    pub transformed_at: DateTime<Utc>,
}

/// MySIMKARI data transformer
pub struct MySIMKARITransformer {
    satker_mapper: SatkerCodeMapper,
}

impl MySIMKARITransformer {
    pub fn new(satker_mapper: SatkerCodeMapper) -> Self {
        Self { satker_mapper }
    }

    /// Transform satker data from MySIMKARI format to internal format
    pub fn transform_satker(&self, raw_data: &Value) -> Result<TransformedSatker, MonsaktiError> {
        // Validate required fields
        let id = raw_data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MonsaktiError::ApiError("Missing required field: id in satker data".to_string())
            })?
            .to_string();

        let kode_satker = raw_data
            .get("kode_satker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MonsaktiError::ApiError(
                    "Missing required field: kode_satker in satker data".to_string(),
                )
            })?
            .to_string();

        let nama_satker = raw_data
            .get("nama_satker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MonsaktiError::ApiError(
                    "Missing required field: nama_satker in satker data".to_string(),
                )
            })?
            .to_string();

        // Optional fields
        let wilayah = raw_data
            .get("wilayah")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tipe_satker = raw_data
            .get("tipe_satker")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Map satker code
        let simpel_satker_code = self.satker_mapper.map_satker_code(&kode_satker);

        if simpel_satker_code.is_none() {
            warn!(
                "No SIMPEL satker code mapping found for MySIMKARI code: {}",
                kode_satker
            );
        }

        Ok(TransformedSatker {
            id,
            kode_satker,
            nama_satker,
            wilayah,
            tipe_satker,
            simpel_satker_code,
            raw_data: raw_data.clone(),
            transformed_at: Utc::now(),
        })
    }

    /// Transform pegawai data from MySIMKARI format to internal format
    pub fn transform_pegawai(&self, raw_data: &Value) -> Result<TransformedPegawai, MonsaktiError> {
        // Validate required fields
        let nip = raw_data
            .get("nip")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MonsaktiError::ApiError("Missing required field: nip in pegawai data".to_string())
            })?
            .to_string();

        let nama = raw_data
            .get("nama")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MonsaktiError::ApiError("Missing required field: nama in pegawai data".to_string())
            })?
            .to_string();

        // Optional fields
        let satker_id = raw_data
            .get("satker_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let satker_code = raw_data
            .get("satker_code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let jabatan = raw_data
            .get("jabatan")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let golongan = raw_data
            .get("golongan")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let status_pegawai = raw_data
            .get("status_pegawai")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Determine if pegawai is active
        let is_active = status_pegawai
            .as_ref()
            .map(|s| s.to_lowercase() == "aktif" || s.to_lowercase() == "active")
            .unwrap_or(true);

        // Map satker code if available
        let simpel_satker_code = satker_code
            .as_ref()
            .and_then(|code| self.satker_mapper.map_satker_code(code));

        if satker_code.is_some() && simpel_satker_code.is_none() {
            warn!(
                "No SIMPEL satker code mapping found for MySIMKARI code: {} (NIP: {})",
                satker_code.as_ref().unwrap(),
                nip
            );
        }

        Ok(TransformedPegawai {
            nip,
            nama,
            satker_id,
            satker_code,
            simpel_satker_code,
            jabatan,
            golongan,
            status_pegawai,
            is_active,
            raw_data: raw_data.clone(),
            transformed_at: Utc::now(),
        })
    }

    /// Transform batch of satker data
    pub fn transform_satker_batch(
        &self,
        raw_data_array: &Value,
    ) -> Result<Vec<TransformedSatker>, MonsaktiError> {
        let array = raw_data_array
            .as_array()
            .ok_or_else(|| MonsaktiError::ApiError("Expected array of satker data".to_string()))?;

        let mut results = Vec::new();
        let mut errors = 0;

        for item in array {
            match self.transform_satker(item) {
                Ok(transformed) => results.push(transformed),
                Err(e) => {
                    error!("Failed to transform satker data: {:?}", e);
                    errors += 1;
                }
            }
        }

        if errors > 0 {
            warn!(
                "Transformed {} satker records with {} errors",
                results.len(),
                errors
            );
        }

        Ok(results)
    }

    /// Transform batch of pegawai data
    pub fn transform_pegawai_batch(
        &self,
        raw_data_array: &Value,
    ) -> Result<Vec<TransformedPegawai>, MonsaktiError> {
        let array = raw_data_array
            .as_array()
            .ok_or_else(|| MonsaktiError::ApiError("Expected array of pegawai data".to_string()))?;

        let mut results = Vec::new();
        let mut errors = 0;

        for item in array {
            match self.transform_pegawai(item) {
                Ok(transformed) => results.push(transformed),
                Err(e) => {
                    error!("Failed to transform pegawai data: {:?}", e);
                    errors += 1;
                }
            }
        }

        if errors > 0 {
            warn!(
                "Transformed {} pegawai records with {} errors",
                results.len(),
                errors
            );
        }

        Ok(results)
    }

    /// Validate NIP format (18 digits)
    pub fn validate_nip(nip: &str) -> bool {
        nip.len() == 18 && nip.chars().all(|c| c.is_ascii_digit())
    }

    /// Validate satker code format
    pub fn validate_satker_code(code: &str) -> bool {
        !code.is_empty() && code.len() <= 20
    }

    /// Get satker mapper reference
    pub fn satker_mapper(&self) -> &SatkerCodeMapper {
        &self.satker_mapper
    }

    /// Get mutable satker mapper reference
    pub fn satker_mapper_mut(&mut self) -> &mut SatkerCodeMapper {
        &mut self.satker_mapper
    }
}

impl Default for MySIMKARITransformer {
    fn default() -> Self {
        Self::new(SatkerCodeMapper::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_nip() {
        assert!(MySIMKARITransformer::validate_nip("199001012020011001"));
        assert!(!MySIMKARITransformer::validate_nip("19900101202001100")); // 17 digits
        assert!(!MySIMKARITransformer::validate_nip("1990010120200110011")); // 19 digits
        assert!(!MySIMKARITransformer::validate_nip("19900101202001100A")); // contains letter
    }

    #[test]
    fn test_validate_satker_code() {
        assert!(MySIMKARITransformer::validate_satker_code("123456"));
        assert!(MySIMKARITransformer::validate_satker_code("KEJARI-001"));
        assert!(!MySIMKARITransformer::validate_satker_code(""));
    }

    #[test]
    fn test_transform_satker() {
        let mut mapper = SatkerCodeMapper::new();
        mapper.add_mapping("123456".to_string(), "SIMPEL-001".to_string());

        let transformer = MySIMKARITransformer::new(mapper);

        let raw_data = json!({
            "id": "1",
            "kode_satker": "123456",
            "nama_satker": "Kejaksaan Negeri Jakarta Pusat",
            "wilayah": "DKI Jakarta",
            "tipe_satker": "Kejari"
        });

        let result = transformer.transform_satker(&raw_data).unwrap();

        assert_eq!(result.id, "1");
        assert_eq!(result.kode_satker, "123456");
        assert_eq!(result.nama_satker, "Kejaksaan Negeri Jakarta Pusat");
        assert_eq!(result.simpel_satker_code, Some("SIMPEL-001".to_string()));
    }

    #[test]
    fn test_transform_pegawai() {
        let mut mapper = SatkerCodeMapper::new();
        mapper.add_mapping("123456".to_string(), "SIMPEL-001".to_string());

        let transformer = MySIMKARITransformer::new(mapper);

        let raw_data = json!({
            "nip": "199001012020011001",
            "nama": "John Doe",
            "satker_id": "1",
            "satker_code": "123456",
            "jabatan": "Jaksa",
            "golongan": "III/a",
            "status_pegawai": "Aktif"
        });

        let result = transformer.transform_pegawai(&raw_data).unwrap();

        assert_eq!(result.nip, "199001012020011001");
        assert_eq!(result.nama, "John Doe");
        assert_eq!(result.simpel_satker_code, Some("SIMPEL-001".to_string()));
        assert!(result.is_active);
    }
}
