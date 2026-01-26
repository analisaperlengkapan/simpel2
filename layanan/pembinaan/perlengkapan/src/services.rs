//! # Services Layer
//!
//! Business logic for the Perlengkapan service

use crate::{errors::*, models::*, repository::PerlengkapanRepository};
use std::sync::Arc;
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

    // ============ Aset Services ============

    pub async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)> {
        self.repo.get_all_aset(page, per_page).await
    }

    pub async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset> {
        self.repo.get_aset_by_id(id).await
    }

    pub async fn create_aset(
        &self,
        request: CreateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        request.validate()?;

        // Business logic: Check if code exists
        if self.repo.check_aset_code_exists(&request.kode_bmn).await? {
            return Err(conflict("Kode BMN sudah digunakan"));
        }

        self.repo.create_aset(request, user_id).await
    }

    pub async fn update_aset(
        &self,
        id: Uuid,
        request: UpdateAsetRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Aset> {
        request.validate()?;
        // Check existence
        self.get_aset_by_id(id).await?;
        self.repo.update_aset(id, request, user_id).await
    }

    pub async fn delete_aset(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete_aset(id).await
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

    // ============ Analisis Kebutuhan Services ============

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        self.repo.get_all_analisis(page, per_page).await
    }

    pub async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;
        self.repo.create_analisis(request, user_id).await
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

    pub async fn process_siman_assets(
        &self,
        assets: Vec<Value>,
        category_label: &str,
    ) -> AppResult<(usize, usize)> {
        let mut count = 0;
        let mut error_count = 0;

        for asset in assets {
            if let Some((nama, kode_bmn, merk, nup, kondisi, lokasi, nilai_perolehan)) =
                Self::parse_siman_asset(&asset)
            {
                let id = Uuid::new_v4();

                match self.repo.upsert_siman_asset(
                    id, nama, category_label.to_string(), kode_bmn.clone(), merk, nup, kondisi, lokasi, nilai_perolehan
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
