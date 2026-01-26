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
use serde_json::Value;

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

    fn parse_siman_asset(
        asset: &Value,
    ) -> Option<(
        String,         // nama
        String,         // kode_bmn
        Option<String>, // merk
        String,         // nup
        String,         // kondisi
        String,         // lokasi
        Option<f64>,    // nilai_perolehan
    )> {
        let obj = asset.as_object()?;

        // Helper to extract string from either String or Number/Int
        let get_string = |key: &str| -> Option<String> {
            obj.get(key).and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_i64() {
                    Some(n.to_string())
                } else if let Some(f) = v.as_f64() {
                    Some(f.to_string())
                } else {
                    None
                }
            })
        };

        let kode_barang = get_string("KD_BRG").unwrap_or_default();
        let nup = get_string("NO_ASET").unwrap_or_default();

        if kode_barang.is_empty() || nup.is_empty() {
            return None;
        }

        let kode_bmn = format!("{}.{}", kode_barang, nup);
        let nama = get_string("NM_BRG").unwrap_or_else(|| "Unknown Asset".to_string());
        let merk = get_string("MERK");
        let kondisi_raw = get_string("KONDISI").unwrap_or_else(|| "BAIK".to_string());
        let kondisi = match kondisi_raw.to_uppercase().as_str() {
            "RUSAK BERAT" => "rusak berat".to_string(),
            "RUSAK RINGAN" => "rusak ringan".to_string(),
            _ => "baik".to_string(),
        };
        let lokasi = get_string("NM_SATKER").unwrap_or_else(|| "-".to_string());

        let nilai_perolehan = obj.get("RPH_ASET").and_then(|v| {
            if let Some(f) = v.as_f64() {
                Some(f)
            } else if let Some(i) = v.as_i64() {
                Some(i as f64)
            } else {
                None
            }
        });

        Some((
            nama,
            kode_bmn,
            merk,
            nup,
            kondisi,
            lokasi,
            nilai_perolehan,
        ))
    }

    async fn process_siman_assets(
        &self,
        assets: Vec<Value>,
        category_label: &str,
    ) -> AppResult<(usize, usize)> {
        let db_client = self.db.pool().get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get database connection: {}", e))
        })?;

        let mut count = 0;
        let mut error_count = 0;

        for asset in assets {
            if let Some((nama, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan)) =
                Self::parse_siman_asset(&asset)
            {
                let id = Uuid::new_v4();

                match db_client.execute(
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
                        &category_label,
                        &kode_bmn,
                        &merk,
                        &Some(nup),
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
            } else {
                error_count += 1;
            }
        }
        Ok((count, error_count))
    }

    pub async fn sync_from_siman(&self) -> AppResult<String> {
        // Load config from environment
        let config = IntegrasiConfig::from_env()
            .map_err(|e| AppError::Internal(format!("Integration config error: {}", e)))?;

        // Initialize client
        let mut client = MonsaktiClient::new(config)
            .await
            .map_err(|e| AppError::Internal(format!("Integration client error: {}", e)))?;

        let categories = vec![
            (SimanAssetCategory::Tanah, "Tanah"),
            (SimanAssetCategory::GedungBangunan, "Gedung dan Bangunan"),
            (SimanAssetCategory::JalandanJembatan, "Jalan dan Jembatan"),
            (SimanAssetCategory::AlatBesar, "Alat Besar"),
            (SimanAssetCategory::AngkutanBermotor, "Angkutan Bermotor"),
            (SimanAssetCategory::NonTIK, "Non TIK"),
            (SimanAssetCategory::InstalasiJaringan, "Instalasi dan Jaringan"),
            (SimanAssetCategory::AlatPersenjataan, "Alat Persenjataan"),
            (SimanAssetCategory::Rumah, "Rumah Negara"),
            (SimanAssetCategory::TakBerwujud, "Aset Tak Berwujud"),
            (SimanAssetCategory::TetapLainnya, "Aset Tetap Lainnya"),
        ];

        let mut total_synced = 0;
        let mut total_errors = 0;

        for (category, label) in categories {
            info!("Syncing category: {}", label);
            match fetch_all_aset_paginated(&mut client, category, 100).await {
                Ok(assets) => {
                     match self.process_siman_assets(assets, label).await {
                        Ok((c, e)) => {
                            total_synced += c;
                            total_errors += e;
                        }
                        Err(e) => {
                            error!("Error processing assets for {}: {}", label, e);
                            total_errors += 1;
                        }
                     }
                }
                Err(e) => {
                    error!("Error fetching assets for {}: {}", label, e);
                    total_errors += 1;
                }
            }
        }

        info!("Synced total {} assets from SIMAN ({} errors)", total_synced, total_errors);
        Ok(format!("Synced {} assets from SIMAN ({} errors)", total_synced, total_errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_siman_asset_success() {
        let json_data = json!({
            "KD_BRG": "101",
            "NO_ASET": "1",
            "NM_BRG": "Laptop",
            "MERK": "Dell",
            "KONDISI": "BAIK",
            "NM_SATKER": "Pusat",
            "RPH_ASET": 15000000
        });

        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert!(result.is_some());
        let (nama, kode_bmn, merk, nup, kondisi, lokasi, nilai) = result.unwrap();

        assert_eq!(nama, "Laptop");
        assert_eq!(kode_bmn, "101.1");
        assert_eq!(merk, Some("Dell".to_string()));
        assert_eq!(nup, "1");
        assert_eq!(kondisi, "baik");
        assert_eq!(lokasi, "Pusat");
        assert_eq!(nilai, Some(15000000.0));
    }

    #[test]
    fn test_parse_siman_asset_missing_ids() {
        let json_data = json!({
            "NM_BRG": "Laptop",
        });

        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_siman_asset_kondisi_mapping() {
         let json_data = json!({
            "KD_BRG": "101",
            "NO_ASET": "1",
            "KONDISI": "RUSAK BERAT",
        });

        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert_eq!(result.unwrap().4, "rusak berat");
    }

    #[test]
    fn test_parse_siman_asset_numeric_fields() {
        let json_data = json!({
            "KD_BRG": 101, // Integer
            "NO_ASET": 5,  // Integer
            "NM_BRG": "Numeric Asset",
            "RPH_ASET": 50000 // Integer
        });

        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert!(result.is_some());
        let (_, kode_bmn, _, nup, _, _, nilai) = result.unwrap();

        assert_eq!(kode_bmn, "101.5");
        assert_eq!(nup, "5");
        assert_eq!(nilai, Some(50000.0));
    }

    #[test]
    fn test_parse_siman_asset_invalid_input() {
        // Not an object
        let json_data = json!(["item1", "item2"]);
        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert!(result.is_none());

        // Null
        let json_data = json!(null);
        let result = PerlengkapanService::parse_siman_asset(&json_data);
        assert!(result.is_none());
    }
}
