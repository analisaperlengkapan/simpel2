/// Database operations for users
use crate::{
    database::Database,
    error::Result,
    models::{
        User,
        user::{CreateUserRequest, SecretonAccessPolicy, SecurityContext, UpdateUserRequest},
    },
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Helper function to merge legacy Secreton policy into attributes
///
/// This ensures backward compatibility by reconstructing a default policy
/// from the satker_code if one is not present in the attributes JSON.
fn enrich_attributes_with_policy(
    mut attributes_json: Option<serde_json::Value>,
    satker_code: &str,
) -> Option<serde_json::Value> {
    // Check if policy is missing from attributes
    if attributes_json.is_none()
        || attributes_json
            .as_ref()
            .map(|a| a.get("secreton_access_policy").is_none())
            .unwrap_or(false)
    {
        // Create default policy based on satker_code (legacy behavior)
        let policy = SecretonAccessPolicy {
            allowed_satker_secrets: vec![if satker_code.is_empty() {
                "UNKNOWN".to_string()
            } else {
                satker_code.to_string()
            }],
            access_level: crate::models::user::AccessLevel::ReadOnly,
            time_restrictions: None,
            audit_required: true,
            rate_limit: Some(100),
            allowed_paths: None,
            denied_paths: None,
        };

        if let Ok(policy_json) = serde_json::to_value(policy) {
            if let Some(ref mut attr) = attributes_json {
                if let Some(obj) = attr.as_object_mut() {
                    obj.insert("secreton_access_policy".to_string(), policy_json);
                }
            } else {
                attributes_json = Some(serde_json::json!({
                    "secreton_access_policy": policy_json
                }));
            }
        }
    }
    attributes_json
}

pub async fn create_user(db: &Database, request: &CreateUserRequest) -> Result<User> {
    // Prepare all data outside the async block
    let username = request.username.clone();
    let email = request.email.clone();
    let first_name = request.first_name.clone();
    let last_name = request.last_name.clone();
    let phone_number = request.phone_number.clone();
    let password_hash = request.password.as_ref().map(|p| {
        use argon2::{
            Argon2,
            password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
        };
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(p.as_bytes(), &salt)
            .map(|h| h.to_string())
            .unwrap_or_default()
    });
    let realm_id = request.realm_id;
    let organization_id = request.organization_id;
    let _attributes_json = request
        .attributes
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    // Create user ID and timestamp outside
    let _user_id = Uuid::new_v4();
    let _now = Utc::now();

    let client = db.get_connection().await?;
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO users (
            id, username, email, first_name, last_name,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_failed_login_at,
            password_changed_at, password_expires_at, require_password_change,
            organization_id, attributes, email_verified, enabled,
            realm_id, federated, created_at, updated_at, deleted_at,
            last_login_at, login_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
        RETURNING
            id, username, email, first_name, last_name,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_failed_login_at,
            password_changed_at, password_expires_at, require_password_change,
            organization_id, attributes, email_verified, enabled,
            realm_id, federated, created_at, updated_at, deleted_at,
            last_login_at, login_count
    "#;

    let row = client
        .query_one(
            query,
            &[
                &user_id,
                &username,
                &email,
                &first_name,
                &last_name,
                &phone_number,
                &false, // phone_verified
                &password_hash,
                &None::<String>,                // totp_secret
                &None::<Vec<String>>,           // totp_backup_codes
                &false,                         // webauthn_enabled
                &false,                         // account_locked
                &None::<chrono::DateTime<Utc>>, // account_locked_until
                &0i32,                          // failed_login_attempts
                &None::<chrono::DateTime<Utc>>, // last_failed_login_at
                &None::<chrono::DateTime<Utc>>, // password_changed_at
                &None::<chrono::DateTime<Utc>>, // password_expires_at
                &false,                         // require_password_change
                &organization_id,
                &request.attributes,
                &true,                            // email_verified
                &request.enabled.unwrap_or(true), // enabled
                &realm_id,
                &false, // federated (default to false for regular user creation)
                &now,
                &now,
                &None::<chrono::DateTime<Utc>>, // deleted_at
                &None::<chrono::DateTime<Utc>>, // last_login_at
                &0i32,                          // login_count
            ],
        )
        .await?;

    // Convert row to User by extracting values directly
    let user = row_to_user(&row);

    Ok(user)
}

pub async fn get_user_by_id(db: &Database, user_id: Uuid) -> Result<Option<User>> {
    let query = r#"
        SELECT
            id, username, email, email_verified, first_name, last_name,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_login_at,
            last_failed_login_at, password_changed_at, password_expires_at,
            require_password_change, realm_id, organization_id, attributes,
            enabled, federated, created_at, updated_at, deleted_at, login_count
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_opt(query, &[&user_id]).await?;
    Ok(row.map(|r| row_to_user(&r)))
}

pub async fn get_user_by_username(db: &Database, username: &str) -> Result<Option<User>> {
    let query = r#"
        SELECT
            id, username, email, email_verified, first_name, last_name,
            nip, nama, jabatan, satker_code,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, mfa_enabled, mfa_setup_at, mfa_last_used,
            webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_login_at,
            last_failed_login_at, password_changed_at, password_expires_at,
            require_password_change, realm_id, organization_id, attributes,
            enabled, federated, created_at, updated_at, deleted_at, login_count
        FROM users
        WHERE username = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_opt(query, &[&username]).await?;
    Ok(row.map(|r| row_to_user(&r)))
}

pub async fn get_user_by_email(db: &Database, email: &str) -> Result<Option<User>> {
    let query = r#"
        SELECT
            id, username, email, email_verified, first_name, last_name,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_login_at,
            last_failed_login_at, password_changed_at, password_expires_at,
            require_password_change, realm_id, organization_id, attributes,
            enabled, federated, created_at, updated_at, deleted_at, last_login_at, login_count
        FROM users
        WHERE email = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_one(query, &[&email]).await?;
    Ok(Some(row_to_user(&row)))
}

pub async fn update_user(
    db: &Database,
    user_id: Uuid,
    request: &UpdateUserRequest,
) -> Result<User> {
    let now = Utc::now();

    let query = r#"
        UPDATE users SET
            username = COALESCE($2, username),
            email = COALESCE($3, email),
            first_name = COALESCE($4, first_name),
            last_name = COALESCE($5, last_name),
            phone_number = COALESCE($6, phone_number),
            enabled = COALESCE($7, enabled),
            email_verified = COALESCE($8, email_verified),
            phone_verified = COALESCE($9, phone_verified),
            require_password_change = COALESCE($10, require_password_change),
            attributes = COALESCE($11, attributes),
            updated_at = $12
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING
            id, username, email, first_name, last_name,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_failed_login_at,
            password_changed_at, password_expires_at, require_password_change,
            organization_id, attributes, email_verified, enabled,
            realm_id, federated, created_at, updated_at, deleted_at,
            last_login_at, login_count
    "#;

    let row = db
        .query_one(
            query,
            &[
                &user_id,
                &request.username,
                &request.email,
                &request.first_name,
                &request.last_name,
                &request.phone_number,
                &request.enabled,
                &request.email_verified,
                &request.phone_verified,
                &request.require_password_change,
                &request
                    .attributes
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_default()),
                &now,
            ],
        )
        .await?;

    Ok(row_to_user(&row))
}

pub async fn delete_user(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = "UPDATE users SET deleted_at = $2, updated_at = $2 WHERE id = $1";
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

pub async fn record_login(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE users SET
            last_login_at = $2,
            failed_login_attempts = 0,
            account_locked = false,
            account_locked_until = NULL,
            updated_at = $2
        WHERE id = $1
    "#;
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

pub async fn record_failed_login(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE users SET
            failed_login_attempts = failed_login_attempts + 1,
            last_failed_login_at = $2,
            updated_at = $2
        WHERE id = $1
    "#;
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

pub async fn update_password(db: &Database, user_id: Uuid, password_hash: &str) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE users SET
            password_hash = $2,
            password_changed_at = $3,
            require_password_change = false,
            updated_at = $3
        WHERE id = $1
    "#;
    db.execute(query, &[&user_id, &password_hash, &now]).await?;
    Ok(())
}

pub async fn enable_webauthn(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = "UPDATE users SET webauthn_enabled = true, updated_at = $2 WHERE id = $1";
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

pub async fn disable_webauthn(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = "UPDATE users SET webauthn_enabled = false, updated_at = $2 WHERE id = $1";
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

pub async fn lock_account(
    db: &Database,
    user_id: Uuid,
    until: Option<DateTime<Utc>>,
) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE users SET
            account_locked = true,
            account_locked_until = $2,
            updated_at = $3
        WHERE id = $1
    "#;
    db.execute(query, &[&user_id, &until, &now]).await?;
    Ok(())
}

pub async fn unlock_account(db: &Database, user_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = r#"
        UPDATE users SET
            account_locked = false,
            account_locked_until = NULL,
            failed_login_attempts = 0,
            updated_at = $2
        WHERE id = $1
    "#;
    db.execute(query, &[&user_id, &now]).await?;
    Ok(())
}

fn row_to_user(row: &tokio_postgres::Row) -> User {
    // Extract satker_code first to use in fallback policy
    let satker_code_val: String = if let Ok(val) = row.try_get("satker_code") {
        val
    } else {
        // Fallback for queries that might select by index or alias, though here we mostly select by name
        // Or if column is missing (should panic in get but try_get is safer if we wanted)
        // Since we used get() in original code, we stick to safe retrieval where possible
        // But for this helper, let's use the index fallback if needed or empty string
        "UNKNOWN".to_string()
    };

    let attributes_raw: Option<serde_json::Value> = row.try_get("attributes").ok();
    let enriched_attributes = enrich_attributes_with_policy(attributes_raw, &satker_code_val);

    User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        email_verified: row.get("email_verified"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        nip: row.try_get("nip").ok(),
        nama: row.try_get("nama").ok(),
        jabatan: row.try_get("jabatan").ok(),
        satker_code: satker_code_val,
        phone_number: row.get("phone_number"),
        phone_verified: row.get("phone_verified"),
        password_hash: row.get("password_hash"),
        totp_secret: row.get("totp_secret"),
        totp_backup_codes: row.get("totp_backup_codes"),
        mfa_enabled: row.try_get("mfa_enabled").unwrap_or(false),
        mfa_setup_at: row.try_get("mfa_setup_at").ok().flatten(),
        mfa_last_used: row.try_get("mfa_last_used").ok().flatten(),
        webauthn_enabled: row.get("webauthn_enabled"),
        account_locked: row.get("account_locked"),
        account_locked_until: row.get("account_locked_until"),
        failed_login_attempts: row.get("failed_login_attempts"),
        last_login_at: row.get("last_login_at"),
        last_failed_login_at: row.get("last_failed_login_at"),
        password_changed_at: row.get("password_changed_at"),
        password_expires_at: row.get("password_expires_at"),
        require_password_change: row.get("require_password_change"),
        realm_id: row.get("realm_id"),
        organization_id: row.get("organization_id"),
        roles: Vec::new(),       // Roles would be loaded separately
        permissions: Vec::new(), // Permissions would be loaded separately
        session_data: None,      // Default to None for now
        security_context: SecurityContext {
            ip_address: None,
            user_agent: None,
            session_id: None,
            timestamp: chrono::Utc::now(),
            risk_score: None,
            metadata: None,
        },
        attributes: enriched_attributes,
        enabled: row.get("enabled"),
        federated: row.get("federated"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        deleted_at: row.get("deleted_at"),
        login_count: row.try_get("login_count").unwrap_or(0),
    }
}

pub async fn get_all_users(db: &Database) -> Result<Vec<User>> {
    let client = db.get_connection().await?;
    let query = r#"
        SELECT
            id, username, email, first_name, last_name, nip, nama, jabatan, satker_code,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, mfa_enabled, mfa_setup_at, mfa_last_used, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_failed_login_at,
            password_changed_at, password_expires_at, require_password_change,
            organization_id, attributes, email_verified, enabled,
            realm_id, federated, created_at, updated_at, deleted_at,
            last_login_at, login_count
        FROM users
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
    "#;

    let rows = client.query(query, &[]).await?;
    let mut users = Vec::new();

    for row in rows {
        users.push(row_to_user(&row));
    }

    Ok(users)
}

pub async fn get_users_by_realm(db: &Database, realm_id: Uuid) -> Result<Vec<User>> {
    let client = db.get_connection().await?;
    let query = r#"
        SELECT
            id, username, email, first_name, last_name,
            COALESCE(nip, '') as nip,
            COALESCE(nama, '') as nama,
            COALESCE(jabatan, '') as jabatan,
            COALESCE(satker_code, '') as satker_code,
            phone_number, phone_verified, password_hash, totp_secret,
            totp_backup_codes, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_failed_login_at,
            password_changed_at, password_expires_at, require_password_change,
            organization_id, attributes, email_verified, enabled,
            realm_id, federated, created_at, updated_at, deleted_at,
            last_login_at, login_count,
            COALESCE(mfa_enabled, false) as mfa_enabled,
            mfa_setup_at, mfa_last_used
        FROM users
        WHERE realm_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
    "#;

    let rows = client.query(query, &[&realm_id]).await?;
    let mut users = Vec::new();

    for row in rows {
        users.push(row_to_user(&row));
    }

    Ok(users)
}

pub async fn bulk_create_users(db: &Database, users: Vec<CreateUserRequest>) -> Result<Vec<User>> {
    if users.is_empty() {
        return Ok(Vec::new());
    }

    let mut client = db.get_connection().await?;
    let transaction = client.transaction().await?;

    let mut created_users = Vec::new();

    for user_req in users {
        let query = r#"
            INSERT INTO users (
                username, email, email_verified, first_name, last_name,
                phone_number, phone_verified, password_hash, enabled,
                realm_id, organization_id, attributes, federated
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING
                id, username, email, first_name, last_name,
                phone_number, phone_verified, password_hash, totp_secret,
                totp_backup_codes, webauthn_enabled, account_locked,
                account_locked_until, failed_login_attempts, last_failed_login_at,
                password_changed_at, password_expires_at, require_password_change,
                organization_id, attributes, email_verified, enabled,
                realm_id, federated, created_at, updated_at, deleted_at,
                last_login_at, login_count
        "#;

        // Hash password if provided
        let password_hash = if let Some(password) = &user_req.password {
            use argon2::{
                Argon2,
                password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
            };
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| {
                    crate::error::AuthencError::database(format!("Password hashing failed: {}", e))
                })?
        } else {
            String::new()
        };

        let attributes_json =
            serde_json::to_string(&user_req.attributes.clone().unwrap_or_default())
                .map_err(|e| crate::error::AuthencError::database(e.to_string()))?;

        let enabled = user_req.enabled.unwrap_or(true);
        let row = transaction
            .query_one(
                query,
                &[
                    &user_req.username,
                    &user_req.email,
                    &false, // email_verified - default false
                    &user_req.first_name,
                    &user_req.last_name,
                    &user_req.phone_number,
                    &false, // phone_verified - default false
                    &password_hash,
                    &enabled, // enabled - default true
                    &user_req.realm_id,
                    &user_req.organization_id,
                    &attributes_json,
                    &false, // federated - default false
                ],
            )
            .await?;

        created_users.push(row_to_user(&row));
    }

    transaction.commit().await?;
    Ok(created_users)
}

pub async fn bulk_update_users(
    db: &Database,
    updates: Vec<(Uuid, serde_json::Value)>,
) -> Result<usize> {
    if updates.is_empty() {
        return Ok(0);
    }

    let mut client = db.get_connection().await?;
    let transaction = client.transaction().await?;

    let mut updated_count = 0;
    let now = Utc::now();

    for (user_id, update_data) in updates {
        let mut set_clauses = Vec::new();
        let mut param_index = 2; // Start from 2 since $1 is user_id
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = vec![Box::new(user_id)];

        // Build dynamic UPDATE query based on provided fields
        if let Some(email) = update_data.get("email").and_then(|v| v.as_str()) {
            set_clauses.push(format!("email = ${}", param_index));
            params.push(Box::new(email.to_string()));
            param_index += 1;
        }

        if let Some(first_name) = update_data.get("first_name").and_then(|v| v.as_str()) {
            set_clauses.push(format!("first_name = ${}", param_index));
            params.push(Box::new(first_name.to_string()));
            param_index += 1;
        }

        if let Some(last_name) = update_data.get("last_name").and_then(|v| v.as_str()) {
            set_clauses.push(format!("last_name = ${}", param_index));
            params.push(Box::new(last_name.to_string()));
            param_index += 1;
        }

        if let Some(enabled) = update_data.get("enabled").and_then(|v| v.as_bool()) {
            set_clauses.push(format!("enabled = ${}", param_index));
            params.push(Box::new(enabled));
            param_index += 1;
        }

        if let Some(email_verified) = update_data.get("email_verified").and_then(|v| v.as_bool()) {
            set_clauses.push(format!("email_verified = ${}", param_index));
            params.push(Box::new(email_verified));
            param_index += 1;
        }

        if set_clauses.is_empty() {
            continue; // No fields to update
        }

        set_clauses.push(format!("updated_at = ${}", param_index));
        params.push(Box::new(now));

        let query = format!("UPDATE users SET {} WHERE id = $1", set_clauses.join(", "));

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref()).collect();

        let affected = transaction.execute(query.as_str(), &params_refs).await?;
        updated_count += affected as usize;
    }

    transaction.commit().await?;
    Ok(updated_count)
}

pub async fn bulk_delete_users(db: &Database, user_ids: Vec<Uuid>) -> Result<usize> {
    if user_ids.is_empty() {
        return Ok(0);
    }

    let now = Utc::now();
    let query = r#"
        UPDATE users
        SET deleted_at = $1, updated_at = $1
        WHERE id = ANY($2) AND deleted_at IS NULL
    "#;

    let count = db.execute(query, &[&now, &user_ids]).await?;
    Ok(count as usize)
}

pub async fn bulk_assign_roles(
    db: &Database,
    assignments: Vec<(Uuid, Uuid)>, // (user_id, role_id) pairs
) -> Result<usize> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let mut client = db.get_connection().await?;
    let transaction = client.transaction().await?;

    let query = r#"
        INSERT INTO user_roles (user_id, role_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, role_id) DO NOTHING
    "#;

    let mut assigned_count = 0;
    for (user_id, role_id) in assignments {
        let affected = transaction.execute(query, &[&user_id, &role_id]).await?;
        assigned_count += affected as usize;
    }

    transaction.commit().await?;
    Ok(assigned_count)
}

pub async fn bulk_remove_roles(
    db: &Database,
    removals: Vec<(Uuid, Uuid)>, // (user_id, role_id) pairs
) -> Result<usize> {
    if removals.is_empty() {
        return Ok(0);
    }

    let mut client = db.get_connection().await?;
    let transaction = client.transaction().await?;

    let query = "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2";

    let mut removed_count = 0;
    for (user_id, role_id) in removals {
        let affected = transaction.execute(query, &[&user_id, &role_id]).await?;
        removed_count += affected as usize;
    }

    transaction.commit().await?;
    Ok(removed_count)
}

pub async fn export_users(db: &Database, realm_id: Option<Uuid>) -> Result<Vec<serde_json::Value>> {
    let query = if realm_id.is_some() {
        r#"
            SELECT
                id, username, email, first_name, last_name,
                phone_number, phone_verified, email_verified,
                enabled, realm_id, organization_id, attributes,
                federated, created_at, updated_at
            FROM users
            WHERE realm_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
        "#
    } else {
        r#"
            SELECT
                id, username, email, first_name, last_name,
                phone_number, phone_verified, email_verified,
                enabled, realm_id, organization_id, attributes,
                federated, created_at, updated_at
            FROM users
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
        "#
    };

    let rows: Vec<tokio_postgres::Row> = if let Some(rid) = realm_id {
        db.query(query, &[&rid]).await?
    } else {
        db.query(query, &[]).await?
    };

    let mut users = Vec::new();
    for row in rows {
        users.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "username": row.get::<_, String>("username"),
            "email": row.get::<_, String>("email"),
            "first_name": row.get::<_, Option<String>>("first_name"),
            "last_name": row.get::<_, Option<String>>("last_name"),
            "phone_number": row.get::<_, Option<String>>("phone_number"),
            "phone_verified": row.get::<_, bool>("phone_verified"),
            "email_verified": row.get::<_, bool>("email_verified"),
            "enabled": row.get::<_, bool>("enabled"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "organization_id": row.get::<_, Option<Uuid>>("organization_id"),
            "attributes": row.get::<_, Option<String>>("attributes"),
            "federated": row.get::<_, bool>("federated"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at"),
        }));
    }

    Ok(users)
}

#[allow(clippy::too_many_arguments)]
pub async fn query_users_advanced(
    db: &Database,
    realm_id: Option<Uuid>,
    search: Option<&str>,
    email_filter: Option<&str>,
    enabled_filter: Option<bool>,
    email_verified_filter: Option<bool>,
    organization_id_filter: Option<Uuid>,
    sort_by: Option<&str>, // "username", "email", "created_at", "last_login_at"
    sort_order: Option<&str>, // "asc" or "desc"
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<(Vec<serde_json::Value>, i64)> {
    let mut where_clauses: Vec<String> = vec!["deleted_at IS NULL".to_string()];
    let mut param_index = 1;
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

    // Realm filter
    if let Some(rid) = realm_id {
        where_clauses.push(format!("realm_id = ${}", param_index));
        params.push(Box::new(rid));
        param_index += 1;
    }

    // Full-text search across username, email, first_name, last_name
    if let Some(search_term) = search
        && !search_term.is_empty()
    {
        where_clauses.push(format!(
                "(username ILIKE ${} OR email ILIKE ${} OR first_name ILIKE ${} OR last_name ILIKE ${})",
                param_index, param_index, param_index, param_index
            ));
        let search_pattern = format!("%{}%", search_term);
        params.push(Box::new(search_pattern));
        param_index += 1;
    }

    // Email filter
    if let Some(email_pattern) = email_filter
        && !email_pattern.is_empty()
    {
        where_clauses.push(format!("email ILIKE ${}", param_index));
        params.push(Box::new(format!("%{}%", email_pattern)));
        param_index += 1;
    }

    // Enabled filter
    if let Some(enabled) = enabled_filter {
        where_clauses.push(format!("enabled = ${}", param_index));
        params.push(Box::new(enabled));
        param_index += 1;
    }

    // Email verified filter
    if let Some(verified) = email_verified_filter {
        where_clauses.push(format!("email_verified = ${}", param_index));
        params.push(Box::new(verified));
        param_index += 1;
    }

    // Organization filter
    if let Some(org_id) = organization_id_filter {
        where_clauses.push(format!("organization_id = ${}", param_index));
        params.push(Box::new(org_id));
        param_index += 1;
    }

    let where_clause = where_clauses.join(" AND ");

    // Sorting
    let sort_column = match sort_by {
        Some("email") => "email",
        Some("created_at") => "created_at",
        Some("last_login_at") => "last_login_at",
        Some("updated_at") => "updated_at",
        _ => "username",
    };

    let sort_direction = match sort_order {
        Some("desc") => "DESC",
        _ => "ASC",
    };

    // Count query
    let count_query = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);

    let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();

    let count_row: tokio_postgres::Row = db.query_one(&count_query, &params_refs).await?;
    let total_count: i64 = count_row.get(0);

    // Data query with pagination
    let data_query = format!(
        r#"
            SELECT
                id, username, email, first_name, last_name,
                phone_number, phone_verified, email_verified,
                enabled, realm_id, organization_id, attributes,
                federated, created_at, updated_at, last_login_at, login_count
            FROM users
            WHERE {}
            ORDER BY {} {}
            LIMIT ${} OFFSET ${}
        "#,
        where_clause,
        sort_column,
        sort_direction,
        param_index,
        param_index + 1
    );

    let limit_val = limit.unwrap_or(20);
    let offset_val = offset.unwrap_or(0);

    let mut data_params = params;
    data_params.push(Box::new(limit_val));
    data_params.push(Box::new(offset_val));

    let data_params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        data_params.iter().map(|p| p.as_ref()).collect();

    let rows: Vec<tokio_postgres::Row> = db.query(&data_query, &data_params_refs).await?;

    let mut users = Vec::new();
    for row in rows {
        users.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "username": row.get::<_, String>("username"),
            "email": row.get::<_, String>("email"),
            "first_name": row.get::<_, Option<String>>("first_name"),
            "last_name": row.get::<_, Option<String>>("last_name"),
            "phone_number": row.get::<_, Option<String>>("phone_number"),
            "phone_verified": row.get::<_, bool>("phone_verified"),
            "email_verified": row.get::<_, bool>("email_verified"),
            "enabled": row.get::<_, bool>("enabled"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "organization_id": row.get::<_, Option<Uuid>>("organization_id"),
            "attributes": row.get::<_, Option<String>>("attributes"),
            "federated": row.get::<_, bool>("federated"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at"),
            "last_login_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("last_login_at"),
            "login_count": row.get::<_, i32>("login_count"),
        }));
    }

    Ok((users, total_count))
}

pub async fn search_users_fulltext(
    db: &Database,
    realm_id: Uuid,
    search_query: &str,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    let query = r#"
        SELECT
            id, username, email, first_name, last_name,
            phone_number, phone_verified, email_verified,
            enabled, realm_id, organization_id, attributes,
            federated, created_at, updated_at, last_login_at, login_count,
            ts_rank(
                to_tsvector('english',
                    COALESCE(username, '') || ' ' ||
                    COALESCE(email, '') || ' ' ||
                    COALESCE(first_name, '') || ' ' ||
                    COALESCE(last_name, '')
                ),
                plainto_tsquery('english', $2)
            ) AS rank
        FROM users
        WHERE realm_id = $1
            AND deleted_at IS NULL
            AND to_tsvector('english',
                COALESCE(username, '') || ' ' ||
                COALESCE(email, '') || ' ' ||
                COALESCE(first_name, '') || ' ' ||
                COALESCE(last_name, '')
            ) @@ plainto_tsquery('english', $2)
        ORDER BY rank DESC
        LIMIT $3
    "#;

    let limit_val = limit.unwrap_or(20);
    let rows: Vec<tokio_postgres::Row> = db
        .query(query, &[&realm_id, &search_query, &limit_val])
        .await?;

    let mut users = Vec::new();
    for row in rows {
        users.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "username": row.get::<_, String>("username"),
            "email": row.get::<_, String>("email"),
            "first_name": row.get::<_, Option<String>>("first_name"),
            "last_name": row.get::<_, Option<String>>("last_name"),
            "phone_number": row.get::<_, Option<String>>("phone_number"),
            "phone_verified": row.get::<_, bool>("phone_verified"),
            "email_verified": row.get::<_, bool>("email_verified"),
            "enabled": row.get::<_, bool>("enabled"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "organization_id": row.get::<_, Option<Uuid>>("organization_id"),
            "federated": row.get::<_, bool>("federated"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at"),
            "last_login_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>("last_login_at"),
            "login_count": row.get::<_, i32>("login_count"),
            "relevance_score": row.get::<_, f32>("rank"),
        }));
    }

    Ok(users)
}

