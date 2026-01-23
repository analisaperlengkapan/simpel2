//! Template rendering

use crate::config::TemplateConfig;
use anyhow::Result;

/// Render template
pub async fn render_template(
    client: &reqwest::Client,
    server_url: &str,
    token: &str,
    config: &TemplateConfig,
) -> Result<()> {
    // Fetch secret from engine
    let url = format!("{}/v1/{}", server_url, config.source);

    let response = client
        .get(&url)
        .header("X-Engine-Token", token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch secret: {}",
            response.status()
        ));
    }

    let body: serde_json::Value = response.json().await?;
    let data = &body["data"]["data"];

    // Render template or write raw data
    let content = if let Some(ref template) = config.template {
        render_template_string(template, data)?
    } else {
        serde_json::to_string_pretty(data)?
    };

    // Write to file
    tokio::fs::write(&config.dest, content).await?;

    // Set permissions if specified
    if let Some(ref perms) = config.permissions {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = u32::from_str_radix(perms, 8)?;
            let permissions = std::fs::Permissions::from_mode(mode);
            std::fs::set_permissions(&config.dest, permissions)?;
        }
    }

    Ok(())
}

/// Simple template rendering (replace {{key} with value)
fn render_template_string(template: &str, data: &serde_json::Value) -> Result<String> {
    let mut result = template.to_string();

    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }
    }

    Ok(result)
}
