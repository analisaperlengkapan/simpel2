//! The directory of known users, and who a handover notification goes to.
//!
//! The directory lives in [`crate::shared::pengguna`] (every module's handovers read it, and every
//! authenticated request writes it); these are the pakaian dinas repository's names for it.
//! See `migrations/V017__pengguna_dikenal.sql` for why it exists.

use super::PakaianDinasRepository;
use crate::shared::error::AppResult;
use uuid::Uuid;

impl PakaianDinasRepository {
    /// Remember that `user_id` exists, holds `roles`, and sits in `satker_code`.
    pub async fn catat_pengguna(
        &self,
        user_id: Uuid,
        nama: Option<&str>,
        roles: &[String],
        satker_code: Option<&str>,
    ) -> AppResult<()> {
        crate::shared::pengguna::catat(&self.pool, user_id, nama, roles, satker_code).await
    }

    /// The users to tell when work reaches `peran` for the satker `satker_code`.
    pub async fn penerima_peran(&self, peran: &str, satker_code: &str) -> AppResult<Vec<Uuid>> {
        crate::shared::pengguna::penerima_peran(&self.pool, peran, satker_code).await
    }
}
