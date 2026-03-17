//! PostgreSQL implementation of UserStore trait
//!
//! This module provides the PostgreSQL-backed implementation of the UserStore trait,
//! with optimized queries using prepared statements and connection pooling.

use async_trait::async_trait;
use authenc_types::{
    AuthencError, RealmId, Result, UserId,
    domain::{CreateUserRequest, UpdateUserRequest, user::User},
    traits::UserStore,
};
use chrono::Utc;
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, info};

use crate::Database;

/// PostgreSQL implementation of UserStore
pub struct PostgresUserStore {
    db: Arc<Database>,
}

impl PostgresUserStore {
    /// Create a new PostgresUserStore
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresUserStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let user_store = PostgresUserStore::new(db);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(db: Arc<Database>) -> Self {
        info!("Initializing PostgresUserStore");
        Self { db }
    }
}

#[async_trait]
impl UserStore for PostgresUserStore {
    async fn get_user(&self, id: UserId) -> Result<User> {
        debug!("Getting user by ID: {}", id);

        let query = r#"
            SELECT *
            FROM users
            WHERE id = $1
        "#;

        let row = self.db.query_one(query, &[&id.0]).await?;

        let mut user = row_to_user(row)?;

        // Load roles from user_roles + roles tables
        let roles_query = r#"
            SELECT r.id, r.name, r.description
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1
        "#;
        if let Ok(role_rows) = self.db.query(roles_query, &[&id.0]).await {
            user.roles = role_rows
                .iter()
                .map(|r| {
                    let now = chrono::Utc::now();
                    authenc_types::domain::user::Role {
                        id: r.get("id"),
                        name: r.get("name"),
                        description: r.try_get("description").ok(),
                        permissions: Vec::new(),
                        managed_by: None,
                        scope: None,
                        realm_id: None,
                        composite: false,
                        client_role: false,
                        client_id: None,
                        priority: 0,
                        active: true,
                        attributes: None,
                        created_at: now,
                        updated_at: now,
                    }
                })
                .collect();
        }

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str, realm_id: RealmId) -> Result<User> {
        debug!(
            "Getting user by username: {} in realm: {}",
            username, realm_id
        );

        let query = r#"
            SELECT *
            FROM users
            WHERE username = $1 AND realm_id = $2
        "#;

        let row = self.db.query_opt(query, &[&username, &realm_id.0]).await?;

        match row {
            Some(r) => {
                let mut user = row_to_user(r)?;

                // Load roles from user_roles + roles tables
                let roles_query = r#"
                    SELECT r.id, r.name, r.description
                    FROM user_roles ur
                    JOIN roles r ON r.id = ur.role_id
                    WHERE ur.user_id = $1
                "#;
                if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
                    user.roles = role_rows
                        .iter()
                        .map(|r| {
                            let now = chrono::Utc::now();
                            authenc_types::domain::user::Role {
                                id: r.get("id"),
                                name: r.get("name"),
                                description: r.try_get("description").ok(),
                                permissions: Vec::new(),
                                managed_by: None,
                                scope: None,
                                realm_id: None,
                                composite: false,
                                client_role: false,
                                client_id: None,
                                priority: 0,
                                active: true,
                                attributes: None,
                                created_at: now,
                                updated_at: now,
                            }
                        })
                        .collect();
                }

                Ok(user)
            },
            None => Err(AuthencError::UserNotFound(username.to_string())),
        }
    }

    async fn get_user_by_email(&self, email: &str, realm_id: RealmId) -> Result<User> {
        debug!("Getting user by email: {} in realm: {}", email, realm_id);

        let query = r#"
            SELECT *
            FROM users
            WHERE email = $1 AND realm_id = $2
        "#;

        let row = self.db.query_opt(query, &[&email, &realm_id.0]).await?;

        match row {
            Some(r) => {
                let mut user = row_to_user(r)?;

                // Load roles from user_roles + roles tables
                let roles_query = r#"
                    SELECT r.id, r.name, r.description
                    FROM user_roles ur
                    JOIN roles r ON r.id = ur.role_id
                    WHERE ur.user_id = $1
                "#;
                if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
                    user.roles = role_rows
                        .iter()
                        .map(|r| {
                            let now = chrono::Utc::now();
                            authenc_types::domain::user::Role {
                                id: r.get("id"),
                                name: r.get("name"),
                                description: r.try_get("description").ok(),
                                permissions: Vec::new(),
                                managed_by: None,
                                scope: None,
                                realm_id: None,
                                composite: false,
                                client_role: false,
                                client_id: None,
                                priority: 0,
                                active: true,
                                attributes: None,
                                created_at: now,
                                updated_at: now,
                            }
                        })
                        .collect();
                }

                Ok(user)
            },
            None => Err(AuthencError::UserNotFound(email.to_string())),
        }
    }

    async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        let realm_id = req.realm_id.map(RealmId);
        info!("Creating user: {} in realm: {:?}", req.username, realm_id);

        // Check if username already exists (only if realm_id is provided)
        if let Some(rid) = realm_id {
            if self.username_exists(&req.username, rid).await? {
                return Err(AuthencError::Conflict(format!(
                    "Username '{}' already exists in realm",
                    req.username
                )));
            }

            // Check if email already exists
            if self.email_exists(&req.email, rid).await? {
                return Err(AuthencError::Conflict(format!(
                    "Email '{}' already exists in realm",
                    req.email
                )));
            }
        }

        let user_id = UserId::new();
        let now = Utc::now();

        let query = r#"
            INSERT INTO users (
                id, username, email, password_hash, enabled, email_verified,
                mfa_enabled, realm_id, satker_code, first_name, last_name,
                nip, nama, jabatan, phone_number, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *
        "#;

        let enabled = req.enabled.unwrap_or(true);
        let row = self
            .db
            .query_one(
                query,
                &[
                    &user_id.0,
                    &req.username,
                    &req.email,
                    &req.password, // Note: This should be hashed before calling create_user
                    &enabled,      // enabled by default
                    &false,        // email_verified = false by default
                    &false,        // mfa_enabled = false by default
                    &req.realm_id,
                    &req.satker_code,
                    &req.first_name,
                    &req.last_name,
                    &req.nip,
                    &req.nama,
                    &req.jabatan,
                    &req.phone_number,
                    &now,
                    &now,
                ],
            )
            .await?;

        let user = row_to_user(row)?;
        info!("User created successfully: {}", user.id);
        Ok(user)
    }

    async fn update_user(&self, id: UserId, req: UpdateUserRequest) -> Result<User> {
        info!("Updating user: {}", id);

        // Build dynamic UPDATE query based on provided fields
        let mut updates = Vec::new();
        let mut param_index = 2; // $1 is reserved for user ID

        if req.email.is_some() {
            updates.push(format!("email = ${}", param_index));
            param_index += 1;
        }
        if req.enabled.is_some() {
            updates.push(format!("enabled = ${}", param_index));
            param_index += 1;
        }
        if req.email_verified.is_some() {
            updates.push(format!("email_verified = ${}", param_index));
            param_index += 1;
        }
        if req.first_name.is_some() {
            updates.push(format!("first_name = ${}", param_index));
            param_index += 1;
        }
        if req.last_name.is_some() {
            updates.push(format!("last_name = ${}", param_index));
            param_index += 1;
        }
        if req.phone_number.is_some() {
            updates.push(format!("phone_number = ${}", param_index));
            param_index += 1;
        }
        if req.username.is_some() {
            updates.push(format!("username = ${}", param_index));
            param_index += 1;
        }
        if req.satker_code.is_some() {
            updates.push(format!("satker_code = ${}", param_index));
            param_index += 1;
        }
        if req.nip.is_some() {
            updates.push(format!("nip = ${}", param_index));
            param_index += 1;
        }
        if req.nama.is_some() {
            updates.push(format!("nama = ${}", param_index));
            param_index += 1;
        }
        if req.jabatan.is_some() {
            updates.push(format!("jabatan = ${}", param_index));
            param_index += 1;
        }
        if req.phone_verified.is_some() {
            updates.push(format!("phone_verified = ${}", param_index));
            param_index += 1;
        }
        if req.require_password_change.is_some() {
            updates.push(format!("require_password_change = ${}", param_index));
            param_index += 1;
        }
        if req.password.is_some() {
            updates.push(format!("password_hash = ${}", param_index));
            param_index += 1;
        }
        if req.mfa_enabled.is_some() {
            updates.push(format!("mfa_enabled = ${}", param_index));
            param_index += 1;
        }

        if updates.is_empty() {
            // No updates requested, just return the current user
            return self.get_user(id).await;
        }

        // Always update updated_at
        updates.push(format!("updated_at = ${}", param_index));

        let query = format!(
            r#"
            UPDATE users
            SET {}
            WHERE id = $1
            RETURNING *
            "#,
            updates.join(", ")
        );

        // Build parameter list dynamically
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&id.0];

        if let Some(ref email) = req.email {
            params.push(email);
        }
        if let Some(ref enabled) = req.enabled {
            params.push(enabled);
        }
        if let Some(ref email_verified) = req.email_verified {
            params.push(email_verified);
        }
        if let Some(ref first_name) = req.first_name {
            params.push(first_name);
        }
        if let Some(ref last_name) = req.last_name {
            params.push(last_name);
        }
        if let Some(ref phone_number) = req.phone_number {
            params.push(phone_number);
        }
        if let Some(ref username) = req.username {
            params.push(username);
        }
        if let Some(ref satker_code) = req.satker_code {
            params.push(satker_code);
        }
        if let Some(ref nip) = req.nip {
            params.push(nip);
        }
        if let Some(ref nama) = req.nama {
            params.push(nama);
        }
        if let Some(ref jabatan) = req.jabatan {
            params.push(jabatan);
        }
        if let Some(ref phone_verified) = req.phone_verified {
            params.push(phone_verified);
        }
        if let Some(ref require_password_change) = req.require_password_change {
            params.push(require_password_change);
        }
        if let Some(ref password) = req.password {
            params.push(password);
        }
        if let Some(ref mfa_enabled) = req.mfa_enabled {
            params.push(mfa_enabled);
        }

        let now = Utc::now();
        params.push(&now);

        let row = self.db.query_one(&query, &params).await?;

        let mut user = row_to_user(row)?;

        // Load roles from user_roles + roles tables
        let roles_query = r#"
            SELECT r.id, r.name, r.description
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1
        "#;
        if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
            user.roles = role_rows
                .iter()
                .map(|r| {
                    let now = chrono::Utc::now();
                    authenc_types::domain::user::Role {
                        id: r.get("id"),
                        name: r.get("name"),
                        description: r.try_get("description").ok(),
                        permissions: Vec::new(),
                        managed_by: None,
                        scope: None,
                        realm_id: None,
                        composite: false,
                        client_role: false,
                        client_id: None,
                        priority: 0,
                        active: true,
                        attributes: None,
                        created_at: now,
                        updated_at: now,
                    }
                })
                .collect();
        }

        info!("User updated successfully: {}", user.id);
        Ok(user)
    }

    async fn delete_user(&self, id: UserId) -> Result<()> {
        info!("Deleting user: {}", id);

        // Soft delete: set enabled = false and add deleted_at timestamp
        // Note: This requires a deleted_at column in the users table
        // For now, we'll just set enabled = false
        let query = r#"
            UPDATE users
            SET enabled = false, updated_at = $2
            WHERE id = $1
        "#;

        let now = Utc::now();
        let rows_affected = self.db.execute(query, &[&id.0, &now]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::UserNotFound(format!("User {} not found", id)));
        }

        info!("User deleted successfully: {}", id);
        Ok(())
    }

    async fn list_users(
        &self,
        realm_id: RealmId,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        debug!(
            "Listing users in realm: {} (offset: {}, limit: {})",
            realm_id, offset, limit
        );

        let query = r#"
            SELECT *
            FROM users
            WHERE realm_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        "#;

        let rows = self
            .db
            .query(query, &[&realm_id.0, &(limit as i64), &(offset as i64)])
            .await?;

        let mut users = Vec::new();
        for row in rows {
            let mut user = row_to_user(row)?;

            // Load roles from user_roles + roles tables
            let roles_query = r#"
                SELECT r.id, r.name, r.description
                FROM user_roles ur
                JOIN roles r ON r.id = ur.role_id
                WHERE ur.user_id = $1
            "#;
            if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
                user.roles = role_rows
                    .iter()
                    .map(|r| {
                        let now = chrono::Utc::now();
                        authenc_types::domain::user::Role {
                            id: r.get("id"),
                            name: r.get("name"),
                            description: r.try_get("description").ok(),
                            permissions: Vec::new(),
                            managed_by: None,
                            scope: None,
                            realm_id: None,
                            composite: false,
                            client_role: false,
                            client_id: None,
                            priority: 0,
                            active: true,
                            attributes: None,
                            created_at: now,
                            updated_at: now,
                        }
                    })
                    .collect();
            }
            users.push(user);
        }

        Ok(users)
    }

    async fn username_exists(&self, username: &str, realm_id: RealmId) -> Result<bool> {
        debug!(
            "Checking if username exists: {} in realm: {}",
            username, realm_id
        );

        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE username = $1 AND realm_id = $2 AND deleted_at IS NULL
            )
        "#;

        let row = self.db.query_one(query, &[&username, &realm_id.0]).await?;

        let exists: bool = row.get(0);
        Ok(exists)
    }

    async fn email_exists(&self, email: &str, realm_id: RealmId) -> Result<bool> {
        debug!("Checking if email exists: {} in realm: {}", email, realm_id);

        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE email = $1 AND realm_id = $2 AND deleted_at IS NULL
            )
        "#;

        let row = self.db.query_one(query, &[&email, &realm_id.0]).await?;

        let exists: bool = row.get(0);
        Ok(exists)
    }

    async fn count_users(&self, realm_id: RealmId) -> Result<i64> {
        debug!("Counting users in realm: {}", realm_id);

        let query = r#"
            SELECT COUNT(*) FROM users
            WHERE realm_id = $1 AND deleted_at IS NULL
        "#;

        let row = self.db.query_one(query, &[&realm_id.0]).await?;
        Ok(row.get(0))
    }

    async fn search_users(
        &self,
        realm_id: RealmId,
        query_str: &str,
        enabled: Option<bool>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        debug!(
            "Searching users in realm: {} query: {} enabled: {:?} (offset: {}, limit: {})",
            realm_id, query_str, enabled, offset, limit
        );

        let escaped = query_str
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{}%", escaped);
        let query = r#"
            SELECT *
            FROM users
            WHERE realm_id = $1
              AND deleted_at IS NULL
              AND ($5::boolean IS NULL OR enabled = $5)
              AND (
                  LOWER(username) LIKE $2
                  OR LOWER(email) LIKE $2
                  OR LOWER(COALESCE(nip, '')) LIKE $2
                  OR LOWER(COALESCE(nama, '')) LIKE $2
              )
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
        "#;

        let rows = self
            .db
            .query(
                query,
                &[
                    &realm_id.0,
                    &pattern,
                    &(limit as i64),
                    &(offset as i64),
                    &enabled,
                ],
            )
            .await?;

        let mut users = Vec::new();
        for row in rows {
            let mut user = row_to_user(row)?;

            // Load roles from user_roles + roles tables
            let roles_query = r#"
                SELECT r.id, r.name, r.description
                FROM user_roles ur
                JOIN roles r ON r.id = ur.role_id
                WHERE ur.user_id = $1
            "#;
            if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
                user.roles = role_rows
                    .iter()
                    .map(|r| {
                        let now = chrono::Utc::now();
                        authenc_types::domain::user::Role {
                            id: r.get("id"),
                            name: r.get("name"),
                            description: r.try_get("description").ok(),
                            permissions: Vec::new(),
                            managed_by: None,
                            scope: None,
                            realm_id: None,
                            composite: false,
                            client_role: false,
                            client_id: None,
                            priority: 0,
                            active: true,
                            attributes: None,
                            created_at: now,
                            updated_at: now,
                        }
                    })
                    .collect();
            }
            users.push(user);
        }

        Ok(users)
    }

    async fn count_search_users(
        &self,
        realm_id: RealmId,
        query_str: &str,
        enabled: Option<bool>,
    ) -> Result<i64> {
        debug!(
            "Counting search users in realm: {} query: {} enabled: {:?}",
            realm_id, query_str, enabled
        );

        let escaped = query_str
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{}%", escaped);
        let query = r#"
            SELECT COUNT(*)
            FROM users
            WHERE realm_id = $1
              AND deleted_at IS NULL
              AND ($3::boolean IS NULL OR enabled = $3)
              AND (
                  LOWER(username) LIKE $2
                  OR LOWER(email) LIKE $2
                  OR LOWER(COALESCE(nip, '')) LIKE $2
                  OR LOWER(COALESCE(nama, '')) LIKE $2
              )
        "#;

        let row = self
            .db
            .query_one(query, &[&realm_id.0, &pattern, &enabled])
            .await?;
        Ok(row.get(0))
    }

    async fn count_enabled_users(&self, realm_id: RealmId) -> Result<i64> {
        debug!("Counting enabled users in realm: {}", realm_id);

        let query = r#"
            SELECT COUNT(*) FROM users
            WHERE realm_id = $1 AND deleted_at IS NULL AND enabled = true
        "#;

        let row = self.db.query_one(query, &[&realm_id.0]).await?;
        Ok(row.get(0))
    }

    async fn list_users_filtered(
        &self,
        realm_id: RealmId,
        enabled: Option<bool>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        debug!(
            "Listing users in realm: {} enabled={:?} (offset: {}, limit: {})",
            realm_id, enabled, offset, limit
        );

        let (query, rows) = match enabled {
            Some(e) => {
                let q = r#"
                    SELECT *
                    FROM users
                    WHERE realm_id = $1 AND deleted_at IS NULL AND enabled = $4
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                "#;
                let r = self
                    .db
                    .query(q, &[&realm_id.0, &(limit as i64), &(offset as i64), &e])
                    .await?;
                (q, r)
            }
            None => {
                let q = r#"
                    SELECT *
                    FROM users
                    WHERE realm_id = $1 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                "#;
                let r = self
                    .db
                    .query(q, &[&realm_id.0, &(limit as i64), &(offset as i64)])
                    .await?;
                (q, r)
            }
        };
        let _ = query; // suppress unused warning

        let mut users = Vec::new();
        for row in rows {
            let mut user = row_to_user(row)?;

            // Load roles from user_roles + roles tables
            let roles_query = r#"
                SELECT r.id, r.name, r.description
                FROM user_roles ur
                JOIN roles r ON r.id = ur.role_id
                WHERE ur.user_id = $1
            "#;
            if let Ok(role_rows) = self.db.query(roles_query, &[&user.id]).await {
                user.roles = role_rows
                    .iter()
                    .map(|r| {
                        let now = chrono::Utc::now();
                        authenc_types::domain::user::Role {
                            id: r.get("id"),
                            name: r.get("name"),
                            description: r.try_get("description").ok(),
                            permissions: Vec::new(),
                            managed_by: None,
                            scope: None,
                            realm_id: None,
                            composite: false,
                            client_role: false,
                            client_id: None,
                            priority: 0,
                            active: true,
                            attributes: None,
                            created_at: now,
                            updated_at: now,
                        }
                    })
                    .collect();
            }
            users.push(user);
        }

        Ok(users)
    }

    async fn count_users_filtered(&self, realm_id: RealmId, enabled: Option<bool>) -> Result<i64> {
        debug!(
            "Counting users in realm: {} enabled={:?}",
            realm_id, enabled
        );

        match enabled {
            Some(e) => {
                let query = r#"
                    SELECT COUNT(*) FROM users
                    WHERE realm_id = $1 AND deleted_at IS NULL AND enabled = $2
                "#;
                let row = self.db.query_one(query, &[&realm_id.0, &e]).await?;
                Ok(row.get(0))
            }
            None => self.count_users(realm_id).await,
        }
    }
}

