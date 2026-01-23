//! # Database Connection
//!
//! Database connection and migration management using deadpool-postgres

use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::info;

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
