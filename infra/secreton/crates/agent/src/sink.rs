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
///
/// Note: Setting environment variables at runtime is not recommended in production
/// as it can lead to race conditions in multi-threaded applications.
/// Consider using file-based configuration or process environment setup instead.
fn write_to_env(token: &str, config: &SinkConfig) -> Result<()> {
    if let Some(ref var_name) = config.env_var {
        // SAFETY: This is safe because:
        // 1. We're in a single-threaded initialization context
        // 2. No other threads are reading this environment variable yet
        // 3. This is only used by the agent during startup
        //
        // However, note that modifying environment variables at runtime
        // is generally discouraged in Rust. For production use, consider:
        // - Writing to a secure file that applications can read
        // - Using a configuration service
        // - Setting the variable before process startup
        unsafe {
            std::env::set_var(var_name, token);
        }
        tracing::warn!(
            var_name = %var_name,
            "Token set in environment variable - this is not recommended for production use"
        );
    }
    Ok(())
}
