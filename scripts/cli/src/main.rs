use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "simpel")]
#[command(about = "SIMPelv2 CLI Tool - Complete Workspace Management like Laravel Artisan")]
#[command(version = "2.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build operations (replaces menu items 1-5)
    Build {
        /// Build target (all, frontend, backend, clean)
        #[arg(default_value = "all")]
        target: String,
        /// Build mode (dev, release)
        #[arg(short, long, default_value = "dev")]
        mode: String,
    },
    /// Development operations (replaces menu item 4)
    Dev {
        /// Dev action (start, stop, restart, logs)
        #[arg(default_value = "start")]
        action: String,
        /// Service name (optional)
        service: Option<String>,
    },
    /// Testing operations (replaces menu items 6-9)
    Test {
        /// Test type (all, unit, integration, performance, security, validation)
        #[arg(default_value = "all")]
        test_type: String,
        /// Run in watch mode
        #[arg(short, long)]
        watch: bool,
    },
    /// Deployment operations (replaces menu items 10-12)
    Deploy {
        /// Environment (dev, staging, prod, k8s)
        #[arg(default_value = "dev")]
        environment: String,
        /// Force deployment
        #[arg(short, long)]
        force: bool,
    },
    /// Monitoring and observability (replaces menu item 11)
    Monitor {
        /// Monitor action (start, stop, status, logs)
        #[arg(default_value = "status")]
        action: String,
        /// Service to monitor
        service: Option<String>,
    },
    /// Tool operations (replaces menu items 13-18)
    Tool {
        /// Tool type (wasm, cargo, ai, vscode, nginx, vault)
        tool_type: String,
        /// Tool action
        action: String,
        /// Additional arguments
        args: Vec<String>,
    },
    /// Project initialization and management (replaces menu item 15)
    Init {
        /// Component type (service, frontend, tool, project)
        component_type: String,
        /// Name of the component
        name: String,
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Show comprehensive project status (replaces 'v' command)
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
        /// Check specific component
        component: Option<String>,
    },
    /// Configuration management
    Config {
        /// Operation (get, set, list, validate)
        operation: String,
        /// Configuration key
        key: Option<String>,
        /// Configuration value
        value: Option<String>,
    },
    /// Maintenance operations (replaces menu item 12)
    Clean {
        /// What to clean (all, build, cache, logs, temp)
        #[arg(default_value = "all")]
        target: String,
        /// Force clean without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Performance benchmarking
    Benchmark {
        /// Benchmark type (build, runtime, memory, wasm)
        bench_type: String,
        /// Number of iterations
        #[arg(short, long, default_value = "1")]
        iterations: u32,
    },
    /// Security operations
    Security {
        /// Security action (audit, scan, update, report)
        action: String,
        /// Severity level filter
        #[arg(short, long)]
        severity: Option<String>,
    },
    /// AI-powered operations (replaces menu item 17)
    Ai {
        /// AI operation (generate, optimize, analyze, chat)
        operation: String,
        /// Target for AI operation
        target: Option<String>,
        /// Additional context or prompt
        prompt: Option<String>,
    },
    /// Run arbitrary Makefile targets
    Make {
        /// Makefile target
        target: String,
        /// Additional make arguments
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize colored output
    if cli.no_color {
        colored::control::set_override(false);
    }

    match cli.command {
        Commands::Build { target, mode } => handle_build(target, mode).await,
        Commands::Dev { action, service } => handle_dev(action, service).await,
        Commands::Test { test_type, watch } => handle_test(test_type, watch).await,
        Commands::Deploy { environment, force } => handle_deploy(environment, force).await,
        Commands::Monitor { action, service } => handle_monitor(action, service).await,
        Commands::Tool {
            tool_type,
            action,
            args,
        } => handle_tool(tool_type, action, args).await,
        Commands::Init {
            component_type,
            name,
            template,
        } => handle_init(component_type, name, template).await,
        Commands::Status {
            detailed,
            component,
        } => handle_status(detailed, component).await,
        Commands::Config {
            operation,
            key,
            value,
        } => handle_config(operation, key, value).await,
        Commands::Clean { target, force } => handle_clean(target, force).await,
        Commands::Benchmark {
            bench_type,
            iterations,
        } => handle_benchmark(bench_type, iterations).await,
        Commands::Security { action, severity } => handle_security(action, severity).await,
        Commands::Ai {
            operation,
            target,
            prompt,
        } => handle_ai(operation, target, prompt).await,
        Commands::Make { target, args } => handle_make(target, args).await,
    }
}

async fn handle_build(target: String, mode: String) -> Result<()> {
    println!(
        "{}",
        format!("🔨 Building {} in {} mode", target, mode).bright_blue()
    );

    match target.as_str() {
        "all" => {
            run_bash_script("scripts/simpel.sh", &["build", "all"]).await?;
        }
        "frontend" | "frontends" => {
            run_bash_script("scripts/simpel.sh", &["build", "frontend"]).await?;
        }
        "backend" | "services" => {
            run_bash_script("scripts/simpel.sh", &["build", "backend"]).await?;
        }
        "clean" => {
            run_bash_script("scripts/simpel.sh", &["clean"]).await?;
        }
        specific => {
            // Try to build specific service or frontend
            if std::path::Path::new(&format!("layanan/{}", specific)).exists() {
                run_command(
                    "cargo",
                    &["build", "--bin", &format!("layanan-{}", specific)],
                )
                .await?;
            } else if std::path::Path::new(&format!("antarmuka/{}", specific)).exists() {
                run_command(
                    "trunk",
                    &[
                        "build",
                        "--config",
                        &format!("antarmuka/{}/Trunk.toml", specific),
                    ],
                )
                .await?;
            } else {
                println!("{}", format!("❌ Unknown build target: {}", specific).red());
                return Ok(());
            }
        }
    }

    println!("{}", "✅ Build completed successfully!".green());
    Ok(())
}

async fn handle_dev(action: String, service: Option<String>) -> Result<()> {
    match action.as_str() {
        "start" => {
            if let Some(svc) = service {
                println!(
                    "{}",
                    format!("🚀 Starting development for service: {}", svc).bright_blue()
                );
                run_command("cargo", &["run", "--bin", &format!("layanan-{}", svc)]).await?;
            } else {
                println!(
                    "{}",
                    "🚀 Starting full development environment".bright_blue()
                );
                run_bash_script("scripts/simpel.sh", &["dev"]).await?;
            }
        }
        "stop" => {
            println!("{}", "🛑 Stopping development environment".bright_blue());
            run_command("docker", &["compose", "down"]).await?;
        }
        "restart" => {
            println!("{}", "🔄 Restarting development environment".bright_blue());
            run_command("docker", &["compose", "restart"]).await?;
        }
        "logs" => {
            if let Some(svc) = service {
                run_command("docker", &["compose", "logs", "-f", &svc]).await?;
            } else {
                run_command("docker", &["compose", "logs", "-f"]).await?;
            }
        }
        _ => {
            println!("{}", format!("❌ Unknown dev action: {}", action).red());
        }
    }
    Ok(())
}

async fn handle_test(test_type: String, watch: bool) -> Result<()> {
    println!(
        "{}",
        format!("🧪 Running {} tests", test_type).bright_blue()
    );

    match test_type.as_str() {
        "all" => {
            run_bash_script("scripts/simpel.sh", &["test", "all"]).await?;
        }
        "unit" => {
            if watch {
                run_command("cargo", &["watch", "-x", "test", "--lib"]).await?;
            } else {
                run_command("cargo", &["test", "--lib"]).await?;
            }
        }
        "integration" => {
            run_bash_script("scripts/test/test-runner.sh", &["integration"]).await?;
        }
        "performance" => {
            run_bash_script("scripts/simpel.sh", &["benchmark"]).await?;
        }
        "security" => {
            run_bash_script("scripts/simpel.sh", &["test", "security"]).await?;
        }
        "validation" => {
            run_bash_script("scripts/simpel.sh", &["test", "validation"]).await?;
        }
        specific => {
            let args = vec!["test", "--package", specific];
            if watch {
                run_command("cargo", &["watch", "-x", "test", "--package", specific]).await?;
            } else {
                run_command("cargo", &args).await?;
            }
        }
    }

    println!("{}", "✅ Tests completed!".green());
    Ok(())
}

async fn handle_deploy(environment: String, force: bool) -> Result<()> {
    println!(
        "{}",
        format!("🚀 Deploying to {} environment", environment).bright_blue()
    );

    match environment.as_str() {
        "dev" | "development" => {
            run_bash_script("scripts/simpel.sh", &["deploy", "dev"]).await?;
        }
        "k8s" | "kubernetes" => {
            if force {
                run_bash_script("scripts/deploy-k8s.sh", &["--force"]).await?;
            } else {
                run_bash_script("scripts/deploy-k8s.sh", &[]).await?;
            }
        }
        "prod" | "production" => {
            if !force {
                println!(
                    "{}",
                    "⚠️  Production deployment requires --force flag".yellow()
                );
                return Ok(());
            }
            run_command(
                "docker",
                &["compose", "-f", "docker-compose.prod.yml", "up", "-d"],
            )
            .await?;
        }
        _ => {
            println!(
                "{}",
                format!("❌ Unknown environment: {}", environment).red()
            );
        }
    }

    println!("{}", "✅ Deployment completed!".green());
    Ok(())
}

async fn handle_monitor(action: String, service: Option<String>) -> Result<()> {
    match action.as_str() {
        "start" => {
            println!("{}", "📊 Starting monitoring stack".bright_blue());
            run_bash_script("scripts/simpel.sh", &["monitor"]).await?;
        }
        "stop" => {
            println!("{}", "🛑 Stopping monitoring".bright_blue());
            run_command(
                "docker",
                &["compose", "down", "prometheus", "grafana", "loki"],
            )
            .await?;
        }
        "status" => {
            println!("{}", "📈 Monitoring Status".bright_blue());
            show_monitoring_status().await?;
        }
        "logs" => {
            if let Some(svc) = service {
                run_command("docker", &["compose", "logs", "-f", &svc]).await?;
            } else {
                run_command("docker", &["compose", "logs", "-f"]).await?;
            }
        }
        _ => {
            println!("{}", format!("❌ Unknown monitor action: {}", action).red());
        }
    }
    Ok(())
}

async fn handle_tool(tool_type: String, action: String, args: Vec<String>) -> Result<()> {
    match tool_type.as_str() {
        "wasm" => {
            let mut script_args = vec![action.as_str()];
            let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            script_args.extend(arg_strs);
            run_bash_script("scripts/tools/wasm-optimizer.sh", &script_args).await?;
        }
        "cargo" => {
            let mut script_args = vec![action.as_str()];
            let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            script_args.extend(arg_strs);
            run_bash_script("scripts/tools/cargo-maintenance.sh", &script_args).await?;
        }
        "ai" => {
            let mut script_args = vec![action.as_str()];
            let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            script_args.extend(arg_strs);
            run_bash_script("scripts/tools/ai/ai-tools.sh", &script_args).await?;
        }
        "nginx" => {
            let mut script_args = vec![action.as_str()];
            let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            script_args.extend(arg_strs);
            run_bash_script("scripts/tools/nginx-manager.sh", &script_args).await?;
        }
        "vault" => {
            let mut script_args = vec![action.as_str()];
            let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            script_args.extend(arg_strs);
            run_bash_script("scripts/tools/vault/vault-manager.sh", &script_args).await?;
        }
        "vscode" => {
            println!("{}", "🔧 VS Code validation".bright_blue());
            run_bash_script("scripts/simpel.sh", &["vscode"]).await?;
        }
        _ => {
            println!("{}", format!("❌ Unknown tool type: {}", tool_type).red());
        }
    }
    Ok(())
}

async fn handle_init(component_type: String, name: String, template: Option<String>) -> Result<()> {
    println!(
        "{}",
        format!("🚀 Initializing new {}: {}", component_type, name).bright_blue()
    );

    let mut args = vec![component_type.as_str(), name.as_str()];
    if let Some(tmpl) = &template {
        args.push(tmpl.as_str());
    }

    run_bash_script("scripts/tools/project-init.sh", &args).await?;
    println!(
        "{}",
        format!("✅ Component {} created successfully!", name).green()
    );
    Ok(())
}

async fn handle_status(detailed: bool, component: Option<String>) -> Result<()> {
    println!("{}", "📋 SIMPelv2 Status Information".bright_blue());
    println!("==================================");

    if detailed {
        run_bash_script("scripts/simpel.sh", &["version"]).await?;
    } else {
        show_basic_status().await?;
    }

    if let Some(comp) = component {
        show_component_status(&comp).await?;
    }

    Ok(())
}

async fn handle_config(
    operation: String,
    key: Option<String>,
    value: Option<String>,
) -> Result<()> {
    match operation.as_str() {
        "get" => {
            if let Some(k) = key {
                show_config_value(&k).await?;
            } else {
                show_all_config().await?;
            }
        }
        "set" => {
            if let (Some(k), Some(v)) = (key, value) {
                set_config_value(&k, &v).await?;
            } else {
                println!(
                    "{}",
                    "❌ Both key and value required for set operation".red()
                );
            }
        }
        "list" => {
            show_all_config().await?;
        }
        "validate" => {
            validate_config().await?;
        }
        _ => {
            println!(
                "{}",
                format!("❌ Unknown config operation: {}", operation).red()
            );
        }
    }
    Ok(())
}

async fn handle_clean(target: String, force: bool) -> Result<()> {
    println!("{}", format!("🧹 Cleaning {}", target).bright_blue());

    if !force {
        println!(
            "{}",
            "⚠️  This will delete build artifacts. Use --force to confirm.".yellow()
        );
        return Ok(());
    }

    match target.as_str() {
        "all" => {
            run_bash_script("scripts/simpel.sh", &["clean"]).await?;
        }
        "build" => {
            run_command("cargo", &["clean"]).await?;
        }
        "cache" => {
            run_command("rm", &["-rf", ".cargo/", "target/debug/incremental/"]).await?;
        }
        "logs" => {
            run_command("rm", &["-rf", "logs/", "*.log"]).await?;
        }
        "temp" => {
            run_command("rm", &["-rf", "/tmp/simpelv2*"]).await?;
        }
        _ => {
            println!("{}", format!("❌ Unknown clean target: {}", target).red());
        }
    }

    println!("{}", "✅ Cleanup completed!".green());
    Ok(())
}

async fn handle_benchmark(bench_type: String, iterations: u32) -> Result<()> {
    println!(
        "{}",
        format!(
            "⚡ Running {} benchmark ({} iterations)",
            bench_type, iterations
        )
        .bright_blue()
    );

    match bench_type.as_str() {
        "build" => {
            run_bash_script(
                "scripts/test/performance/build-benchmark.sh",
                &[&iterations.to_string()],
            )
            .await?;
        }
        "runtime" => {
            run_bash_script(
                "scripts/test/performance/runtime-benchmark.sh",
                &[&iterations.to_string()],
            )
            .await?;
        }
        "memory" => {
            run_bash_script(
                "scripts/test/performance/memory-benchmark.sh",
                &[&iterations.to_string()],
            )
            .await?;
        }
        "wasm" => {
            run_bash_script(
                "scripts/test/performance/wasm-benchmark.sh",
                &[&iterations.to_string()],
            )
            .await?;
        }
        _ => {
            println!(
                "{}",
                format!("❌ Unknown benchmark type: {}", bench_type).red()
            );
        }
    }

    Ok(())
}

async fn handle_security(action: String, severity: Option<String>) -> Result<()> {
    println!(
        "{}",
        format!("🔒 Security operation: {}", action).bright_blue()
    );

    match action.as_str() {
        "audit" => {
            run_command("cargo", &["audit"]).await?;
        }
        "scan" => {
            run_bash_script("scripts/security-optimizer.sh", &["scan"]).await?;
        }
        "update" => {
            run_command("cargo", &["update"]).await?;
            run_command("cargo", &["audit", "fix"]).await?;
        }
        "report" => {
            if let Some(sev) = &severity {
                run_bash_script(
                    "scripts/test/security/security-report.sh",
                    &[&action, "--severity", sev],
                )
                .await?;
            } else {
                run_bash_script("scripts/test/security/security-report.sh", &[&action]).await?;
            }
        }
        _ => {
            println!(
                "{}",
                format!("❌ Unknown security action: {}", action).red()
            );
        }
    }

    Ok(())
}

async fn handle_ai(
    operation: String,
    target: Option<String>,
    prompt: Option<String>,
) -> Result<()> {
    println!(
        "{}",
        format!("🤖 AI operation: {}", operation).bright_blue()
    );

    let mut args = vec![operation.as_str()];
    if let Some(tgt) = &target {
        args.push(tgt.as_str());
    }
    if let Some(prmt) = &prompt {
        args.push(prmt.as_str());
    }

    run_bash_script("scripts/tools/ai/ai-tools.sh", &args).await?;
    Ok(())
}

async fn handle_make(target: String, args: Vec<String>) -> Result<()> {
    println!(
        "{}",
        format!("🔧 Running make target: {}", target).bright_blue()
    );

    let mut make_args = vec![target.as_str()];
    make_args.extend(args.iter().map(|s| s.as_str()));

    run_command("make", &make_args).await?;
    Ok(())
}

// Helper functions
async fn run_bash_script(script_path: &str, args: &[&str]) -> Result<()> {
    let mut cmd = tokio::process::Command::new("bash");
    cmd.arg(script_path);
    cmd.args(args);

    let status = cmd.status().await?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Script failed: {} {}",
            script_path,
            args.join(" ")
        ));
    }

    Ok(())
}

async fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = tokio::process::Command::new(cmd)
        .args(args)
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Command failed: {} {}",
            cmd,
            args.join(" ")
        ));
    }

    Ok(())
}

async fn show_basic_status() -> Result<()> {
    println!("{}", "🔍 System Check:".bright_blue());

    // Check Cargo
    if run_command("cargo", &["version"]).await.is_ok() {
        println!("  ✅ Cargo available");
    } else {
        println!("  ❌ Cargo not found");
    }

    // Check Docker
    if run_command("docker", &["version"]).await.is_ok() {
        println!("  ✅ Docker available");
    } else {
        println!("  ❌ Docker not found");
    }

    // Check Trunk
    if run_command("trunk", &["--version"]).await.is_ok() {
        println!("  ✅ Trunk available");
    } else {
        println!("  ⚠️  Trunk not found");
    }

    // Check if in workspace
    if std::path::Path::new("Cargo.toml").exists() {
        println!("  ✅ In SIMPelv2 workspace");

        // Count services and frontends
        let services = count_directories("layanan")?;
        let frontends = count_directories("antarmuka")?;

        println!("🔧 Backend Services: {}", services);
        println!("🎨 Frontend Apps: {}", frontends);
    } else {
        println!("  ⚠️  Not in SIMPelv2 workspace");
    }

    Ok(())
}

