use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::AppResult;
use crate::shared::satker_scope::SatkerScope;

impl PemakaianBmnService {
    /// Get BMN usage history, restricted to the caller's satker scope.
    ///
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(
        &self,
        bmn_nup: &str,
        scope: &SatkerScope,
    ) -> AppResult<BmnUsageStats> {
        self.repository.get_bmn_usage_history(bmn_nup, scope).await
    }

    /// Get pegawai usage history, restricted to the caller's satker scope.
    ///
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(
        &self,
        pegawai_nip: &str,
        scope: &SatkerScope,
    ) -> AppResult<PegawaiUsageStats> {
        self.repository
            .get_pegawai_usage_history(pegawai_nip, scope)
            .await
    }
}
