//! # Services Layer
//!
//! Business logic for the Perlengkapan service

use crate::{database::Database, errors::*, models::*};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct PerlengkapanService {
    db: Database,
}

impl PerlengkapanService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // Dashboard Services
    pub async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let pool = self.db.pool();

        let total_aset: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.aset")
            .fetch_one(pool)
            .await?
            .get("count");

        let total_pengadaan: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.pengadaan")
            .fetch_one(pool)
            .await?
            .get("count");

        let total_analisis: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan")
            .fetch_one(pool)
            .await?
            .get("count");

        let aset_aktif: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.aset WHERE status = 'aktif'")
            .fetch_one(pool)
            .await?
            .get("count");

        let pengadaan_berjalan: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.pengadaan WHERE status IN ('perencanaan', 'proses', 'tender')")
            .fetch_one(pool)
            .await?
            .get("count");

        let analisis_pending: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan WHERE status IN ('draft', 'review')")
            .fetch_one(pool)
            .await?
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

    // Aset Services
    pub async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)> {
        let pool = self.db.pool();
        let offset = (page - 1) * per_page;

        let total: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.aset")
            .fetch_one(pool)
            .await?
            .get("count");

        let aset = sqlx::query_as::<_, Aset>(
            "SELECT * FROM perlengkapan.aset ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((aset, total))
    }

    pub async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset> {
        let pool = self.db.pool();

        let aset = sqlx::query_as::<_, Aset>("SELECT * FROM perlengkapan.aset WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        aset.ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    pub async fn create_aset(&self, request: CreateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset> {
        request.validate()?;

        let pool = self.db.pool();
        let id = Uuid::new_v4();

        // Check if kode_bmn already exists
        let existing = sqlx::query("SELECT id FROM perlengkapan.aset WHERE kode_bmn = $1")
            .bind(&request.kode_bmn)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(conflict("Kode BMN sudah digunakan"));
        }

        let aset = sqlx::query_as::<_, Aset>(r#"
            INSERT INTO perlengkapan.aset
            (id, nama, kategori, kode_bmn, kondisi, lokasi, nilai_perolehan, tanggal_perolehan, keterangan, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
        "#)
        .bind(id)
        .bind(&request.nama)
        .bind(&request.kategori)
        .bind(&request.kode_bmn)
        .bind(&request.kondisi)
        .bind(&request.lokasi)
        .bind(&request.nilai_perolehan)
        .bind(&request.tanggal_perolehan)
        .bind(&request.keterangan)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(aset)
    }

    pub async fn update_aset(&self, id: Uuid, request: UpdateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset> {
        request.validate()?;

        let pool = self.db.pool();

        // Check if aset exists
        self.get_aset_by_id(id).await?;

        let aset = sqlx::query_as::<_, Aset>(r#"
            UPDATE perlengkapan.aset
            SET nama = COALESCE($2, nama),
                kategori = COALESCE($3, kategori),
                kondisi = COALESCE($4, kondisi),
                lokasi = COALESCE($5, lokasi),
                nilai_perolehan = COALESCE($6, nilai_perolehan),
                tanggal_perolehan = COALESCE($7, tanggal_perolehan),
                status = COALESCE($8, status),
                keterangan = COALESCE($9, keterangan),
                updated_by = $10,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#)
        .bind(id)
        .bind(&request.nama)
        .bind(&request.kategori)
        .bind(&request.kondisi)
        .bind(&request.lokasi)
        .bind(&request.nilai_perolehan)
        .bind(&request.tanggal_perolehan)
        .bind(&request.status)
        .bind(&request.keterangan)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(aset)
    }

    pub async fn delete_aset(&self, id: Uuid) -> AppResult<()> {
        let pool = self.db.pool();

        let result = sqlx::query("DELETE FROM perlengkapan.aset WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(not_found("Aset", &id.to_string()));
        }

        Ok(())
    }

    // Pengadaan Services
    pub async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)> {
        let pool = self.db.pool();
        let offset = (page - 1) * per_page;

        let total: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.pengadaan")
            .fetch_one(pool)
            .await?
            .get("count");

        let pengadaan = sqlx::query_as::<_, Pengadaan>(
            "SELECT * FROM perlengkapan.pengadaan ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((pengadaan, total))
    }

    pub async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan> {
        let pool = self.db.pool();

        let pengadaan = sqlx::query_as::<_, Pengadaan>("SELECT * FROM perlengkapan.pengadaan WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        pengadaan.ok_or_else(|| not_found("Pengadaan", &id.to_string()))
    }

    pub async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan> {
        request.validate()?;

        let pool = self.db.pool();
        let id = Uuid::new_v4();

        let pengadaan = sqlx::query_as::<_, Pengadaan>(r#"
            INSERT INTO perlengkapan.pengadaan
            (id, judul, deskripsi, jenis, anggaran, target_selesai, pic_user_id, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#)
        .bind(id)
        .bind(&request.judul)
        .bind(&request.deskripsi)
        .bind(&request.jenis)
        .bind(&request.anggaran)
        .bind(&request.target_selesai)
        .bind(&request.pic_user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(pengadaan)
    }

    // Analisis Kebutuhan Services
    pub async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let pool = self.db.pool();
        let offset = (page - 1) * per_page;

        let total: i64 = sqlx::query("SELECT COUNT(*) as count FROM perlengkapan.analisis_kebutuhan")
            .fetch_one(pool)
            .await?
            .get("count");

        let analisis = sqlx::query_as::<_, AnalisisKebutuhan>(
            "SELECT * FROM perlengkapan.analisis_kebutuhan ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((analisis, total))
    }

    pub async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;

        let pool = self.db.pool();
        let id = Uuid::new_v4();

        let analisis = sqlx::query_as::<_, AnalisisKebutuhan>(r#"
            INSERT INTO perlengkapan.analisis_kebutuhan
            (id, judul, kategori, deskripsi, prioritas, estimasi_biaya, justifikasi, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#)
        .bind(id)
        .bind(&request.judul)
        .bind(&request.kategori)
        .bind(&request.deskripsi)
        .bind(&request.prioritas)
        .bind(&request.estimasi_biaya)
        .bind(&request.justifikasi)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(analisis)
    }
}
