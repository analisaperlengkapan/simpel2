//! # Mapping Kodefikasi Services
//!
//! Business logic for mapping kodefikasi

use crate::errors::AppError;
use crate::mapping_kodefikasi::models::*;
use crate::mapping_kodefikasi::repository::MappingRepository;
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Service for mapping kodefikasi operations
pub struct MappingService {
    repository: MappingRepository,
}

impl MappingService {
    pub fn new(pool: Pool) -> Self {
        Self {
            repository: MappingRepository::new(pool),
        }
    }

    /// Detect non-standard codes and suggest mappings
    pub async fn detect_non_standard_codes_with_suggestions(
        &self,
    ) -> Result<Vec<NonStandardCode>, AppError> {
        let mut non_standard = self.repository.detect_non_standard_codes().await?;

        // Add suggestions for each non-standard code
        for code in &mut non_standard {
            let suggestions = self
                .repository
                .suggest_mapping(&code.nama_lama)
                .await?;

            if let Some(best_match) = suggestions.first() {
                code.suggested_mapping = Some(best_match.clone());
            }
        }

        Ok(non_standard)
    }

    /// Detect non-standard codes without suggestions
    pub async fn detect_non_standard_codes(&self) -> Result<Vec<NonStandardCode>, AppError> {
        self.repository.detect_non_standard_codes().await
    }

    /// Get mapping suggestions for a specific code
    pub async fn get_mapping_suggestions(
        &self,
        nama_lama: &str,
    ) -> Result<Vec<MappingSuggestion>, AppError> {
        self.repository.suggest_mapping(nama_lama).await
    }

    /// Create a mapping proposal
    pub async fn create_proposal(
        &self,
        request: MappingProposalRequest,
    ) -> Result<MappingProposal, AppError> {
        // Validate that the kode_baru_id exists
        // This would be done in a real implementation

        self.repository.create_proposal(&request).await
    }

    /// Get all mapping proposals
    pub async fn get_all_proposals(&self) -> Result<Vec<MappingProposal>, AppError> {
        self.repository.get_all_proposals().await
    }

    /// Get proposal by ID
    pub async fn get_proposal_by_id(&self, id: Uuid) -> Result<MappingProposal, AppError> {
        self.repository.get_proposal_by_id(id).await
    }

    /// Verify a mapping proposal
    pub async fn verify_proposal(
        &self,
        proposal_id: Uuid,
        request: VerifyMappingRequest,
    ) -> Result<MappingProposal, AppError> {
        let proposal = self
            .repository
            .verify_proposal(proposal_id, request.approved, request.catatan_verifikasi)
            .await?;

        // If approved, apply the mapping to SIMAN assets
        if request.approved {
            let rows_affected = self.repository.apply_mapping(proposal_id).await?;
            tracing::info!(
                "Applied mapping {} to {} assets",
                proposal_id,
                rows_affected
            );
        }

        Ok(proposal)
    }

    /// Get mapping progress statistics
    pub async fn get_mapping_progress(&self) -> Result<MappingProgress, AppError> {
        self.repository.get_mapping_progress().await
    }

    /// Get mapping progress by satker
    pub async fn get_mapping_progress_by_satker(
        &self,
    ) -> Result<Vec<MappingProgressBySatker>, AppError> {
        self.repository.get_mapping_progress_by_satker().await
    }

    /// Get mapping progress by wilayah
    pub async fn get_mapping_progress_by_wilayah(
        &self,
    ) -> Result<Vec<MappingProgressByWilayah>, AppError> {
        // Get progress by satker first
        let satker_progress = self.get_mapping_progress_by_satker().await?;

        // Group by wilayah (this would require wilayah information in satkers table)
        // For now, return empty vec as placeholder
        // In a real implementation, this would query the satkers table with wilayah info

        Ok(vec![])
    }
}
