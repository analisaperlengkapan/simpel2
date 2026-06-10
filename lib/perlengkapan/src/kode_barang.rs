//! Kode Barang Utilities
//!
//! Provides validation and utilities for BMN kode barang (asset codes)

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// Kode barang validation result
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KodeBarangValidation {
    Valid,
    Invalid(String),
}

impl KodeBarangValidation {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(self, KodeBarangValidation::Valid)
    }

    /// Get error message if invalid
    pub fn error_message(&self) -> Option<&str> {
        match self {
            KodeBarangValidation::Valid => None,
            KodeBarangValidation::Invalid(msg) => Some(msg),
        }
    }
}

/// BMN kode barang format validator
pub struct KodeBarangValidator;

impl KodeBarangValidator {
    /// Validate BMN kode barang format
    ///
    /// Standard BMN format: X.XX.XX.XX.XXX
    /// - 5 segments separated by dots
    /// - First segment: 1 digit (category)
    /// - Segments 2-4: 2 digits each (subcategories)
    /// - Last segment: 3 digits (item code)
    ///
    /// Example: 1.01.01.01.001
    pub fn validate(kode: &str) -> KodeBarangValidation {
        // Check if empty
        if kode.is_empty() {
            return KodeBarangValidation::Invalid("Kode barang cannot be empty".to_string());
        }

        // Split by dots
        let segments: Vec<&str> = kode.split('.').collect();

        // Check number of segments
        if segments.len() != 5 {
            return KodeBarangValidation::Invalid(format!(
                "Kode barang must have 5 segments, got {}",
                segments.len()
            ));
        }

        // Validate first segment (1 digit)
        if segments[0].len() != 1 || !segments[0].chars().all(|c| c.is_ascii_digit()) {
            return KodeBarangValidation::Invalid("First segment must be 1 digit".to_string());
        }

        // Validate segments 2-4 (2 digits each)
        for (i, segment) in segments.iter().enumerate().skip(1).take(3) {
            if segment.len() != 2 || !segment.chars().all(|c| c.is_ascii_digit()) {
                return KodeBarangValidation::Invalid(format!(
                    "Segment {} must be 2 digits",
                    i + 1
                ));
            }
        }

        // Validate last segment (3 digits)
        if segments[4].len() != 3 || !segments[4].chars().all(|c| c.is_ascii_digit()) {
            return KodeBarangValidation::Invalid("Last segment must be 3 digits".to_string());
        }

        KodeBarangValidation::Valid
    }

    /// Check if kode barang is in standard format
    pub fn is_standard_format(kode: &str) -> bool {
        Self::validate(kode).is_valid()
    }

    /// Normalize kode barang (remove extra spaces, convert to uppercase)
    pub fn normalize(kode: &str) -> String {
        kode.trim().to_uppercase()
    }
}

/// Parsed kode barang structure
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParsedKodeBarang {
    pub category: String,
    pub subcategory1: String,
    pub subcategory2: String,
    pub subcategory3: String,
    pub item_code: String,
    pub full_code: String,
}

impl ParsedKodeBarang {
    /// Parse kode barang into components
    pub fn parse(kode: &str) -> Result<Self, String> {
        let validation = KodeBarangValidator::validate(kode);
        if !validation.is_valid() {
            return Err(validation.error_message().unwrap().to_string());
        }

        let segments: Vec<&str> = kode.split('.').collect();

        Ok(Self {
            category: segments[0].to_string(),
            subcategory1: segments[1].to_string(),
            subcategory2: segments[2].to_string(),
            subcategory3: segments[3].to_string(),
            item_code: segments[4].to_string(),
            full_code: kode.to_string(),
        })
    }

    /// Get category code (first segment)
    pub fn category_code(&self) -> &str {
        &self.category
    }

    /// Get full subcategory path (first 4 segments)
    pub fn subcategory_path(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.category, self.subcategory1, self.subcategory2, self.subcategory3
        )
    }
}

