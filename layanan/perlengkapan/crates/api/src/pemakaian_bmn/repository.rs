//! # Pemakaian BMN Repository
//!
//! Data access layer for BMN usage permits.
//! Requirements: REQ-P001, REQ-P002, REQ-P003, REQ-P008, REQ-P011, REQ-P012, REQ-P013

use chrono::{DateTime, NaiveDate, Utc};
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::models::*;

/// Repository for pemakaian BMN data access
#[derive(Clone)]
pub struct PemakaianBmnRepository {
    pool: Pool,
}

impl PemakaianBmnRepository {
    /// Create a new repository instance
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a new permit
    /// Requirements: REQ-P001
    pub async fn create(&self, request: CreateIzinPemakaianRequest, created_by: Uuid, created_by_nama: String) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let id = Uuid::new_v4();
        let is_renewal = request.is_renewal.unwrap_or(false);

        let query = r#"
            INSERT INTO perlengkapan.izin_pemakaian_bmn (
                id, pegawai_nip, pegawai_nama, pegawai_satker_id, pegawai_satker_nama, pegawai_jabatan,
                jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang, bmn_merk, bmn_tahun_perolehan,
                no_polisi, no_bpkb, no_stnk, no_rangka, no_mesin,
                alamat, luas_tanah, luas_bangunan,
                serial_number, spesifikasi,
                tanggal_mulai, tanggal_selesai, keperluan, lokasi_pemakaian,
                is_renewal, previous_permit_id, file_pendukung,
                status, created_by, created_by_nama, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22,
                $23, $24, $25, $26,
                $27, $28, $29,
                'DRAFT', $30, $31, NOW(), NOW()
            )
            RETURNING *
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &id,
                    &request.pegawai_nip,
                    &request.pegawai_nama,
                    &request.pegawai_satker_id,
                    &request.pegawai_satker_nama,
                    &request.pegawai_jabatan,
                    &request.jenis_bmn,
                    &request.bmn_nup,
                    &request.bmn_kode_barang,
                    &request.bmn_nama_barang,
                    &request.bmn_merk,
                    &request.bmn_tahun_perolehan,
                    &request.no_polisi,
                    &request.no_bpkb,
                    &request.no_stnk,
                    &request.no_rangka,
                    &request.no_mesin,
                    &request.alamat,
                    &request.luas_tanah,
                    &request.luas_bangunan,
                    &request.serial_number,
                    &request.spesifikasi,
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &request.lokasi_pemakaian,
                    &is_renewal,
                    &request.previous_permit_id,
                    &request.file_pendukung,
                    &created_by,
                    &created_by_nama,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    /// Get permit by ID
    /// Requirements: REQ-P001
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Izin pemakaian not found: {}", id)))?;

