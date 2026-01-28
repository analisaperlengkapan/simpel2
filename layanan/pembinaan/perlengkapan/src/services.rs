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

    pub async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)> {
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

    // ============ Pemakaian Services ============

    pub async fn get_all_pemakaian(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pemakaian>, i64)> {
        self.repo.get_all_pemakaian(page, per_page).await
    }

    pub async fn create_pemakaian(
        &self,
        request: CreatePemakaianRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pemakaian> {
        request.validate()?;
        self.repo.create_pemakaian(request, user_id).await
    }

    // ============ Hibah Services ============

    pub async fn get_all_hibah(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Hibah>, i64)> {
        self.repo.get_all_hibah(page, per_page).await
    }

    pub async fn create_hibah(
        &self,
        request: CreateHibahRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Hibah> {
        request.validate()?;
        self.repo.create_hibah(request, user_id).await
    }

    // ============ Mutasi Services ============

    pub async fn get_all_mutasi(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Mutasi>, i64)> {
        self.repo.get_all_mutasi(page, per_page).await
    }

    pub async fn create_mutasi(
        &self,
        request: CreateMutasiRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Mutasi> {
        request.validate()?;
        self.repo.create_mutasi(request, user_id).await
    }

    // ============ Penghapusan Services ============

    pub async fn get_all_penghapusan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Penghapusan>, i64)> {
        self.repo.get_all_penghapusan(page, per_page).await
    }

    pub async fn create_penghapusan(
        &self,
        request: CreatePenghapusanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Penghapusan> {
        request.validate()?;
        self.repo.create_penghapusan(request, user_id).await
    }

    // ============ Pengalihan Services ============

    pub async fn get_all_pengalihan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pengalihan>, i64)> {
        self.repo.get_all_pengalihan(page, per_page).await
    }

    pub async fn create_pengalihan(
        &self,
        request: CreatePengalihanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pengalihan> {
        request.validate()?;
        self.repo.create_pengalihan(request, user_id).await
    }

    // ============ Pemeliharaan Services ============

    pub async fn get_all_pemeliharaan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pemeliharaan>, i64)> {
        self.repo.get_all_pemeliharaan(page, per_page).await
    }

    pub async fn create_pemeliharaan(
        &self,
        request: CreatePemeliharaanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pemeliharaan> {
        request.validate()?;
        self.repo.create_pemeliharaan(request, user_id).await
    }
}