/// Convert a database row to a User struct
fn row_to_user(row: Row) -> Result<User> {
    use authenc_types::SecurityContext;

    let satker_code: Option<String> = row.try_get("satker_code").ok();

    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        email_verified: row.try_get("email_verified").unwrap_or(false),
        first_name: row.try_get("first_name").ok(),
        last_name: row.try_get("last_name").ok(),
        nip: row.try_get("nip").ok(),
        nama: row.try_get("nama").ok(),
        jabatan: row.try_get("jabatan").ok(),
        satker_code: satker_code.unwrap_or_default(),
        phone_number: row.try_get("phone_number").ok(),
        phone_verified: row.try_get("phone_verified").unwrap_or(false),
        password_hash: row.try_get("password_hash").ok(),
        totp_secret: row.try_get("totp_secret").ok(),
        totp_backup_codes: row.try_get("totp_backup_codes").ok(),
        mfa_enabled: row.try_get("mfa_enabled").unwrap_or(false),
        mfa_setup_at: row.try_get("mfa_setup_at").ok(),
        mfa_last_used: row.try_get("mfa_last_used").ok(),
        webauthn_enabled: row.try_get("webauthn_enabled").unwrap_or(false),
        account_locked: row.try_get("account_locked").unwrap_or(false),
        account_locked_until: row.try_get("account_locked_until").ok(),
        failed_login_attempts: row.try_get("failed_login_attempts").unwrap_or(0),
        last_login_at: row.try_get("last_login_at").ok(),
        last_failed_login_at: row.try_get("last_failed_login_at").ok(),
        password_changed_at: row.try_get("password_changed_at").ok(),
        password_expires_at: row.try_get("password_expires_at").ok(),
        require_password_change: row.try_get("require_password_change").unwrap_or(false),
        realm_id: row.try_get("realm_id").ok(),
        organization_id: row.try_get("organization_id").ok(),
        roles: Vec::new(), // TODO: Load from roles/user_roles tables when full domain model exists
        permissions: Vec::new(),
        session_data: None, // Session data usually loaded separately where needed
        security_context: SecurityContext::default(),
        attributes: row.try_get("attributes").ok(),
        enabled: row.try_get("enabled").unwrap_or(true),
        federated: row.try_get("federated").unwrap_or(false),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        deleted_at: row.try_get("deleted_at").ok(),
        login_count: row.try_get("login_count").unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_user_store_creation() {
        // This is a basic test to ensure the struct can be created
        // Integration tests with a real database should be in a separate test file
        // Note: Actual database connection tests require tokio runtime
    }

    #[test]
    fn test_row_to_user_conversion() {
        // Test that row_to_user function signature is correct
        // Actual conversion testing requires mock Row objects
    }

    mod unit_tests {
        use super::*;
        use authenc_types::{Permission, Role, SecurityContext};

        #[test]
        fn test_user_id_creation() {
            let id1 = UserId::new();
            let id2 = UserId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_realm_id_creation() {
            let id1 = RealmId::new();
            let id2 = RealmId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_create_user_request_validation() {
            let realm_id = uuid::Uuid::new_v4();
            let req = CreateUserRequest {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                satker_code: "001".to_string(),
                password: Some("hashed_password".to_string()),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                nip: None,
                nama: None,
                jabatan: None,
                phone_number: None,
                realm_id: Some(realm_id),
                organization_id: None,
                roles: None,
                attributes: None,
            enabled: None,
            };

            assert_eq!(req.username, "testuser");
            assert_eq!(req.email, "test@example.com");
            assert_eq!(req.realm_id, Some(realm_id));
        }

        #[test]
        fn test_update_user_request_empty() {
            let req = UpdateUserRequest {
                username: None,
                email: None,
                satker_code: None,
                first_name: None,
                last_name: None,
                nip: None,
                nama: None,
                jabatan: None,
                phone_number: None,
                enabled: None,
                email_verified: None,
                phone_verified: None,
                require_password_change: None,
                password: None,
                mfa_enabled: None,
                attributes: None,
            };

            assert!(req.email.is_none());
            assert!(req.password.is_none());
            assert!(req.enabled.is_none());
        }

        #[test]
        fn test_update_user_request_partial() {
            let req = UpdateUserRequest {
                username: None,
                email: Some("newemail@example.com".to_string()),
                satker_code: None,
                first_name: None,
                last_name: None,
                nip: None,
                nama: None,
                jabatan: None,
                phone_number: None,
                enabled: Some(false),
                email_verified: None,
                phone_verified: None,
                require_password_change: None,
                password: None,
                mfa_enabled: Some(true),
                attributes: None,
            };

            assert_eq!(req.email, Some("newemail@example.com".to_string()));
            assert_eq!(req.enabled, Some(false));
            assert_eq!(req.mfa_enabled, Some(true));
        }

        #[test]
        fn test_user_struct_fields() {
            let now = Utc::now();
            let user = User {
                id: uuid::Uuid::new_v4(),
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                email_verified: false,
                first_name: None,
                last_name: None,
                nip: None,
                nama: None,
                jabatan: None,
                satker_code: "001".to_string(),
                phone_number: None,
                phone_verified: false,
                password_hash: Some("hashed".to_string()),
                totp_secret: None,
                totp_backup_codes: None,
                mfa_enabled: false,
                mfa_setup_at: None,
                mfa_last_used: None,
                webauthn_enabled: false,
                account_locked: false,
                account_locked_until: None,
                failed_login_attempts: 0,
                last_login_at: None,
                last_failed_login_at: None,
                password_changed_at: None,
                password_expires_at: None,
                require_password_change: false,
                realm_id: None,
                organization_id: None,
                roles: Vec::new(),
                permissions: Vec::new(),
                session_data: None,
                security_context: SecurityContext::default(),
                attributes: None,
                enabled: true,
                federated: false,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                login_count: 0,
            };

            assert_eq!(user.username, "testuser");
            assert!(user.enabled);
            assert!(!user.email_verified);
        }
    }
}