        Ok(self.row_to_permit(row))
    }

    /// Update permit (only in DRAFT status)
    /// Requirements: REQ-P001
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateIzinPemakaianRequest,
        updated_by: Uuid,
        updated_by_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // First check if permit is in DRAFT status
        let current = self.get_by_id(id).await?;
        if current.status != "DRAFT" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status DRAFT yang dapat diubah".to_string(),
            ));
        }

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET tanggal_mulai = COALESCE($1, tanggal_mulai),
                tanggal_selesai = COALESCE($2, tanggal_selesai),
                keperluan = COALESCE($3, keperluan),
                lokasi_pemakaian = COALESCE($4, lokasi_pemakaian),
                file_pendukung = COALESCE($5, file_pendukung),
                updated_by = $6,
                updated_by_nama = $7,
                updated_at = NOW()
            WHERE id = $8
            RETURNING *
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &request.tanggal_mulai,
                    &request.tanggal_selesai,
                    &request.keperluan,
                    &request.lokasi_pemakaian,
                    &request.file_pendukung,
                    &updated_by,
                    &updated_by_nama,
                    &id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    /// Update permit status
    /// Requirements: REQ-P004
    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        user_id: Uuid,
        user_nama: String,
        catatan: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = match status {
            "APPROVED" => r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    approved_by = $2,
                    approved_by_nama = $3,
                    approved_at = NOW(),
                    catatan_approval = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#,
            "REVOKED" => r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    revoked_by = $2,
                    revoked_by_nama = $3,
                    revoked_at = NOW(),
                    catatan_revocation = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#,
            _ => r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status = $1,
                    catatan_approval = $4,
                    updated_by = $2,
                    updated_by_nama = $3,
                    updated_at = NOW()
                WHERE id = $5
                RETURNING *
            "#,
        };

        let row = client
            .query_one(query, &[&status, &user_id, &user_nama, &catatan, &id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    /// Generate permit number
    /// Requirements: REQ-P005
    pub async fn generate_permit_number(&self, id: Uuid) -> AppResult<String> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // Format: IP/YYYY/MM/NNNN
        // IP = Izin Pemakaian
        // YYYY = Year
        // MM = Month
        // NNNN = Sequential number
        let query = r#"
            WITH next_number AS (
                SELECT COALESCE(MAX(
                    CAST(SUBSTRING(nomor_izin FROM 'IP/\d{4}/\d{2}/(\d{4})') AS INTEGER)
                ), 0) + 1 AS num
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE nomor_izin LIKE 'IP/' || TO_CHAR(NOW(), 'YYYY/MM') || '/%'
            )
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET nomor_izin = 'IP/' || TO_CHAR(NOW(), 'YYYY/MM/') || LPAD((SELECT num FROM next_number)::TEXT, 4, '0'),
                updated_at = NOW()
            WHERE id = $1
            RETURNING nomor_izin
        "#;

        let row = client
            .query_one(query, &[&id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.get("nomor_izin"))
    }

    /// Update document fields after document generation
    /// Requirements: REQ-P006, REQ-D002
    pub async fn update_document_fields(
        &self,
        id: Uuid,
        document_id: Uuid,
        document_url: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET document_id = $1,
                document_url = $2,
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
        "#;

        let row = client
            .query_one(query, &[&document_id, &document_url, &id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(self.row_to_permit(row))
    }

    /// Check if BMN is available (no active permit)
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(&self, bmn_nup: &str) -> AppResult<BmnAvailabilityResponse> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            SELECT id, nomor_izin, pegawai_nama, tanggal_selesai
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1 AND status = 'ACTIVE'
            LIMIT 1
        "#;

        let row_opt = client
            .query_opt(query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = row_opt {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: false,
                active_permit_id: Some(row.get("id")),
                active_permit_holder: Some(row.get("pegawai_nama")),
                active_permit_expires: Some(row.get("tanggal_selesai")),
            })
        } else {
            Ok(BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: true,
                active_permit_id: None,
                active_permit_holder: None,
                active_permit_expires: None,
            })
        }
    }

    /// List permits with pagination and filters
    /// Requirements: REQ-P001, REQ-P011
    pub async fn list(&self, query: ListPermitsQuery) -> AppResult<PaginatedPermitsResponse> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let mut where_clauses = vec![];
        let mut param_count = 1;

        // Build WHERE clause dynamically
        let mut sql_params: Vec<String> = vec![];

        if query.status.is_some() {
            where_clauses.push(format!("status = ${}", param_count));
            param_count += 1;
        }

        if query.jenis_bmn.is_some() {
            where_clauses.push(format!("jenis_bmn = ${}", param_count));
            param_count += 1;
        }

        if query.pegawai_nip.is_some() {
            where_clauses.push(format!("pegawai_nip = ${}", param_count));
            param_count += 1;
        }

        if query.satker_id.is_some() {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_count));
            param_count += 1;
        }

        if query.search.is_some() {
            where_clauses.push(format!(
                "(bmn_nama_barang ILIKE ${} OR pegawai_nama ILIKE ${} OR nomor_izin ILIKE ${})",
                param_count, param_count, param_count
            ));
            param_count += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total - simplified for now
        let count_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn {}",
            where_clause
        );

        let total_row = client
            .query_one(&count_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total: i64 = total_row.get("total");

        // Get data
        let data_query = format!(
            "SELECT * FROM perlengkapan.izin_pemakaian_bmn {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause, per_page, offset
        );

        let rows = client
            .query(&data_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let data = rows.into_iter().map(|row| self.row_to_permit(row)).collect();

        let total_pages = (total as f64 / per_page as f64).ceil() as i64;

        Ok(PaginatedPermitsResponse {
            data,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Get permit history for a BMN
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(&self, bmn_nup: &str) -> AppResult<BmnUsageStats> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            SELECT
                bmn_nup,
                bmn_nama_barang,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits,
                SUM(tanggal_selesai - tanggal_mulai) as total_days_used,
                (SELECT pegawai_nama FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = $1 AND status = 'ACTIVE' LIMIT 1) as current_holder
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1
            GROUP BY bmn_nup, bmn_nama_barang
        "#;

        let row = client
            .query_opt(query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("No usage history for BMN: {}", bmn_nup)))?;

        // Get history entries
        let history_query = r#"
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1
            ORDER BY created_at DESC
            LIMIT 10
        "#;

        let history_rows = client
            .query(history_query, &[&bmn_nup])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(BmnUsageStats {
            bmn_nup: row.get("bmn_nup"),
            bmn_nama: row.get("bmn_nama_barang"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            total_days_used: row.get::<_, Option<i32>>("total_days_used").unwrap_or(0) as i64,
            current_holder: row.get("current_holder"),
            permit_history,
        })
    }

    /// Get permit history for a pegawai
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(&self, pegawai_nip: &str) -> AppResult<PegawaiUsageStats> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            SELECT
                pegawai_nip,
                pegawai_nama,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1
            GROUP BY pegawai_nip, pegawai_nama
        "#;

        let row = client
            .query_opt(query, &[&pegawai_nip])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("No usage history for pegawai: {}", pegawai_nip)))?;

        // Get history entries
        let history_query = r#"
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1
            ORDER BY created_at DESC
            LIMIT 10
        "#;

        let history_rows = client
            .query(history_query, &[&pegawai_nip])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(PegawaiUsageStats {
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            permit_history,
        })
    }

    /// Auto-expire permits that have passed their end date
    /// Requirements: REQ-P010
    pub async fn auto_expire_permits(&self) -> AppResult<usize> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            UPDATE perlengkapan.izin_pemakaian_bmn
            SET status = 'EXPIRED',
                updated_at = NOW()
            WHERE status = 'ACTIVE'
            AND tanggal_selesai < CURRENT_DATE
        "#;

        let result = client
            .execute(query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get permits expiring soon (for notifications)
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(&self, days_threshold: i32) -> AppResult<Vec<IzinPemakaianBmn>> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let query = r#"
            SELECT * FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
            AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + $1
            ORDER BY tanggal_selesai ASC
        "#;

        let rows = client
            .query(query, &[&days_threshold])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|row| self.row_to_permit(row)).collect())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================

    /// Get active usage monitoring dashboard data
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: super::models::MonitoringDashboardQuery,
    ) -> AppResult<super::models::ActiveUsageMonitoringDashboard> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // Build WHERE clause for filters
        let mut where_clauses = vec!["status = 'ACTIVE'".to_string()];
        let mut param_idx = 1;

        if query.satker_id.is_some() {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_idx));
            param_idx += 1;
        }

        if query.jenis_bmn.is_some() {
            where_clauses.push(format!("jenis_bmn = ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = where_clauses.join(" AND ");

        // Total active permits
        let total_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn WHERE {}",
            where_clause
        );

        let total_row = client
            .query_one(&total_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_active_permits: i64 = total_row.get("total");

        // Permits by jenis BMN
        let jenis_query = format!(
            r#"
            SELECT jenis_bmn, COUNT(*) as count
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {}
            GROUP BY jenis_bmn
            ORDER BY count DESC
            "#,
            where_clause
        );

        let jenis_rows = client
            .query(&jenis_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_jenis_bmn = jenis_rows
            .into_iter()
            .map(|row| {
                let count: i64 = row.get("count");
                let percentage = if total_active_permits > 0 {
                    (count as f64 / total_active_permits as f64) * 100.0
                } else {
                    0.0
                };
                super::models::PermitsByJenisBmn {
                    jenis_bmn: row.get("jenis_bmn"),
                    count,
                    percentage,
                }
            })
            .collect();

        // Permits by satker
        let satker_query = format!(
            r#"
            SELECT pegawai_satker_id, pegawai_satker_nama, COUNT(*) as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {}
            GROUP BY pegawai_satker_id, pegawai_satker_nama
            ORDER BY active_permits DESC
            LIMIT 10
            "#,
            where_clause
        );

        let satker_rows = client
            .query(&satker_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_satker = satker_rows
            .into_iter()
            .map(|row| super::models::PermitsBySatker {
                satker_id: row.get("pegawai_satker_id"),
                satker_nama: row.get("pegawai_satker_nama"),
                active_permits: row.get("active_permits"),
            })
            .collect();

        // Expiring soon (next 30 days)
        let expiring_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, tanggal_selesai,
                   (tanggal_selesai - CURRENT_DATE) as days_until_expiry
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30
            ORDER BY tanggal_selesai ASC
            LIMIT 10
            "#,
            where_clause
        );

        let expiring_rows = client
            .query(&expiring_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let expiring_soon = expiring_rows
            .into_iter()
            .map(|row| super::models::ExpiringPermitInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                tanggal_selesai: row.get("tanggal_selesai"),
                days_until_expiry: row.get("days_until_expiry"),
            })
            .collect();

        // Recent activations (last 7 days)
        let recent_query = format!(
            r#"
            SELECT id, nomor_izin, bmn_nama_barang, pegawai_nama, approved_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE {} AND approved_at >= CURRENT_DATE - 7
            ORDER BY approved_at DESC
            LIMIT 10
            "#,
            where_clause
        );

        let recent_rows = client
            .query(&recent_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let recent_activations = recent_rows
            .into_iter()
            .map(|row| super::models::RecentActivationInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                activated_at: row.get("approved_at"),
            })
            .collect();

        Ok(super::models::ActiveUsageMonitoringDashboard {
            total_active_permits,
            permits_by_jenis_bmn,
            permits_by_satker,
            expiring_soon,
            recent_activations,
        })
    }

    /// Get BMN utilization report
    /// Requirements: REQ-P013
    pub async fn get_bmn_utilization_report(
        &self,
        query: super::models::MonitoringDashboardQuery,
    ) -> AppResult<super::models::BmnUtilizationReport> {
        let client = self.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // For this report, we need to query SIMAN data (from integrasi schema)
        // to get total BMN count and compare with permits

        // Total BMN count (from SIMAN integration)
        let total_bmn_query = r#"
            SELECT COUNT(DISTINCT nup) as total
            FROM integrasi.siman_aset_tanah
            WHERE kondisi = 'BAIK'
        "#;

        let total_bmn_row = client
            .query_one(total_bmn_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_bmn: i64 = total_bmn_row.get("total");

        // BMN with active permits
        let utilized_query = r#"
            SELECT COUNT(DISTINCT bmn_nup) as count
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE status = 'ACTIVE'
        "#;

        let utilized_row = client
            .query_one(utilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let bmn_with_active_permits: i64 = utilized_row.get("count");

        let bmn_without_permits = total_bmn - bmn_with_active_permits;
        let utilization_rate = if total_bmn > 0 {
            (bmn_with_active_permits as f64 / total_bmn as f64) * 100.0
        } else {
            0.0
        };

        // BMN utilization by type
        let by_type_query = r#"
            WITH bmn_counts AS (
                SELECT
                    CASE
                        WHEN kode_barang LIKE '03.01%' THEN 'KENDARAAN_BERMOTOR'
                        WHEN kode_barang LIKE '03.02%' THEN 'RUMAH_NEGARA'
                        WHEN kode_barang LIKE '03.03%' THEN 'LAPTOP'
                        ELSE 'LAINNYA'
                    END as jenis_bmn,
                    COUNT(DISTINCT nup) as total_bmn
                FROM integrasi.siman_aset_tanah
                WHERE kondisi = 'BAIK'
                GROUP BY jenis_bmn
            ),
            utilized_counts AS (
                SELECT jenis_bmn, COUNT(DISTINCT bmn_nup) as utilized_bmn
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE status = 'ACTIVE'
                GROUP BY jenis_bmn
            )
            SELECT
                bc.jenis_bmn,
                bc.total_bmn,
                COALESCE(uc.utilized_bmn, 0) as utilized_bmn
            FROM bmn_counts bc
            LEFT JOIN utilized_counts uc ON bc.jenis_bmn = uc.jenis_bmn
        "#;

        let by_type_rows = client
            .query(by_type_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let bmn_by_type = by_type_rows
            .into_iter()
            .map(|row| {
                let total: i64 = row.get("total_bmn");
                let utilized: i64 = row.get("utilized_bmn");
                let rate = if total > 0 {
                    (utilized as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                super::models::BmnUtilizationByType {
                    jenis_bmn: row.get("jenis_bmn"),
                    total_bmn: total,
                    utilized_bmn: utilized,
                    utilization_rate: rate,
                }
            })
            .collect();

        // Top utilized BMN
        let top_utilized_query = r#"
            SELECT
                bmn_nup,
                bmn_nama_barang,
                jenis_bmn,
                COUNT(*) as total_permits,
                SUM(tanggal_selesai - tanggal_mulai) as total_days_used,
                (SELECT pegawai_nama FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = i.bmn_nup AND status = 'ACTIVE' LIMIT 1) as current_holder
            FROM perlengkapan.izin_pemakaian_bmn i
            GROUP BY bmn_nup, bmn_nama_barang, jenis_bmn
            ORDER BY total_permits DESC, total_days_used DESC
            LIMIT 10
        "#;

        let top_rows = client
            .query(top_utilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let top_utilized_bmn = top_rows
            .into_iter()
            .map(|row| super::models::TopUtilizedBmn {
                bmn_nup: row.get("bmn_nup"),
                bmn_nama: row.get("bmn_nama_barang"),
                jenis_bmn: row.get("jenis_bmn"),
                total_permits: row.get("total_permits"),
                total_days_used: row.get::<_, Option<i32>>("total_days_used").unwrap_or(0) as i64,
                current_holder: row.get("current_holder"),
            })
            .collect();

        // Underutilized BMN (no permits in last 180 days)
        let underutilized_query = r#"
            SELECT
                s.nup as bmn_nup,
                s.nama_barang as bmn_nama,
                CASE
                    WHEN s.kode_barang LIKE '03.01%' THEN 'KENDARAAN_BERMOTOR'
                    WHEN s.kode_barang LIKE '03.02%' THEN 'RUMAH_NEGARA'
                    WHEN s.kode_barang LIKE '03.03%' THEN 'LAPTOP'
                    ELSE 'LAINNYA'
                END as jenis_bmn,
                (SELECT MAX(tanggal_selesai) FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = s.nup) as last_used_date,
                (CURRENT_DATE - (SELECT MAX(tanggal_selesai) FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = s.nup)) as days_since_last_use
            FROM integrasi.siman_aset_tanah s
            WHERE s.kondisi = 'BAIK'
            AND s.nup NOT IN (
                SELECT DISTINCT bmn_nup FROM perlengkapan.izin_pemakaian_bmn
                WHERE tanggal_selesai >= CURRENT_DATE - 180
            )
            LIMIT 20
        "#;

        let underutilized_rows = client
            .query(underutilized_query, &[])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let underutilized_bmn = underutilized_rows
            .into_iter()
            .map(|row| super::models::UnderutilizedBmn {
                bmn_nup: row.get("bmn_nup"),
                bmn_nama: row.get("bmn_nama"),
                jenis_bmn: row.get("jenis_bmn"),
                last_used_date: row.get("last_used_date"),
                days_since_last_use: row.get("days_since_last_use"),
            })
            .collect();

        Ok(super::models::BmnUtilizationReport {
            total_bmn,
            bmn_with_active_permits,
            bmn_without_permits,
            utilization_rate,
            bmn_by_type,
            top_utilized_bmn,
            underutilized_bmn,
        })
    }

    /// Helper function to convert database row to IzinPemakaianBmn
    fn row_to_permit(&self, row: tokio_postgres::Row) -> IzinPemakaianBmn {
        IzinPemakaianBmn {
            id: row.get("id"),
            nomor_izin: row.get("nomor_izin"),
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            pegawai_satker_id: row.get("pegawai_satker_id"),
            pegawai_satker_nama: row.get("pegawai_satker_nama"),
            pegawai_jabatan: row.get("pegawai_jabatan"),
            jenis_bmn: row.get("jenis_bmn"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.get("bmn_merk"),
            bmn_tahun_perolehan: row.get("bmn_tahun_perolehan"),
            no_polisi: row.get("no_polisi"),
            no_bpkb: row.get("no_bpkb"),
            no_stnk: row.get("no_stnk"),
            no_rangka: row.get("no_rangka"),
            no_mesin: row.get("no_mesin"),
            alamat: row.get("alamat"),
            luas_tanah: row.get("luas_tanah"),
            luas_bangunan: row.get("luas_bangunan"),
            serial_number: row.get("serial_number"),
            spesifikasi: row.get("spesifikasi"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            keperluan: row.get("keperluan"),
            lokasi_pemakaian: row.get("lokasi_pemakaian"),
            is_renewal: row.get("is_renewal"),
            previous_permit_id: row.get("previous_permit_id"),
            file_pendukung: row.get("file_pendukung"),
            document_id: row.get("document_id"),
            document_url: row.get("document_url"),
            status: row.get("status"),
            catatan_approval: row.get("catatan_approval"),
            catatan_revocation: row.get("catatan_revocation"),
            approved_by: row.get("approved_by"),
            approved_by_nama: row.get("approved_by_nama"),
            approved_at: row.get("approved_at"),
            revoked_by: row.get("revoked_by"),
            revoked_by_nama: row.get("revoked_by_nama"),
            revoked_at: row.get("revoked_at"),
            created_by: row.get("created_by"),
            created_by_nama: row.get("created_by_nama"),
            updated_by: row.get("updated_by"),
            updated_by_nama: row.get("updated_by_nama"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
