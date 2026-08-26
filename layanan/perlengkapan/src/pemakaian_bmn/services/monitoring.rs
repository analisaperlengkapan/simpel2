use super::PemakaianBmnService;
use crate::bank_aset::scope::AsetScope;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use tracing::info;

impl PemakaianBmnService {
    /// Get active usage monitoring dashboard data
    ///
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: MonitoringDashboardQuery,
        scope: &SatkerScope,
    ) -> AppResult<ActiveUsageMonitoringDashboard> {
        info!("Fetching active usage monitoring dashboard");
        self.repository
            .get_active_usage_dashboard(query, scope)
            .await
    }

    /// Daftar pemakaian BMN ter-scope: satker mana, nama barang, NUP, pegawai
    /// pemakai, dan jangka waktunya.
    pub async fn list_pemakaian_monitoring(
        &self,
        query: PemakaianMonitoringQuery,
        scope: &SatkerScope,
    ) -> AppResult<PemakaianBmnMonitoringPage> {
        info!("Listing scoped pemakaian BMN monitoring rows");
        self.repository
            .list_pemakaian_monitoring(query, scope)
            .await
    }

    /// Tiga kartu agregat headline monitoring (Fase 2.6):
    /// sedang dipakai / tidak dipakai / akan expired.
    ///
    /// Dua scope karena kartunya melintasi dua sumber: izin dikunci lewat
    /// MySIMKARI `satker_code`, aset SIMAN lewat `kdsatker_keu`.
    pub async fn get_monitoring_summary(
        &self,
        query: MonitoringDashboardQuery,
        scope: &SatkerScope,
        aset_scope: &AsetScope,
    ) -> AppResult<MonitoringSummaryCards> {
        info!("Fetching pemakaian BMN monitoring summary cards");
        self.repository
            .get_monitoring_summary(query, scope, aset_scope)
            .await
    }

    /// Validate BMN type-specific required fields
    ///
    /// Requirements: REQ-P001
    pub(crate) fn validate_bmn_type_fields(
        &self,
        request: &CreateIzinPemakaianRequest,
    ) -> AppResult<()> {
        let jenis_bmn = JenisBmn::from_str(&request.jenis_bmn).ok_or_else(|| {
            AppError::BadRequest(format!("Invalid jenis_bmn: {}", request.jenis_bmn))
        })?;

        match jenis_bmn {
            JenisBmn::KendaraanBermotor => {
                if request.no_polisi.is_none() {
                    return Err(AppError::BadRequest(
                        "Nomor polisi harus diisi untuk kendaraan bermotor".to_string(),
                    ));
                }
            }
            JenisBmn::RumahNegara => {
                if request.alamat.is_none() {
                    return Err(AppError::BadRequest(
                        "Alamat harus diisi untuk rumah negara".to_string(),
                    ));
                }
                if request.luas_tanah.is_none() {
                    return Err(AppError::BadRequest(
                        "Luas tanah harus diisi untuk rumah negara".to_string(),
                    ));
                }
                if request.luas_bangunan.is_none() {
                    return Err(AppError::BadRequest(
                        "Luas bangunan harus diisi untuk rumah negara".to_string(),
                    ));
                }
            }
            JenisBmn::Laptop => {
                if request.serial_number.is_none() {
                    return Err(AppError::BadRequest(
                        "Serial number harus diisi untuk laptop".to_string(),
                    ));
                }
            }
            JenisBmn::Lainnya => {
                // No specific validation for other types
            }
        }

        Ok(())
    }
}
