//! # Kebutuhan BMN Repository
//!
//! Database operations for BMN needs analysis system.
//! Uses PostgreSQL via tokio-postgres with connection pooling.

use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::{Value, json};
use tokio_postgres::Row;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::models::*;
use crate::errors::{AppError, AppResult};

// ============================================================================
// Repository Trait
// ============================================================================

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait KebutuhanBmnRepository: Send + Sync {
    // Pengajuan CRUD
    async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmn>;

    async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        filter: Option<PengajuanFilter>,
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)>;

    async fn update_pengajuan(
        &self,
        id: Uuid,
        request: UpdatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()>;

    // Pengajuan Assets
    async fn get_pengajuan_assets(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAsset>>;
    async fn create_pengajuan_asset(
        &self,
        pengajuan_id: Uuid,
        request: CreateAssetTypeRequest,
    ) -> AppResult<PengajuanKebutuhanBmnAsset>;

    // Satker Operations
    async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>>;
    async fn get_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnSatker>;
    async fn create_pengajuan_satker(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker>;
    async fn update_satker_status(
        &self,
        satker_id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker>;

    // Barang Operations
    async fn get_satker_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
        filter: Option<BarangFilter>,
    ) -> AppResult<(Vec<PengajuanKebutuhanBmnBarang>, i64)>;
    async fn get_barang_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn delete_barang(&self, id: Uuid) -> AppResult<()>;
    async fn set_barang_prioritas(&self, items: Vec<PrioritasItem>) -> AppResult<()>;

    // Workflow Aktivitas
    async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>>;
    async fn create_aktivitas(
        &self,
        satker_id: Uuid,
        from_status: Option<i32>,
        to_status: i32,
        aksi: &str,
        komentar: Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnAktivitas>;

    // Dashboard Statistics
    async fn get_dashboard_stats(&self) -> AppResult<KebutuhanBmnDashboardStats>;

    // Pengajuan Status Update
    async fn update_pengajuan_status(
        &self,
        id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    // Satker Submit Data (lampiran, catatan) for SubmitWilayah
    async fn update_satker_submit_data(
        &self,
        satker_id: Uuid,
        catatan_satker: Option<String>,
        lampiran_surat_permohonan: Option<String>,
        lampiran_pendukung: Option<Vec<LampiranItem>>,
    ) -> AppResult<()>;

    // Validator Wilayah info update
    async fn update_satker_validator_wilayah(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
    ) -> AppResult<()>;

    // Validator Pusat info update
    async fn update_satker_validator_pusat(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
        is_approved: bool,
    ) -> AppResult<()>;

    // Update pengajuan laporan URL
    async fn update_pengajuan_laporan(
        &self,
        id: Uuid,
        laporan_url: &str,
        laporan_format: &str,
    ) -> AppResult<()>;
}

/// User information for audit trail
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

#[derive(Clone)]
pub struct PgKebutuhanBmnRepository {
    pool: Pool,
}

impl PgKebutuhanBmnRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Get a reference to the database pool
    ///
    /// Used by search engine and other components that need direct pool access
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    async fn get_client(&self) -> AppResult<deadpool_postgres::Client> {
        self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::Internal(format!("Database connection error: {}", e))
        })
    }
}

#[async_trait]
impl KebutuhanBmnRepository for PgKebutuhanBmnRepository {
    async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let pilihan_satker = request
            .pilihan_satker
            .unwrap_or_else(|| "semua".to_string());
        let asset_ids: Value = json!(
            request
                .asset_types
                .iter()
                .filter_map(|a| a.ms_jenis_asset_id)
                .collect::<Vec<_>>()
        );

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
                    (nama, deskripsi, tahun, tgl_mulai, tgl_selesai, pilihan_satker,
                     id_jenis_asset, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
                RETURNING *
                "#,
                &[
                    &request.nama,
                    &request.deskripsi,
                    &request.tahun,
                    &request.tgl_mulai,
                    &request.tgl_selesai,
                    &pilihan_satker,
                    &asset_ids,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to create pengajuan: {}", e);
                AppError::Internal(format!("Database error: {}", e))
            })?;

        let pengajuan = PengajuanKebutuhanBmn::from_row(&row);
        info!("Created pengajuan kebutuhan BMN: {}", pengajuan.id);

        // Create asset types
        for asset in &request.asset_types {
            self.create_pengajuan_asset(pengajuan.id, asset.clone())
                .await?;
        }

        // Create satker entries if specified
        for satker_id in &request.satker_ids {
            self.create_pengajuan_satker(pengajuan.id, satker_id, None, user_id)
                .await?;
        }

        Ok(pengajuan)
    }

    async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Pengajuan {} not found", id)))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        filter: Option<PengajuanFilter>,
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)> {
        let client = self.get_client().await?;
        let offset = (page - 1) * per_page;

        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref f) = filter {
            if let Some(tahun) = f.tahun {
                conditions.push(format!("tahun = ${}", param_idx));
                params.push(Box::new(tahun));
                param_idx += 1;
            }
            if let Some(status) = f.status_kode {
                conditions.push(format!("status_kode = ${}", param_idx));
                params.push(Box::new(status));
                param_idx += 1;
            }
            if let Some(ref search) = f.search {
                conditions.push(format!("nama ILIKE ${}", param_idx));
                params.push(Box::new(format!("%{}%", search)));
                param_idx += 1;
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.vw_kebutuhan_bmn_summary {}",
            where_clause
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client
            .query_one(&count_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        params.push(Box::new(per_page));
        params.push(Box::new(offset));

        let data_query = format!(
            r#"
            SELECT * FROM perlengkapan.vw_kebutuhan_bmn_summary
            {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&data_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let summaries: Vec<KebutuhanBmnSummary> =
            rows.iter().map(KebutuhanBmnSummary::from_row).collect();

        Ok((summaries, total))
    }

    async fn update_pengajuan(
        &self,
        id: Uuid,
        request: UpdatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        // First check version for optimistic locking
        let current = self.get_pengajuan_by_id(id).await?;
        if current.version != request.version {
            return Err(AppError::Conflict(
                "Data has been modified by another user. Please refresh and try again.".to_string(),
            ));
        }

        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref nama) = request.nama {
            updates.push(format!("nama = ${}", param_idx));
            params.push(Box::new(nama.clone()));
            param_idx += 1;
        }
        if let Some(ref deskripsi) = request.deskripsi {
            updates.push(format!("deskripsi = ${}", param_idx));
            params.push(Box::new(deskripsi.clone()));
            param_idx += 1;
        }
        if let Some(tgl_mulai) = request.tgl_mulai {
            updates.push(format!("tgl_mulai = ${}", param_idx));
            params.push(Box::new(tgl_mulai));
            param_idx += 1;
        }
        if let Some(tgl_selesai) = request.tgl_selesai {
            updates.push(format!("tgl_selesai = ${}", param_idx));
            params.push(Box::new(tgl_selesai));
            param_idx += 1;
        }
        if let Some(ref pilihan) = request.pilihan_satker {
            updates.push(format!("pilihan_satker = ${}", param_idx));
            params.push(Box::new(pilihan.clone()));
            param_idx += 1;
        }

        // Always update version and updated_by
        updates.push(format!("version = version + 1"));
        updates.push(format!("updated_by = ${}", param_idx));
        params.push(Box::new(user_id));
        param_idx += 1;

        // Add id and version for WHERE clause
        params.push(Box::new(id));
        params.push(Box::new(request.version));

        let query = format!(
            r#"
            UPDATE perlengkapan.pengajuan_kebutuhan_bmn
            SET {}
            WHERE id = ${} AND version = ${}
            RETURNING *
            "#,
            updates.join(", "),
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let row = client
            .query_opt(&query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::Conflict("Concurrent modification detected".to_string()))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()> {
        let client = self.get_client().await?;

        // Check if pengajuan exists and is in draft status
        let pengajuan = self.get_pengajuan_by_id(id).await?;
        if pengajuan.status != KebutuhanBmnStatus::Draft {
            return Err(AppError::BadRequest(
                "Only draft pengajuan can be deleted".to_string(),
            ));
        }

        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Deleted pengajuan kebutuhan BMN: {}", id);
        Ok(())
    }

    async fn get_pengajuan_assets(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAsset>> {
        let client = self.get_client().await?;

        let rows = client
            .query(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_asset WHERE pengajuan_id = $1",
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnAsset::from_row)
            .collect())
    }

    async fn create_pengajuan_asset(
        &self,
        pengajuan_id: Uuid,
        request: CreateAssetTypeRequest,
    ) -> AppResult<PengajuanKebutuhanBmnAsset> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_asset
                    (pengajuan_id, kode_barang, nm_barang, ms_jenis_asset_id, keterangan)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#,
                &[
                    &pengajuan_id,
                    &request.kode_barang,
                    &request.nm_barang,
                    &request.ms_jenis_asset_id,
                    &request.keterangan,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnAsset::from_row(&row))
    }

    async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>> {
        let client = self.get_client().await?;

        let rows = client
            .query(
                r#"
                SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker
                WHERE pengajuan_id = $1
                ORDER BY prioritas ASC, nm_satker ASC
                "#,
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnSatker::from_row)
            .collect())
    }

    async fn get_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Satker {} not found", id)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn create_pengajuan_satker(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
                    (pengajuan_id, ms_satker_id, nm_satker, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $4)
                ON CONFLICT (pengajuan_id, ms_satker_id) DO UPDATE
                SET updated_at = NOW()
                RETURNING *
                "#,
                &[&pengajuan_id, &satker_id, &satker_name, &user_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn update_satker_status(
        &self,
        satker_id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET status_kode = $1, updated_by = $2
                WHERE id = $3
                RETURNING *
                "#,
                &[&new_status, &user_id, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnSatker::from_row(&row))
    }

    async fn get_satker_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
        filter: Option<BarangFilter>,
    ) -> AppResult<(Vec<PengajuanKebutuhanBmnBarang>, i64)> {
        let client = self.get_client().await?;
        let offset = (page - 1) * per_page;

        let mut conditions = vec!["pengajuan_satker_id = $1".to_string()];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(satker_id)];
        let mut param_idx = 2;

        if let Some(ref f) = filter {
            if let Some(ref kode) = f.kode_barang {
                conditions.push(format!("kode_barang = ${}", param_idx));
                params.push(Box::new(kode.clone()));
                param_idx += 1;
            }
            if let Some(min_prioritas) = f.prioritas_min {
                conditions.push(format!("prioritas >= ${}", param_idx));
                params.push(Box::new(min_prioritas));
                param_idx += 1;
            }
            if let Some(ref search) = f.search {
                conditions.push(format!("nama ILIKE ${}", param_idx));
                params.push(Box::new(format!("%{}%", search)));
                param_idx += 1;
            }
        }

        let where_clause = conditions.join(" AND ");

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE {}",
            where_clause
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let count_row = client
            .query_one(&count_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get(0);

        // Get paginated data
        params.push(Box::new(per_page));
        params.push(Box::new(offset));

        let data_query = format!(
            r#"
            SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
            WHERE {}
            ORDER BY prioritas ASC, created_at ASC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&data_query, &params_refs)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let items: Vec<PengajuanKebutuhanBmnBarang> = rows
            .iter()
            .map(PengajuanKebutuhanBmnBarang::from_row)
            .collect();

        Ok((items, total))
    }

    async fn get_barang_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Barang {} not found", id)))?;

        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let file_pendukung: Value = json!(request.file_pendukung);

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                    (pengajuan_satker_id, nama, kode_barang, jumlah, satuan,
                     alasan, keterangan, file_pendukung, created_by, updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
                RETURNING *
                "#,
                &[
                    &satker_id,
                    &request.nama,
                    &request.kode_barang,
                    &request.jumlah,
                    &request.satuan,
                    &request.alasan,
                    &request.keterangan,
                    &file_pendukung,
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Created barang for satker {}: {}", satker_id, request.nama);
        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                SET jml_setuju = $1, keterangan = COALESCE($2, keterangan), updated_by = $3
                WHERE id = $4
                RETURNING *
                "#,
                &[
                    &request.jml_setuju,
                    &request.keterangan,
                    &user_id,
                    &barang_id,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnBarang::from_row(&row))
    }

    async fn delete_barang(&self, id: Uuid) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        info!("Deleted barang: {}", id);
        Ok(())
    }

    async fn set_barang_prioritas(&self, items: Vec<PrioritasItem>) -> AppResult<()> {
        let client = self.get_client().await?;

        for item in items {
            client
                .execute(
                    r#"
                    UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                    SET prioritas = $1, skor = COALESCE($2, skor)
                    WHERE id = $3
                    "#,
                    &[&item.prioritas, &item.skor, &item.barang_id],
                )
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
        }

        Ok(())
    }

    async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>> {
        let client = self.get_client().await?;

        let rows = client
            .query(
                r#"
                SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_satker_id = $1
                ORDER BY created_at DESC
                "#,
                &[&satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(rows
            .iter()
            .map(PengajuanKebutuhanBmnAktivitas::from_row)
            .collect())
    }

    async fn create_aktivitas(
        &self,
        satker_id: Uuid,
        from_status: Option<i32>,
        to_status: i32,
        aksi: &str,
        komentar: Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnAktivitas> {
        let client = self.get_client().await?;

        let (nip, nama, pangkat, jabatan, role) = user_info
            .map(|u| (u.nip, u.nama, u.pangkat, u.jabatan, u.role))
            .unwrap_or_default();

        let row = client
            .query_one(
                r#"
                INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                    (pengajuan_satker_id, from_status_kode, to_status_kode, user_id,
                     nip, nama, pangkat, jabatan, role, aksi, komentar)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *
                "#,
                &[
                    &satker_id,
                    &from_status,
                    &to_status,
                    &user_id,
                    &nip,
                    &nama,
                    &pangkat,
                    &jabatan,
                    &role,
                    &aksi,
                    &komentar,
                ],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmnAktivitas::from_row(&row))
    }

    async fn get_dashboard_stats(&self) -> AppResult<KebutuhanBmnDashboardStats> {
        let client = self.get_client().await?;

        // Get aggregate stats
        let stats_row = client
            .query_one(
                r#"
                SELECT
                    COUNT(*) as total_pengajuan,
                    COUNT(*) FILTER (WHERE status_kode = 2000) as pengajuan_draft,
                    COUNT(*) FILTER (WHERE status_kode BETWEEN 2001 AND 2005) as pengajuan_in_progress,
                    COUNT(*) FILTER (WHERE status_kode IN (2006, 2008)) as pengajuan_completed
                FROM perlengkapan.pengajuan_kebutuhan_bmn
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        // Get satker and barang stats
        let satker_barang_row = client
            .query_one(
                r#"
                SELECT
                    COUNT(DISTINCT ps.ms_satker_id) as total_satker,
                    COALESCE(SUM(psb.jumlah), 0) as total_diminta,
                    COALESCE(SUM(psb.jml_setuju), 0) as total_disetujui
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
                LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang psb
                    ON ps.id = psb.pengajuan_satker_id
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        // Get by tahun
        let tahun_rows = client
            .query(
                r#"
                SELECT tahun, COUNT(*) as total
                FROM perlengkapan.pengajuan_kebutuhan_bmn
                GROUP BY tahun
                ORDER BY tahun DESC
                LIMIT 5
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        // Get by status
        let status_rows = client
            .query(
                r#"
                SELECT p.status_kode, m.nama as status_nama, COUNT(*) as total
                FROM perlengkapan.pengajuan_kebutuhan_bmn p
                JOIN perlengkapan.ms_aktivitas_bmn m ON p.status_kode = m.kode
                GROUP BY p.status_kode, m.nama
                ORDER BY p.status_kode
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(KebutuhanBmnDashboardStats {
            total_pengajuan: stats_row.get("total_pengajuan"),
            pengajuan_draft: stats_row.get("pengajuan_draft"),
            pengajuan_in_progress: stats_row.get("pengajuan_in_progress"),
            pengajuan_completed: stats_row.get("pengajuan_completed"),
            total_satker_terlibat: satker_barang_row.get("total_satker"),
            total_barang_diminta: satker_barang_row.get("total_diminta"),
            total_barang_disetujui: satker_barang_row.get("total_disetujui"),
            by_tahun: tahun_rows
                .iter()
                .map(|r| StatsByTahun {
                    tahun: r.get("tahun"),
                    total: r.get("total"),
                })
                .collect(),
            by_status: status_rows
                .iter()
                .map(|r| StatsByStatus {
                    status_kode: r.get("status_kode"),
                    status_nama: r.get("status_nama"),
                    total: r.get("total"),
                })
                .collect(),
        })
    }

    async fn update_pengajuan_status(
        &self,
        id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn
                SET status_kode = $1, updated_by = $2, version = version + 1
                WHERE id = $3
                RETURNING *
                "#,
                &[&new_status, &user_id, &id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(PengajuanKebutuhanBmn::from_row(&row))
    }

    async fn update_satker_submit_data(
        &self,
        satker_id: Uuid,
        catatan_satker: Option<String>,
        lampiran_surat_permohonan: Option<String>,
        lampiran_pendukung: Option<Vec<LampiranItem>>,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        let lampiran_json = lampiran_pendukung
            .map(|v| serde_json::to_value(v).unwrap_or_default());

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET catatan_satker = COALESCE($1, catatan_satker),
                    lampiran_surat_permohonan = COALESCE($2, lampiran_surat_permohonan),
                    lampiran_pendukung = COALESCE($3, lampiran_pendukung),
                    updated_at = NOW()
                WHERE id = $4
                "#,
                &[&catatan_satker, &lampiran_surat_permohonan, &lampiran_json, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update_satker_validator_wilayah(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET validator_wilayah_id = COALESCE($1, validator_wilayah_id),
                    catatan_validator_wilayah = COALESCE($2, catatan_validator_wilayah),
                    tanggal_submit_wilayah = NOW(),
                    updated_at = NOW()
                WHERE id = $3
                "#,
                &[&validator_id, &catatan, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update_satker_validator_pusat(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
        is_approved: bool,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker
                SET validator_pusat_id = COALESCE($1, validator_pusat_id),
                    catatan_validator_pusat = COALESCE($2, catatan_validator_pusat),
                    tanggal_submit_pusat = NOW(),
                    is_approved = $3,
                    alasan_keputusan = $2,
                    updated_at = NOW()
                WHERE id = $4
                "#,
                &[&validator_id, &catatan, &is_approved, &satker_id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update_pengajuan_laporan(
        &self,
        id: Uuid,
        laporan_url: &str,
        laporan_format: &str,
    ) -> AppResult<()> {
        let client = self.get_client().await?;

        client
            .execute(
                r#"
                UPDATE perlengkapan.pengajuan_kebutuhan_bmn
                SET laporan_url = $1,
                    laporan_format = $2,
                    laporan_generated_at = NOW(),
                    updated_at = NOW()
                WHERE id = $3
                "#,
                &[&laporan_url, &laporan_format, &id],
            )
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Repository tests would use testcontainers for PostgreSQL
    // Example test structure:

    #[tokio::test]
    #[ignore] // Requires PostgreSQL testcontainer
    async fn test_create_pengajuan() {
        // Setup testcontainer
        // Create pool
        // Create repository
        // Test create operation
    }
}
