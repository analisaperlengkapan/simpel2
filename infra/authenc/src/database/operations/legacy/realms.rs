/// Database operations for realms
use crate::{

    database::Database,
    error::Result,
    models::{Realm, realm::CreateRealmRequest, realm::UpdateRealmRequest},
};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_realm(db: &Database, request: &CreateRealmRequest) -> Result<Realm> {
    let realm_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO realms (
            id, name, display_name, description, enabled, ssl_required,
            registration_allowed, registration_email_as_username, remember_me,
            verify_email, login_with_email_allowed, duplicate_emails_allowed,
            reset_password_allowed, edit_username_allowed, brute_force_protected,
            max_failure_wait_seconds, minimum_quick_login_wait_seconds,
            wait_increment_seconds, quick_login_check_milli_seconds,
            max_delta_time_seconds, failure_factor, default_signature_algorithm,
            revoke_refresh_token, refresh_token_max_reuse, access_token_lifespan,
            access_token_lifespan_for_implicit_flow, sso_session_idle_timeout,
            sso_session_max_lifespan, sso_session_idle_timeout_remember_me,
            sso_session_max_lifespan_remember_me, offline_session_idle_timeout,
            offline_session_max_lifespan, client_session_idle_timeout,
            client_session_max_lifespan, access_code_lifespan,
            access_code_lifespan_user_action, access_code_lifespan_login,
            action_token_generated_by_admin_lifespan,
            action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
            oauth2_device_polling_interval, attributes, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
        RETURNING *
    "#;

    let row = db
        .query_one(
            query,
            &[
                &realm_id,
                &request.name,
                &request.display_name,
                &request.description,
                &request.enabled.unwrap_or(true),
                &"external", // ssl_required
                &false,      // registration_allowed
                &false,      // registration_email_as_username
                &true,       // remember_me
                &false,      // verify_email
                &true,       // login_with_email_allowed
                &false,      // duplicate_emails_allowed
                &true,       // reset_password_allowed
                &false,      // edit_username_allowed
                &true,       // brute_force_protected
                &900i32,     // max_failure_wait_seconds
                &60i32,      // minimum_quick_login_wait_seconds
                &60i32,      // wait_increment_seconds
                &1000i64,    // quick_login_check_milli_seconds
                &43200i32,   // max_delta_time_seconds (12 hours)
                &30i32,      // failure_factor
                &"RS256",    // default_signature_algorithm
                &false,      // revoke_refresh_token
                &0i32,       // refresh_token_max_reuse
                &300i32,     // access_token_lifespan (5 minutes)
                &900i32,     // access_token_lifespan_for_implicit_flow (15 minutes)
                &1800i32,    // sso_session_idle_timeout (30 minutes)
                &36000i32,   // sso_session_max_lifespan (10 hours)
                &0i32,       // sso_session_idle_timeout_remember_me
                &0i32,       // sso_session_max_lifespan_remember_me
                &2592000i32, // offline_session_idle_timeout (30 days)
                &5184000i32, // offline_session_max_lifespan (60 days)
                &0i32,       // client_session_idle_timeout
                &0i32,       // client_session_max_lifespan
                &60i32,      // access_code_lifespan (1 minute)
                &300i32,     // access_code_lifespan_user_action (5 minutes)
                &1800i32,    // access_code_lifespan_login (30 minutes)
                &43200i32,   // action_token_generated_by_admin_lifespan (12 hours)
                &300i32,     // action_token_generated_by_user_lifespan (5 minutes)
                &600i32,     // oauth2_device_code_lifespan (10 minutes)
                &5i32,       // oauth2_device_polling_interval
                &request
                    .attributes
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_default()),
                &now,
                &now,
            ],
        )
        .await?;

    Ok(row_to_realm(row))
}

pub async fn get_realm_by_id(db: &Database, realm_id: Uuid) -> Result<Option<Realm>> {
    let query = r#"
        SELECT * FROM realms
        WHERE id = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_one(query, &[&realm_id]).await?;
    Ok(Some(row_to_realm(row)))
}

