use authenc::config::DatabaseConfig;
use authenc::database::Database;
use authenc::database::operations as db_ops;
use authenc::models::user::CreateUserRequest;
use uuid::Uuid;

#[tokio::test]


#[ignore = "Requires database/infrastructure"]
async fn test_bulk_create_users() {
    // Setup database connection
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "authenc".to_string(),
        password: "authenc".to_string(),
        database: "authenc_test".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };
    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to database");

    // Create test realm
    let realm_id = Uuid::new_v4();

    // Prepare bulk users
    let users = vec![
        CreateUserRequest {
            username: "testuser1".to_string(),
            email: "test1@example.com".to_string(),
            satker_code: "SATKER_TEST".to_string(),
            password: Some("password123".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User1".to_string()),
            nip: Some("198001012000011001".to_string()),
            nama: Some("Test User".to_string()),
            jabatan: Some("Jaksa Muda".to_string()),
            phone_number: None,
            realm_id: Some(realm_id),
            organization_id: None,
            roles: None,
            attributes: None,
        },
        CreateUserRequest {
            username: "testuser2".to_string(),
            email: "test2@example.com".to_string(),
            satker_code: "SATKER_TEST".to_string(),
            password: Some("password123".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User2".to_string()),
            nip: Some("198001012000011001".to_string()),
            nama: Some("Test User".to_string()),
            jabatan: Some("Jaksa Muda".to_string()),
            phone_number: None,
            realm_id: Some(realm_id),
            organization_id: None,
            roles: None,
            attributes: None,
        },
    ];

    // Test bulk create
    let result = db_ops::users::bulk_create_users(&db, users).await;
    assert!(result.is_ok(), "Bulk create should succeed");

    let created = result.unwrap();
    assert_eq!(created.len(), 2, "Should create 2 users");

    println!("✓ Bulk create test passed: {} users created", created.len());
}

#[tokio::test]


#[ignore = "Requires database/infrastructure"]
async fn test_advanced_user_query() {
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "authenc".to_string(),
        password: "authenc".to_string(),
        database: "authenc_test".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };
    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to database");

    let realm_id = Uuid::new_v4();

    // Test query with filters
    let result = db_ops::users::query_users_advanced(
        &db,
        Some(realm_id),
        Some("test"),
        None,
        Some(true), // enabled only
        None,
        None,
        Some("username"),
        Some("asc"),
        Some(0),
        Some(10),
    )
    .await;

    assert!(result.is_ok(), "Advanced query should succeed");

    let (users, total) = result.unwrap();
    println!(
        "✓ Advanced query test passed: {} users found, total: {}",
        users.len(),
        total
    );
}

#[tokio::test]


#[ignore = "Requires database/infrastructure"]
async fn test_session_management() {
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "authenc".to_string(),
        password: "authenc".to_string(),
        database: "authenc_test".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };
    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to database");

    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    // Test create session
    let result = db_ops::sessions::create_user_session(
        &db,
        db_ops::sessions::NewUserSession {
            user_id,
            realm_id,
            client_id: None,
            token: "test_token_123",
            refresh_token: Some("refresh_token_456"),
            expires_in: 3600, // 1 hour
            ip_address: Some("127.0.0.1"),
            user_agent: Some("Mozilla/5.0"),
            authentication_method: Some("password"),
            protocol: Some("openid-connect"),
        },
    )
    .await;

    assert!(result.is_ok(), "Create session should succeed");

    let session = result.unwrap();
    println!("✓ Session create test passed: {:?}", session);

    // Test get session by token
    let get_result = db_ops::sessions::get_session_by_token(&db, "test_token_123").await;
    assert!(get_result.is_ok(), "Get session by token should succeed");

    println!("✓ Session retrieval test passed");
}

#[tokio::test]


#[ignore = "Requires database/infrastructure"]
async fn test_offline_tokens() {
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "authenc".to_string(),
        password: "authenc".to_string(),
        database: "authenc_test".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };
    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to database");

    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let client_id = Uuid::new_v4();

    // Test create offline token
    let result = db_ops::sessions::create_offline_token(
        &db,
        db_ops::sessions::NewOfflineToken {
            user_id,
            realm_id,
            client_id,
            token: "offline_token_789",
            scope: Some("openid email profile"),
            expires_at: None, // Never expires
            data: Some(serde_json::json!({"device": "mobile"})),
        },
    )
    .await;

    assert!(result.is_ok(), "Create offline token should succeed");

    let token = result.unwrap();
    println!("✓ Offline token create test passed: {:?}", token);

    // Test get offline token
    let get_result = db_ops::sessions::get_offline_token(&db, "offline_token_789").await;
    assert!(get_result.is_ok(), "Get offline token should succeed");

    println!("✓ Offline token retrieval test passed");
}

#[tokio::test]


#[ignore = "Requires database/infrastructure"]
async fn test_refresh_token_rotation() {
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "authenc".to_string(),
        password: "authenc".to_string(),
        database: "authenc_test".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };
    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to database");

    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    // Create session first
    let session_result = db_ops::sessions::create_user_session(
        &db,
        db_ops::sessions::NewUserSession {
            user_id,
            realm_id,
            client_id: None,
            token: "access_token",
            refresh_token: Some("old_refresh_token"),
            expires_in: 3600,
            ip_address: Some("127.0.0.1"),
            user_agent: Some("Mozilla/5.0"),
            authentication_method: Some("password"),
            protocol: Some("openid-connect"),
        },
    )
    .await;

    assert!(session_result.is_ok(), "Create session should succeed");

    let session = session_result.unwrap();
    let session_id = session["id"].as_str().unwrap();
    let session_uuid = Uuid::parse_str(session_id).unwrap();

    // Test token rotation
    let rotate_result = db_ops::sessions::rotate_refresh_token(
        &db,
        session_uuid,
        "old_refresh_token",
        "new_refresh_token",
        Some("127.0.0.1"),
        Some("Mozilla/5.0"),
    )
    .await;

    assert!(rotate_result.is_ok(), "Token rotation should succeed");
    assert!(rotate_result.unwrap(), "Token rotation should return true");

    println!("✓ Refresh token rotation test passed");
}
