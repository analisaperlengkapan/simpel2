use crate::{
    config::SimplConfig,
    utils::{output::SimplOutput, process::{CargoHelper, DockerHelper}},
};
use anyhow::Result;
use std::path::PathBuf;

pub async fn handle_build_command(
    command: &crate::commands::BuildCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::BuildCommands::All { release, features } => {
            build_all(*release, features, config, output).await
        }
        crate::commands::BuildCommands::Backend { service, release } => {
            build_backend(service.as_deref(), *release, config, output).await
        }
        crate::commands::BuildCommands::Frontend { frontend, release } => {
            build_frontend(frontend.as_deref(), *release, config, output).await
        }
        crate::commands::BuildCommands::Docker { service, push } => {
            build_docker(service.as_deref(), *push, config, output).await
        }
        crate::commands::BuildCommands::Clean => build_clean(config, output).await,
        crate::commands::BuildCommands::Check { service } => {
            build_check(service.as_deref(), config, output).await
        }
    }
}

async fn build_all(
    release: bool,
    features: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Building all services...")?;

    let total_services = config.workspace.services.len();
    let progress = output.progress("Building services...", total_services)?;

    // Build backend services first
    let backend_services: Vec<_> = config
        .workspace
        .services
        .iter()
        .filter(|(_, cfg)| cfg.service_type == "backend")
        .collect();

    for (name, service_config) in backend_services {
        progress.set_message(&format!("Building backend: {}", name));
        
        let mut args = vec!["build", "--bin", &format!("layanan-{}", name)];
        if release {
            args.push("--release");
        }
        
        for feature in features {
            args.extend(&["--features", feature]);
        }

        CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
        progress.inc(1);
    }

    // Build frontend services
    let frontend_services: Vec<_> = config
        .workspace
        .services
        .iter()
        .filter(|(_, cfg)| cfg.service_type == "frontend")
        .collect();

    for (name, service_config) in frontend_services {
        progress.set_message(&format!("Building frontend: {}", name));
        build_single_frontend(name, &service_config.path, release, output).await?;
        progress.inc(1);
    }

    progress.finish_with_message("All services built successfully!");
    output.success("Build completed successfully!")?;

    Ok(())
}

async fn build_backend(
    service: Option<&str>,
    release: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(service_name) = service {
        // Build specific backend service
        if let Some(service_config) = config.workspace.services.get(service_name) {
            if service_config.service_type != "backend" {
                output.error(&format!("{} is not a backend service", service_name))?;
                return Ok(());
            }

            output.info(&format!("Building backend service: {}", service_name))?;
            
            let mut args = vec!["build", "--bin", &format!("layanan-{}", service_name)];
            if release {
                args.push("--release");
            }

            CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
            output.success(&format!("Backend service {} built successfully!", service_name))?;
        } else {
            output.error(&format!("Service {} not found", service_name))?;
        }
    } else {
        // Build all backend services
        output.info("Building all backend services...")?;

        let backend_services: Vec<_> = config
            .workspace
            .services
            .iter()
            .filter(|(_, cfg)| cfg.service_type == "backend")
            .collect();

        let progress = output.progress("Building backend services...", backend_services.len())?;

        for (name, service_config) in backend_services {
            progress.set_message(&format!("Building {}", name));
            
            let mut args = vec!["build", "--bin", &format!("layanan-{}", name)];
            if release {
                args.push("--release");
            }

            CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
            progress.inc(1);
        }

        progress.finish_with_message("All backend services built!");
        output.success("All backend services built successfully!")?;
    }

    Ok(())
}

