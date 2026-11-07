//! The directory of known users, and who a handover notification goes to.
//!
//! See `migrations/V017__pengguna_dikenal.sql` for why this exists.

use super::PakaianDinasRepository;
use crate::shared::error::{AppResult, bad_request};
use uuid::Uuid;

/// A user is re-written at most this often. The table is read when a handover
/// happens, not on every request, so a stale `last_seen` costs nothing; writing
/// it on every list call would turn each page load into a write.
const CATAT_ULANG_MENIT: i32 = 10;

impl PakaianDinasRepository {
    /// Remember that `user_id` exists, holds `roles`, and sits in `satker_code`.
    ///
    /// Roles are merged, never replaced: with session-scoped role activation a
    /// token carries one role, and a user who switched from operator to validator
    /// is still reachable as both.
    pub async fn catat_pengguna(
        &self,
        user_id: Uuid,
        nama: Option<&str>,
        roles: &[String],
        satker_code: Option<&str>,
    ) -> AppResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        client
            .execute(
                "INSERT INTO perlengkapan.pengguna_dikenal \
                    (user_id, nama, roles, satker_code, last_seen) \
                 VALUES ($1, $2, $3, $4, now()) \
                 ON CONFLICT (user_id) DO UPDATE SET \
                    nama = COALESCE(EXCLUDED.nama, pengguna_dikenal.nama), \
                    roles = ARRAY(SELECT DISTINCT unnest(pengguna_dikenal.roles || EXCLUDED.roles)), \
                    satker_code = COALESCE(EXCLUDED.satker_code, pengguna_dikenal.satker_code), \
                    last_seen = now() \
                 WHERE pengguna_dikenal.last_seen < now() - make_interval(mins => $5) \
                    OR NOT (pengguna_dikenal.roles @> EXCLUDED.roles) \
                    OR pengguna_dikenal.satker_code IS DISTINCT FROM EXCLUDED.satker_code",
                &[&user_id, &nama, &roles, &satker_code, &CATAT_ULANG_MENIT],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }

    /// The users to tell when work reaches `peran` for the satker `satker_code`.
    ///
    /// - `operator_satker`: operators of that satker.
    /// - `validator_wilayah`: wilayah validators whose own satker shares the
    ///   Kejati above `satker_code` (the same definition every scope reads,
    ///   `integrasi.v_satker_wilayah`).
    /// - `validator_pusat`: every pusat validator.
    pub async fn penerima_peran(&self, peran: &str, satker_code: &str) -> AppResult<Vec<Uuid>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows =
            match peran {
                "operator_satker" => {
                    client
                        .query(
                            "SELECT user_id FROM perlengkapan.pengguna_dikenal \
                         WHERE $1 = ANY(roles) AND satker_code = $2",
                            &[&peran, &satker_code],
                        )
                        .await
                }
                "validator_wilayah" => {
                    client
                        .query(
                            "SELECT p.user_id FROM perlengkapan.pengguna_dikenal p \
                         WHERE $1 = ANY(p.roles) AND EXISTS ( \
                             SELECT 1 FROM integrasi.v_satker_wilayah mine \
                             JOIN integrasi.v_satker_wilayah theirs \
                               ON theirs.wilayah_code = mine.wilayah_code \
                             WHERE mine.kode_satker = p.satker_code \
                               AND theirs.kode_satker = $2)",
                            &[&peran, &satker_code],
                        )
                        .await
                }
                _ => client
                    .query(
                        "SELECT user_id FROM perlengkapan.pengguna_dikenal WHERE $1 = ANY(roles)",
                        &[&peran],
                    )
                    .await,
            }
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(|r| r.get::<_, Uuid>("user_id")).collect())
    }
}
