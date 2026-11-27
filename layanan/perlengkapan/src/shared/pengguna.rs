//! The directory of known users: who can be told things.
//!
//! A notification needs a recipient, and this service cannot list users by role (that lives in
//! authenc). So it learns them from use: every authenticated request records the caller's id, the
//! roles their token carried and their satker (`perlengkapan.pengguna_dikenal`), and "tell the next
//! role" is a lookup in that table. The people who can receive a notification are exactly the
//! people who use the app, because the bell is the only place one is read.
//!
//! It used to be written by four pakaian dinas handlers only, so a validator who had never opened
//! that screen could never be told that work was waiting for them, whichever module it was in.
//! [`catat_pengguna_middleware`] now records every authenticated caller, wherever they land.

use crate::shared::error::{AppResult, bad_request};
use crate::shared::middleware::Claims;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_postgres::Pool;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

/// A user is re-written at most this often. The table is read when a handover happens, not on
/// every request, so a stale `last_seen` costs nothing; writing it on every request would turn
/// each page load into a write.
const CATAT_ULANG: Duration = Duration::from_secs(10 * 60);
const CATAT_ULANG_MENIT: i32 = 10;

/// Above this many remembered users the throttle map is swept of entries that have expired. A
/// bound, not a business rule: it keeps the map from growing for the life of the process.
const BATAS_INGATAN: usize = 20_000;

/// Remember that `user_id` exists, holds `roles`, and sits in `satker_code`.
///
/// Roles are merged, never replaced: with session-scoped role activation a token carries one role,
/// and a user who switched from operator to validator is still reachable as both.
pub async fn catat(
    pool: &Pool,
    user_id: Uuid,
    nama: Option<&str>,
    roles: &[String],
    satker_code: Option<&str>,
) -> AppResult<()> {
    let client = pool.get().await.map_err(|e| bad_request(&e.to_string()))?;
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
/// - `operator_satker`, `validator_satker`, `approver_satker`: the holders of that role in that
///   satker. (The satker-bound roles used to fall into the last arm and reach EVERY holder in the
///   country.)
/// - `validator_wilayah`: wilayah validators whose own satker shares the Kejati above
///   `satker_code` (the same definition every scope reads, `integrasi.v_satker_wilayah`).
/// - anything else (`validator_pusat`, `admin`): every holder of the role.
pub async fn penerima_peran(pool: &Pool, peran: &str, satker_code: &str) -> AppResult<Vec<Uuid>> {
    penerima_peran_untuk(pool, peran, &[satker_code.to_string()]).await
}

/// [`penerima_peran`] for several satkers at once (a campaign that reaches many): one query, and a
/// person who qualifies through more than one satker appears once. `satker_codes` is ignored for
/// the roles that are not bound to a satker.
pub async fn penerima_peran_untuk(
    pool: &Pool,
    peran: &str,
    satker_codes: &[String],
) -> AppResult<Vec<Uuid>> {
    let client = pool.get().await.map_err(|e| bad_request(&e.to_string()))?;
    let rows = match peran {
        "operator_satker" | "validator_satker" | "approver_satker" => {
            client
                .query(
                    "SELECT DISTINCT user_id FROM perlengkapan.pengguna_dikenal \
                     WHERE $1 = ANY(roles) AND satker_code = ANY($2::text[])",
                    &[&peran, &satker_codes],
                )
                .await
        }
        "validator_wilayah" => {
            client
                .query(
                    "SELECT DISTINCT p.user_id FROM perlengkapan.pengguna_dikenal p \
                     WHERE $1 = ANY(p.roles) AND EXISTS ( \
                         SELECT 1 FROM integrasi.v_satker_wilayah mine \
                         JOIN integrasi.v_satker_wilayah theirs \
                           ON theirs.wilayah_code = mine.wilayah_code \
                         WHERE mine.kode_satker = p.satker_code \
                           AND theirs.kode_satker = ANY($2::text[]))",
                    &[&peran, &satker_codes],
                )
                .await
        }
        _ => {
            client
                .query(
                    "SELECT user_id FROM perlengkapan.pengguna_dikenal WHERE $1 = ANY(roles)",
                    &[&peran],
                )
                .await
        }
    }
    .map_err(|e| bad_request(&e.to_string()))?;
    Ok(rows.iter().map(|r| r.get::<_, Uuid>("user_id")).collect())
}

/// What was last written for a user, to tell whether a new request changes anything.
struct Jejak {
    pada: Instant,
    sidik: u64,
}

/// Records every authenticated caller, without a write per request.
///
/// A caller is written when they have not been seen for [`CATAT_ULANG`], or when what their token
/// carries (roles, satker) differs from what was last written. Everything else is a lookup in a map
/// in memory. The write itself runs detached, so a slow or failing database never delays the
/// request it rides on.
pub struct PenggunaRegistry {
    pool: Pool,
    terakhir: Mutex<HashMap<Uuid, Jejak>>,
}

impl PenggunaRegistry {
    pub fn new(pool: Pool) -> Arc<Self> {
        Arc::new(Self {
            pool,
            terakhir: Mutex::new(HashMap::new()),
        })
    }

    /// A fingerprint of what the table records about a caller.
    fn sidik(claims: &Claims) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        claims.roles.hash(&mut h);
        claims.satker_code.hash(&mut h);
        h.finish()
    }

    /// Whether this caller is due a write now. Calling it counts as having written: the caller
    /// who gets `true` is expected to write.
    fn perlu_catat(&self, claims: &Claims, sekarang: Instant) -> bool {
        let sidik = Self::sidik(claims);
        let Ok(mut peta) = self.terakhir.lock() else {
            // A poisoned lock only costs the throttle; writing is always safe.
            return true;
        };
        if peta.len() >= BATAS_INGATAN {
            peta.retain(|_, j| sekarang.duration_since(j.pada) < CATAT_ULANG);
        }
        let due = match peta.get(&claims.user_id) {
            Some(j) => j.sidik != sidik || sekarang.duration_since(j.pada) >= CATAT_ULANG,
            None => true,
        };
        if due {
            peta.insert(
                claims.user_id,
                Jejak {
                    pada: sekarang,
                    sidik,
                },
            );
        }
        due
    }

    /// Forget the throttle entry, so a failed write is tried again by the next request instead of
    /// waiting out the interval.
    fn lupakan(&self, user_id: Uuid) {
        if let Ok(mut peta) = self.terakhir.lock() {
            peta.remove(&user_id);
        }
    }

    async fn tulis(&self, claims: &Claims) {
        let nama = claims.nama.as_deref().or(claims.name.as_deref());
        if let Err(e) = catat(
            &self.pool,
            claims.user_id,
            nama,
            &claims.roles,
            claims.satker_code.as_deref(),
        )
        .await
        {
            self.lupakan(claims.user_id);
            tracing::warn!(error = %e, "could not record the user for notifications");
        }
    }
}

