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

    // ============ Pengadaan Sub-Documents Services ============

    pub async fn create_pengadaan_hps(
        &self,
        request: CreatePengadaanHpsRequest,
    ) -> AppResult<PengadaanHps> {
        request.validate()?;
        self.repo.create_pengadaan_hps(request).await
    }

    pub async fn get_pengadaan_hps(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanHps>> {
        self.repo.get_pengadaan_hps(pengadaan_id).await
    }

    pub async fn create_pengadaan_skppbj(
        &self,
        request: CreatePengadaanSkppbjRequest,
    ) -> AppResult<PengadaanSkppbj> {
        request.validate()?;
        self.repo.create_pengadaan_skppbj(request).await
    }

    pub async fn get_pengadaan_skppbj(
        &self,
        pengadaan_id: Uuid,
    ) -> AppResult<Vec<PengadaanSkppbj>> {
        self.repo.get_pengadaan_skppbj(pengadaan_id).await
    }

    pub async fn create_pengadaan_spk(
        &self,
        request: CreatePengadaanSpkRequest,
    ) -> AppResult<PengadaanSpk> {
        request.validate()?;
        self.repo.create_pengadaan_spk(request).await
    }

    pub async fn get_pengadaan_spk(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanSpk>> {
        self.repo.get_pengadaan_spk(pengadaan_id).await
    }

    pub async fn create_pengadaan_ringkasan(
        &self,
        request: CreatePengadaanRingkasanRequest,
    ) -> AppResult<PengadaanRingkasan> {
        request.validate()?;
        self.repo.create_pengadaan_ringkasan(request).await
    }

    pub async fn get_pengadaan_ringkasan(
        &self,
        pengadaan_id: Uuid,
    ) -> AppResult<Vec<PengadaanRingkasan>> {
        self.repo.get_pengadaan_ringkasan(pengadaan_id).await
    }

    pub async fn create_pengadaan_kontrak(
        &self,
        request: CreatePengadaanKontrakRequest,
    ) -> AppResult<PengadaanKontrak> {
        request.validate()?;
        self.repo.create_pengadaan_kontrak(request).await
    }

    pub async fn get_pengadaan_kontrak(
        &self,
        pengadaan_id: Uuid,
    ) -> AppResult<Vec<PengadaanKontrak>> {
        self.repo.get_pengadaan_kontrak(pengadaan_id).await
    }

    pub async fn create_pengadaan_bast(
        &self,
        request: CreatePengadaanBastRequest,
    ) -> AppResult<PengadaanBast> {
        request.validate()?;
        self.repo.create_pengadaan_bast(request).await
    }

    pub async fn get_pengadaan_bast(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanBast>> {
        self.repo.get_pengadaan_bast(pengadaan_id).await
    }

    pub async fn create_pengadaan_nodis(
        &self,
        request: CreatePengadaanNodisRequest,
    ) -> AppResult<PengadaanNodis> {
        request.validate()?;
        self.repo.create_pengadaan_nodis(request).await
    }

    pub async fn get_pengadaan_nodis(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanNodis>> {
        self.repo.get_pengadaan_nodis(pengadaan_id).await
    }

    // ============ Analisis Kebutuhan Services ============

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        self.repo.get_all_analisis(page, per_page).await
    }

    pub async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan> {
        self.repo.get_analisis_by_id(id).await
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

    pub async fn get_pemakaian_by_id(&self, id: Uuid) -> AppResult<Pemakaian> {
        self.repo.get_pemakaian_by_id(id).await
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

    pub async fn get_all_hibah(&self, page: i32, per_page: i32) -> AppResult<(Vec<Hibah>, i64)> {
        self.repo.get_all_hibah(page, per_page).await
    }

    pub async fn get_hibah_by_id(&self, id: Uuid) -> AppResult<Hibah> {
        self.repo.get_hibah_by_id(id).await
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

    pub async fn get_all_mutasi(&self, page: i32, per_page: i32) -> AppResult<(Vec<Mutasi>, i64)> {
        self.repo.get_all_mutasi(page, per_page).await
    }

    pub async fn get_mutasi_by_id(&self, id: Uuid) -> AppResult<Mutasi> {
        self.repo.get_mutasi_by_id(id).await
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

    pub async fn get_penghapusan_by_id(&self, id: Uuid) -> AppResult<Penghapusan> {
        self.repo.get_penghapusan_by_id(id).await
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

    pub async fn get_pengalihan_by_id(&self, id: Uuid) -> AppResult<Pengalihan> {
        self.repo.get_pengalihan_by_id(id).await
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

    pub async fn get_pemeliharaan_by_id(&self, id: Uuid) -> AppResult<Pemeliharaan> {
        self.repo.get_pemeliharaan_by_id(id).await
    }

    pub async fn create_pemeliharaan(
        &self,
        request: CreatePemeliharaanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pemeliharaan> {
        request.validate()?;
        self.repo.create_pemeliharaan(request, user_id).await
    }

    // ============ Export Services ============

    /// Queue an async export job for large datasets
    pub async fn queue_export_job(
        &self,
        query: crate::handlers::ExportQuery,
    ) -> AppResult<Uuid> {
        self.repo.queue_export_job(query).await
    }

    /// Export data to Excel synchronously (for small datasets)
    pub async fn export_to_excel_sync(
        &self,
        query: crate::handlers::ExportQuery,
    ) -> AppResult<Vec<u8>> {
        self.repo.export_to_excel_sync(query).await
    }

    /// Get export job status
    pub async fn get_export_job_status(
        &self,
        job_id: Uuid,
    ) -> AppResult<crate::handlers::ExportJobStatusResponse> {
        self.repo.get_export_job_status(job_id).await
    }

    /// Download completed export job
    pub async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)> {
        self.repo.download_export_job(job_id).await
    }
}
