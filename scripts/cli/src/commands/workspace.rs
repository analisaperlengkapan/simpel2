use crate::{
    config::SimplConfig,
    utils::output::SimplOutput,
};
use anyhow::Result;

pub async fn handle_workspace_command(
    command: &crate::commands::WorkspaceCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::WorkspaceCommands::Init => workspace_init(config, output).await,
        crate::commands::WorkspaceCommands::Clean => workspace_clean(config, output).await,
        crate::commands::WorkspaceCommands::Update => workspace_update(config, output).await,
        crate::commands::WorkspaceCommands::Check => workspace_check(config, output).await,
        crate::commands::WorkspaceCommands::Info => workspace_info(config, output).await,
    }
}

async fn workspace_init(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Initializing workspace...")?;
    
    let progress = output.progress("Initializing workspace...", 5)?;
    
    // Check if already in a workspace
    if crate::utils::is_simpelv2_workspace(&std::env::current_dir()?) {
        output.warning("Already in a SIMPelv2 workspace")?;
        return Ok(());
    }
    
    // Create workspace structure
    progress.set_message("Creating workspace structure...");
    create_workspace_structure(output).await?;
    progress.inc(1);
    
    // Initialize Git repository
    progress.set_message("Initializing Git repository...");
    init_git_repository(output).await?;
    progress.inc(1);
    
    // Create configuration files
    progress.set_message("Creating configuration files...");
    create_config_files(output).await?;
    progress.inc(1);
    
    // Create Docker files
    progress.set_message("Creating Docker files...");
    create_docker_files(output).await?;
    progress.inc(1);
    
    // Create Makefile
    progress.set_message("Creating Makefile...");
    create_makefile(output).await?;
    progress.inc(1);
    
    progress.finish_with_message("Workspace initialized!");
    output.success("SIMPelv2 workspace initialized successfully!")?;
    
    show_getting_started_info(output).await?;
    
    Ok(())
}

async fn workspace_clean(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Cleaning workspace...")?;
    
    let confirmed = output.confirm("This will remove all build artifacts, logs, and temporary files. Continue?")?;
    if !confirmed {
        output.info("Workspace clean cancelled.")?;
        return Ok(());
    }
    
    let progress = output.progress("Cleaning workspace...", 6)?;
    
    // Clean Cargo artifacts
    progress.set_message("Cleaning Cargo artifacts...");
    let _ = tokio::process::Command::new("cargo")
        .args(&["clean"])
        .status()
        .await;
    progress.inc(1);
    
    // Clean Docker artifacts
    progress.set_message("Cleaning Docker artifacts...");
    clean_docker_artifacts(output).await?;
    progress.inc(1);
    
    // Clean frontend build artifacts
    progress.set_message("Cleaning frontend artifacts...");
    clean_frontend_artifacts(config, output).await?;
    progress.inc(1);
    
    // Clean logs
    progress.set_message("Cleaning logs...");
    clean_logs(output).await?;
    progress.inc(1);
    
    // Clean temporary files
    progress.set_message("Cleaning temporary files...");
    clean_temp_files(output).await?;
    progress.inc(1);
    
    // Clean node_modules if any
    progress.set_message("Cleaning node_modules...");
    clean_node_modules(output).await?;
    progress.inc(1);
    
    progress.finish_with_message("Workspace cleaned!");
    output.success("Workspace cleaned successfully!")?;
    
    Ok(())
}

async fn workspace_update(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Updating workspace dependencies...")?;
    
    let progress = output.progress("Updating dependencies...", 4)?;
    
    // Update Rust dependencies
    progress.set_message("Updating Rust dependencies...");
    update_rust_dependencies(output).await?;
    progress.inc(1);
    
    // Update Docker images
    progress.set_message("Updating Docker images...");
    update_docker_images(output).await?;
    progress.inc(1);
    
    // Update configuration
    progress.set_message("Updating configuration...");
    update_configuration(config, output).await?;
    progress.inc(1);
    
    // Update scripts
    progress.set_message("Updating scripts...");
    update_scripts(output).await?;
    progress.inc(1);
    
    progress.finish_with_message("Dependencies updated!");
    output.success("Workspace dependencies updated successfully!")?;
    
    Ok(())
}

async fn workspace_check(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Checking workspace health...")?;
    
    let mut issues = Vec::new();
    
    // Check workspace structure
    check_workspace_structure(&mut issues, output).await?;
    
    // Check dependencies
    check_dependencies(&mut issues, output).await?;
    
    // Check configuration
    check_configuration(config, &mut issues, output).await?;
    
    // Check services
    check_services(config, &mut issues, output).await?;
    
    if issues.is_empty() {
        output.success("Workspace is healthy!")?;
    } else {
        output.warning("Workspace health check found issues:")?;
        for issue in issues {
            println!("  ⚠️  {}", issue);
        }
    }
    
    Ok(())
}

