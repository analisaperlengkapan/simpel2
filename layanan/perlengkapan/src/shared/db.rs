//! # Database Connection
//!
//! Database connection and migration management using deadpool-postgres

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::repository::PerlengkapanRepository;
use crate::shared::error::{AppError, AppResult, not_found};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Use optimized configuration from lib-common
        let db_config = lib_backend::db::DbConfig::new(database_url.to_string())
            .with_max_size(50) // Max 50 connections (NFR-SC001)
            .with_min_idle(10) // Min 10 idle connections
            .with_connection_timeout(std::time::Duration::from_secs(30))
            .with_idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes
            .with_max_lifetime(std::time::Duration::from_secs(1800)); // 30 minutes

        let pool = lib_backend::db::create_postgres_pool(db_config)?;

        // Test the connection
        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;

        info!("Database connection pool established (min: 10, max: 50)");

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get a database connection from the pool
    pub async fn get_connection(&self) -> Result<deadpool_postgres::Object, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Database(format!("Failed to get database connection: {}", e)))
    }

    /// Execute a query that returns exactly one row
    pub async fn query_one(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, AppError> {
        let client = self.get_connection().await?;
        client
            .query_one(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a query that returns zero or more rows
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, AppError> {
        let client = self.get_connection().await?;
        client
            .query(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a query that returns zero or one row
    pub async fn query_opt(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>, AppError> {
        let client = self.get_connection().await?;
        client
            .query_opt(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a statement that modifies data (INSERT, UPDATE, DELETE)
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, AppError> {
        let client = self.get_connection().await?;
        client
            .execute(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }
}

#[async_trait]
impl PerlengkapanRepository for Database {
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

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
                &[],
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

        let categories = cat_rows
            .iter()
            .map(|row| CategoryStat {
                category: row.get("kategori_aset"),
                count: row.get("total_aset"),
                value: row.get("total_nilai"),
            })
            .collect();

        Ok(DashboardStats {
            total_aset,
            total_nilai_aset,
            total_satker,
            aset_baik,
            aset_rusak,
            categories,
        })
    }

    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        let client =
            self.pool.get().await.map_err(|e| {
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

    async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let row = client
            .query_opt("SELECT id, judul, kategori, deskripsi, prioritas, status, estimasi_biaya::FLOAT8, justifikasi, created_at, updated_at, created_by, updated_by FROM perlengkapan.analisis_kebutuhan WHERE id = $1", &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|r| AnalisisKebutuhan::from_row(&r))
            .ok_or_else(|| not_found("Analisis Kebutuhan", &id.to_string()))
    }

    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        let client =
            self.pool.get().await.map_err(|e| {
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

    // ============================================================================
    // Export Implementation
    // ============================================================================

    async fn queue_export_job(&self, query: crate::handlers::ExportQuery) -> AppResult<Uuid> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let job_id = Uuid::new_v4();

        // Store job in database
        client
            .execute(
                r#"
                INSERT INTO perlengkapan.export_jobs
                (id, entity_type, filters, status, created_at)
                VALUES ($1, $2, $3, 'queued', NOW())
                "#,
                &[&job_id, &query.entity_type, &query.filters],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Spawn background task to process export
        let pool = self.pool.clone();
        tokio::spawn(async move {
            if let Err(e) = process_export_job_background(pool, job_id, query).await {
                tracing::error!("Export job {} failed: {}", job_id, e);
            }
        });

        Ok(job_id)
    }

    async fn export_to_excel_sync(
        &self,
        query: crate::handlers::ExportQuery,
    ) -> AppResult<Vec<u8>> {
        // Fetch data based on entity type
        let data = match query.entity_type.as_str() {
            "kebutuhan_bmn" => self.fetch_kebutuhan_bmn_for_export(&query).await?,
            "pakaian_dinas" => self.fetch_pakaian_dinas_for_export(&query).await?,
            "roadmap_sarpras" => self.fetch_roadmap_for_export(&query).await?,
            "riwayat_pemenuhan" => self.fetch_riwayat_for_export(&query).await?,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unknown entity type: {}",
                    query.entity_type
                )));
            }
        };

        // Generate Excel
        generate_excel(&query.entity_type, data)
    }

    async fn get_export_job_status(
        &self,
        job_id: Uuid,
    ) -> AppResult<crate::handlers::ExportJobStatusResponse> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let row = client
            .query_opt(
                r#"
                SELECT id, status, progress, document_id, error_message, created_at, completed_at
                FROM perlengkapan.export_jobs
                WHERE id = $1
                "#,
                &[&job_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Export job not found: {}", job_id)))?;

        Ok(crate::handlers::ExportJobStatusResponse {
            job_id: row.get("id"),
            status: row.get("status"),
            progress: row.get("progress"),
            document_id: row.get("document_id"),
            error_message: row.get("error_message"),
            created_at: row
                .get::<_, chrono::DateTime<chrono::Utc>>("created_at")
                .to_rfc3339(),
            completed_at: row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>("completed_at")
                .map(|dt| dt.to_rfc3339()),
        })
    }

    async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        // Get job status
        let row = client
            .query_opt(
                r#"
                SELECT status, document_id, entity_type
                FROM perlengkapan.export_jobs
                WHERE id = $1
                "#,
                &[&job_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Export job not found: {}", job_id)))?;

        let status: String = row.get("status");
        if status != "completed" {
            return Err(AppError::BadRequest(format!(
                "Export job is not completed. Current status: {}",
                status
            )));
        }

        let document_id: Option<Uuid> = row.get("document_id");
        let _document_id = document_id.ok_or_else(|| {
            AppError::Internal("Export job completed but no document_id found".to_string())
        })?;

        let entity_type: String = row.get("entity_type");

        // For now, we'll fetch the data again and generate Excel
        // In production, this should fetch from document storage
        let query = crate::handlers::ExportQuery {
            entity_type: entity_type.clone(),
            filters: None,
            limit: Some(50000),
            tahun_anggaran: None,
            satker_id: None,
            status: None,
        };

        let data = self.export_to_excel_sync(query).await?;
        let filename = format!("export_{}_{}.xlsx", entity_type, job_id);

        Ok((filename, data))
    }
}

// Helper function to process export job in background
async fn process_export_job_background(
    pool: deadpool_postgres::Pool,
    job_id: Uuid,
    query: crate::handlers::ExportQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update status to processing
    {
        let client = pool.get().await?;
        client
            .execute(
                r#"
                UPDATE perlengkapan.export_jobs
                SET status = 'processing', started_at = NOW()
                WHERE id = $1
                "#,
                &[&job_id],
            )
            .await?;
    }

    // Create database instance for export
    let db = Database { pool: pool.clone() };

    // Generate Excel
    let result = db.export_to_excel_sync(query).await;

    match result {
        Ok(_data) => {
            // Update status to completed
            let client = pool.get().await?;
            client
                .execute(
                    r#"
                    UPDATE perlengkapan.export_jobs
                    SET status = 'completed', completed_at = NOW(), progress = 100
                    WHERE id = $1
                    "#,
                    &[&job_id],
                )
                .await?;
        }
        Err(e) => {
            // Update status to failed
            let client = pool.get().await?;
            client
                .execute(
                    r#"
                    UPDATE perlengkapan.export_jobs
                    SET status = 'failed', error_message = $2, completed_at = NOW()
                    WHERE id = $1
                    "#,
                    &[&job_id, &e.to_string()],
                )
                .await?;
        }
    }

    Ok(())
}

// Helper function to generate Excel from data
fn generate_excel(entity_type: &str, data: Vec<serde_json::Value>) -> AppResult<Vec<u8>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Set worksheet name
    worksheet
        .set_name(entity_type)
        .map_err(|e| AppError::Internal(format!("Failed to set worksheet name: {}", e)))?;

    // Header format
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White);

    // Write headers based on entity type
    let headers = match entity_type {
        "kebutuhan_bmn" => vec![
            "No",
            "Satker",
            "Kode Barang",
            "Nama Barang",
            "Jumlah Kebutuhan",
            "Tahun Anggaran",
            "Status",
            "Created At",
        ],
        "pakaian_dinas" => vec![
            "No",
            "NIP",
            "Nama Pegawai",
            "Jenis Pakaian",
            "Ukuran",
            "Jumlah",
            "Tahun Anggaran",
            "Status",
        ],
        "roadmap_sarpras" => vec![
            "No",
            "Satker",
            "Kode Barang",
            "Tahun Rencana",
            "Jumlah Kebutuhan",
            "Jumlah Terpenuhi",
            "Estimasi Anggaran",
            "Status",
        ],
        "riwayat_pemenuhan" => vec![
            "No",
            "Satker",
            "Kode Barang",
            "Tahun Anggaran",
            "Jumlah Terpenuhi",
            "Sumber Data",
            "Tanggal Pemenuhan",
        ],
        _ => vec!["No", "Data"],
    };

    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col as u16, *header, &header_format)
            .map_err(|e| AppError::Internal(format!("Failed to write header: {}", e)))?;
    }

    // Write data rows
    for (idx, item) in data.iter().enumerate() {
        let row = (idx + 1) as u32;

        worksheet
            .write_number(row, 0, (idx + 1) as f64)
            .map_err(|e| AppError::Internal(format!("Failed to write row number: {}", e)))?;

        // Write data based on entity type
        match entity_type {
            "kebutuhan_bmn" => {
                write_string_safe(worksheet, row, 1, item.get("satker_nama"))?;
                write_string_safe(worksheet, row, 2, item.get("kode_barang"))?;
                write_string_safe(worksheet, row, 3, item.get("nama_barang"))?;
                write_number_safe(worksheet, row, 4, item.get("jumlah_kebutuhan"))?;
                write_number_safe(worksheet, row, 5, item.get("tahun_anggaran"))?;
                write_string_safe(worksheet, row, 6, item.get("status"))?;
                write_string_safe(worksheet, row, 7, item.get("created_at"))?;
            }
            "pakaian_dinas" => {
                write_string_safe(worksheet, row, 1, item.get("nip"))?;
                write_string_safe(worksheet, row, 2, item.get("nama_pegawai"))?;
                write_string_safe(worksheet, row, 3, item.get("jenis_pakaian"))?;
                write_string_safe(worksheet, row, 4, item.get("ukuran"))?;
                write_number_safe(worksheet, row, 5, item.get("jumlah"))?;
                write_number_safe(worksheet, row, 6, item.get("tahun_anggaran"))?;
                write_string_safe(worksheet, row, 7, item.get("status"))?;
            }
            "roadmap_sarpras" => {
                write_string_safe(worksheet, row, 1, item.get("satker_nama"))?;
                write_string_safe(worksheet, row, 2, item.get("kode_barang"))?;
                write_number_safe(worksheet, row, 3, item.get("tahun_rencana"))?;
                write_number_safe(worksheet, row, 4, item.get("jumlah_kebutuhan"))?;
                write_number_safe(worksheet, row, 5, item.get("jumlah_terpenuhi"))?;
                write_number_safe(worksheet, row, 6, item.get("estimasi_anggaran"))?;
                write_string_safe(worksheet, row, 7, item.get("status_pemenuhan"))?;
            }
            "riwayat_pemenuhan" => {
                write_string_safe(worksheet, row, 1, item.get("satker_nama"))?;
                write_string_safe(worksheet, row, 2, item.get("kode_barang"))?;
                write_number_safe(worksheet, row, 3, item.get("tahun_anggaran"))?;
                write_number_safe(worksheet, row, 4, item.get("jumlah_terpenuhi"))?;
                write_string_safe(worksheet, row, 5, item.get("sumber_data"))?;
                write_string_safe(worksheet, row, 6, item.get("tanggal_pemenuhan"))?;
            }
            _ => {}
        }
    }

    // Auto-fit columns
    worksheet.autofit();

    // Add metadata sheet
    let metadata_sheet = workbook.add_worksheet();
    metadata_sheet
        .set_name("Metadata")
        .map_err(|e| AppError::Internal(format!("Failed to set metadata sheet name: {}", e)))?;

    metadata_sheet
        .write_string(0, 0, "Export Date")
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;
    metadata_sheet
        .write_string(0, 1, chrono::Utc::now().to_rfc3339())
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;
    metadata_sheet
        .write_string(1, 0, "Total Rows")
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;
    metadata_sheet
        .write_number(1, 1, data.len() as f64)
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;
    metadata_sheet
        .write_string(2, 0, "Entity Type")
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;
    metadata_sheet
        .write_string(2, 1, entity_type)
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;

    // Save to buffer
    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(format!("Failed to save workbook: {}", e)))?;

    Ok(buffer)
}

// Helper functions for safe writing to Excel
fn write_string_safe(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: Option<&serde_json::Value>,
) -> AppResult<()> {
    let str_value = value.and_then(|v| v.as_str()).unwrap_or("");
    worksheet
        .write_string(row, col, str_value)
        .map_err(|e| AppError::Internal(format!("Failed to write string: {}", e)))?;
    Ok(())
}

fn write_number_safe(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: Option<&serde_json::Value>,
) -> AppResult<()> {
    let num_value = value.and_then(|v| v.as_f64()).unwrap_or(0.0);
    worksheet
        .write_number(row, col, num_value)
        .map_err(|e| AppError::Internal(format!("Failed to write number: {}", e)))?;
    Ok(())
}

// Helper methods for Database to fetch export data
impl Database {
    async fn fetch_kebutuhan_bmn_for_export(
        &self,
        query: &crate::handlers::ExportQuery,
    ) -> AppResult<Vec<serde_json::Value>> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let limit = query.limit.unwrap_or(50000).min(50000);

        // Build query with filters
        let mut sql = String::from(
            r#"
            SELECT
                k.id,
                s.nama as satker_nama,
                k.kode_barang,
                k.nama_barang,
                k.jumlah_kebutuhan,
                k.tahun_anggaran,
                k.status,
                k.created_at
            FROM perlengkapan.kebutuhan_bmn k
            LEFT JOIN authenc.satkers s ON k.satker_id = s.id
            WHERE 1=1
            "#,
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(tahun) = query.tahun_anggaran {
            sql.push_str(&format!(" AND k.tahun_anggaran = ${}", param_idx));
            params.push(Box::new(tahun));
            param_idx += 1;
        }

        if let Some(satker_id) = query.satker_id {
            sql.push_str(&format!(" AND k.satker_id = ${}", param_idx));
            params.push(Box::new(satker_id));
            param_idx += 1;
        }

        if let Some(ref status) = query.status {
            sql.push_str(&format!(" AND k.status = ${}", param_idx));
            params.push(Box::new(status.clone()));
            param_idx += 1;
        }

        sql.push_str(&format!(" ORDER BY k.created_at DESC LIMIT ${}", param_idx));
        params.push(Box::new(limit as i64));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&sql, &param_refs[..])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "satker_nama".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("satker_nama")
                        .unwrap_or_default(),
                ),
            );
            obj.insert(
                "kode_barang".to_string(),
                serde_json::Value::String(row.get("kode_barang")),
            );
            obj.insert(
                "nama_barang".to_string(),
                serde_json::Value::String(row.get("nama_barang")),
            );
            obj.insert(
                "jumlah_kebutuhan".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("jumlah_kebutuhan"),
                )),
            );
            obj.insert(
                "tahun_anggaran".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("tahun_anggaran"),
                )),
            );
            obj.insert(
                "status".to_string(),
                serde_json::Value::String(row.get("status")),
            );
            obj.insert(
                "created_at".to_string(),
                serde_json::Value::String(
                    row.get::<_, chrono::DateTime<chrono::Utc>>("created_at")
                        .to_rfc3339(),
                ),
            );
            results.push(serde_json::Value::Object(obj));
        }

        Ok(results)
    }

    async fn fetch_pakaian_dinas_for_export(
        &self,
        query: &crate::handlers::ExportQuery,
    ) -> AppResult<Vec<serde_json::Value>> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let limit = query.limit.unwrap_or(50000).min(50000);

        let rows = client
            .query(
                r#"
                SELECT
                    p.nip,
                    p.nama as nama_pegawai,
                    j.nama as jenis_pakaian,
                    u.nama as ukuran,
                    1 as jumlah,
                    EXTRACT(YEAR FROM CURRENT_DATE)::INTEGER as tahun_anggaran,
                    'ACTIVE' as status
                FROM perlengkapan.ukuran_pakaian_pegawai p
                LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON p.jenis_pakaian_id = j.id
                LEFT JOIN perlengkapan.ms_ukuran u ON p.ukuran_id = u.id
                ORDER BY p.created_at DESC
                LIMIT $1
                "#,
                &[&(limit as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            obj.insert("nip".to_string(), serde_json::Value::String(row.get("nip")));
            obj.insert(
                "nama_pegawai".to_string(),
                serde_json::Value::String(row.get("nama_pegawai")),
            );
            obj.insert(
                "jenis_pakaian".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("jenis_pakaian")
                        .unwrap_or_default(),
                ),
            );
            obj.insert(
                "ukuran".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("ukuran").unwrap_or_default(),
                ),
            );
            obj.insert(
                "jumlah".to_string(),
                serde_json::Value::Number(serde_json::Number::from(row.get::<_, i32>("jumlah"))),
            );
            obj.insert(
                "tahun_anggaran".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("tahun_anggaran"),
                )),
            );
            obj.insert(
                "status".to_string(),
                serde_json::Value::String(row.get("status")),
            );
            results.push(serde_json::Value::Object(obj));
        }

        Ok(results)
    }

    async fn fetch_roadmap_for_export(
        &self,
        query: &crate::handlers::ExportQuery,
    ) -> AppResult<Vec<serde_json::Value>> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let limit = query.limit.unwrap_or(50000).min(50000);

        let rows = client
            .query(
                r#"
                SELECT
                    s.nama as satker_nama,
                    r.kode_barang,
                    r.tahun_rencana,
                    r.jumlah_kebutuhan,
                    r.jumlah_terpenuhi,
                    COALESCE(r.estimasi_anggaran, 0)::FLOAT8 as estimasi_anggaran,
                    r.status_pemenuhan
                FROM perlengkapan.roadmap_sarpras r
                LEFT JOIN authenc.satkers s ON r.satker_id = s.id
                ORDER BY r.created_at DESC
                LIMIT $1
                "#,
                &[&(limit as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "satker_nama".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("satker_nama")
                        .unwrap_or_default(),
                ),
            );
            obj.insert(
                "kode_barang".to_string(),
                serde_json::Value::String(row.get("kode_barang")),
            );
            obj.insert(
                "tahun_rencana".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("tahun_rencana"),
                )),
            );
            obj.insert(
                "jumlah_kebutuhan".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("jumlah_kebutuhan"),
                )),
            );
            obj.insert(
                "jumlah_terpenuhi".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("jumlah_terpenuhi"),
                )),
            );

            if let Some(anggaran) = row.get::<_, Option<f64>>("estimasi_anggaran") {
                obj.insert(
                    "estimasi_anggaran".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(anggaran)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }

            obj.insert(
                "status_pemenuhan".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("status_pemenuhan")
                        .unwrap_or_default(),
                ),
            );
            results.push(serde_json::Value::Object(obj));
        }

        Ok(results)
    }

    async fn fetch_riwayat_for_export(
        &self,
        query: &crate::handlers::ExportQuery,
    ) -> AppResult<Vec<serde_json::Value>> {
        let client =
            self.pool.get().await.map_err(|e| {
                AppError::Internal(format!("Failed to get database connection: {}", e))
            })?;

        let limit = query.limit.unwrap_or(50000).min(50000);

        let rows = client
            .query(
                r#"
                SELECT
                    s.nama as satker_nama,
                    r.kode_barang,
                    r.tahun_anggaran,
                    r.jumlah_terpenuhi,
                    r.sumber_data,
                    r.tanggal_pemenuhan
                FROM perlengkapan.riwayat_pemenuhan r
                LEFT JOIN authenc.satkers s ON r.satker_id = s.id
                ORDER BY r.created_at DESC
                LIMIT $1
                "#,
                &[&(limit as i64)],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "satker_nama".to_string(),
                serde_json::Value::String(
                    row.get::<_, Option<String>>("satker_nama")
                        .unwrap_or_default(),
                ),
            );
            obj.insert(
                "kode_barang".to_string(),
                serde_json::Value::String(row.get("kode_barang")),
            );
            obj.insert(
                "tahun_anggaran".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("tahun_anggaran"),
                )),
            );
            obj.insert(
                "jumlah_terpenuhi".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    row.get::<_, i32>("jumlah_terpenuhi"),
                )),
            );
            obj.insert(
                "sumber_data".to_string(),
                serde_json::Value::String(row.get("sumber_data")),
            );
            obj.insert(
                "tanggal_pemenuhan".to_string(),
                serde_json::Value::String(
                    row.get::<_, chrono::NaiveDate>("tanggal_pemenuhan")
                        .to_string(),
                ),
            );
            results.push(serde_json::Value::Object(obj));
        }

        Ok(results)
    }
}
