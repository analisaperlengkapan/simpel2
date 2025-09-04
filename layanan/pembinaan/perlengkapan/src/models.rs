//! # Data Models for Perlengkapan Service

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use validator::Validate;
use bigdecimal::BigDecimal;

// Aset Models
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Aset {
    pub id: Uuid,
    pub nama: String,
    pub kategori: String,
    pub kode_bmn: String,
    pub kondisi: String,
    pub lokasi: String,
    pub nilai_perolehan: Option<BigDecimal>,
    pub tanggal_perolehan: Option<NaiveDate>,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAsetRequest {
    #[validate(length(min = 1, max = 255))]
    pub nama: String,
    #[validate(length(min = 1, max = 100))]
    pub kategori: String,
    #[validate(length(min = 1, max = 50))]
    pub kode_bmn: String,
    #[validate(length(min = 1, max = 50))]
    pub kondisi: String,
    #[validate(length(min = 1, max = 255))]
    pub lokasi: String,
    pub nilai_perolehan: Option<BigDecimal>,
    pub tanggal_perolehan: Option<NaiveDate>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateAsetRequest {
    #[validate(length(min = 1, max = 255))]
    pub nama: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub kategori: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub kondisi: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub lokasi: Option<String>,
    pub nilai_perolehan: Option<BigDecimal>,
    pub tanggal_perolehan: Option<NaiveDate>,
    #[validate(length(min = 1, max = 50))]
    pub status: Option<String>,
    pub keterangan: Option<String>,
}

// Dashboard Models
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub aset_aktif: i64,
}

// API Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i32, per_page: i32, message: String) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

        Self {
            success: true,
            message,
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
