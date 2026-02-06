use authenc::config::DatabaseConfig;
use authenc::database::Database;
use authenc::models::user::CreateUserRequest;
use authenc::services::stores::user_store::{UserStore, UserStoreTrait};
use std::sync::Arc;

#[tokio::test]
#[ignore = "Requires PostgreSQL database to be running"]
async fn user_store_basic_flow() {
    // Create a test database configuration
    let database_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "test".to_string(),
        password: "test".to_string(),
        database: "test_db".to_string(),
        max_connections: 10,
        min_connections: 1,
        idle_timeout: 600,
        max_lifetime: 1800,
        connection_timeout: 30,
        audit_log_url: None,
    };

    let database_result = Database::new(&database_config).await;
    let database = match database_result {
        Ok(db) => Arc::new(db),
        Err(_) => {
            println!("Skipping test due to database connection issues");
            return;
        }
    };
    let store = UserStore::new(database);

    let request = CreateUserRequest {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        satker_code: "SATKER_TEST".to_string(),
        password: Some("password123".to_string()),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        nip: Some("198001012000011001".to_string()),
        nama: Some("Test User".to_string()),
        jabatan: Some("Jaksa Muda".to_string()),
        phone_number: None,
        realm_id: None,
        organization_id: None,
        roles: None,
        attributes: None,
    };

    let user = store.add_user(request).await.unwrap();
    let id = user.id;

    let all_users = store.get_all().await.unwrap();
    assert_eq!(all_users.len(), 1);
    assert!(store.get_user(id).await.unwrap().is_some());
    assert!(store.get_user_by_username("alice").await.unwrap().is_some());
    // TODO: Implement proper password hashing and verification
    // assert_eq!(store.verify_password("alice", "password").await.unwrap(), true);
}
