#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;
    use std::time::Duration;

    const PORTAL_URL: &str = "http://127.0.0.1:8081";
    const AUTHENC_URL: &str = "http://127.0.0.1:8088";

    #[tokio::test]
    async fn test_portal_authenc_integration_flow() {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        // 1. Verify Portal Health
        let resp = client.get(format!("{}/api/v1/health", PORTAL_URL)).send().await;
        assert!(resp.is_ok(), "Portal should be reachable");
        assert!(resp.unwrap().status().is_success(), "Portal health check failed");

        // 2. Attempt Login (expect failure or specific response, but verifying connectivity)
        // Using a non-existent user to verify the interaction chain:
        // Portal -> Authenc gRPC -> DB -> "User not found"
        // This confirms the gRPC link is active.
        let login_body = json!({
            "username": "integration_test_non_existent",
            "password": "wrongpassword"
        });

        let resp = client.post(format!("{}/api/v1/auth/login", PORTAL_URL))
            .json(&login_body)
            .send()
            .await
            .expect("Failed to send login request");

        // We expect 401 Unauthorized or 400 Bad Request if user doesn't exist
        // But crucially, if the link to Authenc was broken, we'd get 500 Internal Server Error
        // or a specific Gateway Error.

        let status = resp.status();
        println!("Login Response Status: {}", status);

        // Read body to see if it mentions Authenc error or Invalid Credentials
        let body = resp.text().await.unwrap_or_default();
        println!("Login Response Body: {}", body);

        // Assert that the response is 401 Unauthorized for invalid credentials
        assert!(
            status.as_u16() == 401 || status.as_u16() == 400 || body.contains("Authentication error"),
            "Expected 401/400 or Authentication error, got {}. Body: {}", status, body
        );

        // If we get "Authenc gRPC error: transport error", then integration is broken.
        assert!(!body.to_lowercase().contains("transport error"), "gRPC Transport error detected!");
        assert!(!body.to_lowercase().contains("connection refused"), "Connection refused detected!");
    }
}
