//! Kubernetes authentication

use anyhow::Result;
use std::collections::HashMap;

pub async fn authenticate(
    client: &reqwest::Client,
    server_url: &str,
    config: &HashMap<String, String>,
) -> Result<String> {
    let role = config
        .get("role")
        .ok_or_else(|| anyhow::anyhow!("Missing role"))?;

    let jwt_path = config
        .get("jwt_path")
        .unwrap_or(&"/var/run/secrets/kubernetes.io/serviceaccount/token".to_string());

    let jwt = tokio::fs::read_to_string(jwt_path).await?;

    let url = format!("{}/v1/auth/kubernetes/login", server_url);

    let body = serde_json::json!({
        "role": role,
        "jwt": jwt.trim()
    });

    let response = client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Authentication failed: {}",
            response.status()
        ));
    }

    let body: serde_json::Value = response.json().await?;
    let token = body["auth"]["client_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No token in response"))?;

    Ok(token.to_string())
}
