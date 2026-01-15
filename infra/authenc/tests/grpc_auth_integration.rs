use authenc::app::AppState;
use authenc::config::AppConfig;
use authenc::grpc::authenc_service::AuthencGrpcService;
use authenc::grpc::proto::{AuthenticateRequest, AuthencService};
use authenc::models::user::{AccessLevel, SecretonAccessPolicy, SecurityContext, User};
use chrono::Utc;
use std::sync::Arc;
use tonic::Request;
use uuid::Uuid;

mod test_utils {
    use super::*;
    use authenc::error::{AuthencError, Result};

    pub async fn create_integration_db_pool() -> deadpool_postgres::Pool {
        let mut cfg = deadpool_postgres::Config::new();
        cfg.host = Some(std::env::var("TEST_DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
        cfg.port = Some(std::env::var("TEST_DB_PORT").unwrap_or_else(|_| "5432".to_string()).parse().unwrap_or(5432));
        cfg.dbname = Some(std::env::var("TEST_DB_NAME").unwrap_or_else(|_| "authenc_integration_test".to_string()));
        cfg.user = Some(std::env::var("TEST_DB_USER").unwrap_or_else(|_| "postgres".to_string()));
        cfg.password = Some(std::env::var("TEST_DB_PASSWORD").unwrap_or_else(|_| "password".to_string()));

        cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)
            .expect("Failed to create test database pool")
    }

    pub fn create_integration_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: format!("test_user_{}", Uuid::new_v4().to_string()[..8].to_string()),
            email: "integration.test@kejaksaan.go.id".to_string(),
            email_verified: true,
            first_name: Some("Integration".to_string()),
            last_name: Some("Test".to_string()),
            nip: Some(format!("19{:08}", rand::random::<u32>() % 100000000)),
            nama: Some("Integration Test User".to_string()),
            jabatan: Some("Jaksa Muda".to_string()),
            satker_code: "001".to_string(),
            phone_number: None,
            phone_verified: false,
            password_hash: Some("$argon2id$v=19$m=4096,t=3,p=1$c2FsdHNhbHQ$qU8g/Rk/3k...".to_string()), // Dummy hash
            totp_secret: Some("JBSWY3DPEHPK3PXP".to_string()), // valid base32 secret
            totp_backup_codes: None,
            mfa_enabled: true, // ENABLED for this test
            mfa_setup_at: Some(Utc::now()),
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
            realm_id: Some(Uuid::new_v4()),
            organization_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_data: None,
            secreton_access_policy: SecretonAccessPolicy {
                allowed_satker_secrets: vec!["001".to_string()],
                access_level: AccessLevel::ReadWrite,
                time_restrictions: None,
                audit_required: true,
                rate_limit: Some(100),
                allowed_paths: None,
                denied_paths: None,
            },
            security_context: SecurityContext {
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("integration-test-agent".to_string()),
                session_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
                risk_score: Some(0.0),
                metadata: None,
            },
            attributes: None,
            enabled: true,
            federated: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            login_count: 0,
        }
    }

    pub async fn setup_test_user_in_db(pool: &deadpool_postgres::Pool, user: &User) -> Result<()> {
        let client = pool.get().await.map_err(|e| AuthencError::database(e.to_string()))?;

        // Ensure table exists (simplified)
        client.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username VARCHAR NOT NULL UNIQUE,
                email VARCHAR NOT NULL,
                password_hash VARCHAR,
                nip VARCHAR,
                nama VARCHAR,
                jabatan VARCHAR,
                satker_code VARCHAR,
                mfa_enabled BOOLEAN DEFAULT FALSE,
                mfa_setup_at TIMESTAMP WITH TIME ZONE,
                mfa_last_used TIMESTAMP WITH TIME ZONE,
                enabled BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_login_at TIMESTAMP WITH TIME ZONE
            )", &[]
        ).await.map_err(|e| AuthencError::database(e.to_string()))?;

        client.execute(
            "INSERT INTO users (id, username, email, password_hash, nip, nama, jabatan, satker_code, mfa_enabled, mfa_setup_at, mfa_last_used, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            &[
                &user.id, &user.username, &user.email, &user.password_hash,
                &user.nip, &user.nama, &user.jabatan, &user.satker_code,
                &user.mfa_enabled, &user.mfa_setup_at, &user.mfa_last_used, &user.enabled
            ],
        ).await.map_err(|e| AuthencError::database(e.to_string()))?;
        Ok(())
    }

    pub async fn cleanup_test_user(pool: &deadpool_postgres::Pool, user_id: Uuid) -> Result<()> {
        let client = pool.get().await.map_err(|e| AuthencError::database(e.to_string()))?;
        client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.map_err(|e| AuthencError::database(e.to_string()))?;
        Ok(())
    }
}