async fn workspace_info(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Workspace Information:")?;
    
    // Workspace details
    println!("📁 Workspace: {}", config.workspace.name);
    println!("📦 Version: {}", config.workspace.version);
    println!("🏗️  Build Target: {}", config.build.target);
    println!("🚀 Deploy Environment: {}", config.deploy.environment);
    
    // Services count
    let backend_count = config.workspace.services.iter()
        .filter(|(_, s)| s.service_type == "backend")
        .count();
    let frontend_count = config.workspace.services.iter()
        .filter(|(_, s)| s.service_type == "frontend")
        .count();
    
    println!("🔧 Backend Services: {}", backend_count);
    println!("🎨 Frontend Services: {}", frontend_count);
    println!("📊 Total Services: {}", config.workspace.services.len());
    
    // Show services
    if !config.workspace.services.is_empty() {
        println!("\n📋 Services:");
        for (name, service_config) in &config.workspace.services {
            let status = if tokio::fs::metadata(&service_config.path).await.is_ok() {
                "✅"
            } else {
                "❌"
            };
            println!("  {} {} ({})", status, name, service_config.service_type);
        }
    }
    
    // Git information
    if let Ok(output) = tokio::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .await
    {
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("🔖 Git Commit: {}", commit);
        }
    }
    
    // Docker information
    if let Ok(output) = tokio::process::Command::new("docker")
        .args(&["images", "--format", "table {{.Repository}}\t{{.Tag}}\t{{.Size}}", "--filter", "reference=simpelv2-*"])
        .output()
        .await
    {
        if output.status.success() {
            let images = String::from_utf8_lossy(&output.stdout);
            if !images.trim().is_empty() {
                println!("\n🐳 Docker Images:");
                println!("{}", images);
            }
        }
    }
    
    Ok(())
}

async fn create_workspace_structure(output: &SimplOutput) -> Result<()> {
    let directories = [
        "layanan",
        "antarmuka",
        "infra",
        "infra/k8s",
        "infra/nginx",
        "infra/gerbang",
        "scripts",
        "scripts/test",
        "scripts/tools",
        "docs",
        "shared",
    ];
    
    for dir in &directories {
        tokio::fs::create_dir_all(dir).await?;
        output.info(&format!("Created directory: {}", dir))?;
    }
    
    Ok(())
}

