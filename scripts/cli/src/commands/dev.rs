use crate::{
    config::SimplConfig,
    utils::{output::SimplOutput, process::CargoHelper},
};
use anyhow::Result;
use std::collections::HashMap;

pub async fn handle_dev_command(
    command: &crate::commands::DevCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::DevCommands::Up { services, hot_reload } => {
            dev_up(services, *hot_reload, config, output).await
        }
        crate::commands::DevCommands::Down { services } => {
            dev_down(services, config, output).await
        }
        crate::commands::DevCommands::Restart { services } => {
            dev_restart(services, config, output).await
        }
        crate::commands::DevCommands::Logs { service, follow, lines } => {
            dev_logs(service, *follow, *lines, config, output).await
        }
        crate::commands::DevCommands::Setup => dev_setup(config, output).await,
        crate::commands::DevCommands::Clean => dev_clean(config, output).await,
    }
}

async fn dev_up(
    services: &[String],
    hot_reload: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Starting development environment...")?;

    let services_to_start = if services.is_empty() {
        config.workspace.services.keys().cloned().collect::<Vec<_>>()
    } else {
        services.to_vec()
    };

    // Create docker-compose override for development
    create_dev_override(hot_reload, &services_to_start, config, output).await?;

    // Start Docker Compose services
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.dev.yml", "up", "-d"]);

    if !services_to_start.is_empty() {
        cmd.args(&services_to_start);
    }

    output.info(&format!("Starting services: {}", services_to_start.join(", ")))?;
    
    let status = cmd.status().await?;
    if status.success() {
        output.success("Development environment started successfully!")?;
        
        if hot_reload {
            output.info("Hot reload is enabled. Changes will be automatically detected.")?;
            start_hot_reload(&services_to_start, config, output).await?;
        }
    } else {
        output.error("Failed to start development environment")?;
    }

    Ok(())
}

async fn dev_down(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Stopping development environment...")?;

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "down"]);

    if !services.is_empty() {
        output.info(&format!("Stopping services: {}", services.join(", ")))?;
        // For selective service stopping, we use stop instead of down
        cmd.args(&["--remove-orphans"]);
    }

    let status = cmd.status().await?;
    if status.success() {
        output.success("Development environment stopped successfully!")?;
    } else {
        output.error("Failed to stop development environment")?;
    }

    Ok(())
}

async fn dev_restart(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Restarting development environment...")?;

    // Stop first
    dev_down(services, config, output).await?;
    
    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Start again
    dev_up(services, false, config, output).await?;

    Ok(())
}

async fn dev_logs(
    service: &str,
    follow: bool,
    lines: u32,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Showing logs for service: {}", service))?;

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "logs"]);
    
    if follow {
        cmd.arg("-f");
    }
    
    cmd.args(&["--tail", &lines.to_string()]);
    cmd.arg(service);

    let status = cmd.status().await?;
    if !status.success() {
        output.error(&format!("Failed to get logs for service: {}", service))?;
    }

    Ok(())
}

async fn dev_setup(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Setting up development environment...")?;

    let progress = output.progress("Setting up development environment...", 6)?;

    // Step 1: Check dependencies
    progress.set_message("Checking dependencies...");
    check_dev_dependencies(output).await?;
    progress.inc(1);

    // Step 2: Build Docker images
    progress.set_message("Building Docker images...");
    build_dev_images(config, output).await?;
    progress.inc(1);

    // Step 3: Setup database
    progress.set_message("Setting up database...");
    setup_dev_database(config, output).await?;
    progress.inc(1);

    // Step 4: Run migrations
    progress.set_message("Running migrations...");
    run_dev_migrations(config, output).await?;
    progress.inc(1);

    // Step 5: Create networks
    progress.set_message("Creating networks...");
    create_dev_networks(output).await?;
    progress.inc(1);

    // Step 6: Start base services
    progress.set_message("Starting base services...");
    start_base_services(config, output).await?;
    progress.inc(1);

    progress.finish_with_message("Development environment setup complete!");
    output.success("Development environment is ready!")?;

    Ok(())
}

async fn dev_clean(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Cleaning development environment...")?;

    let confirmed = output.confirm("This will remove all containers, images, and volumes. Continue?")?;
    if !confirmed {
        output.info("Cleanup cancelled.")?;
        return Ok(());
    }

    let progress = output.progress("Cleaning development environment...", 4)?;

    // Stop all containers
    progress.set_message("Stopping containers...");
    let _ = tokio::process::Command::new("docker")
        .args(&["compose", "down", "--remove-orphans"])
        .status()
        .await?;
    progress.inc(1);

    // Remove images
    progress.set_message("Removing images...");
    clean_dev_images(config, output).await?;
    progress.inc(1);

    // Remove volumes
    progress.set_message("Removing volumes...");
    let _ = tokio::process::Command::new("docker")
        .args(&["volume", "prune", "-f"])
        .status()
        .await?;
    progress.inc(1);

    // Clean build artifacts
    progress.set_message("Cleaning build artifacts...");
    CargoHelper::clean().await?;
    progress.inc(1);

    progress.finish_with_message("Development environment cleaned!");
    output.success("Development environment cleaned successfully!")?;

    Ok(())
}