async fn show_monitoring_status() -> Result<()> {
    println!("📊 Monitoring Services:");

    // Check if monitoring containers are running
    let services = ["prometheus", "grafana", "loki", "tempo"];

    for service in &services {
        if check_container_running(service).await {
            println!("  ✅ {} is running", service);
        } else {
            println!("  ❌ {} is not running", service);
        }
    }

    Ok(())
}

async fn check_container_running(container: &str) -> bool {
    tokio::process::Command::new("docker")
        .args(&["ps", "-q", "-f", &format!("name={}", container)])
        .output()
        .await
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}

async fn show_component_status(component: &str) -> Result<()> {
    println!(
        "{}",
        format!("📊 Status for component: {}", component).bright_blue()
    );

    // Check if it's a service
    let service_path = format!("layanan/{}", component);
    if std::path::Path::new(&service_path).exists() {
        println!("  📦 Type: Backend Service");
        println!("  📁 Path: {}", service_path);

        // Check if service has a Dockerfile
        if std::path::Path::new(&format!("{}/Dockerfile", service_path)).exists() {
            println!("  🐳 Docker: Available");
        }

        // Check if service has tests
        if std::path::Path::new(&format!("{}/src", service_path)).exists() {
            println!("  🧪 Source: Available");
        }
    }

    // Check if it's a frontend
    let frontend_path = format!("antarmuka/{}", component);
    if std::path::Path::new(&frontend_path).exists() {
        println!("  📦 Type: Frontend Application");
        println!("  📁 Path: {}", frontend_path);

        // Check if frontend has Trunk.toml
        if std::path::Path::new(&format!("{}/Trunk.toml", frontend_path)).exists() {
            println!("  🌐 Trunk: Configured");
        }
    }

    Ok(())
}

