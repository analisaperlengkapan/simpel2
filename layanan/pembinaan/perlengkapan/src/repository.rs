use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::*,
};

#[async_trait]
pub trait PerlengkapanRepository: Send + Sync {
    // Dashboard
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;

    // Aset
    async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)>;
    async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset>;
    async fn create_aset(&self, request: CreateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset>;
    async fn update_aset(&self, id: Uuid, request: UpdateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset>;
    async fn delete_aset(&self, id: Uuid) -> AppResult<()>;
    async fn check_aset_code_exists(&self, code: &str) -> AppResult<bool>;

    // Pengadaan
    async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)>;
    async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan>;
    async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan>;

    // Analisis
    async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
    async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;

    // Integration
    async fn upsert_siman_asset(&self, id: Uuid, nama: String, kategori: String, kode_bmn: String, merk: Option<String>, nup: String, kondisi: String, lokasi: String, nilai_perolehan: Option<f64>) -> AppResult<()>;
}
