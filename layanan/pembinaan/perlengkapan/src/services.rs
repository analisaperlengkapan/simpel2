//! # Services Layer
//!
//! Business logic for the Perlengkapan service using tokio-postgres

use crate::{database::Database, errors::*, models::*};
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

    // ============ Dashboard Services ============

    pub async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let total_aset: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.aset", &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let total_pengadaan: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.pengadaan", &[])
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

    // ============ Aset Services ============

    pub async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.aset", &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT * FROM perlengkapan.aset ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let aset: Vec<Aset> = rows.iter().map(Aset::from_row).collect();

        Ok((aset, total))
    }

    pub async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let row = client
            .query_opt("SELECT * FROM perlengkapan.aset WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| Aset::from_row(&r))
            .ok_or_else(|| not_found("Aset", &id.to_string()))
    }

    pub async fn create_aset(
        &self,
        request: CreateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        request.validate()?;

        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let id = Uuid::new_v4();

        // Check if kode_bmn already exists
        let existing = client
            .query_opt(
                "SELECT id FROM perlengkapan.aset WHERE kode_bmn = $1",
                &[&request.kode_bmn],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing.is_some() {
            return Err(conflict("Kode BMN sudah digunakan"));
        }

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.aset
                (id, nama, kategori, kode_bmn, kondisi, lokasi, nilai_perolehan, tanggal_perolehan, keterangan, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.kategori,
                    &request.kode_bmn,
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
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(Aset::from_row(&row))
    }

    pub async fn update_aset(
        &self,
        id: Uuid,
        request: UpdateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        request.validate()?;

        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        // Check if aset exists
        self.get_aset_by_id(id).await?;

        let row = client
            .query_one(
                r#"
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
                "#,
                &[
                    &id,
                    &request.nama,
                    &request.kategori,
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

    pub async fn delete_aset(&self, id: Uuid) -> AppResult<()> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let result = client
            .execute("DELETE FROM perlengkapan.aset WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result == 0 {
            return Err(not_found("Aset", &id.to_string()));
        }

        Ok(())
    }

    // ============ Pengadaan Services ============

    pub async fn get_all_pengadaan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pengadaan>, i64)> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let offset = (page - 1) * per_page;

        let total: i64 = client
            .query_one("SELECT COUNT(*) as count FROM perlengkapan.pengadaan", &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("count");

        let rows = client
            .query(
                "SELECT * FROM perlengkapan.pengadaan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let pengadaan: Vec<Pengadaan> = rows.iter().map(Pengadaan::from_row).collect();

        Ok((pengadaan, total))
    }

    pub async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let row = client
            .query_opt("SELECT * FROM perlengkapan.pengadaan WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| Pengadaan::from_row(&r))
            .ok_or_else(|| not_found("Pengadaan", &id.to_string()))
    }

    pub async fn create_pengadaan(
        &self,
        request: CreatePengadaanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pengadaan> {
        request.validate()?;

        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let id = Uuid::new_v4();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengadaan
                (id, judul, deskripsi, jenis, anggaran, target_selesai, pic_user_id, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
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

    // ============ Analisis Kebutuhan Services ============

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

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
                "SELECT * FROM perlengkapan.analisis_kebutuhan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let analisis: Vec<AnalisisKebutuhan> =
            rows.iter().map(AnalisisKebutuhan::from_row).collect();

        Ok((analisis, total))
    }

    pub async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;

        let client =
            self.db.pool().get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let id = Uuid::new_v4();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.analisis_kebutuhan
                (id, judul, kategori, deskripsi, prioritas, estimasi_biaya, justifikasi, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
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
