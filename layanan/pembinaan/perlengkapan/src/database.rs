//! # Database Connection
//!
//! Database connection and migration management using deadpool-postgres

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    errors::{conflict, not_found, AppError, AppResult},
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

        // Create aset table
        client
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS perlengkapan.aset (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                nama VARCHAR NOT NULL,
                kategori VARCHAR NOT NULL,
                kode_bmn VARCHAR UNIQUE NOT NULL,
                merk VARCHAR,
                nup VARCHAR,
                kondisi VARCHAR NOT NULL DEFAULT 'baik',
                lokasi VARCHAR NOT NULL,
                nilai_perolehan DECIMAL(15,2),
                tanggal_perolehan DATE,
                status VARCHAR NOT NULL DEFAULT 'aktif',
                keterangan TEXT,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                created_by UUID,
                updated_by UUID
            )
        "#,
                &[],
            )
            .await?;

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

        let total_aset: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.aset", &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let total_pengadaan: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.pengadaan",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let total_analisis: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let aset_aktif: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.aset WHERE status = 'aktif'",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let pengadaan_berjalan: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.pengadaan WHERE status IN ('perencanaan', 'proses', 'tender')",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let analisis_pending: i64 = client
            .query_one(
                "SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan WHERE status IN ('draft', 'review')",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        Ok(DashboardStats {
            total_aset,
            total_pengadaan,
            total_analisis,
            aset_aktif,
            pengadaan_berjalan,
            analisis_pending,
        })
    }

    async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.aset", &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by FROM perlengkapan.aset ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let aset: Vec<Aset> = rows.iter().map(Aset::from_row).collect();

        Ok((aset, total))
    }

    async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let row = client
            .query_opt("SELECT id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by FROM perlengkapan.aset WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| Aset::from_row(&r))
            .ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    async fn create_aset(
        &self,
        request: CreateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let id = Uuid::new_v4();

        // Check is done in Service if necessary, or handled by DB unique constraint
        // But for repo, we just try to insert. If unique violation, we catch it.
        // However, the original service checked explicitly. Let's move the check to `check_aset_code_exists`.

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.aset
                (id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan, tanggal_perolehan, keterangan, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.kategori,
                    &request.kode_bmn,
                    &request.merk,
                    &request.nup,
                    &request.kondisi,
                    &request.lokasi,
                    &request.nilai_perolehan,
                    &request.tanggal_perolehan,
                    &request.keterangan,
                    &user_id,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| {
                if e.code().map(|c| c.code() == "23505").unwrap_or(false) { // Unique violation
                     conflict("Kode BMN sudah digunakan")
                } else {
                     AppError::Database(e.to_string())
                }
            })?;

        Ok(Aset::from_row(&row))
    }

    async fn check_aset_code_exists(&self, code: &str) -> AppResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let existing = client
            .query_opt(
                "SELECT id FROM perlengkapan.aset WHERE kode_bmn = $1",
                &[&code],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(existing.is_some())
    }

    async fn update_aset(
        &self,
        id: Uuid,
        request: UpdateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.aset
                SET nama = COALESCE($2, nama),
                    kategori = COALESCE($3, kategori),
                    merk = COALESCE($4, merk),
                    nup = COALESCE($5, nup),
                    kondisi = COALESCE($6, kondisi),
                    lokasi = COALESCE($7, lokasi),
                    nilai_perolehan = COALESCE($8, nilai_perolehan),
                    tanggal_perolehan = COALESCE($9, tanggal_perolehan),
                    status = COALESCE($10, status),
                    keterangan = COALESCE($11, keterangan),
                    updated_by = $12,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.kategori,
                    &request.merk,
                    &request.nup,
                    &request.kondisi,
                    &request.lokasi,
                    &request.nilai_perolehan,
                    &request.tanggal_perolehan,
                    &request.status,
                    &request.keterangan,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(Aset::from_row(&row))
    }

    async fn delete_aset(&self, id: Uuid) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        let result = client
            .execute("DELETE FROM perlengkapan.aset WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result == 0 {
            return Err(not_found("Aset", &id.to_string()));
        }

        Ok(())
    }

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

    async fn upsert_siman_asset(&self, id: Uuid, nama: String, kategori: String, kode_bmn: String, merk: Option<String>, nup: String, kondisi: String, lokasi: String, nilai_perolehan: Option<f64>) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

        client.execute(
            r#"
            INSERT INTO perlengkapan.aset
            (id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan, status, updated_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'aktif', NOW(), NOW())
            ON CONFLICT (kode_bmn)
            DO UPDATE SET
                nama = EXCLUDED.nama,
                merk = EXCLUDED.merk,
                nup = EXCLUDED.nup,
                kondisi = EXCLUDED.kondisi,
                lokasi = EXCLUDED.lokasi,
                nilai_perolehan = EXCLUDED.nilai_perolehan,
                kategori = EXCLUDED.kategori,
                updated_at = NOW()
            "#,
            &[
                &id,
                &nama,
                &kategori,
                &kode_bmn,
                &merk,
                &Some(nup),
                &kondisi,
                &lokasi,
                &nilai_perolehan
            ]
        ).await.map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