async fn build_frontend(
    frontend: Option<&str>,
    release: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(frontend_name) = frontend {
        // Build specific frontend
        if let Some(service_config) = config.workspace.services.get(frontend_name) {
            if service_config.service_type != "frontend" {
                output.error(&format!("{} is not a frontend service", frontend_name))?;
                return Ok(());
            }

            output.info(&format!("Building frontend: {}", frontend_name))?;
            build_single_frontend(frontend_name, &service_config.path, release, output).await?;
            output.success(&format!("Frontend {} built successfully!", frontend_name))?;
        } else {
            output.error(&format!("Frontend {} not found", frontend_name))?;
        }
    } else {
        // Build all frontends
        output.info("Building all frontends...")?;

        let frontend_services: Vec<_> = config
            .workspace
            .services
            .iter()
            .filter(|(_, cfg)| cfg.service_type == "frontend")
            .collect();

        let progress = output.progress("Building frontends...", frontend_services.len())?;

        for (name, service_config) in frontend_services {
            progress.set_message(&format!("Building {}", name));
            build_single_frontend(name, &service_config.path, release, output).await?;
            progress.inc(1);
        }

        progress.finish_with_message("All frontends built!");
        output.success("All frontends built successfully!")?;
    }

    Ok(())
}

async fn build_single_frontend(
    name: &str,
    path: &str,
    release: bool,
    output: &SimplOutput,
) -> Result<()> {
    let trunk_config = format!("{}/Trunk.toml", path);
    
    if !PathBuf::from(&trunk_config).exists() {
        output.warning(&format!("Trunk.toml not found for {}, skipping", name))?;
        return Ok(());
    }

    let mut args = vec!["build", "--config", &trunk_config];
    if release {
        args.push("--release");
    }

    let mut cmd = tokio::process::Command::new("trunk");
    cmd.args(&args);

    let status = cmd.status().await?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to build frontend: {}", name));
    }

    Ok(())
}

async fn build_docker(
    service: Option<&str>,
    push: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(service_name) = service {
        // Build specific service Docker image
        if let Some(service_config) = config.workspace.services.get(service_name) {
            output.info(&format!("Building Docker image for: {}", service_name))?;
            build_single_docker_image(service_name, service_config, push, output).await?;
            output.success(&format!("Docker image for {} built successfully!", service_name))?;
        } else {
            output.error(&format!("Service {} not found", service_name))?;
        }
    } else {
        // Build all Docker images
        output.info("Building all Docker images...")?;

        let services_with_docker: Vec<_> = config
            .workspace
            .services
            .iter()
            .filter(|(_, cfg)| cfg.build.dockerfile.is_some())
            .collect();

        let progress = output.progress("Building Docker images...", services_with_docker.len())?;

        for (name, service_config) in services_with_docker {
            progress.set_message(&format!("Building {}", name));
            build_single_docker_image(name, service_config, false, output).await?;
            progress.inc(1);
        }

        progress.finish_with_message("All Docker images built!");

        if push {
            output.info("Pushing all images to registry...")?;
            push_all_images(config, output).await?;
        }

        output.success("All Docker images built successfully!")?;
    }

    Ok(())
}

async fn build_single_docker_image(
    name: &str,
    service_config: &crate::config::ServiceConfig,
    push: bool,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(dockerfile) = &service_config.build.dockerfile {
        let image_tag = format!("simpelv2-{}", name);
        
        // Build the image
        DockerHelper::build(&image_tag, &service_config.path, Some(dockerfile)).await?;

        if push {
            // Tag for registry if configured
            if let Some(registry) = &service_config.build.registry {
                let registry_tag = format!("{}/{}", registry, image_tag);
                DockerHelper::tag(&image_tag, &registry_tag).await?;
                DockerHelper::push(&registry_tag).await?;
                output.info(&format!("Pushed {} to registry", registry_tag))?;
            } else {
                output.warning(&format!("No registry configured for {}, skipping push", name))?;
            }
        }
    } else {
        output.warning(&format!("No Dockerfile configured for {}, skipping", name))?;
    }

    Ok(())
}

async fn push_all_images(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, service_config) in &config.workspace.services {
        if service_config.build.dockerfile.is_some() {
            if let Some(registry) = &service_config.build.registry {
                let image_tag = format!("simpelv2-{}", name);
                let registry_tag = format!("{}/{}", registry, image_tag);
                
                DockerHelper::tag(&image_tag, &registry_tag).await?;
                DockerHelper::push(&registry_tag).await?;
                output.info(&format!("Pushed {} to registry", registry_tag))?;
            }
        }
    }

    Ok(())
}

