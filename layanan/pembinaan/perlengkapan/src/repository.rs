use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::*,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PerlengkapanRepository: Send + Sync {
    // Dashboard
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;

    // Asset (Read-Only from Integrasi/SIMAN)
    async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)>;
    async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset>;

    // Pengadaan (Local)
    async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)>;
    async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan>;
    async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan>;

    // Analisis (Local)
    async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
    async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;

    // Pemakaian (Local)
    async fn get_all_pemakaian(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pemakaian>, i64)>;
    async fn create_pemakaian(&self, request: CreatePemakaianRequest, user_id: Option<Uuid>) -> AppResult<Pemakaian>;

    // Hibah (Local)
    async fn get_all_hibah(&self, page: i32, per_page: i32) -> AppResult<(Vec<Hibah>, i64)>;
    async fn create_hibah(&self, request: CreateHibahRequest, user_id: Option<Uuid>) -> AppResult<Hibah>;
}
