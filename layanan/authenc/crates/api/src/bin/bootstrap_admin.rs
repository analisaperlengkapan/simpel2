//! One-shot production bootstrap for authenc: retire the baked-in accounts and
//! create the first real administrator.
//!
//! # Why this exists
//!
//! `migrations/002_seed.sql` runs in EVERY environment and creates two accounts:
//!
//! * `admin` — a superuser whose password nobody has documented;
//! * `199203142014031001` — holder of operator + validator_wilayah +
//!   validator_pusat + admin, whose password is its own NIP. That NIP is in the
//!   public repo, in the e2e specs and in `docker-compose.e2e.yml`.
//!
//! On a government system reachable from the internet that is a login for
//! anyone who can read the repository. `docs/audit-pr-930.md` (A13) flagged it
//! as "needs confirming it was rotated or removed in prod"; production had
//! simply never been deployed, so nobody could confirm anything. There is also
//! no other way to obtain a first administrator: user creation needs an
//! authenticated admin, so a fresh production database has nobody who can log
//! in once the seeds are gone.
//!
//! This binary does both halves inside ONE transaction, so there is no moment at
//! which neither (or both) exist:
//!
//! 1. disable + lock the seeded accounts (unless one of them is the NIP the
//!    operator chose as first admin — then it is reclaimed with a fresh
//!    password rather than left with the public one);
//! 2. create the first admin (role `admin` only — least privilege; every other
//!    role is granted from the IAM UI by a person) with
//!    `require_password_change = true`, so the initial password handed over out
//!    of band is good for exactly one login.
//!
//! It is idempotent: a second run (every `helm upgrade` re-runs the hook) never
//! resets a password that has since been rotated.
//!
//! # Configuration (environment)
//!
//! | variable                       | required | meaning                                |
//! |--------------------------------|----------|----------------------------------------|
//! | `DATABASE_URL`                 | yes      | same URL authenc uses                  |
//! | `BOOTSTRAP_ADMIN_USERNAME`     | yes      | login name — the administrator's NIP   |
//! | `BOOTSTRAP_ADMIN_PASSWORD`     | yes      | initial password (12+ chars)           |
//! | `BOOTSTRAP_ADMIN_SATKER_CODE`  | yes      | satker the account belongs to          |
//! | `BOOTSTRAP_ADMIN_EMAIL`        | no       | default `<username>@kejaksaan.go.id`   |
//! | `BOOTSTRAP_ADMIN_NAMA`         | no       | display name                           |
//!
//! The password is never logged.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use authenc_storage::Database;
use tracing::{error, info};
use uuid::Uuid;

/// Ids of the accounts `migrations/002_seed.sql` inserts (`admin`, and the
/// NIP `199203142014031001`). Matched by id AND not by name alone, so a real
/// person who happens to be called `admin` is never mistaken for the seed.
const SEED_USER_IDS: [&str; 2] = [
    "00000000-0000-0000-0000-000000000001",
    "274c04e6-a811-4fe8-a940-13f0f1098a97",
];

/// Minimum length of the initial password. The account must change it on first
/// login (`require_password_change`), so this only has to survive being typed
/// into a chat or a ticket once.
const MIN_PASSWORD_LEN: usize = 12;

/// Reject initial passwords that are obviously guessable. Kept deliberately
/// small — the real password policy runs at the forced change on first login.
pub fn validate_initial_password(username: &str, password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(format!(
            "BOOTSTRAP_ADMIN_PASSWORD must be at least {MIN_PASSWORD_LEN} characters"
        ));
    }
    if password == username || password.contains(username) {
        return Err("BOOTSTRAP_ADMIN_PASSWORD must not contain the username".into());
    }
    let first = password.chars().next().unwrap_or(' ');
    if password.chars().all(|c| c == first) {
        return Err("BOOTSTRAP_ADMIN_PASSWORD must not be one repeated character".into());
    }
    Ok(())
}

fn required(name: &str) -> Result<String, String> {
    match std::env::var(name) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(format!("{name} is not set")),
    }
}

struct Config {
    database_url: String,
    username: String,
    password: String,
    satker_code: String,
    email: String,
    nama: Option<String>,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let username = required("BOOTSTRAP_ADMIN_USERNAME")?.trim().to_string();
        let password = required("BOOTSTRAP_ADMIN_PASSWORD")?;
        validate_initial_password(&username, &password)?;
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            satker_code: required("BOOTSTRAP_ADMIN_SATKER_CODE")?.trim().to_string(),
            email: std::env::var("BOOTSTRAP_ADMIN_EMAIL")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| format!("{username}@kejaksaan.go.id")),
            nama: std::env::var("BOOTSTRAP_ADMIN_NAMA")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            username,
            password,
        })
    }
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("could not hash the initial password: {e}"))
}