pub async fn query_users_by_attributes(
    db: &Database,
    realm_id: Uuid,
    attribute_filters: serde_json::Value,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    // Build JSONB containment query
    let query = r#"
        SELECT
            id, username, email, first_name, last_name,
            phone_number, phone_verified, email_verified,
            enabled, realm_id, organization_id, attributes,
            federated, created_at, updated_at
        FROM users
        WHERE realm_id = $1
            AND deleted_at IS NULL
            AND attributes @> $2::jsonb
        ORDER BY created_at DESC
        LIMIT $3
    "#;

    let limit_val = limit.unwrap_or(100);
    let rows: Vec<tokio_postgres::Row> = db
        .query(
            query,
            &[&realm_id, &attribute_filters.to_string(), &limit_val],
        )
        .await?;

    let mut users = Vec::new();
    for row in rows {
        users.push(serde_json::json!({
            "id": row.get::<_, Uuid>("id"),
            "username": row.get::<_, String>("username"),
            "email": row.get::<_, String>("email"),
            "first_name": row.get::<_, Option<String>>("first_name"),
            "last_name": row.get::<_, Option<String>>("last_name"),
            "phone_number": row.get::<_, Option<String>>("phone_number"),
            "phone_verified": row.get::<_, bool>("phone_verified"),
            "email_verified": row.get::<_, bool>("email_verified"),
            "enabled": row.get::<_, bool>("enabled"),
            "realm_id": row.get::<_, Uuid>("realm_id"),
            "organization_id": row.get::<_, Option<Uuid>>("organization_id"),
            "attributes": row.get::<_, Option<String>>("attributes"),
            "federated": row.get::<_, bool>("federated"),
            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at"),
        }));
    }

    Ok(users)
}