impl fmt::Display for ParsedKodeBarang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_code)
    }
}

// Kode barang autocomplete search

/// Autocomplete result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AutocompleteResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub relevance: f64,
}

impl AutocompleteResult {
    /// Create new autocomplete result
    pub fn new(kode_barang: String, nama_barang: String, relevance: f64) -> Self {
        Self {
            kode_barang,
            nama_barang,
            relevance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_kode() {
        let validation = KodeBarangValidator::validate("1.01.01.01.001");
        assert!(validation.is_valid());
    }

    #[test]
    fn test_validate_invalid_segments() {
        let validation = KodeBarangValidator::validate("1.01.01.001");
        assert!(!validation.is_valid());
        assert!(validation.error_message().unwrap().contains("5 segments"));
    }

    #[test]
    fn test_validate_invalid_first_segment() {
        let validation = KodeBarangValidator::validate("10.01.01.01.001");
        assert!(!validation.is_valid());
        assert!(
            validation
                .error_message()
                .unwrap()
                .contains("First segment")
        );
    }

    #[test]
    fn test_validate_invalid_middle_segment() {
        let validation = KodeBarangValidator::validate("1.1.01.01.001");
        assert!(!validation.is_valid());
        assert!(validation.error_message().unwrap().contains("2 digits"));
    }

    #[test]
    fn test_validate_invalid_last_segment() {
        let validation = KodeBarangValidator::validate("1.01.01.01.01");
        assert!(!validation.is_valid());
        assert!(validation.error_message().unwrap().contains("3 digits"));
    }

    #[test]
    fn test_validate_empty() {
        let validation = KodeBarangValidator::validate("");
        assert!(!validation.is_valid());
        assert!(
            validation
                .error_message()
                .unwrap()
                .contains("cannot be empty")
        );
    }

    #[test]
    fn test_is_standard_format() {
        assert!(KodeBarangValidator::is_standard_format("1.01.01.01.001"));
        assert!(!KodeBarangValidator::is_standard_format("1.01.01.001"));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            KodeBarangValidator::normalize("  1.01.01.01.001  "),
            "1.01.01.01.001"
        );
        assert_eq!(
            KodeBarangValidator::normalize("1.01.01.01.001"),
            "1.01.01.01.001"
        );
    }

    #[test]
    fn test_parse_kode_barang() {
        let parsed = ParsedKodeBarang::parse("1.01.01.01.001").unwrap();
        assert_eq!(parsed.category, "1");
        assert_eq!(parsed.subcategory1, "01");
        assert_eq!(parsed.subcategory2, "01");
        assert_eq!(parsed.subcategory3, "01");
        assert_eq!(parsed.item_code, "001");
        assert_eq!(parsed.full_code, "1.01.01.01.001");
    }

    #[test]
    fn test_parse_invalid_kode() {
        let result = ParsedKodeBarang::parse("1.01.01.001");
        assert!(result.is_err());
    }

    #[test]
    fn test_category_code() {
        let parsed = ParsedKodeBarang::parse("1.01.01.01.001").unwrap();
        assert_eq!(parsed.category_code(), "1");
    }

    #[test]
    fn test_subcategory_path() {
        let parsed = ParsedKodeBarang::parse("1.01.01.01.001").unwrap();
        assert_eq!(parsed.subcategory_path(), "1.01.01.01");
    }

    #[test]
    fn test_display() {
        let parsed = ParsedKodeBarang::parse("1.01.01.01.001").unwrap();
        assert_eq!(format!("{}", parsed), "1.01.01.01.001");
    }

    #[test]
    fn test_autocomplete_result() {
        let result =
            AutocompleteResult::new("1.01.01.01.001".to_string(), "Meja Kerja".to_string(), 0.95);
        assert_eq!(result.kode_barang, "1.01.01.01.001");
        assert_eq!(result.nama_barang, "Meja Kerja");
        assert_eq!(result.relevance, 0.95);
    }
}
