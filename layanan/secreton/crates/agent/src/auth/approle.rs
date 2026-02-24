//! AppRole authentication

use anyhow::Result;
use std::collections::HashMap;

pub async fn authenticate(
    client: &reqwest::Client,
    server_url: &str,
    config: &HashMap<String, String>,
) -> Result<String> {
    let role_id = config
        .get("role_id")
        .ok_or_else(|| anyhow::anyhow!("Missing role_id"))?;
    let secret_id = config
        .get("secret_id")
        .ok_or_else(|| anyhow::anyhow!("Missing secret_id"))?;

    let url = format!("{}/v1/auth/approle/login", server_url);

    let body = serde_json::json!({
        "role_id": role_id,
        "secret_id": secret_id
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
