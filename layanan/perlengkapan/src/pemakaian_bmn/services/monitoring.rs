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

    /// Every rule that governs a permit's asset LIST, checked before a single
    /// row is written.
    ///
    /// Create and draft-edit both go through here. Two doors onto one list
    /// with the rules written at each would drift — that is the shape behind
    /// #93 and the pakaian-dinas scoping gaps, where one entry point was fixed
    /// and its siblings were not.
    ///
    /// `exclude_permit_id` is the permit being edited: its own rows must not
    /// be reported as conflicting with themselves.
    pub(crate) async fn validate_items(
        &self,
        items: &[CreateBmnItemRequest],
        default_mulai: chrono::NaiveDate,
        default_selesai: chrono::NaiveDate,
        satker: &str,
        exclude_permit_id: Option<uuid::Uuid>,
    ) -> AppResult<()> {
        // A permit with no asset is not a permit. The database cannot express
        // "at least one child row", so the rule lives here.
        if items.is_empty() {
            return Err(AppError::BadRequest(
                "Minimal satu BMN harus dipilih untuk permohonan ini".to_string(),
            ));
        }

        let mut terlihat: std::collections::HashSet<(&str, &str)> =
            std::collections::HashSet::new();
        for item in items {
            self.validate_bmn_type_fields(item)?;

            let mulai = item.tanggal_mulai.unwrap_or(default_mulai);
            let selesai = item.tanggal_selesai.unwrap_or(default_selesai);
            if selesai <= mulai {
                return Err(AppError::BadRequest(format!(
                    "BMN {} (kode barang {}): tanggal selesai harus lebih besar dari tanggal mulai",
                    item.bmn_nup, item.bmn_kode_barang
                )));
            }

            // The same asset twice on one permit is a mistake the list makes
            // possible, and the booking check below cannot catch it — neither
            // row exists yet, so neither conflicts with the other.
            if !terlihat.insert((item.bmn_nup.as_str(), item.bmn_kode_barang.as_str())) {
                return Err(AppError::BadRequest(format!(
                    "BMN {} (kode barang {}) tercantum lebih dari sekali pada permohonan ini",
                    item.bmn_nup, item.bmn_kode_barang
                )));
            }

            // Availability is keyed on the full asset identity — kode satker +
            // kode barang + NUP. Keying on `bmn_nup` alone unites 44 017
            // assets across 553 satker that all carry NUP `1` (see the header
            // of `repository::lookup`). Since satker is part of the key, any
            // conflict found belongs to the caller's own satker: nothing leaks
            // that would have to be redacted.
            if let Some(bentrok) = self
                .repository
                .find_booking_conflict(
                    &item.bmn_nup,
                    &item.bmn_kode_barang,
                    satker,
                    exclude_permit_id,
                )
                .await?
            {
                return Err(AppError::BadRequest(format!(
                    "BMN {} (kode barang {}) sedang digunakan oleh {} hingga {}",
                    item.bmn_nup, item.bmn_kode_barang, bentrok.holder, bentrok.expires
                )));
            }
        }

        Ok(())
    }

    /// Validate the kind-specific fields of ONE asset.
    ///
    /// Per item since V014: a permit can hold a vehicle and a laptop at once,
    /// so "which fields are required" is a question about an asset, not about
    /// the permit. Messages name the asset, because on a permit over five BMN
    /// "Nomor polisi harus diisi" does not say which row to fix.
    ///
    /// The DB carries two of these rules as CHECK constraints
    /// (`chk_item_vehicle_fields`, `chk_item_housing_fields`) — deliberately
    /// both places: the CHECK is the backstop that cannot be bypassed, this is
    /// the one that can explain itself.
    pub(crate) fn validate_bmn_type_fields(&self, item: &CreateBmnItemRequest) -> AppResult<()> {
        let jenis_bmn = JenisBmn::from_str(&item.jenis_bmn).ok_or_else(|| {
            AppError::BadRequest(format!("Invalid jenis_bmn: {}", item.jenis_bmn))
        })?;
        let aset = format!(
            "BMN {} (kode barang {})",
            item.bmn_nup, item.bmn_kode_barang
        );

        match jenis_bmn {
            JenisBmn::KendaraanBermotor => {
                if item.no_polisi.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: nomor polisi harus diisi untuk kendaraan bermotor"
                    )));
                }
                // The DB requires BPKB too; rejecting it there surfaces as a
                // bare constraint violation rather than a sentence.
                if item.no_bpkb.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: nomor BPKB harus diisi untuk kendaraan bermotor"
                    )));
                }
            }
            JenisBmn::RumahNegara => {
                if item.alamat.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: alamat harus diisi untuk rumah negara"
                    )));
                }
                if item.luas_tanah.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: luas tanah harus diisi untuk rumah negara"
                    )));
                }
                if item.luas_bangunan.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: luas bangunan harus diisi untuk rumah negara"
                    )));
                }
            }
            JenisBmn::Laptop => {
                if item.serial_number.is_none() {
                    return Err(AppError::BadRequest(format!(
                        "{aset}: serial number harus diisi untuk laptop"
                    )));
                }
            }
            JenisBmn::Lainnya => {
                // No specific validation for other types
            }
        }

        Ok(())
    }
}
