//! Postgres backings for `authenc_mfa::TotpStore` and `BackupCodesStore`.
//!
//! These let the REST `MfaApiService` adapter persist user MFA state across
//! pod restarts. Both tables (`authenc.totp_secrets`,
//! `authenc.mfa_backup_codes`) ship in migration 049.

use async_trait::async_trait;
use authenc_mfa::{BackupCode, BackupCodesStore, totp::TotpStore};
use authenc_storage::Database;
use authenc_types::{AuthencError, Result, UserId};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::debug;

/// Postgres-backed [`TotpStore`].
///
/// One row per user, keyed by `user_id`. `store_totp_secret` is an upsert so
/// re-enrollment overwrites the prior secret instead of inserting a duplicate.
#[derive(Clone)]
pub struct PgTotpStore {
    db: Arc<Database>,
}

impl PgTotpStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TotpStore for PgTotpStore {
    async fn store_totp_secret(&self, user_id: UserId, secret: &str) -> Result<()> {
        let uid = user_id.0;
        debug!(user_id = %uid, "storing TOTP secret");
        self.db
            .execute(
                r#"
                INSERT INTO authenc.totp_secrets (user_id, secret, created_at, updated_at)
                VALUES ($1, $2, NOW(), NOW())
                ON CONFLICT (user_id) DO UPDATE
                  SET secret = EXCLUDED.secret,
                      updated_at = NOW()
                "#,
                &[&uid, &secret],
            )
            .await?;
        Ok(())
    }

    async fn get_totp_secret(&self, user_id: UserId) -> Result<Option<String>> {
        let uid = user_id.0;
        let row = self
            .db
            .query_opt(
                "SELECT secret FROM authenc.totp_secrets WHERE user_id = $1",
                &[&uid],
            )
            .await?;
        Ok(row.map(|r| r.get::<_, String>("secret")))
    }

    async fn delete_totp_secret(&self, user_id: UserId) -> Result<()> {
        let uid = user_id.0;
        debug!(user_id = %uid, "deleting TOTP secret");
        self.db
            .execute(
                "DELETE FROM authenc.totp_secrets WHERE user_id = $1",
                &[&uid],
            )
            .await?;
        Ok(())
    }
}

/// Postgres-backed [`BackupCodesStore`].
///
/// `store_backup_codes` wipes any prior set in a single statement before
/// inserting the new batch — this matches the upstream service semantics
/// (calling `regenerate_backup_codes` always produces a fresh set).
#[derive(Clone)]
pub struct PgBackupCodesStore {
    db: Arc<Database>,
}

impl PgBackupCodesStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BackupCodesStore for PgBackupCodesStore {
    async fn store_backup_codes(&self, user_id: UserId, codes: &[BackupCode]) -> Result<()> {
        let uid = user_id.0;
        let mut client = self.db.get_connection().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| AuthencError::database(format!("begin tx: {}", e)))?;

        tx.execute(
            "DELETE FROM authenc.mfa_backup_codes WHERE user_id = $1",
            &[&uid],
        )
        .await
        .map_err(|e| AuthencError::database(format!("wipe backup codes: {}", e)))?;

        let insert = tx
            .prepare_cached(
                r#"
                INSERT INTO authenc.mfa_backup_codes (user_id, code, used, used_at, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .await
            .map_err(|e| AuthencError::database(format!("prepare insert: {}", e)))?;

        for code in codes {
            tx.execute(
                &insert,
                &[
                    &uid,
                    &code.code,
                    &code.used,
                    &code.used_at,
                    &code.created_at,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("insert backup code: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AuthencError::database(format!("commit backup codes: {}", e)))?;
        Ok(())
    }

    async fn get_backup_codes(&self, user_id: UserId) -> Result<Option<Vec<BackupCode>>> {
        let uid = user_id.0;
        let rows = self
            .db
            .query(
                r#"
                SELECT code, used, used_at, created_at
                FROM authenc.mfa_backup_codes
                WHERE user_id = $1
                ORDER BY created_at ASC
                "#,
                &[&uid],
            )
            .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let codes = rows
            .into_iter()
            .map(|r| BackupCode {
                code: r.get::<_, String>("code"),
                used: r.get::<_, bool>("used"),
                used_at: r.get::<_, Option<DateTime<Utc>>>("used_at"),
                created_at: r.get::<_, DateTime<Utc>>("created_at"),
            })
            .collect();
        Ok(Some(codes))
    }

    async fn update_backup_code(&self, user_id: UserId, code: &str, used: bool) -> Result<()> {
        let uid = user_id.0;
        // Always stamp `used_at` on transition so admin queries can see when
        // the code burned. NULL keeps it idempotent when un-using (we never
        // do, but the contract permits it).
        let used_at: Option<DateTime<Utc>> = if used { Some(Utc::now()) } else { None };
        self.db
            .execute(
                r#"
                UPDATE authenc.mfa_backup_codes
                SET used = $3, used_at = $4
                WHERE user_id = $1 AND code = $2
                "#,
                &[&uid, &code, &used, &used_at],
            )
            .await?;
        Ok(())
    }

    async fn delete_backup_codes(&self, user_id: UserId) -> Result<()> {
        let uid = user_id.0;
        self.db
            .execute(
                "DELETE FROM authenc.mfa_backup_codes WHERE user_id = $1",
                &[&uid],
            )
            .await?;
        Ok(())
    }
}