async fn create_dev_override(
    hot_reload: bool,
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let mut override_content = String::from("version: '3.8'\nservices:\n");

    for service in services {
        if let Some(service_config) = config.workspace.services.get(service) {
            override_content.push_str(&format!("  {}:\n", service));
            override_content.push_str("    environment:\n");
            override_content.push_str("      - RUST_LOG=debug\n");
            override_content.push_str("      - RUST_BACKTRACE=1\n");
            
            if hot_reload {
                override_content.push_str("    volumes:\n");
                override_content.push_str(&format!("      - ./{}:/app\n", service_config.path));
                override_content.push_str("      - /app/target\n");
            }
            
            override_content.push_str("\n");
        }
    }

    tokio::fs::write("docker-compose.dev.override.yml", override_content).await?;
    output.info("Created development override configuration")?;

    Ok(())
}

async fn start_hot_reload(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Starting hot reload watchers...")?;

    for service in services {
        if let Some(service_config) = config.workspace.services.get(service) {
            match service_config.service_type.as_str() {
                "backend" => {
                    // Start cargo watch for backend services
                    let mut cmd = tokio::process::Command::new("cargo");
                    cmd.args(&["watch", "-x", &format!("build --bin layanan-{}", service)]);
                    cmd.current_dir(&service_config.path);
                    
                    tokio::spawn(async move {
                        let _ = cmd.status().await;
                    });
                }
                "frontend" => {
                    // Start trunk serve for frontend services
                    let mut cmd = tokio::process::Command::new("trunk");
                    cmd.args(&["serve", "--config", &format!("{}/Trunk.toml", service_config.path)]);
                    
                    tokio::spawn(async move {
                        let _ = cmd.status().await;
                    });
                }
                _ => {}
            }
        }
    }

    output.success("Hot reload watchers started")?;
    Ok(())
}

async fn check_dev_dependencies(output: &SimplOutput) -> Result<()> {
    let dependencies = ["docker", "docker-compose", "cargo", "trunk"];
    
    for dep in dependencies {
        let status = tokio::process::Command::new("which")
            .arg(dep)
            .status()
            .await?;
            
        if !status.success() {
            output.error(&format!("Missing dependency: {}", dep))?;
            return Err(anyhow::anyhow!("Missing dependency: {}", dep));
        }
    }
    
    output.success("All dependencies are available")?;
    Ok(())
}

async fn build_dev_images(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, service_config) in &config.workspace.services {
        if service_config.build.dockerfile.is_some() {
            output.info(&format!("Building image for {}", name))?;
            
            let mut cmd = tokio::process::Command::new("docker");
            cmd.args(&["build", "-t", &format!("simpelv2-{}", name), "."]);
            cmd.current_dir(&service_config.path);
            
            let status = cmd.status().await?;
            if !status.success() {
                output.error(&format!("Failed to build image for {}", name))?;
            }
        }
    }
    
    Ok(())
}

async fn setup_dev_database(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Starting PostgreSQL container...")?;
    
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&[
        "run", "-d",
        "--name", "simpelv2-postgres",
        "--network", "simpelv2-network",
        "-e", "POSTGRES_DB=simpelv2",
        "-e", "POSTGRES_USER=simpelv2",
        "-e", "POSTGRES_PASSWORD=simpelv2",
        "-p", "5432:5432",
        "postgres:15"
    ]);
    
    let status = cmd.status().await?;
    if !status.success() {
        output.warning("PostgreSQL container might already be running")?;
    }
    
    Ok(())
}

async fn run_dev_migrations(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    // Wait for database to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    for (name, service_config) in &config.workspace.services {
        if service_config.service_type == "backend" {
            let migration_dir = format!("{}/migrations", service_config.path);
            if tokio::fs::metadata(&migration_dir).await.is_ok() {
                output.info(&format!("Running migrations for {}", name))?;
                
                // This would typically use sqlx or diesel CLI
                // For now, we'll just log that migrations would run
                output.info(&format!("Migrations for {} would run here", name))?;
            }
        }
    }
    
    Ok(())
}

async fn create_dev_networks(output: &SimplOutput) -> Result<()> {
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["network", "create", "simpelv2-network"]);
    
    let status = cmd.status().await?;
    if !status.success() {
        output.info("Network might already exist")?;
    }
    
    Ok(())
}

async fn start_base_services(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    let base_services = ["redis", "postgres", "vault"];
    
    for service in base_services {
        output.info(&format!("Starting {}", service))?;
        
        let mut cmd = tokio::process::Command::new("docker");
        cmd.args(&["compose", "up", "-d", service]);
        
        let status = cmd.status().await?;
        if !status.success() {
            output.warning(&format!("Failed to start {}", service))?;
        }
    }
    
    Ok(())
}

async fn clean_dev_images(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, _) in &config.workspace.services {
        let image_name = format!("simpelv2-{}", name);
        
        let mut cmd = tokio::process::Command::new("docker");
        cmd.args(&["rmi", "-f", &image_name]);
        
        let _ = cmd.status().await; // Ignore errors
    }
    
    // Remove dangling images
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["image", "prune", "-f"]);
    let _ = cmd.status().await;
    
    Ok(())
}
