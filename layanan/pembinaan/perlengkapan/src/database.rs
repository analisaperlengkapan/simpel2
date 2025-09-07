//! # Database Connection
//!
//! Database connection and migration management

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;

        // Test the connection
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await?;

        tracing::info!("Database connection established");

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        // Here we would run database migrations
        // For now, let's create basic tables
        self.create_tables().await?;
        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        // Create schema if not exists
        sqlx::query("CREATE SCHEMA IF NOT EXISTS perlengkapan")
            .execute(&self.pool)
            .await?;

        // Create aset table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS perlengkapan.aset (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                nama VARCHAR NOT NULL,
                kategori VARCHAR NOT NULL,
                kode_bmn VARCHAR UNIQUE NOT NULL,
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
        "#)
        .execute(&self.pool)
        .await?;

        // Create pengadaan table
        sqlx::query(r#"
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
        "#)
        .execute(&self.pool)
        .await?;

        // Create analisis table
        sqlx::query(r#"
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
        "#)
        .execute(&self.pool)
        .await?;

        tracing::info!("Database tables created successfully");
        Ok(())
    }
}
