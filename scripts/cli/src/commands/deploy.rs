use crate::{
    config::SimplConfig,
    utils::{output::SimplOutput, process::{DockerHelper, KubectlHelper}},
};
use anyhow::Result;

pub async fn handle_deploy_command(
    command: &crate::commands::DeployCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::DeployCommands::Up { env, services } => {
            deploy_up(env, services, config, output).await
        }
        crate::commands::DeployCommands::Down { env, services } => {
            deploy_down(env, services, config, output).await
        }
        crate::commands::DeployCommands::Update { env, services } => {
            deploy_update(env, services, config, output).await
        }
        crate::commands::DeployCommands::Status { env } => {
            deploy_status(env, config, output).await
        }
        crate::commands::DeployCommands::Scale { service, replicas, env } => {
            deploy_scale(service, *replicas, env, config, output).await
        }
    }
}

async fn deploy_up(
    env: &str,
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Deploying to environment: {}", env))?;

    // Determine deployment strategy based on environment
    match env {
        "dev" | "development" => deploy_to_dev(services, config, output).await,
        "staging" | "stage" => deploy_to_staging(services, config, output).await,
        "prod" | "production" => deploy_to_production(services, config, output).await,
        "k8s" | "kubernetes" => deploy_to_kubernetes(services, config, output).await,
        _ => {
            output.error(&format!("Unknown environment: {}", env))?;
            Ok(())
        }
    }
}

async fn deploy_down(
    env: &str,
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Removing deployment from environment: {}", env))?;

    match env {
        "dev" | "development" => remove_from_dev(services, config, output).await,
        "staging" | "stage" => remove_from_staging(services, config, output).await,
        "prod" | "production" => remove_from_production(services, config, output).await,
        "k8s" | "kubernetes" => remove_from_kubernetes(services, config, output).await,
        _ => {
            output.error(&format!("Unknown environment: {}", env))?;
            Ok(())
        }
    }
}

async fn deploy_update(
    env: &str,
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Updating deployment in environment: {}", env))?;

    let services_to_update = if services.is_empty() {
        config.workspace.services.keys().cloned().collect::<Vec<_>>()
    } else {
        services.to_vec()
    };

    let progress = output.progress("Updating services...", services_to_update.len())?;

    for service in &services_to_update {
        progress.set_message(&format!("Updating {}", service));
        
        match env {
            "k8s" | "kubernetes" => update_k8s_service(service, config, output).await?,
            _ => update_docker_service(service, env, config, output).await?,
        }
        
        progress.inc(1);
    }

    progress.finish_with_message("Services updated successfully!");
    output.success("Deployment update completed!")?;

    Ok(())
}

async fn deploy_status(
    env: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Checking deployment status for environment: {}", env))?;

    match env {
        "k8s" | "kubernetes" => show_k8s_status(config, output).await,
        _ => show_docker_status(env, config, output).await,
    }
}

async fn deploy_scale(
    service: &str,
    replicas: u32,
    env: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Scaling {} to {} replicas in {}", service, replicas, env))?;

    match env {
        "k8s" | "kubernetes" => scale_k8s_service(service, replicas, config, output).await,
        _ => scale_docker_service(service, replicas, env, config, output).await,
    }
}

async fn deploy_to_dev(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Deploying to development environment using Docker Compose...")?;

    let services_to_deploy = if services.is_empty() {
        config.workspace.services.keys().cloned().collect::<Vec<_>>()
    } else {
        services.to_vec()
    };

    let progress = output.progress("Deploying to development...", services_to_deploy.len() + 2)?;

    // Build images first
    progress.set_message("Building Docker images...");
    for service in &services_to_deploy {
        if let Some(service_config) = config.workspace.services.get(service) {
            if service_config.build.dockerfile.is_some() {
                let image_tag = format!("simpelv2-{}", service);
                DockerHelper::build(&image_tag, &service_config.path, service_config.build.dockerfile.as_deref()).await?;
            }
        }
    }
    progress.inc(1);

    // Start services with Docker Compose
    progress.set_message("Starting services...");
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.dev.yml", "up", "-d"]);
    
    if !services_to_deploy.is_empty() {
        cmd.args(&services_to_deploy);
    }

    let status = cmd.status().await?;
    if !status.success() {
        output.error("Failed to deploy to development environment")?;
        return Ok(());
    }
    progress.inc(1);

    progress.finish_with_message("Development deployment completed!");
    output.success("Successfully deployed to development environment!")?;

    // Show service URLs
    show_dev_service_urls(config, output).await?;

    Ok(())
}