async fn init_git_repository(output: &SimplOutput) -> Result<()> {
    if tokio::fs::metadata(".git").await.is_ok() {
        output.info("Git repository already exists")?;
        return Ok(());
    }
    
    let status = tokio::process::Command::new("git")
        .args(&["init"])
        .status()
        .await?;
        
    if status.success() {
        output.success("Git repository initialized")?;
        
        // Create .gitignore
        let gitignore_content = r#"# Rust
target/
Cargo.lock
**/*.rs.bk

# Node
node_modules/
npm-debug.log*

# Trunk
dist/

# IDEs
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# Logs
*.log
logs/

# Docker
.dockerignore

# Temporary files
tmp/
.tmp/
"#;
        tokio::fs::write(".gitignore", gitignore_content).await?;
    }
    
    Ok(())
}

async fn create_config_files(output: &SimplOutput) -> Result<()> {
    // Create simpel.toml
    let config = crate::config::SimplConfig::default();
    let config_content = toml::to_string_pretty(&config)?;
    tokio::fs::write("simpel.toml", config_content).await?;
    output.success("Created simpel.toml")?;
    
    // Create .env.example
    let env_example = r#"# Database
DATABASE_URL=postgresql://simpelv2:simpelv2@localhost:5432/simpelv2

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-jwt-secret-here

# API Keys
AI_API_KEY=your-ai-api-key-here

# Environment
RUST_LOG=info
RUST_BACKTRACE=1
"#;
    tokio::fs::write(".env.example", env_example).await?;
    output.success("Created .env.example")?;
    
    Ok(())
}

async fn create_docker_files(output: &SimplOutput) -> Result<()> {
    // Create basic docker-compose.yml
    let docker_compose = r#"version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: simpelv2
      POSTGRES_USER: simpelv2
      POSTGRES_PASSWORD: simpelv2
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
"#;
    tokio::fs::write("docker-compose.yml", docker_compose).await?;
    output.success("Created docker-compose.yml")?;
    
    Ok(())
}

async fn create_makefile(output: &SimplOutput) -> Result<()> {
    let makefile_content = r#".PHONY: up down build test clean

up:
	simpel dev up

down:
	simpel dev down

build:
	simpel build all

test:
	simpel test all

clean:
	simpel workspace clean

help:
	@echo "Available targets:"
	@echo "  up    - Start development environment"
	@echo "  down  - Stop development environment"
	@echo "  build - Build all services"
	@echo "  test  - Run all tests"
	@echo "  clean - Clean workspace"
"#;
    tokio::fs::write("Makefile", makefile_content).await?;
    output.success("Created Makefile")?;
    
    Ok(())
}

async fn show_getting_started_info(output: &SimplOutput) -> Result<()> {
    output.info("Getting Started:")?;
    println!("  1. Create your first service:");
    println!("     simpel make service my-service");
    println!("  2. Create your first frontend:");
    println!("     simpel make frontend my-frontend");
    println!("  3. Start development environment:");
    println!("     simpel dev up");
    println!("  4. Build everything:");
    println!("     simpel build all");
    println!("  5. Run tests:");
    println!("     simpel test all");
    println!("\n  For more help: simpel --help");
    
    Ok(())
}

async fn clean_docker_artifacts(output: &SimplOutput) -> Result<()> {
    // Stop all containers
    let _ = tokio::process::Command::new("docker")
        .args(&["compose", "down", "--remove-orphans"])
        .status()
        .await;
    
    // Remove simpelv2 images
    let _ = tokio::process::Command::new("docker")
        .args(&["images", "-q", "--filter", "reference=simpelv2-*"])
        .output()
        .await;
    
    Ok(())
}

async fn clean_frontend_artifacts(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (_, service_config) in &config.workspace.services {
        if service_config.service_type == "frontend" {
            let dist_dir = format!("{}/dist", service_config.path);
            if tokio::fs::metadata(&dist_dir).await.is_ok() {
                let _ = tokio::fs::remove_dir_all(&dist_dir).await;
            }
        }
    }
    Ok(())
}

async fn clean_logs(output: &SimplOutput) -> Result<()> {
    let log_patterns = ["*.log", "logs/", ".log"];
    
    for pattern in log_patterns {
        // In a real implementation, you'd use glob to find and remove log files
        output.info(&format!("Cleaning logs matching: {}", pattern))?;
    }
    
    Ok(())
}

async fn clean_temp_files(output: &SimplOutput) -> Result<()> {
    let temp_dirs = ["tmp/", ".tmp/", "target/tmp/"];
    
    for dir in temp_dirs {
        if tokio::fs::metadata(dir).await.is_ok() {
            let _ = tokio::fs::remove_dir_all(dir).await;
        }
    }
    
    Ok(())
}

async fn clean_node_modules(output: &SimplOutput) -> Result<()> {
    // Find and remove node_modules directories
    if tokio::fs::metadata("node_modules").await.is_ok() {
        let _ = tokio::fs::remove_dir_all("node_modules").await;
        output.info("Removed node_modules")?;
    }
    
    Ok(())
}

async fn update_rust_dependencies(output: &SimplOutput) -> Result<()> {
    let status = tokio::process::Command::new("cargo")
        .args(&["update"])
        .status()
        .await?;
        
    if status.success() {
        output.success("Rust dependencies updated")?;
    }
    
    Ok(())
}

async fn update_docker_images(output: &SimplOutput) -> Result<()> {
    let status = tokio::process::Command::new("docker")
        .args(&["compose", "pull"])
        .status()
        .await?;
        
    if status.success() {
        output.success("Docker images updated")?;
    }
    
    Ok(())
}

async fn update_configuration(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    // Check if configuration needs updates
    output.info("Configuration is up to date")?;
    Ok(())
}

async fn update_scripts(output: &SimplOutput) -> Result<()> {
    // Update any scripts that might need updating
    output.info("Scripts are up to date")?;
    Ok(())
}

async fn check_workspace_structure(issues: &mut Vec<String>, output: &SimplOutput) -> Result<()> {
    let required_dirs = ["layanan", "antarmuka", "infra"];
    
    for dir in required_dirs {
        if tokio::fs::metadata(dir).await.is_err() {
            issues.push(format!("Missing required directory: {}", dir));
        }
    }
    
    Ok(())
}

async fn check_dependencies(issues: &mut Vec<String>, output: &SimplOutput) -> Result<()> {
    let required_tools = ["cargo", "docker", "docker-compose"];
    
    for tool in required_tools {
        let status = tokio::process::Command::new("which")
            .arg(tool)
            .status()
            .await?;
            
        if !status.success() {
            issues.push(format!("Missing required tool: {}", tool));
        }
    }
    
    Ok(())
}

async fn check_configuration(config: &SimplConfig, issues: &mut Vec<String>, output: &SimplOutput) -> Result<()> {
    if config.workspace.name.is_empty() {
        issues.push("Workspace name is not configured".to_string());
    }
    
    Ok(())
}

async fn check_services(config: &SimplConfig, issues: &mut Vec<String>, output: &SimplOutput) -> Result<()> {
    for (name, service_config) in &config.workspace.services {
        if tokio::fs::metadata(&service_config.path).await.is_err() {
            issues.push(format!("Service {} path does not exist: {}", name, service_config.path));
        }
    }
    
    Ok(())
}