/// Layer this INSIDE `require_authentication`: it reads the claims that layer put in the request.
/// A request without claims (a self-authenticating route) passes through untouched.
pub async fn catat_pengguna_middleware(
    State(registry): State<Arc<PenggunaRegistry>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(claims) = request.extensions().get::<Claims>().cloned()
        && registry.perlu_catat(&claims, Instant::now())
    {
        let registry = Arc::clone(&registry);
        tokio::spawn(async move { registry.tulis(&claims).await });
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Arc<PenggunaRegistry> {
        // The pool is never used by these tests, which only exercise the throttle.
        let cfg = deadpool_postgres::Config {
            host: Some("127.0.0.1".into()),
            port: Some(1),
            dbname: Some("x".into()),
            user: Some("x".into()),
            ..Default::default()
        };
        let pool = cfg
            .create_pool(
                Some(deadpool_postgres::Runtime::Tokio1),
                tokio_postgres::NoTls,
            )
            .unwrap();
        PenggunaRegistry::new(pool)
    }

    fn claims(user: u128, role: &str, satker: Option<&str>) -> Claims {
        let mut c = Claims::with_roles(Uuid::from_u128(user), "u", [role]);
        c.satker_code = satker.map(str::to_string);
        c
    }

    #[tokio::test]
    async fn a_caller_is_written_once_per_interval_not_once_per_request() {
        let r = registry();
        let c = claims(1, "validator_wilayah", Some("02.28"));
        let t0 = Instant::now();
        assert!(r.perlu_catat(&c, t0), "first sight is written");
        assert!(
            !r.perlu_catat(&c, t0 + Duration::from_secs(5)),
            "again at once is not"
        );
        assert!(
            !r.perlu_catat(&c, t0 + CATAT_ULANG - Duration::from_secs(1)),
            "still inside the interval"
        );
        assert!(
            r.perlu_catat(&c, t0 + CATAT_ULANG),
            "after the interval last_seen is refreshed"
        );
    }

    #[tokio::test]
    async fn a_change_of_role_or_satker_is_written_at_once() {
        let r = registry();
        let t0 = Instant::now();
        assert!(r.perlu_catat(&claims(1, "operator_satker", Some("02.28")), t0));
        // The same user, now acting as a validator: reachable as both, so this must not wait.
        assert!(r.perlu_catat(&claims(1, "validator_wilayah", Some("02.28")), t0));
        assert!(r.perlu_catat(&claims(1, "validator_wilayah", Some("03.01")), t0));
        assert!(!r.perlu_catat(&claims(1, "validator_wilayah", Some("03.01")), t0));
    }

    #[tokio::test]
    async fn users_do_not_throttle_each_other_and_a_failed_write_is_retried() {
        let r = registry();
        let t0 = Instant::now();
        assert!(r.perlu_catat(&claims(1, "validator_pusat", None), t0));
        assert!(r.perlu_catat(&claims(2, "validator_pusat", None), t0));
        assert!(!r.perlu_catat(&claims(1, "validator_pusat", None), t0));
        r.lupakan(Uuid::from_u128(1));
        assert!(
            r.perlu_catat(&claims(1, "validator_pusat", None), t0),
            "after a failed write the next request tries again"
        );
    }

    #[tokio::test]
    async fn the_throttle_map_is_swept_when_it_grows_past_its_bound() {
        let r = registry();
        let t0 = Instant::now();
        for i in 0..BATAS_INGATAN as u128 {
            r.perlu_catat(&claims(i, "operator_satker", None), t0);
        }
        // Past the interval every old entry has expired; the next caller triggers the sweep.
        let later = t0 + CATAT_ULANG + Duration::from_secs(1);
        assert!(r.perlu_catat(&claims(u128::MAX, "operator_satker", None), later));
        assert_eq!(
            r.terakhir.lock().unwrap().len(),
            1,
            "expired entries were dropped"
        );
    }
}
