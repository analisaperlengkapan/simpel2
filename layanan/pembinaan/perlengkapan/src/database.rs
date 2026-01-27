//! # Database Connection
//!
//! Database connection and migration management using deadpool-postgres

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::{not_found, AppError, AppResult},
    models::*,
    repository::PerlengkapanRepository,
};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut config = Config::new();
        config.url = Some(database_url.to_string());

        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;

        // Test the connection
        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;

        info!("Database connection established");

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        // Here we would run database migrations
        // For now, let's create basic tables
        self.create_tables().await?;
        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        let client = self.pool.get().await?;

        // Create schema if not exists
        client
            .execute("CREATE SCHEMA IF NOT EXISTS perlengkapan", &[])
            .await?;

        // We do NOT create perlengkapan.aset anymore, as we use integrasi.siman_aset

        // Create pengadaan table
        client
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS perlengkapan.pengadaan (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                judul VARCHAR NOT NULL,
                deskripsi TEXT,
                jenis VARCHAR NOT NULL,
                status VARCHAR NOT NULL DEFAULT 'perencanaan',
                anggaran DECIMAL(15,2),
                target_selesai DATE,
                pic_user_id UUID,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                created_by UUID,
                updated_by UUID
            )
        "#,
                &[],
            )
            .await?;

        // Create analisis table
        client
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS perlengkapan.analisis_kebutuhan (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                judul VARCHAR NOT NULL,
                kategori VARCHAR NOT NULL,
                deskripsi TEXT,
                prioritas VARCHAR NOT NULL DEFAULT 'sedang',
                status VARCHAR NOT NULL DEFAULT 'draft',
                estimasi_biaya DECIMAL(15,2),
                justifikasi TEXT,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                created_by UUID,
                updated_by UUID
            )
        "#,
                &[],
            )
            .await?;

        info!("Database tables created successfully");
        Ok(())
    }
}

#[async_trait]
impl PerlengkapanRepository for Database {
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        // Query v_siman_summary_total with explicit casts to be safe.
        let summary = client
             .query_one(
                 "SELECT
                    total_aset,
                    COALESCE(total_nilai_perolehan, 0)::FLOAT8 as total_nilai,
                    total_satker,
                    total_baik,
                    total_rusak
                  FROM integrasi.v_siman_summary_total",
                 &[]
             )
             .await
             .map_err(|e| AppError::Database(format!("Failed to query summary with cast: {}", e)))?;

        let total_aset: i64 = summary.get("total_aset");
        let total_nilai_aset: f64 = summary.get("total_nilai");
        let total_satker: i64 = summary.get("total_satker");
        let aset_baik: i64 = summary.get("total_baik");
        let aset_rusak: i64 = summary.get("total_rusak");

        // Categories
        let cat_rows = client.query(
            "SELECT kategori_aset, total_aset, COALESCE(total_nilai_perolehan, 0)::FLOAT8 as total_nilai FROM integrasi.v_siman_summary_per_kategori ORDER BY total_aset DESC",
            &[]
        ).await.map_err(|e| AppError::Database(format!("Failed to query categories: {}", e)))?;

        let categories = cat_rows.iter().map(|row| CategoryStat {
            category: row.get("kategori_aset"),
            count: row.get("total_aset"),
            value: row.get("total_nilai"),
        }).collect();

        Ok(DashboardStats {
            total_aset,
            total_nilai_aset,
            total_satker,
            aset_baik,
            aset_rusak,
            categories,
        })
    }

    async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let offset = (page - 1) * per_page;

        // Easier way:
        let total: i64;
        if let Some(cat) = &category {
            total = client.query_one("SELECT COUNT(*) as count FROM integrasi.siman_aset WHERE kategori_aset = $1", &[cat])
                .await.map_err(|e| AppError::Database(e.to_string()))?.get("count");
        } else {
             total = client.query_one("SELECT COUNT(*) as count FROM integrasi.siman_aset", &[])
                .await.map_err(|e| AppError::Database(e.to_string()))?.get("count");
        }

        let query_str = if category.is_some() {
            "SELECT id, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, rph_aset::FLOAT8, tgl_perlh, updated_at
             FROM integrasi.siman_aset
             WHERE kategori_aset = $1
             ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
        } else {
             "SELECT id, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, rph_aset::FLOAT8, tgl_perlh, updated_at
             FROM integrasi.siman_aset
             ORDER BY updated_at DESC LIMIT $1 OFFSET $2"
        };

        let rows = if let Some(cat) = &category {
            client.query(query_str, &[cat, &(per_page as i64), &(offset as i64)]).await
        } else {
             client.query(query_str, &[&(per_page as i64), &(offset as i64)]).await
        }.map_err(|e| AppError::Database(e.to_string()))?;

        let assets: Vec<Asset> = rows.iter().map(Asset::from_row).collect();

        Ok((assets, total))
    }

    async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let row = client
            .query_opt("SELECT id, kategori_aset, no_aset, ur_sskel, nama, kd_brg, merk, tipe, ur_kondisi, alamat, nama_satker, rph_aset::FLOAT8, tgl_perlh, updated_at
                        FROM integrasi.siman_aset WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| Asset::from_row(&r))
            .ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    // Pengadaan & Analisis remain same
    async fn get_all_pengadaan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pengadaan>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.pengadaan",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT id, judul, deskripsi, jenis, status, anggaran::FLOAT8, target_selesai, pic_user_id, created_at, updated_at, created_by, updated_by FROM perlengkapan.pengadaan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let pengadaan: Vec<Pengadaan> = rows.iter().map(Pengadaan::from_row).collect();

        Ok((pengadaan, total))
    }

    async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let row = client
            .query_opt("SELECT id, judul, deskripsi, jenis, status, anggaran::FLOAT8, target_selesai, pic_user_id, created_at, updated_at, created_by, updated_by FROM perlengkapan.pengadaan WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| Pengadaan::from_row(&r))
            .ok_or_else(|| not_found("Pengadaan", &id.to_string()))
    }

    async fn create_pengadaan(
        &self,
        request: CreatePengadaanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pengadaan> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let id = Uuid::new_v4();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengadaan
                (id, judul, deskripsi, jenis, anggaran, target_selesai, pic_user_id, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, judul, deskripsi, jenis, status, anggaran::FLOAT8, target_selesai, pic_user_id, created_at, updated_at, created_by, updated_by
                "#,
                &[
                    &id,
                    &request.judul,
                    &request.deskripsi,
                    &request.jenis,
                    &request.anggaran,
                    &request.target_selesai,
                    &request.pic_user_id,
                    &user_id,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(Pengadaan::from_row(&row))
    }

    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by FROM perlengkapan.analisis_kebutuhan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let analisis: Vec<AnalisisKebutuhan> =
            rows.iter().map(AnalisisKebutuhan::from_row).collect();

        Ok((analisis, total))
    }

    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let id = Uuid::new_v4();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.analisis_kebutuhan
                (id, judul, kategori, deskripsi, prioritas, estimasi_biaya, justifikasi, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by
                "#,
                &[
                    &id,
                    &request.judul,
                    &request.kategori,
                    &request.deskripsi,
                    &request.prioritas,
                    &request.estimasi_biaya,
                    &request.justifikasi,
                    &user_id,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(AnalisisKebutuhan::from_row(&row))
    }
}
