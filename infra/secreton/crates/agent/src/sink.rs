//! Token sink

use crate::config::SinkConfig;
use anyhow::Result;

/// Write token to configured sinks
pub async fn write_token(token: &str, config: &SinkConfig) -> Result<()> {
    for sink_type in &config.types {
        match sink_type.as_str() {
            "file" => write_to_file(token, config).await?,
            "env" => write_to_env(token, config)?,
            _ => tracing::warn!("Unknown sink type: {}", sink_type),
        }
    }
    Ok(())
}

/// Write token to file
async fn write_to_file(token: &str, config: &SinkConfig) -> Result<()> {
    if let Some(ref path) = config.file_path {
        tokio::fs::write(path, token).await?;

        // Set permissions if specified
        if let Some(ref perms) = config.file_permissions {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = u32::from_str_radix(perms, 8)?;
                let permissions = std::fs::Permissions::from_mode(mode);
                std::fs::set_permissions(path, permissions)?;
            }
        }

        tracing::info!("Token written to file: {}", path);
    }
    Ok(())
}

/// Write token to environment variable
fn write_to_env(token: &str, config: &SinkConfig) -> Result<()> {
    if let Some(ref var_name) = config.env_var {
        unsafe {
            std::env::set_var(var_name, token);
        }
        tracing::info!("Token set in environment: {}", var_name);
    }
    Ok(())
}
