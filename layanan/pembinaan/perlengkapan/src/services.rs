//! # Services Layer
//!
//! Business logic for the Perlengkapan service

use crate::{errors::*, models::*, repository::PerlengkapanRepository};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct PerlengkapanService {
    repo: Arc<dyn PerlengkapanRepository>,
}

impl PerlengkapanService {
    pub fn new(repo: Arc<dyn PerlengkapanRepository>) -> Self {
        Self { repo }
    }

    // ============ Dashboard Services ============

    pub async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        self.repo.get_dashboard_stats().await
    }

    // ============ Asset Services (Read-Only) ============

    pub async fn get_all_assets(
        &self,
        page: i32,
        per_page: i32,
        category: Option<String>,
    ) -> AppResult<(Vec<Asset>, i64)> {
        self.repo.get_all_assets(page, per_page, category).await
    }

    pub async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset> {
        self.repo.get_asset_by_id(id).await
    }

    // ============ Pengadaan Services ============

    pub async fn get_all_pengadaan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pengadaan>, i64)> {
        self.repo.get_all_pengadaan(page, per_page).await
    }

    pub async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan> {
        self.repo.get_pengadaan_by_id(id).await
    }

    pub async fn create_pengadaan(
        &self,
        request: CreatePengadaanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pengadaan> {
        request.validate()?;
        self.repo.create_pengadaan(request, user_id).await
    }

    // ============ Analisis Kebutuhan Services ============

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        self.repo.get_all_analisis(page, per_page).await
    }

    pub async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;
        self.repo.create_analisis(request, user_id).await
    }
}