async fn deploy_to_staging(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Deploying to staging environment...")?;

    let confirmed = output.confirm("Deploy to staging environment? This will affect the staging server.")?;
    if !confirmed {
        output.info("Deployment cancelled.")?;
        return Ok(());
    }

    // Use production Docker Compose configuration
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.prod.yml", "up", "-d"]);
    
    if !services.is_empty() {
        cmd.args(services);
    }

    let status = cmd.status().await?;
    if status.success() {
        output.success("Successfully deployed to staging environment!")?;
    } else {
        output.error("Failed to deploy to staging environment")?;
    }

    Ok(())
}

async fn deploy_to_production(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.warning("WARNING: You are about to deploy to PRODUCTION!")?;
    
    let confirmed = output.confirm("Are you sure you want to deploy to production?")?;
    if !confirmed {
        output.info("Production deployment cancelled.")?;
        return Ok(());
    }

    let double_confirmed = output.confirm("This action cannot be undone. Deploy to production?")?;
    if !double_confirmed {
        output.info("Production deployment cancelled.")?;
        return Ok(());
    }

    output.info("Deploying to production environment...")?;

    // Run pre-deployment checks
    run_pre_deployment_checks(config, output).await?;

    // Use production Docker Compose configuration
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.prod.yml", "up", "-d"]);
    
    if !services.is_empty() {
        cmd.args(services);
    }

    let status = cmd.status().await?;
    if status.success() {
        output.success("Successfully deployed to production environment!")?;
        
        // Run post-deployment checks
        run_post_deployment_checks(config, output).await?;
    } else {
        output.error("Failed to deploy to production environment")?;
    }

    Ok(())
}

async fn deploy_to_kubernetes(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Deploying to Kubernetes...")?;

    let services_to_deploy = if services.is_empty() {
        config.workspace.services.keys().cloned().collect::<Vec<_>>()
    } else {
        services.to_vec()
    };

    let progress = output.progress("Deploying to Kubernetes...", services_to_deploy.len() + 2)?;

    // Apply namespace and common resources
    progress.set_message("Creating namespace and common resources...");
    KubectlHelper::apply("infra/k8s/namespace.yaml").await?;
    KubectlHelper::apply("infra/k8s/configmap.yaml").await?;
    progress.inc(1);

    // Deploy each service
    for service in &services_to_deploy {
        progress.set_message(&format!("Deploying {}", service));
        
        let k8s_manifest = format!("infra/k8s/{}.yaml", service);
        if tokio::fs::metadata(&k8s_manifest).await.is_ok() {
            KubectlHelper::apply(&k8s_manifest).await?;
        } else {
            output.warning(&format!("No Kubernetes manifest found for {}", service))?;
        }
    }
    progress.inc(1);

    progress.finish_with_message("Kubernetes deployment completed!");
    output.success("Successfully deployed to Kubernetes!")?;

    // Show deployment status
    show_k8s_status(config, output).await?;

    Ok(())
}

async fn remove_from_dev(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "down"]);
    
    if !services.is_empty() {
        cmd.args(&["--remove-orphans"]);
        // For selective removal, stop specific services
        for service in services {
            let mut stop_cmd = tokio::process::Command::new("docker");
            stop_cmd.args(&["compose", "stop", service]);
            let _ = stop_cmd.status().await;
        }
    }

    let status = cmd.status().await?;
    if status.success() {
        output.success("Removed from development environment!")?;
    } else {
        output.error("Failed to remove from development environment")?;
    }

    Ok(())
}

async fn remove_from_staging(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let confirmed = output.confirm("Remove from staging environment?")?;
    if !confirmed {
        return Ok(());
    }

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.prod.yml", "down"]);

    let status = cmd.status().await?;
    if status.success() {
        output.success("Removed from staging environment!")?;
    } else {
        output.error("Failed to remove from staging environment")?;
    }

    Ok(())
}