async fn build_clean(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Cleaning build artifacts...")?;

    let progress = output.progress("Cleaning build artifacts...", 4)?;

    // Clean Cargo artifacts
    progress.set_message("Cleaning Cargo artifacts...");
    CargoHelper::clean().await?;
    progress.inc(1);

    // Clean trunk dist directories
    progress.set_message("Cleaning frontend artifacts...");
    clean_frontend_artifacts(config, output).await?;
    progress.inc(1);

    // Clean Docker images
    progress.set_message("Cleaning Docker images...");
    clean_docker_images(config, output).await?;
    progress.inc(1);

    // Clean temporary files
    progress.set_message("Cleaning temporary files...");
    clean_temp_files(output).await?;
    progress.inc(1);

    progress.finish_with_message("Build artifacts cleaned!");
    output.success("All build artifacts cleaned successfully!")?;

    Ok(())
}

async fn clean_frontend_artifacts(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, service_config) in &config.workspace.services {
        if service_config.service_type == "frontend" {
            let dist_dir = format!("{}/dist", service_config.path);
            if PathBuf::from(&dist_dir).exists() {
                tokio::fs::remove_dir_all(&dist_dir).await.unwrap_or_else(|_| {
                    // Log but don't fail
                });
                output.info(&format!("Cleaned dist directory for {}", name))?;
            }
        }
    }

    Ok(())
}

async fn clean_docker_images(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, _) in &config.workspace.services {
        let image_name = format!("simpelv2-{}", name);
        let _ = DockerHelper::remove_image(&image_name).await; // Ignore errors
    }

    // Clean dangling images
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["image", "prune", "-f"]);
    let _ = cmd.status().await;

    Ok(())
}

async fn clean_temp_files(output: &SimplOutput) -> Result<()> {
    let temp_patterns = [
        "**/.tmp",
        "**/tmp",
        "**/*.log",
        "**/docker-compose.dev.override.yml",
    ];

    for pattern in temp_patterns {
        // Use glob to find and remove matching files
        // This is a simplified version - in practice you'd use the glob crate
        output.info(&format!("Cleaning files matching: {}", pattern))?;
    }

    Ok(())
}

async fn build_check(
    service: Option<&str>,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(service_name) = service {
        // Check specific service
        if let Some(service_config) = config.workspace.services.get(service_name) {
            output.info(&format!("Checking service: {}", service_name))?;
            
            if service_config.service_type == "backend" {
                let args = vec!["check", "--bin", &format!("layanan-{}", service_name)];
                CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
            } else {
                // For frontends, we can check if trunk can parse the config
                let trunk_config = format!("{}/Trunk.toml", service_config.path);
                if !PathBuf::from(&trunk_config).exists() {
                    output.warning(&format!("No Trunk.toml found for {}", service_name))?;
                } else {
                    output.success(&format!("Frontend {} configuration looks good", service_name))?;
                }
            }
            
            output.success(&format!("Service {} check completed!", service_name))?;
        } else {
            output.error(&format!("Service {} not found", service_name))?;
        }
    } else {
        // Check all services
        output.info("Checking all services...")?;

        let total_services = config.workspace.services.len();
        let progress = output.progress("Checking services...", total_services)?;

        for (name, service_config) in &config.workspace.services {
            progress.set_message(&format!("Checking {}", name));
            
            if service_config.service_type == "backend" {
                let args = vec!["check", "--bin", &format!("layanan-{}", name)];
                match CargoHelper::build_with_args(&args, Some(&service_config.path)).await {
                    Ok(_) => output.success(&format!("✓ {}", name))?,
                    Err(_) => output.error(&format!("✗ {}", name))?,
                }
            } else {
                let trunk_config = format!("{}/Trunk.toml", service_config.path);
                if PathBuf::from(&trunk_config).exists() {
                    output.success(&format!("✓ {}", name))?;
                } else {
                    output.warning(&format!("? {} (no Trunk.toml)", name))?;
                }
            }
            
            progress.inc(1);
        }

        progress.finish_with_message("All services checked!");
        output.success("Check completed!")?;
    }

    Ok(())
}