async fn show_all_config() -> Result<()> {
    println!("{}", "⚙️  Current Configuration:".bright_blue());

    // Show environment variables
    if let Ok(workspace) = std::env::var("WORKSPACE_ROOT") {
        println!("  WORKSPACE_ROOT: {}", workspace);
    }

    // Show Cargo workspace info
    if let Ok(config) = std::fs::read_to_string("Cargo.toml") {
        if config.lines().any(|line| line.contains("[workspace]")) {
            println!("  ✅ Cargo workspace configured");
        }
    }

    // Show Docker Compose files
    if std::path::Path::new("docker-compose.yml").exists() {
        println!("  ✅ Docker Compose: docker-compose.yml");
    }
    if std::path::Path::new("docker-compose.dev.yml").exists() {
        println!("  ✅ Docker Compose Dev: docker-compose.dev.yml");
    }
    if std::path::Path::new("docker-compose.prod.yml").exists() {
        println!("  ✅ Docker Compose Prod: docker-compose.prod.yml");
    }

    Ok(())
}

async fn show_config_value(key: &str) -> Result<()> {
    match key {
        "workspace" => {
            if let Ok(workspace) = std::env::var("WORKSPACE_ROOT") {
                println!("WORKSPACE_ROOT: {}", workspace);
            } else {
                println!("WORKSPACE_ROOT: /var/www/simpelv2 (default)");
            }
        }
        "version" => {
            println!("SIMPelv2 CLI Version: 2.0.0");
        }
        _ => {
            println!("{}", format!("❌ Unknown config key: {}", key).red());
        }
    }
    Ok(())
}

