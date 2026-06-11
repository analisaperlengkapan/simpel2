use secreton_api::config::ApiConfig;
use secreton_api::services::ServiceContainer;
use std::sync::Arc;

#[tokio::test]
#[ignore = "Requires database connection"]
async fn test_auth_service_sessions() {
    let config = ApiConfig::default();
    let services = Arc::new(ServiceContainer::new(&config).await.expect("services"));

    // Create user
    let user = services
        .auth
        .create_user(secreton_api::services::auth::NewUser {
            username: "sessionuser",
            email: "session@example.com",
            password: "password",
            full_name: None,
            roles: vec!["user".to_string()],
            metadata: None,
            is_active: true,
        })
        .await
        .expect("create user");

    // Authenticate
    let auth_token = services
        .auth
        .authenticate("sessionuser", "password", None, "127.0.0.1", "test-agent")
        .await
        .expect("authenticate");

    let token = auth_token.access_token;

    // Validate token (checks session)
    let validated_user = services
        .auth
        .validate_token(&token)
        .await
        .expect("validate");
    assert_eq!(validated_user.username, "sessionuser");

    // List sessions
    let sessions = services
        .auth
        .list_user_sessions(&user.id.to_string())
        .await
        .expect("list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].user_id, "sessionuser"); // username stored in session struct

    // Get token ID
    let jti = services.auth.get_token_id(&token).expect("get jti");
    assert_eq!(sessions[0].id, jti);

    // Revoke session
    services.auth.revoke_session(&jti).await.expect("revoke");

    // Validate token again (should fail)
    let res = services.auth.validate_token(&token).await;
    assert!(res.is_err());

    // Revoke via token (logout)
    // Authenticate again
    let auth_token2 = services
        .auth
        .authenticate("sessionuser", "password", None, "127.0.0.1", "test-agent")
        .await
        .expect("authenticate 2");

    services
        .auth
        .revoke_token(&auth_token2.access_token)
        .await
        .expect("revoke token");

    assert!(
        services
            .auth
            .validate_token(&auth_token2.access_token)
            .await
            .is_err()
    );
}
