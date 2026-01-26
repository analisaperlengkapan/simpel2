//! # Services Layer
//!
//! Business logic for the Perlengkapan service using tokio-postgres

use crate::{database::Database, errors::*, models::*};
use uuid::Uuid;
use validator::Validate;
use tracing::{info, error};
use layanan_integrasi::{
    client::MonsaktiClient,
    config::Config as IntegrasiConfig,
    siman::{fetch_all_aset_paginated, SimanAssetCategory},
};

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
                "SELECT id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by FROM perlengkapan.aset ORDER BY created_at DESC LIMIT $1 OFFSET $2",
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
            .query_opt("SELECT id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan::FLOAT8, tanggal_perolehan, status, keterangan, created_at, updated_at, created_by, updated_by FROM perlengkapan.aset WHERE id = $1", &[&id])
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
                "SELECT id, judul, deskripsi, jenis, status, anggaran::FLOAT8, target_selesai, pic_user_id, created_at, updated_at, created_by, updated_by FROM perlengkapan.pengadaan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
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
            .query_opt("SELECT id, judul, deskripsi, jenis, status, anggaran::FLOAT8, target_selesai, pic_user_id, created_at, updated_at, created_by, updated_by FROM perlengkapan.pengadaan WHERE id = $1", &[&id])
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
                "SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by FROM perlengkapan.analisis_kebutuhan ORDER BY created_at DESC LIMIT $1 OFFSET $2",
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

    // ============ Integration Services ============

    pub async fn sync_from_siman(&self) -> AppResult<String> {
        // Load config from environment
        let config = IntegrasiConfig::from_env()
            .map_err(|e| AppError::Internal(format!("Integration config error: {}", e)))?;

        // Initialize client
        let mut client = MonsaktiClient::new(config)
            .await
            .map_err(|e| AppError::Internal(format!("Integration client error: {}", e)))?;

        // Fetch Non-TIK assets
        let assets = fetch_all_aset_paginated(&mut client, SimanAssetCategory::NonTIK, 100)
            .await
            .map_err(|e| AppError::Internal(format!("Fetch error: {}", e)))?;

        let db_client = self.db.pool().get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        let mut count = 0;
        let mut error_count = 0;
        for asset in assets {
            if let Some(obj) = asset.as_object() {
                // Mapping logic
                let kode_barang = obj.get("KD_BRG").and_then(|v| v.as_str()).unwrap_or_default();
                let nup = obj.get("NO_ASET").and_then(|v| v.as_str()).unwrap_or_default();

                // Validation: Skip if both identifiers are empty to prevent "." kode_bmn
                if kode_barang.is_empty() || nup.is_empty() {
                    error_count += 1;
                    error!("Skipping asset with missing identifiers: KD_BRG='{}', NO_ASET='{}'", kode_barang, nup);
                    continue;
                }

                // Construct Unique Code BMN: KodeBarang.NUP
                let kode_bmn = format!("{}.{}", kode_barang, nup);

                let nama = obj.get("NM_BRG").and_then(|v| v.as_str()).unwrap_or("Unknown Asset");
                let merk = obj.get("MERK").and_then(|v| v.as_str());

                // Map kondisi from SIMAN (KONDISI) to local format
                let kondisi_raw = obj.get("KONDISI").and_then(|v| v.as_str()).unwrap_or("BAIK");
                let kondisi = match kondisi_raw.to_uppercase().as_str() {
                    "RUSAK BERAT" => "rusak berat",
                    "RUSAK RINGAN" => "rusak ringan",
                    _ => "baik", // Default to baik for BAIK or any unknown status
                };

                let lokasi = obj.get("NM_SATKER").and_then(|v| v.as_str()).unwrap_or("-");
                let nilai_perolehan = obj.get("RPH_ASET").and_then(|v| v.as_f64());

                let id = Uuid::new_v4();

                // Upsert logic
                // We use ON CONFLICT (kode_bmn) DO UPDATE
                // Use execute() instead of query() because there is no RETURNING clause
                match db_client.execute(
                    r#"
                    INSERT INTO perlengkapan.aset
                    (id, nama, kategori, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan, status, updated_at, created_at)
                    VALUES ($1, $2, 'Non TIK', $3, $4, $5, $6, $7, $8, 'aktif', NOW(), NOW())
                    ON CONFLICT (kode_bmn)
                    DO UPDATE SET
                        nama = EXCLUDED.nama,
                        merk = EXCLUDED.merk,
                        nup = EXCLUDED.nup,
                        kondisi = EXCLUDED.kondisi,
                        lokasi = EXCLUDED.lokasi,
                        nilai_perolehan = EXCLUDED.nilai_perolehan,
                        updated_at = NOW()
                    "#,
                    &[
                        &id,
                        &nama,
                        &kode_bmn,
                        &merk,
                        &Some(nup.to_string()),
                        &kondisi,
                        &lokasi,
                        &nilai_perolehan
                    ]
                ).await {
                    Ok(_) => count += 1,
                    Err(e) => {
                        error!("Failed to sync asset {}: {}", kode_bmn, e);
                        error_count += 1;
                    }
                }
            }
        }

        info!("Synced {} assets from SIMAN ({} errors)", count, error_count);
        Ok(format!("Synced {} assets from SIMAN ({} errors)", count, error_count))
    }
}