pub async fn get_realm_by_name(db: &Database, name: &str) -> Result<Option<Realm>> {
    let query = r#"
        SELECT * FROM realms
        WHERE name = $1 AND deleted_at IS NULL
    "#;

    let row = db.query_one(query, &[&name]).await?;
    Ok(Some(row_to_realm(row)))
}

pub async fn update_realm(
    db: &Database,
    realm_id: Uuid,
    request: &UpdateRealmRequest,
) -> Result<Realm> {
    let now = Utc::now();

    let query = r#"
        UPDATE realms SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            enabled = COALESCE($4, enabled),
            ssl_required = COALESCE($5, ssl_required),
            registration_allowed = COALESCE($6, registration_allowed),
            verify_email = COALESCE($7, verify_email),
            reset_password_allowed = COALESCE($8, reset_password_allowed),
            brute_force_protected = COALESCE($9, brute_force_protected),
            attributes = COALESCE($10, attributes),
            updated_at = $11
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING *
    "#;

    let row = db
        .query_one(
            query,
            &[
                &realm_id,
                &request.display_name,
                &request.description,
                &request.enabled,
                &request.ssl_required,
                &request.registration_allowed,
                &request.verify_email,
                &request.reset_password_allowed,
                &request.brute_force_protected,
                &request
                    .attributes
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_default()),
                &now,
            ],
        )
        .await?;

    Ok(row_to_realm(row))
}

pub async fn delete_realm(db: &Database, realm_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let query = "UPDATE realms SET deleted_at = $2, updated_at = $2 WHERE id = $1";
    db.execute(query, &[&realm_id, &now]).await?;
    Ok(())
}

pub async fn list_realms(db: &Database) -> Result<Vec<Realm>> {
    let query = r#"
        SELECT * FROM realms
        WHERE deleted_at IS NULL
        ORDER BY name
    "#;

    let rows = db.query(query, &[]).await?;
    Ok(rows.into_iter().map(row_to_realm).collect())
}

fn row_to_realm(row: tokio_postgres::Row) -> Realm {
    Realm {
        id: row.get(0),
        name: row.get(1),
        display_name: row.get(2),
        description: row.get(3),
        enabled: row.get(4),
        ssl_required: row.get(5),
        registration_allowed: row.get(6),
        registration_email_as_username: row.get(7),
        remember_me: row.get(8),
        verify_email: row.get(9),
        login_with_email_allowed: row.get(10),
        duplicate_emails_allowed: row.get(11),
        reset_password_allowed: row.get(12),
        edit_username_allowed: row.get(13),
        brute_force_protected: row.get(14),
        max_failure_wait_seconds: row.get(15),
        minimum_quick_login_wait_seconds: row.get(16),
        wait_increment_seconds: row.get(17),
        quick_login_check_milli_seconds: row.get(18),
        max_delta_time_seconds: row.get(19),
        failure_factor: row.get(20),
        default_signature_algorithm: row.get(21),
        revoke_refresh_token: row.get(22),
        refresh_token_max_reuse: row.get(23),
        access_token_lifespan: row.get(24),
        access_token_lifespan_for_implicit_flow: row.get(25),
        sso_session_idle_timeout: row.get(26),
        sso_session_max_lifespan: row.get(27),
        sso_session_idle_timeout_remember_me: row.get(28),
        sso_session_max_lifespan_remember_me: row.get(29),
        offline_session_idle_timeout: row.get(30),
        offline_session_max_lifespan: row.get(31),
        client_session_idle_timeout: row.get(32),
        client_session_max_lifespan: row.get(33),
        access_code_lifespan: row.get(34),
        access_code_lifespan_user_action: row.get(35),
        access_code_lifespan_login: row.get(36),
        action_token_generated_by_admin_lifespan: row.get(37),
        action_token_generated_by_user_lifespan: row.get(38),
        oauth2_device_code_lifespan: row.get(39),
        oauth2_device_polling_interval: row.get(40),
        attributes: row
            .get::<_, Option<String>>(41)
            .and_then(|s: String| serde_json::from_str(&s).ok()),
        created_at: row.get(42),
        updated_at: row.get(43),
        deleted_at: row.get(44),
    }
}
