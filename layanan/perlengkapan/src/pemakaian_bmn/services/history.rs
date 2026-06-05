use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::AppResult;

impl PemakaianBmnService {
    /// Get BMN usage history
    ///
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(&self, bmn_nup: &str) -> AppResult<BmnUsageStats> {
        self.repository.get_bmn_usage_history(bmn_nup).await
    }

    /// Get pegawai usage history
    ///
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(
        &self,
        pegawai_nip: &str,
    ) -> AppResult<PegawaiUsageStats> {
        self.repository.get_pegawai_usage_history(pegawai_nip).await
    }
}