#[tokio::test]
async fn test_grpc_mfa_required_flow() {
    // 1. Setup
    // Use a minimal config that disables external connections where possible
    unsafe {
        std::env::set_var("TEST_DB_HOST", "localhost");
        std::env::set_var("TEST_DB_NAME", "authenc_integration_test");
        std::env::set_var("TEST_DB_USER", "postgres");
        std::env::set_var("TEST_DB_PASSWORD", "password"); // Adjust as needed for specific env
    }

    let db_pool = test_utils::create_integration_db_pool().await;
    let mut config = AppConfig::default();
    // Configure minimums to allow start
    config.database.host = "localhost".to_string();
    config.database.port = 5432;
    config.database.name = "authenc_integration_test".to_string();
    config.database.username = "postgres".to_string();
    config.database.password = "password".to_string();

    // Disable redis, kafka, etc in default config usually unless explicit?
    // Using default AppConfig usually has defaults. Assuming defaults are sane or optionals are None.

    // Initialize AppState - this might fail if it tries to connect to things.
    // Instead of full AppState::new, we might need a partial construct or mock if possible.
    // But since AppState::new is the only easy way, let's try it.
    // If it fails, we will know.
    let state = match AppState::new(config).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            println!("Failed to init AppState: {:?}", e);
            // If full init fails, we can't easily test without a complex mock.
            // But we can check if it failed on DB or something we can fix.
            panic!("AppState init failed");
        }
    };

    let user = test_utils::create_integration_test_user();

    // Ensure password matches what verify_password expects.
    // In test_utils, we put a dummy hash. We need a real hash for "password123".
    // Or we mock verify_password? No, it's a util call.
    // Let's generate a real hash for "password".
    // Since we can't easily call argon2 here without importing dependencies,
    // we'll assume the dummy hash IS valid or we update it.
    // Actually, create_integration_test_user puts a dummy "$argon2id$..." string.
    // It says "Dummy hash", likely NOT valid for "password".
    // We should use a known valid hash for "password" or generate one.
    // Valid Argon2 hash for "password":
    // $argon2id$v=19$m=4096,t=3,p=1$c2FsdHNhbHQ$qU8g/Rk/3k... is likely truncated/fake.
    // Let's rely on the fact that if password verification fails, we get "Invalid Credentials".
    // We WANT to pass password check to get to MFA.

    // Hack: Assuming we can't easily generate hash, we will SKIP password check by...
    // We can't.
    // We must provide a valid hash.
    // Let's try to perform the test assuming we can verify the hash.
    // I will use a known hash for 'password':
    // $argon2id$v=19$m=16,t=2,p=1$S2F0emVuYmFy$2Z/w/LzZ/w/LzZ/w/LzZ/w
    // (This is just an example, maybe format specific).
    // Better: create user using `state.user_store.add_user` with plain password, it handles hashing!

    // 2. Create User via Store (handles hashing)
    // We need to use `user_store.add_user`.
    // But `add_user` takes `CreateUserRequest`.
    use authenc::models::user::CreateUserRequest;

    let create_req = CreateUserRequest {
        username: user.username.clone(),
        email: user.email.clone(),
        password: Some("password".to_string()),
        satker_code: "001".to_string(),
        nama: "Test User".to_string(),
        ..Default::default()
    };

    let created_user = state.user_store.add_user(create_req).await.expect("Failed to create user");

    // Enable MFA manually in DB because add_user might default to false
    let client = db_pool.get().await.unwrap();
    client.execute("UPDATE users SET mfa_enabled = TRUE WHERE id = $1", &[&created_user.id]).await.unwrap();

    // 3. Test MFA Required Flow
    let service = AuthencGrpcService::new(state.clone());

    let req = Request::new(AuthenticateRequest {
        username: created_user.username.clone(),
        password: "password".to_string(), // correct password
        mfa_code: None, // Missing MFA code
        device_id: None,
        metadata: Default::default(),
        captcha_token: None,
    });

    let result = service.authenticate(req).await;

    // Cleanup
    test_utils::cleanup_test_user(&db_pool, created_user.id).await.ok();

    match result {
        Ok(resp) => {
            let inner = resp.into_inner();
            println!("Got response: {:?}", inner);

            assert!(inner.mfa_required, "mfa_required should be true");
            assert!(!inner.temp_token.is_empty(), "temp_token should not be empty");
            // Access token might be empty or present but effectively useless for full access?
            // The logic sets access_token to "" in this case.
            assert!(inner.access_token.is_empty(), "access_token should be empty when MFA required");
        },
        Err(e) => {
            panic!("Authenticate failed with error: {:?}", e);
        }
    }
}