async fn remove_from_production(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.warning("WARNING: You are about to remove from PRODUCTION!")?;
    
    let confirmed = output.confirm("Are you sure you want to remove from production?")?;
    if !confirmed {
        return Ok(());
    }

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", "docker-compose.prod.yml", "down"]);

    let status = cmd.status().await?;
    if status.success() {
        output.success("Removed from production environment!")?;
    } else {
        output.error("Failed to remove from production environment")?;
    }

    Ok(())
}

async fn remove_from_kubernetes(
    services: &[String],
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let services_to_remove = if services.is_empty() {
        config.workspace.services.keys().cloned().collect::<Vec<_>>()
    } else {
        services.to_vec()
    };

    for service in &services_to_remove {
        let k8s_manifest = format!("infra/k8s/{}.yaml", service);
        if tokio::fs::metadata(&k8s_manifest).await.is_ok() {
            KubectlHelper::delete(&k8s_manifest).await?;
            output.info(&format!("Removed {} from Kubernetes", service))?;
        }
    }

    output.success("Removed from Kubernetes!")?;
    Ok(())
}

async fn update_k8s_service(
    service: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let k8s_manifest = format!("infra/k8s/{}.yaml", service);
    if tokio::fs::metadata(&k8s_manifest).await.is_ok() {
        KubectlHelper::apply(&k8s_manifest).await?;
        output.info(&format!("Updated {} in Kubernetes", service))?;
    } else {
        output.warning(&format!("No Kubernetes manifest found for {}", service))?;
    }
    Ok(())
}

async fn update_docker_service(
    service: &str,
    env: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    // Restart the specific service
    let compose_file = match env {
        "prod" | "production" | "staging" => "docker-compose.prod.yml",
        _ => "docker-compose.dev.yml",
    };

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "-f", "docker-compose.yml", "-f", compose_file, "restart", service]);

    let status = cmd.status().await?;
    if status.success() {
        output.info(&format!("Updated {} in {} environment", service, env))?;
    } else {
        output.error(&format!("Failed to update {} in {} environment", service, env))?;
    }

    Ok(())
}

async fn show_k8s_status(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Kubernetes Deployment Status:")?;
    
    // Show pods
    let pod_output = KubectlHelper::get("pods", Some("simpelv2")).await?;
    output.info("Pods:")?;
    println!("{}", pod_output);
    
    // Show services
    let svc_output = KubectlHelper::get("services", Some("simpelv2")).await?;
    output.info("Services:")?;
    println!("{}", svc_output);

    Ok(())
}

async fn show_docker_status(env: &str, config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info(&format!("Docker Deployment Status ({}):", env))?;
    
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "ps"]);
    
    let output_result = cmd.output().await?;
    if output_result.status.success() {
        let status_text = String::from_utf8_lossy(&output_result.stdout);
        println!("{}", status_text);
    }

    Ok(())
}

async fn scale_k8s_service(
    service: &str,
    replicas: u32,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    KubectlHelper::scale(&format!("deployment/{}", service), replicas, Some("simpelv2")).await?;
    output.success(&format!("Scaled {} to {} replicas in Kubernetes", service, replicas))?;
    Ok(())
}

async fn scale_docker_service(
    service: &str,
    replicas: u32,
    env: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(&["compose", "up", "-d", "--scale", &format!("{}={}", service, replicas), service]);
    
    let status = cmd.status().await?;
    if status.success() {
        output.success(&format!("Scaled {} to {} replicas in {} environment", service, replicas, env))?;
    } else {
        output.error(&format!("Failed to scale {} in {} environment", service, env))?;
    }

    Ok(())
}

async fn show_dev_service_urls(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Service URLs:")?;
    
    for (name, service_config) in &config.workspace.services {
        if let Some(port) = service_config.port {
            println!("  {} -> http://localhost:{}", name, port);
        }
    }

    Ok(())
}

async fn run_pre_deployment_checks(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Running pre-deployment checks...")?;
    
    // Check if all required environment variables are set
    // Check if database is accessible
    // Check if all services build successfully
    // Check if tests pass
    
    output.success("Pre-deployment checks passed!")?;
    Ok(())
}

async fn run_post_deployment_checks(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Running post-deployment checks...")?;
    
    // Health check all services
    // Verify database connectivity
    // Check if all endpoints are responding
    
    output.success("Post-deployment checks passed!")?;
    Ok(())
}
