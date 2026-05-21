//! # Mapping Kodefikasi Services
//!
//! Simplified read-only business logic for mapping kodefikasi.
//! Lists standard/non-standard BMN codes and provides CSV export.

use crate::errors::AppError;
use crate::mapping_kodefikasi::models::*;
use crate::mapping_kodefikasi::repository::MappingRepository;
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Service for mapping kodefikasi operations (read-only)
pub struct MappingService {
    repository: MappingRepository,
}

impl MappingService {
    pub fn new(pool: Pool) -> Self {
        Self {
            repository: MappingRepository::new(pool),
        }
    }

    /// List non-standard BMN codes with optional search and satker filter
    pub async fn list_non_standard_codes(
        &self,
        search: Option<&str>,
        satker_id: Option<Uuid>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<NonStandardCode>, i64), AppError> {
        self.repository
            .list_non_standard_codes(search, satker_id, page, per_page)
            .await
    }

    /// List standard BMN codes from master table
    pub async fn list_standard_codes(
        &self,
        search: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<StandardBmnCode>, i64), AppError> {
        self.repository
            .list_standard_codes(search, page, per_page)
            .await
    }

    /// Get mapping suggestions for a specific code name using fuzzy matching
    pub async fn get_mapping_suggestions(
        &self,
        nama_lama: &str,
    ) -> Result<Vec<MappingSuggestion>, AppError> {
        self.repository.suggest_mapping(nama_lama).await
    }

    /// Get mapping progress statistics
    pub async fn get_mapping_progress(&self) -> Result<MappingProgress, AppError> {
        self.repository.get_mapping_progress().await
    }

    /// Get mapping progress grouped by satker
    pub async fn get_mapping_progress_by_satker(
        &self,
    ) -> Result<Vec<MappingProgressBySatker>, AppError> {
        self.repository.get_mapping_progress_by_satker().await
    }

    /// Export mapping data as CSV
    pub async fn export_csv(&self, satker_id: Option<Uuid>) -> Result<String, AppError> {
        let (codes, _total) = self
            .repository
            .list_non_standard_codes(None, satker_id, 1, 10000)
            .await?;

        let mut csv = String::from(
            "Kode Lama,Nama Lama,Satker ID,Jumlah Aset,Suggested Kode Baru,Suggested Nama Baru,Similarity Score\n",
        );

        for code in &codes {
            let (suggested_kode, suggested_nama, score) = match &code.suggested_mapping {
                Some(s) => (
                    s.kode_baru.as_str(),
                    s.nama_baru.as_str(),
                    format!("{:.2}", s.similarity_score),
                ),
                None => ("", "", String::new()),
            };

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",{},\"{}\",\"{}\",{}\n",
                code.kode_lama,
                code.nama_lama,
                code.satker_id,
                code.jumlah_aset,
                suggested_kode,
                suggested_nama,
                score,
            ));
        }

        Ok(csv)
    }
}
