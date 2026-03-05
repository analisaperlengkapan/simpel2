//! Service traits for Perlengkapan domain
//!
//! These traits define the interfaces that perlengkapan services must implement.


/// Trait for asset repository operations
#[cfg(feature = "backend")]
#[async_trait::async_trait]
pub trait AssetRepository: Send + Sync {
    type Error;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Asset>, Self::Error>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Asset>, Self::Error>;
    async fn count(&self) -> Result<i64, Self::Error>;
}

/// Trait for dashboard statistics
#[cfg(feature = "backend")]
#[async_trait::async_trait]
pub trait DashboardService: Send + Sync {
    type Error;

    async fn get_stats(&self) -> Result<DashboardStats, Self::Error>;
}
