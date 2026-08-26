use super::PemakaianBmnRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
// Which half of `integrasi.siman_aset` is actually written by the SIMAN ingest,
// and in what format. Shared so this report and the bank-aset surfaces cannot
// disagree — they did for four releases, and this file was the last holdout.
use crate::shared::siman_columns::{ASSET_KONDISI_BAIK_PREDICATE, jenis_bmn_sql};

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

    /// Jumlah BMN **kelas ber-izin** (kondisi Baik di SIMAN) yang tidak sedang
    /// dipakai. Feeds the "BMN Tidak Dipakai" card. Dipisah agar kegagalan
    /// SIMAN dapat ditangani best-effort oleh pemanggil.
    ///
    /// Dua perbaikan sekaligus, dan yang kedua BUKAN sekadar bug:
    ///
    /// 1. Kolom mati. `WHERE kondisi = 'BAIK'` cocok 0 dari 624 533 baris —
    ///    kolom `kondisi` tak pernah diisi ingest, dan `ur_kondisi` yang terisi
    ///    menulis "Baik", bukan "BAIK". `total` selalu 0, jadi
    ///    `(0 - utilized).max(0)` = 0: kartu ini menampilkan **0 untuk setiap
    ///    pengguna di setiap satker** sejak ada.
    ///
    /// 2. Penyebutnya salah secara KONSEP, bukan cuma salah kolom. Memperbaiki
    ///    (1) saja mengubah kartu dari 0 menjadi ~548 042 — yaitu seluruh BMN
    ///    kondisi baik se-Indonesia, termasuk 377 985 "Peralatan Mesin Non TIK"
    ///    (kursi, meja, lemari). Izin pemakaian tidak pernah diterbitkan untuk
    ///    kursi, jadi angka itu benar secara aritmatika tapi tak ada artinya —
    ///    dan justru lebih menyesatkan daripada 0 karena terlihat otoritatif.
    ///    Jadi hitungannya dibatasi ke kelas yang MEMANG dipakai lewat izin,
    ///    memakai pemetaan yang sama dengan `jenis_bmn` pada izin itu sendiri.
    ///
    /// ⚠️ Masih NASIONAL, belum ter-scope per-role. Itu disengaja untuk saat
    /// ini: `izin_pemakaian_bmn.pegawai_satker_id` bertipe UUID sedangkan
    /// seluruh sistem lain mengidentifikasi satker lewat MySIMKARI
    /// `kode_satker`, jadi hierarki pusat/wilayah/satker belum bisa di-join ke
    /// tabel ini sama sekali. Pemanggil sudah mengembalikan `None` begitu ada
    /// filter (lihat `get_monitoring_summary`) supaya angka nasional tidak
    /// tersaji seolah-olah angka satker.
    async fn count_idle_bmn(
        client: &deadpool_postgres::Object,
    ) -> Result<i64, tokio_postgres::Error> {
        let total: i64 = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) AS c FROM integrasi.siman_aset \
                     WHERE {kondisi_baik} AND {jenis_bmn} <> 'LAINNYA'",
                    kondisi_baik = ASSET_KONDISI_BAIK_PREDICATE,
                    jenis_bmn = jenis_bmn_sql("")
                ),
                &[],
            )
            .await?
            .get("c");
        // NUP alone is not an asset identity, so this subtraction is an
        // approximation — see the note on ASSET_NUP_SQL. It cannot be made
        // exact until the permit carries kode satker + kode barang + NUP.
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
