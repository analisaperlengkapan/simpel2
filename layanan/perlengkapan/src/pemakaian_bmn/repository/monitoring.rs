use super::PemakaianBmnRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;

impl PemakaianBmnRepository {
    /// Get active usage monitoring dashboard data
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: crate::pemakaian_bmn::models::MonitoringDashboardQuery,
    ) -> AppResult<crate::pemakaian_bmn::models::ActiveUsageMonitoringDashboard> {
        let client = self.pool.client().await?;

        // Build WHERE clause for filters
        let mut where_clauses = vec!["status = 'ACTIVE'".to_string()];
        let mut param_idx = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];

        if let Some(ref satker_id) = query.satker_id {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(*satker_id));
        }

        if let Some(ref jenis_bmn) = query.jenis_bmn {
            where_clauses.push(format!("jenis_bmn = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(jenis_bmn.clone()));
        }
        let _ = param_idx;

        let where_clause = where_clauses.join(" AND ");
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Total active permits
        let total_query = format!(
            "SELECT COUNT(*) as total FROM perlengkapan.izin_pemakaian_bmn WHERE {}",
            where_clause
        );

        let total_row = client
            .query_one(&total_query, &param_refs)
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
            .query(&jenis_query, &param_refs)
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
                crate::pemakaian_bmn::models::PermitsByJenisBmn {
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
            .query(&satker_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permits_by_satker = satker_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::PermitsBySatker {
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
            .query(&expiring_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let expiring_soon = expiring_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::ExpiringPermitInfo {
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
            .query(&recent_query, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let recent_activations = recent_rows
            .into_iter()
            .map(|row| crate::pemakaian_bmn::models::RecentActivationInfo {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                bmn_nama: row.get("bmn_nama_barang"),
                pegawai_nama: row.get("pegawai_nama"),
                activated_at: row.get("approved_at"),
            })
            .collect();

        Ok(
            crate::pemakaian_bmn::models::ActiveUsageMonitoringDashboard {
                total_active_permits,
                permits_by_jenis_bmn,
                permits_by_satker,
                expiring_soon,
                recent_activations,
            },
        )
    }

    /// Get BMN utilization report
    /// Requirements: REQ-P013
    pub async fn get_bmn_utilization_report(
        &self,
        _query: crate::pemakaian_bmn::models::MonitoringDashboardQuery,
    ) -> AppResult<crate::pemakaian_bmn::models::BmnUtilizationReport> {
        let client = self.pool.client().await?;

        // For this report, we need to query SIMAN data (from integrasi schema)
        // to get total BMN count and compare with permits

        // Total BMN count (from SIMAN integration)
        let total_bmn_query = r#"
            SELECT COUNT(DISTINCT nup) as total
            FROM integrasi.siman_aset
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
                FROM integrasi.siman_aset
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
                crate::pemakaian_bmn::models::BmnUtilizationByType {
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
            .map(|row| crate::pemakaian_bmn::models::TopUtilizedBmn {
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
            FROM integrasi.siman_aset s
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
            .map(|row| crate::pemakaian_bmn::models::UnderutilizedBmn {
                bmn_nup: row.get("bmn_nup"),
                bmn_nama: row.get("bmn_nama"),
                jenis_bmn: row.get("jenis_bmn"),
                last_used_date: row.get("last_used_date"),
                days_since_last_use: row.get("days_since_last_use"),
            })
            .collect();

        Ok(crate::pemakaian_bmn::models::BmnUtilizationReport {
            total_bmn,
            bmn_with_active_permits,
            bmn_without_permits,
            utilization_rate,
            bmn_by_type,
            top_utilized_bmn,
            underutilized_bmn,
        })
    }

    /// Tiga kartu agregat headline dashboard monitoring (Fase 2.6):
    /// **sedang dipakai / tidak dipakai / akan expired**.
    ///
    /// Kartu turunan-izin (`sedang_dipakai`, `akan_expired_30d`) menghormati
    /// filter `satker_id`/`jenis_bmn`. `tidak_dipakai` hanya dihitung saat
    /// TANPA filter — angka SIMAN tidak ter-scope per-satker di sini, jadi
    /// menampilkannya saat ter-filter akan menyesatkan (→ `None`). SIMAN
    /// best-effort: jika query SIMAN gagal, `tidak_dipakai = None` dan kartu
    /// lain tetap tersaji (dashboard tidak ikut tumbang).
    pub async fn get_monitoring_summary(
        &self,
        query: crate::pemakaian_bmn::models::MonitoringDashboardQuery,
    ) -> AppResult<crate::pemakaian_bmn::models::MonitoringSummaryCards> {
        let client = self.pool.client().await?;

        // WHERE dinamis utk kartu turunan-izin (parameterized, anti-SQLi).
        let mut where_clauses = vec!["status = 'ACTIVE'".to_string()];
        let mut param_idx = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];

        if let Some(ref satker_id) = query.satker_id {
            where_clauses.push(format!("pegawai_satker_id = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(*satker_id));
        }
        if let Some(ref jenis_bmn) = query.jenis_bmn {
            where_clauses.push(format!("jenis_bmn = ${}", param_idx));
            param_idx += 1;
            params.push(Box::new(jenis_bmn.clone()));
        }
        let _ = param_idx;

        let where_clause = where_clauses.join(" AND ");
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        // Kartu 1: sedang dipakai (izin ACTIVE).
        let sedang_dipakai: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE {}",
                    where_clause
                ),
                &param_refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 2: akan expired dalam 30 hari.
        let akan_expired_30d: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM perlengkapan.izin_pemakaian_bmn \
                     WHERE {} AND tanggal_selesai BETWEEN CURRENT_DATE AND CURRENT_DATE + 30",
                    where_clause
                ),
                &param_refs,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .get("c");

        // Kartu 3: tidak dipakai — hanya saat tanpa filter (SIMAN tidak
        // ter-scope per-satker di sini). Best-effort: error SIMAN → None.
        let tidak_dipakai = if query.satker_id.is_none() && query.jenis_bmn.is_none() {
            match Self::count_idle_bmn(&client).await {
                Ok(n) => Some(n),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "SIMAN tidak tersedia utk kartu 'tidak dipakai' — disajikan null"
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(crate::pemakaian_bmn::models::MonitoringSummaryCards {
            sedang_dipakai,
            akan_expired_30d,
            tidak_dipakai,
        })
    }

    /// Jumlah BMN (kondisi BAIK di SIMAN) yg tidak punya izin ACTIVE =
    /// total distinct NUP − distinct NUP terpakai. Dipisah agar kegagalan
    /// SIMAN dapat ditangani best-effort oleh pemanggil.
    async fn count_idle_bmn(
        client: &deadpool_postgres::Object,
    ) -> Result<i64, tokio_postgres::Error> {
        let total: i64 = client
            .query_one(
                "SELECT COUNT(DISTINCT nup) AS c FROM integrasi.siman_aset WHERE kondisi = 'BAIK'",
                &[],
            )
            .await?
            .get("c");
        let utilized: i64 = client
            .query_one(
                "SELECT COUNT(DISTINCT bmn_nup) AS c FROM perlengkapan.izin_pemakaian_bmn WHERE status = 'ACTIVE'",
                &[],
            )
            .await?
            .get("c");
        Ok((total - utilized).max(0))
    }
}