pub async fn get_user_statistics(db: &Database, realm_id: Uuid) -> Result<serde_json::Value> {
    let query = r#"
        SELECT
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE enabled = true) as enabled_users,
            COUNT(*) FILTER (WHERE enabled = false) as disabled_users,
            COUNT(*) FILTER (WHERE email_verified = true) as verified_emails,
            COUNT(*) FILTER (WHERE email_verified = false) as unverified_emails,
            COUNT(*) FILTER (WHERE federated = true) as federated_users,
            COUNT(*) FILTER (WHERE last_login_at IS NOT NULL) as users_with_login,
            COUNT(*) FILTER (WHERE last_login_at > NOW() - INTERVAL '30 days') as active_last_30_days,
            COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') as new_users_last_7_days
        FROM users
        WHERE realm_id = $1 AND deleted_at IS NULL
    "#;

    let row: tokio_postgres::Row = db.query_one(query, &[&realm_id]).await?;

    Ok(serde_json::json!({
        "total_users": row.get::<_, i64>("total_users"),
        "enabled_users": row.get::<_, i64>("enabled_users"),
        "disabled_users": row.get::<_, i64>("disabled_users"),
        "verified_emails": row.get::<_, i64>("verified_emails"),
        "unverified_emails": row.get::<_, i64>("unverified_emails"),
        "federated_users": row.get::<_, i64>("federated_users"),
        "users_with_login": row.get::<_, i64>("users_with_login"),
        "active_last_30_days": row.get::<_, i64>("active_last_30_days"),
        "new_users_last_7_days": row.get::<_, i64>("new_users_last_7_days"),
    }))
}