async fn run(cfg: Config) -> Result<(), String> {
    let seed_ids: Vec<Uuid> = SEED_USER_IDS
        .iter()
        .map(|s| Uuid::parse_str(s).expect("SEED_USER_IDS are valid uuids"))
        .collect();

    let db = Database::new(&cfg.database_url, 2)
        .await
        .map_err(|e| format!("could not connect to the database: {e}"))?;
    let mut client = db
        .get_connection()
        .await
        .map_err(|e| format!("could not acquire a connection: {e}"))?;
    let tx = client
        .transaction()
        .await
        .map_err(|e| format!("could not begin a transaction: {e}"))?;

    // The role must exist (002_seed.sql); failing here beats creating an
    // administrator who can do nothing.
    let admin_role: Uuid = tx
        .query_opt("SELECT id FROM authenc.roles WHERE name = 'admin'", &[])
        .await
        .map_err(|e| format!("role lookup failed: {e}"))?
        .ok_or("role `admin` does not exist — has authenc-migrate run?")?
        .get(0);

    let existing: Option<Uuid> = tx
        .query_opt(
            "SELECT id FROM authenc.users WHERE username = $1 AND deleted_at IS NULL",
            &[&cfg.username],
        )
        .await
        .map_err(|e| format!("user lookup failed: {e}"))?
        .map(|r| r.get(0));

    let hash = hash_password(&cfg.password)?;
    let admin_id = match existing {
        None => {
            let id = Uuid::new_v4();
            tx.execute(
                "INSERT INTO authenc.users (id, username, email, password_hash, \
                 email_verified, enabled, federated, satker_code, nip, nama, \
                 require_password_change, password_changed_at) \
                 VALUES ($1, $2, $3, $4, true, true, false, $5, $2, $6, true, now())",
                &[
                    &id,
                    &cfg.username,
                    &cfg.email,
                    &hash,
                    &cfg.satker_code,
                    &cfg.nama,
                ],
            )
            .await
            .map_err(|e| format!("creating the first admin failed: {e}"))?;
            info!("created first administrator `{}`", cfg.username);
            id
        }
        Some(id) if seed_ids.contains(&id) => {
            // The operator chose a seeded NIP as the first admin. Keeping the
            // public password would defeat the point, so replace it.
            tx.execute(
                "UPDATE authenc.users SET password_hash = $2, enabled = true, \
                 account_locked = false, account_locked_until = NULL, \
                 failed_login_attempts = 0, require_password_change = true, \
                 password_changed_at = now(), satker_code = $3, updated_at = now() \
                 WHERE id = $1",
                &[&id, &hash, &cfg.satker_code],
            )
            .await
            .map_err(|e| format!("reclaiming the seeded account failed: {e}"))?;
            info!(
                "`{}` is a seeded account: password replaced, change forced at first login",
                cfg.username
            );
            id
        }
        Some(id) => {
            info!(
                "`{}` already exists and is not a seeded account: left untouched \
                 (a rotated password is never reset)",
                cfg.username
            );
            id
        }
    };

    let granted = tx
        .execute(
            "INSERT INTO authenc.user_roles (user_id, role_id) \
             SELECT $1, $2 WHERE NOT EXISTS \
             (SELECT 1 FROM authenc.user_roles WHERE user_id = $1 AND role_id = $2)",
            &[&admin_id, &admin_role],
        )
        .await
        .map_err(|e| format!("granting role `admin` failed: {e}"))?;
    info!("role `admin`: {} new grant(s)", granted);

    let retired = tx
        .execute(
            "UPDATE authenc.users SET enabled = false, account_locked = true, \
             updated_at = now() \
             WHERE id = ANY($1) AND id <> $2 AND (enabled OR NOT account_locked)",
            &[&seed_ids, &admin_id],
        )
        .await
        .map_err(|e| format!("retiring the seeded accounts failed: {e}"))?;
    info!("seeded accounts disabled and locked this run: {retired}");

    tx.commit()
        .await
        .map_err(|e| format!("commit failed: {e}"))?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("authenc-bootstrap-admin: {e}");
            std::process::exit(1);
        }
    };
    match run(cfg).await {
        Ok(()) => info!("authenc-bootstrap-admin: done ✅"),
        Err(e) => {
            error!("authenc-bootstrap-admin: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_password_is_rejected() {
        assert!(validate_initial_password("198001012005011001", "Abcdef12345").is_err());
    }

    #[test]
    fn password_equal_to_or_containing_the_username_is_rejected() {
        let nip = "198001012005011001";
        assert!(validate_initial_password(nip, nip).is_err());
        assert!(validate_initial_password(nip, &format!("x{nip}x")).is_err());
    }

    #[test]
    fn one_repeated_character_is_rejected() {
        assert!(validate_initial_password("u", "aaaaaaaaaaaaaa").is_err());
    }

    #[test]
    fn a_reasonable_password_is_accepted() {
        assert!(validate_initial_password("198001012005011001", "Lonceng-Biru-47!").is_ok());
    }

    #[test]
    fn seed_ids_are_valid_uuids() {
        for s in SEED_USER_IDS {
            assert!(Uuid::parse_str(s).is_ok(), "{s}");
        }
    }

    #[test]
    fn hash_is_argon2id_and_verifies() {
        use argon2::password_hash::{PasswordHash, PasswordVerifier};
        let h = hash_password("Lonceng-Biru-47!").expect("hash");
        assert!(h.starts_with("$argon2id$"), "{h}");
        let parsed = PasswordHash::new(&h).expect("phc");
        assert!(
            Argon2::default()
                .verify_password(b"Lonceng-Biru-47!", &parsed)
                .is_ok()
        );
        assert!(
            Argon2::default()
                .verify_password(b"salah", &parsed)
                .is_err()
        );
    }
}
