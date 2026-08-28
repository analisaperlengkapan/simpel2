use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::KebutuhanBmnRepository;
use crate::kebutuhan_bmn::siman_integration::SimanAsset;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use uuid::Uuid;

impl KebutuhanBmnService {
    /// Search for existing assets from SIMAN
    /// Returns similar assets from inventory for a specific barang
    pub async fn search_siman_assets(
        &self,
        search_term: &str,
        kategori: Option<&str>,
        limit: usize,
    ) -> AppResult<Vec<SimanAsset>> {
        let siman = self
            .siman
            .as_ref()
            .ok_or_else(|| AppError::Internal("SIMAN integration not configured".to_string()))?;

        siman
            .get_matching_assets(search_term, kategori, limit)
            .await
    }

    /// Get satker asset summary from SIMAN
    pub async fn get_satker_siman_summary(
        &self,
        satker_id: &str,
    ) -> AppResult<crate::kebutuhan_bmn::siman_integration::SatkerAssetSummary> {
        let siman = self
            .siman
            .as_ref()
            .ok_or_else(|| AppError::Internal("SIMAN integration not configured".to_string()))?;

        siman.get_satker_asset_summary(satker_id).await
    }

    // ========================================================================
    // Dashboard & Statistics
    // ========================================================================

    /// Get dashboard statistics
    pub async fn get_dashboard_stats(&self) -> AppResult<KebutuhanBmnDashboardStats> {
        self.repository.get_dashboard_stats().await
    }

    /// Get workflow history for a satker
    pub async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>> {
        self.repository.get_satker_aktivitas(satker_id, scope).await
    }

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Search kebutuhan BMN with full-text search and filters
    ///
    /// Uses PostgreSQL's Indonesian text search configuration for relevance ranking
    /// Requirements: REQ-K005
    pub async fn search_kebutuhan(
        &self,
        query: lib_perlengkapan::search::SearchQuery,
    ) -> AppResult<lib_perlengkapan::search::SearchResults<KebutuhanBmnSummary>> {
        use crate::shared::search_db::SearchEngineDb;

        let search_engine = SearchEngineDb::new(self.repository.pool().clone());

        search_engine
            .search_kebutuhan(query)
            .await
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e)))
    }

    /// Get search suggestions based on partial query
    ///
    /// Returns top N most relevant suggestions for autocomplete
    /// Requirements: REQ-K005
    pub async fn get_search_suggestions(
        &self,
        partial_query: &str,
        limit: i32,
    ) -> AppResult<Vec<String>> {
        use crate::shared::search_db::SearchEngineDb;

        let search_engine = SearchEngineDb::new(self.repository.pool().clone());

        search_engine
            .get_suggestions(partial_query, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Suggestions failed: {}", e)))
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================
}
