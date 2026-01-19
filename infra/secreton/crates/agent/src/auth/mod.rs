//! Authentication methods

pub mod approle;
/// Mewakili pub `kubernetes`.
pub mod kubernetes;
/// Mewakili pub `userpass`.
pub mod userpass;

use anyhow::Result;
use std::collections::HashMap;

/// Authenticate with vault
pub async fn authenticate(
    client: &reqwest::Client,
    server_url: &str,
    method: &str,
    config: &HashMap<String, String>,
) -> Result<String> {
    match method {
        "userpass" => userpass::authenticate(client, server_url, config).await,
        "approle" => approle::authenticate(client, server_url, config).await,
        "kubernetes" | "k8s" => kubernetes::authenticate(client, server_url, config).await,
        _ => Err(anyhow::anyhow!("Unsupported auth method: {}", method)),
    }
}

/// Renew token
pub async fn renew_token(
    client: &reqwest::Client,
    server_url: &str,
    token: &str,
) -> Result<String> {
    let url = format!("{}/v1/auth/token/renew-self", server_url);

    let response = client
        .post(&url)
        .header("X-Vault-Token", token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Token renewal failed: {}",
            response.status()
        ));
    }

    let body: serde_json::Value = response.json().await?;
    let new_token = body["auth"]["client_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No token in response"))?;

    Ok(new_token.to_string())
}
