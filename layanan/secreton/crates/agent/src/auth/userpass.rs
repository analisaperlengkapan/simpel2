//! Userpass authentication

use anyhow::Result;
use std::collections::HashMap;

pub async fn authenticate(
    client: &reqwest::Client,
    server_url: &str,
    config: &HashMap<String, String>,
) -> Result<String> {
    let username = config
        .get("username")
        .ok_or_else(|| anyhow::anyhow!("Missing username"))?;
    let password = config
        .get("password")
        .ok_or_else(|| anyhow::anyhow!("Missing password"))?;

    let url = format!("{}/v1/auth/userpass/login/{}", server_url, username);

    let body = serde_json::json!({
        "password": password
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
