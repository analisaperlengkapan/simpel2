use crate::{
    config::SimplConfig,
    utils::output::SimplOutput,
};
use anyhow::Result;

pub async fn handle_config_command(
    command: &crate::commands::ConfigCommands,
    config: &mut SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::ConfigCommands::Show => config_show(config, output).await,
        crate::commands::ConfigCommands::Set { key, value } => {
            config_set(key, value, config, output).await
        }
        crate::commands::ConfigCommands::Get { key } => config_get(key, config, output).await,
        crate::commands::ConfigCommands::Init => config_init(config, output).await,
        crate::commands::ConfigCommands::Validate => config_validate(config, output).await,
    }
}

async fn config_show(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Current Configuration:")?;
    
    let config_toml = toml::to_string_pretty(config)?;
    println!("{}", config_toml);
    
    Ok(())
}

async fn config_set(
    key: &str,
    value: &str,
    config: &mut SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Setting {} = {}", key, value))?;
    
    // Parse the key path (e.g., "workspace.name" or "build.release")
    let parts: Vec<&str> = key.split('.').collect();
    
    match parts.as_slice() {
        ["workspace", "name"] => config.workspace.name = value.to_string(),
        ["workspace", "version"] => config.workspace.version = value.to_string(),
        ["build", "release"] => config.build.release = value.parse().unwrap_or(false),
        ["deploy", "environment"] => config.deploy.environment = value.to_string(),
        _ => {
            output.error(&format!("Unknown configuration key: {}", key))?;
            return Ok(());
        }
    }
    
    // Save the configuration
    config.save().await?;
    output.success(&format!("Configuration updated: {} = {}", key, value))?;
    
    Ok(())
}

async fn config_get(key: &str, config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    
    let value = match parts.as_slice() {
        ["workspace", "name"] => config.workspace.name.clone(),
        ["workspace", "version"] => config.workspace.version.clone(),
        ["build", "release"] => config.build.release.to_string(),
        ["deploy", "environment"] => config.deploy.environment.clone(),
        _ => {
            output.error(&format!("Unknown configuration key: {}", key))?;
            return Ok(());
        }
    };
    
    println!("{}", value);
    Ok(())
}

async fn config_init(config: &mut SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Initializing configuration file...")?;
    
    let config_path = "simpel.toml";
    
    if tokio::fs::metadata(config_path).await.is_ok() {
        let confirmed = output.confirm("Configuration file already exists. Overwrite?")?;
        if !confirmed {
            output.info("Configuration initialization cancelled.")?;
            return Ok(());
        }
    }
    
    // Create default configuration
    *config = SimplConfig::default();
    config.save().await?;
    
    output.success(&format!("Configuration file created: {}", config_path))?;
    Ok(())
}

async fn config_validate(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Validating configuration...")?;
    
    let mut errors = Vec::new();
    
    // Validate workspace configuration
    if config.workspace.name.is_empty() {
        errors.push("workspace.name cannot be empty");
    }
    
    // Validate services exist
    for (name, service_config) in &config.workspace.services {
        let service_path = &service_config.path;
        if tokio::fs::metadata(service_path).await.is_err() {
            errors.push(&format!("Service path does not exist: {}", service_path));
        }
    }
    
    // Validate build configuration
    if config.build.target.is_empty() {
        errors.push("build.target cannot be empty");
    }
    
    if errors.is_empty() {
        output.success("Configuration is valid!")?;
    } else {
        output.error("Configuration validation failed:")?;
        for error in errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}