pub async fn import_users(db: &Database, users_data: Vec<serde_json::Value>) -> Result<usize> {
    if users_data.is_empty() {
        return Ok(0);
    }

    let mut client = db.get_connection().await?;
    let transaction = client.transaction().await?;

    let query = r#"
        INSERT INTO users (
            username, email, first_name, last_name,
            phone_number, phone_verified, email_verified,
            enabled, realm_id, organization_id, attributes,
            federated
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (username, realm_id) DO UPDATE
        SET email = EXCLUDED.email,
            first_name = EXCLUDED.first_name,
            last_name = EXCLUDED.last_name,
            updated_at = NOW()
    "#;

    let mut imported_count = 0;
    for user_data in users_data {
        let affected = transaction
            .execute(
                query,
                &[
                    &user_data["username"].as_str().unwrap_or(""),
                    &user_data["email"].as_str().unwrap_or(""),
                    &user_data.get("first_name").and_then(|v| v.as_str()),
                    &user_data.get("last_name").and_then(|v| v.as_str()),
                    &user_data.get("phone_number").and_then(|v| v.as_str()),
                    &user_data["phone_verified"].as_bool().unwrap_or(false),
                    &user_data["email_verified"].as_bool().unwrap_or(false),
                    &user_data["enabled"].as_bool().unwrap_or(true),
                    &user_data["realm_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(Uuid::nil()),
                    &user_data
                        .get("organization_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok()),
                    &user_data.get("attributes").and_then(|v| v.as_str()),
                    &user_data["federated"].as_bool().unwrap_or(false),
                ],
            )
            .await?;
        imported_count += affected as usize;
    }

    transaction.commit().await?;
    Ok(imported_count)
}
