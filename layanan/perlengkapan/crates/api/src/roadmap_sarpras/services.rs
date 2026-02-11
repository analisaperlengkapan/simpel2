//! Roadmap Sarpras business logic services

use crate::errors::AppError;
use lib_perlengkapan::models::{
    CreateRoadmapSarprasRequest, RoadmapRealizationComparison, RoadmapSarpras,
    UpdateRoadmapRealizationRequest,
};
use uuid::Uuid;
use validator::Validate;

use super::models::{
    CreateRoadmapBatchRequest, CreateRoadmapResponse, ListRoadmapQuery, ListRoadmapResponse,
    RoadmapComparisonQuery, RoadmapComparisonResponse, SyncRealizationRequest,
};
use super::repository::RoadmapRepository;

#[derive(Clone)]
pub struct RoadmapService {
    repository: RoadmapRepository,
}

impl RoadmapService {
    pub fn new(repository: RoadmapRepository) -> Self {
        Self { repository }
    }

    /// Create a single roadmap item
    pub async fn create_roadmap(
        &self,
        request: CreateRoadmapSarprasRequest,
        created_by: Uuid,
    ) -> Result<RoadmapSarpras, AppError> {
        // Validate request
        request.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        // Validate tahun_rencana is within periode
        request.validate_tahun_in_periode().map_err(|e| {
            AppError::BadRequest(e)
        })?;

        // Validate periode is exactly 5 years
        request.validate_periode_duration().map_err(|e| {
            AppError::BadRequest(e)
        })?;

        // Create roadmap
        let roadmap = self
            .repository
            .create(
                request.satker_id,
                request.periode_mulai,
                request.periode_akhir,
                &request.kode_barang,
                &request.nama_barang,
                request.tahun_rencana,
                request.jumlah_kebutuhan,
                request.estimasi_anggaran,
                request.keterangan.as_deref(),
                created_by,
            )
            .await?;

        Ok(roadmap)
    }

    /// Create multiple roadmap items in a batch
    pub async fn create_roadmap_batch(
        &self,
        request: CreateRoadmapBatchRequest,
        created_by: Uuid,
    ) -> Result<CreateRoadmapResponse, AppError> {
        // Validate request
        request.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        // Validate periode is exactly 5 years
        request.validate_periode_duration().map_err(|e| {
            AppError::BadRequest(e)
        })?;

        // Validate each item's tahun_rencana is within periode
        for item in &request.items {
            if item.tahun_rencana < request.periode_mulai
                || item.tahun_rencana > request.periode_akhir
            {
                return Err(AppError::BadRequest(format!(
                    "Tahun rencana {} for item {} must be between {} and {}",
                    item.tahun_rencana,
                    item.kode_barang,
                    request.periode_mulai,
                    request.periode_akhir
                )));
            }
        }

        // Create batch
        let roadmap_ids = self.repository.create_batch(&request, created_by).await?;

        Ok(CreateRoadmapResponse {
            roadmap_ids: roadmap_ids.clone(),
            message: format!("Successfully created {} roadmap items", roadmap_ids.len()),
        })
    }

    /// Get roadmap by ID
    pub async fn get_roadmap(&self, roadmap_id: Uuid) -> Result<RoadmapSarpras, AppError> {
        self.repository
            .get_by_id(roadmap_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Roadmap {} not found", roadmap_id)))
    }

    /// List roadmaps with filters and pagination
    pub async fn list_roadmaps(
        &self,
        query: ListRoadmapQuery,
    ) -> Result<ListRoadmapResponse, AppError> {
        // Validate query
        query.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        let (roadmaps, total) = self.repository.list(&query).await?;

        Ok(ListRoadmapResponse {
            roadmaps,
            total,
            limit: query.limit.unwrap_or(100),
            offset: query.offset.unwrap_or(0),
        })
    }

    /// Update roadmap realization
    pub async fn update_realization(
        &self,
        roadmap_id: Uuid,
        request: UpdateRoadmapRealizationRequest,
        updated_by: Uuid,
    ) -> Result<RoadmapSarpras, AppError> {
        // Validate request
        request.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        // Check if roadmap exists
        let _ = self.get_roadmap(roadmap_id).await?;

        // Update realization
        let roadmap = self
            .repository
            .update_realization(roadmap_id, &request, updated_by)
            .await?;

        Ok(roadmap)
    }

    /// Sync realization increment (called by MonSAKTI integration)
    pub async fn sync_realization_increment(
        &self,
        request: SyncRealizationRequest,
    ) -> Result<RoadmapSarpras, AppError> {
        // Validate request
        request.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        // Check if roadmap exists
        let _ = self.get_roadmap(request.roadmap_id).await?;

        // Sync realization
        let roadmap = self
            .repository
            .sync_realization_increment(&request)
            .await?;

        Ok(roadmap)
    }

    /// Get roadmap vs realization comparison
    pub async fn get_comparison(
        &self,
        query: RoadmapComparisonQuery,
    ) -> Result<RoadmapComparisonResponse, AppError> {
        // Validate query
        query.validate().map_err(|e| {
            AppError::BadRequest(format!("Validation failed: {}", e))
        })?;

        // Validate periode is exactly 5 years
        let duration = query.periode_akhir - query.periode_mulai + 1;
        if duration != 5 {
            return Err(AppError::BadRequest(format!(
                "Periode must be exactly 5 years, got {} years",
                duration
            )));
        }

        // Get comparisons
        let comparisons = self.repository.get_comparison(&query).await?;

        // Get summary
        let summary = self
            .repository
            .get_summary(query.satker_id, query.periode_mulai, query.periode_akhir)
            .await?;

        Ok(RoadmapComparisonResponse {
            comparisons,
            summary,
        })
    }

    /// Delete roadmap
    pub async fn delete_roadmap(&self, roadmap_id: Uuid) -> Result<(), AppError> {
        // Check if roadmap exists
        let _ = self.get_roadmap(roadmap_id).await?;

        // Delete roadmap
        let deleted = self.repository.delete(roadmap_id).await?;

        if !deleted {
            return Err(AppError::Internal(
                "Failed to delete roadmap".to_string(),
            ));
        }

        Ok(())
    }
}