async fn set_config_value(key: &str, value: &str) -> Result<()> {
    println!(
        "{}",
        format!("⚙️  Setting {} = {}", key, value).bright_blue()
    );

    match key {
        "workspace" => {
            // In a real implementation, you'd update a config file
            println!(
                "{}",
                "⚠️  Configuration updates not yet implemented".yellow()
            );
        }
        _ => {
            println!("{}", format!("❌ Unknown config key: {}", key).red());
        }
    }

    Ok(())
}

async fn validate_config() -> Result<()> {
    println!("{}", "🔍 Validating configuration...".bright_blue());

    let mut errors = 0;

    // Check if in workspace
    if !std::path::Path::new("Cargo.toml").exists() {
        println!("  ❌ No Cargo.toml found - not in a Rust workspace");
        errors += 1;
    } else {
        println!("  ✅ Cargo.toml found");
    }

    // Check Docker Compose
    if !std::path::Path::new("docker-compose.yml").exists() {
        println!("  ❌ No docker-compose.yml found");
        errors += 1;
    } else {
        println!("  ✅ docker-compose.yml found");
    }

    // Check key directories
    let dirs = ["layanan", "antarmuka", "scripts", "infra"];
    for dir in &dirs {
        if std::path::Path::new(dir).exists() {
            println!("  ✅ Directory {} exists", dir);
        } else {
            println!("  ⚠️  Directory {} missing", dir);
        }
    }

    if errors == 0 {
        println!("{}", "✅ Configuration is valid!".green());
    } else {
        println!(
            "{}",
            format!("❌ Found {} configuration errors", errors).red()
        );
    }

    Ok(())
}

fn count_directories(path: &str) -> Result<usize> {
    if !std::path::Path::new(path).exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .count();

    Ok(count)
}
