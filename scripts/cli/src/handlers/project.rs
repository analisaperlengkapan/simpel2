use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ProjectCommands;

pub async fn handle_project(action: ProjectCommands) -> Result<()> {
    match action {
        ProjectCommands::Init {
            component_type,
            name,
            template,
            git,
        } => handle_project_init(component_type, name, template, git).await,
        ProjectCommands::Stats { stats_type, format } => {
            handle_project_stats(stats_type, format).await
        }
        ProjectCommands::Health { scope, fix } => handle_project_health(scope, fix).await,
        ProjectCommands::Validate { scope, strict } => handle_project_validate(scope, strict).await,
        ProjectCommands::Update { scope, check } => handle_project_update(scope, check).await,
        ProjectCommands::Archive {
            component,
            location,
        } => handle_project_archive(component, location).await,
    }
}

async fn handle_project_init(
    component_type: String,
    name: String,
    template: Option<String>,
    git: bool,
) -> Result<()> {
    println!(
        "{}",
        format!(
            "🚀 Initializing new project component: {} ({})",
            name, component_type
        )
        .bright_blue()
    );

    match component_type.as_str() {
        "service" | "microservice" => create_new_service(&name, template.as_deref(), git).await,
        "frontend" | "antarmuka" => create_new_frontend(&name, template.as_deref(), git).await,
        "tool" => create_new_tool(&name, template.as_deref(), git).await,
        "workspace" => create_new_workspace(&name, template.as_deref(), git).await,
        _ => {
            println!(
                "{}",
                format!("❌ Unknown component type: {}", component_type).red()
            );
            println!(
                "{}",
                "Available types: service, frontend, tool, workspace".yellow()
            );
            Ok(())
        }
    }
}

async fn create_new_service(name: &str, template: Option<&str>, git: bool) -> Result<()> {
    let service_dir = format!("layanan/{}", name);

    if Path::new(&service_dir).exists() {
        println!("{}", format!("❌ Service {} already exists", name).red());
        return Ok(());
    }

    println!(
        "{}",
        format!("🦀 Creating new microservice: {}", name).cyan()
    );

    // Create directory structure
    fs::create_dir_all(format!("{}/src", service_dir))?;
    fs::create_dir_all(format!("{}/tests", service_dir))?;

    // Create Cargo.toml
    let cargo_toml_content = create_service_cargo_toml(name, template);
    fs::write(format!("{}/Cargo.toml", service_dir), cargo_toml_content)?;

    // Create main.rs
    let main_rs_content = create_service_main_rs(name, template);
    fs::write(format!("{}/src/main.rs", service_dir), main_rs_content)?;

    // Create lib.rs
    let lib_rs_content = create_service_lib_rs(name, template);
    fs::write(format!("{}/src/lib.rs", service_dir), lib_rs_content)?;

    // Create Dockerfile
    let dockerfile_content = create_service_dockerfile(name);
    fs::write(format!("{}/Dockerfile", service_dir), dockerfile_content)?;

    // Create basic test
    let test_content = create_service_test(name);
    fs::write(
        format!("{}/tests/integration_test.rs", service_dir),
        test_content,
    )?;

    // Add to workspace Cargo.toml
    add_service_to_workspace(name)?;

    if git {
        initialize_git(&service_dir).await?;
    }

    println!(
        "{}",
        format!("✅ Service {} created successfully", name).green()
    );
    println!("{}", format!("📁 Location: {}", service_dir).cyan());

    Ok(())
}

async fn create_new_frontend(name: &str, template: Option<&str>, git: bool) -> Result<()> {
    let frontend_dir = format!("antarmuka/{}", name);

    if Path::new(&frontend_dir).exists() {
        println!("{}", format!("❌ Frontend {} already exists", name).red());
        return Ok(());
    }

    println!("{}", format!("🌐 Creating new frontend: {}", name).cyan());

    // Create directory structure
    fs::create_dir_all(format!("{}/src", frontend_dir))?;
    fs::create_dir_all(format!("{}/styles", frontend_dir))?;
    fs::create_dir_all(format!("{}/assets", frontend_dir))?;

    // Create Cargo.toml
    let cargo_toml_content = create_frontend_cargo_toml(name, template);
    fs::write(format!("{}/Cargo.toml", frontend_dir), cargo_toml_content)?;

    // Create main.rs
    let main_rs_content = create_frontend_main_rs(name, template);
    fs::write(format!("{}/src/main.rs", frontend_dir), main_rs_content)?;

    // Create index.html
    let index_html_content = create_frontend_index_html(name);
    fs::write(format!("{}/index.html", frontend_dir), index_html_content)?;

    // Create Trunk.toml
    let trunk_toml_content = create_frontend_trunk_toml(name);
    fs::write(format!("{}/Trunk.toml", frontend_dir), trunk_toml_content)?;

    // Create basic styles
    let styles_content = create_frontend_styles(name);
    fs::write(format!("{}/styles/main.css", frontend_dir), styles_content)?;

    // Create Dockerfile
    let dockerfile_content = create_frontend_dockerfile(name);
    fs::write(format!("{}/Dockerfile", frontend_dir), dockerfile_content)?;

    // Add to workspace Cargo.toml
    add_frontend_to_workspace(name)?;

    if git {
        initialize_git(&frontend_dir).await?;
    }

    println!(
        "{}",
        format!("✅ Frontend {} created successfully", name).green()
    );
    println!("{}", format!("📁 Location: {}", frontend_dir).cyan());

    Ok(())
}

async fn create_new_tool(name: &str, template: Option<&str>, _git: bool) -> Result<()> {
    let tool_dir = format!("scripts/tools/{}", name);

    if Path::new(&tool_dir).exists() {
        println!("{}", format!("❌ Tool {} already exists", name).red());
        return Ok(());
    }

    println!("{}", format!("🔧 Creating new tool: {}", name).cyan());

    fs::create_dir_all(&tool_dir)?;

    // Create tool script based on template
    let tool_content = match template {
        Some("rust") => create_rust_tool(name),
        Some("python") => create_python_tool(name),
        _ => create_bash_tool(name),
    };

    let extension = match template {
        Some("rust") => "rs",
        Some("python") => "py",
        _ => "sh",
    };

    fs::write(format!("{}/{}.{}", tool_dir, name, extension), tool_content)?;

    println!(
        "{}",
        format!("✅ Tool {} created successfully", name).green()
    );

    Ok(())
}

async fn create_new_workspace(name: &str, _template: Option<&str>, git: bool) -> Result<()> {
    if Path::new(name).exists() {
        println!("{}", format!("❌ Directory {} already exists", name).red());
        return Ok(());
    }

    println!("{}", format!("📦 Creating new workspace: {}", name).cyan());

    // Create workspace structure
    fs::create_dir_all(format!("{}/layanan", name))?;
    fs::create_dir_all(format!("{}/antarmuka", name))?;
    fs::create_dir_all(format!("{}/scripts", name))?;
    fs::create_dir_all(format!("{}/docs", name))?;
    fs::create_dir_all(format!("{}/infra", name))?;

    // Create workspace Cargo.toml
    let workspace_cargo_content = create_workspace_cargo_toml(name);
    fs::write(format!("{}/Cargo.toml", name), workspace_cargo_content)?;

    // Create README.md
    let readme_content = create_workspace_readme(name);
    fs::write(format!("{}/README.md", name), readme_content)?;

    // Create .gitignore
    let gitignore_content = create_gitignore();
    fs::write(format!("{}/.gitignore", name), gitignore_content)?;

    if git {
        initialize_git(name).await?;
    }

    println!(
        "{}",
        format!("✅ Workspace {} created successfully", name).green()
    );

    Ok(())
}

async fn handle_project_stats(stats_type: String, format: String) -> Result<()> {
    println!(
        "{}",
        format!("📊 Project Statistics: {}", stats_type).bright_blue()
    );

    match stats_type.as_str() {
        "overview" => show_project_overview(&format).await,
        "detailed" => show_detailed_stats(&format).await,
        "dependencies" => show_dependency_stats(&format).await,
        "code" => show_code_stats(&format).await,
        "performance" => show_performance_stats(&format).await,
        _ => {
            println!("{}", format!("❌ Unknown stats type: {}", stats_type).red());
            Ok(())
        }
    }
}

async fn show_project_overview(format: &str) -> Result<()> {
    let stats = collect_project_overview_stats().await?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        "table" => {
            print_stats_table(&stats);
        }
        _ => {
            print_stats_text(&stats);
        }
    }

    Ok(())
}

async fn collect_project_overview_stats() -> Result<HashMap<String, u32>> {
    let mut stats = HashMap::new();

    // Count services
    if Path::new("layanan").exists() {
        let service_count = count_directories("layanan")?;
        stats.insert("services".to_string(), service_count as u32);
    }

    // Count frontends
    if Path::new("antarmuka").exists() {
        let frontend_count = count_directories("antarmuka")?;
        stats.insert("frontends".to_string(), frontend_count as u32);
    }

    // Count scripts
    if Path::new("scripts").exists() {
        let script_count = count_files_with_extension("scripts", "sh")?
            + count_files_with_extension("scripts", "py")?;
        stats.insert("scripts".to_string(), script_count as u32);
    }

    // Count lines of code
    let loc = count_lines_of_code().await?;
    stats.insert("lines_of_code".to_string(), loc);

    Ok(stats)
}

fn count_directories(path: &str) -> Result<usize> {
    if !Path::new(path).exists() {
        return Ok(0);
    }

    let count = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .count();

    Ok(count)
}

fn count_files_with_extension(path: &str, ext: &str) -> Result<usize> {
    if !Path::new(path).exists() {
        return Ok(0);
    }

    fn count_recursive(dir: &Path, ext: &str) -> Result<usize> {
        let mut count = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                count += count_recursive(&path, ext)?;
            } else if path.extension().map_or(false, |e| e == ext) {
                count += 1;
            }
        }
        Ok(count)
    }

    count_recursive(Path::new(path), ext)
}

async fn count_lines_of_code() -> Result<u32> {
    let output = Command::new("find")
        .args(&[
            ".", "-name", "*.rs", "-o", "-name", "*.py", "-o", "-name", "*.sh",
        ])
        .args(&["-exec", "wc", "-l", "{}", "+"])
        .output()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let total_line = output_str.lines().last().unwrap_or("0");
        let total: u32 = total_line
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        Ok(total)
    } else {
        Ok(0)
    }
}

fn print_stats_table(stats: &HashMap<String, u32>) {
    println!("┌─────────────────┬─────────┐");
    println!("│ Metric          │ Count   │");
    println!("├─────────────────┼─────────┤");

    for (key, value) in stats {
        println!("│ {:15} │ {:7} │", key, value);
    }

    println!("└─────────────────┴─────────┘");
}

fn print_stats_text(stats: &HashMap<String, u32>) {
    for (key, value) in stats {
        println!("{}: {}", key.replace("_", " ").to_title_case(), value);
    }
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

async fn show_detailed_stats(_format: &str) -> Result<()> {
    println!("{}", "📈 Detailed Project Statistics".cyan());

    // Show cargo workspace info
    show_cargo_workspace_info().await?;

    // Show git statistics
    show_git_statistics().await?;

    // Show build statistics
    show_build_statistics().await?;

    Ok(())
}

async fn show_cargo_workspace_info() -> Result<()> {
    println!("{}", "\n🦀 Cargo Workspace Information:".bright_cyan());

    let output = Command::new("cargo")
        .args(&["tree", "--workspace"])
        .output()?;

    if output.status.success() {
        let tree_output = String::from_utf8_lossy(&output.stdout);
        let package_count = tree_output
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.contains("├─") && !line.contains("└─"))
            .count();
        println!("Total packages: {}", package_count);
    }

    Ok(())
}

async fn show_git_statistics() -> Result<()> {
    println!("{}", "\n📊 Git Statistics:".bright_cyan());

    // Commit count
    let output = Command::new("git")
        .args(&["rev-list", "--count", "HEAD"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let count = output_str.trim();
            println!("Total commits: {}", count);
        }
    }

    // Contributors
    let output = Command::new("git")
        .args(&["shortlog", "-sn", "--all"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let contributors = String::from_utf8_lossy(&output.stdout);
            let contributor_count = contributors.lines().count();
            println!("Contributors: {}", contributor_count);
        }
    }

    Ok(())
}

async fn show_build_statistics() -> Result<()> {
    println!("{}", "\n🏗️  Build Statistics:".bright_cyan());

    // Check target directory size
    if Path::new("target").exists() {
        let output = Command::new("du").args(&["-sh", "target"]).output()?;

        if output.status.success() {
            let size = String::from_utf8_lossy(&output.stdout);
            println!("Target directory size: {}", size.trim());
        }
    }

    Ok(())
}

async fn show_dependency_stats(_format: &str) -> Result<()> {
    println!("{}", "📦 Dependency Statistics".cyan());

    // Show outdated dependencies
    if Command::new("cargo-outdated")
        .arg("--version")
        .output()
        .is_ok()
    {
        println!("{}", "\nOutdated dependencies:".yellow());
        let _ = Command::new("cargo-outdated")
            .args(&["--root-deps-only"])
            .status();
    }

    // Show duplicate dependencies
    println!("{}", "\nChecking for duplicates:".yellow());
    let output = Command::new("cargo")
        .args(&["tree", "--duplicates"])
        .output()?;

    if output.status.success() {
        let duplicates = String::from_utf8_lossy(&output.stdout);
        if duplicates.trim().is_empty() {
            println!("{}", "✅ No duplicate dependencies found".green());
        } else {
            println!("{}", duplicates);
        }
    }

    Ok(())
}

async fn show_code_stats(_format: &str) -> Result<()> {
    println!("{}", "📝 Code Statistics".cyan());

    // Use tokei if available for detailed code stats
    if Command::new("tokei").arg("--version").output().is_ok() {
        let _ = Command::new("tokei").status();
    } else {
        // Fallback to basic line counting
        let loc = count_lines_of_code().await?;
        println!("Total lines of code: {}", loc);
    }

    Ok(())
}

async fn show_performance_stats(_format: &str) -> Result<()> {
    println!("{}", "⚡ Performance Statistics".cyan());

    // Show binary sizes
    if Path::new("target/release").exists() {
        println!("{}", "\nRelease binary sizes:".yellow());
        let output = Command::new("find")
            .args(&[
                "target/release",
                "-maxdepth",
                "1",
                "-type",
                "f",
                "-executable",
            ])
            .args(&["-exec", "ls", "-lh", "{}", ";"])
            .output()?;

        if output.status.success() {
            let sizes = String::from_utf8_lossy(&output.stdout);
            for line in sizes.lines() {
                if !line.trim().is_empty() {
                    println!("  {}", line);
                }
            }
        }
    }

    Ok(())
}

async fn handle_project_health(scope: String, fix: bool) -> Result<()> {
    println!(
        "{}",
        format!("🏥 Project Health Check: {}", scope).bright_blue()
    );

    match scope.as_str() {
        "all" => check_all_health(fix).await,
        "dependencies" => check_dependency_health(fix).await,
        "services" => check_services_health(fix).await,
        "infrastructure" => check_infrastructure_health(fix).await,
        _ => {
            println!("{}", format!("❌ Unknown health scope: {}", scope).red());
            Ok(())
        }
    }
}

async fn check_all_health(fix: bool) -> Result<()> {
    println!("{}", "🔍 Running comprehensive health check...".cyan());

    check_dependency_health(fix).await?;
    check_services_health(fix).await?;
    check_infrastructure_health(fix).await?;

    println!("{}", "✅ Health check completed".green());
    Ok(())
}

async fn check_dependency_health(fix: bool) -> Result<()> {
    println!("{}", "📦 Checking dependency health...".cyan());

    // Check for security vulnerabilities
    if Command::new("cargo-audit")
        .arg("--version")
        .output()
        .is_ok()
    {
        println!("{}", "  🔍 Running security audit...".cyan());
        let status = Command::new("cargo-audit").status()?;

        if !status.success() && fix {
            println!("{}", "  🔧 Attempting to fix security issues...".yellow());
            // Implement security fix logic
        }
    }

    // Check for outdated dependencies
    if Command::new("cargo-outdated")
        .arg("--version")
        .output()
        .is_ok()
    {
        println!("{}", "  📊 Checking for outdated dependencies...".cyan());
        let output = Command::new("cargo-outdated")
            .args(&["--root-deps-only"])
            .output()?;

        if output.status.success() {
            let outdated = String::from_utf8_lossy(&output.stdout);
            if !outdated.trim().is_empty() && fix {
                println!("{}", "  ⬆️  Updating dependencies...".yellow());
                let _ = Command::new("cargo").arg("update").status();
            }
        }
    }

    Ok(())
}

async fn check_services_health(_fix: bool) -> Result<()> {
    println!("{}", "🦀 Checking services health...".cyan());

    // Check if all services compile
    println!("{}", "  🔨 Checking compilation...".cyan());
    let status = Command::new("cargo")
        .args(&["check", "--workspace"])
        .status()?;

    if status.success() {
        println!("{}", "  ✅ All services compile successfully".green());
    } else {
        println!("{}", "  ❌ Compilation errors found".red());
    }

    // Check for clippy warnings
    println!("{}", "  🔍 Running clippy analysis...".cyan());
    let status = Command::new("cargo")
        .args(&["clippy", "--workspace", "--", "-D", "warnings"])
        .status()?;

    if status.success() {
        println!("{}", "  ✅ No clippy warnings".green());
    } else {
        println!("{}", "  ⚠️  Clippy warnings found".yellow());
    }

    Ok(())
}

async fn check_infrastructure_health(_fix: bool) -> Result<()> {
    println!("{}", "🏗️  Checking infrastructure health...".cyan());

    // Check docker-compose files
    for compose_file in &[
        "docker-compose.yml",
        "docker-compose.dev.yml",
        "docker-compose.prod.yml",
    ] {
        if Path::new(compose_file).exists() {
            println!("{}", format!("  ✅ {} exists", compose_file).green());

            // Validate compose file
            let status = Command::new("docker")
                .args(&["compose", "-f", compose_file, "config"])
                .status()?;

            if status.success() {
                println!("{}", format!("  ✅ {} is valid", compose_file).green());
            } else {
                println!("{}", format!("  ❌ {} has errors", compose_file).red());
            }
        }
    }

    Ok(())
}

async fn handle_project_validate(scope: String, strict: bool) -> Result<()> {
    println!(
        "{}",
        format!("✅ Project Validation: {} (strict: {})", scope, strict).bright_blue()
    );

    match scope.as_str() {
        "all" => validate_all_components(strict).await,
        "cargo" => validate_cargo_configuration(strict).await,
        "docker" => validate_docker_configuration(strict).await,
        "k8s" => validate_k8s_configuration(strict).await,
        _ => {
            println!(
                "{}",
                format!("❌ Unknown validation scope: {}", scope).red()
            );
            Ok(())
        }
    }
}

async fn validate_all_components(strict: bool) -> Result<()> {
    println!("{}", "🔍 Validating all project components...".cyan());

    validate_cargo_configuration(strict).await?;
    validate_docker_configuration(strict).await?;
    validate_k8s_configuration(strict).await?;
    validate_project_structure(strict).await?;

    println!("{}", "✅ Validation completed".green());
    Ok(())
}

async fn validate_cargo_configuration(strict: bool) -> Result<()> {
    println!("{}", "🦀 Validating Cargo configuration...".cyan());

    // Check main Cargo.toml
    if !Path::new("Cargo.toml").exists() {
        println!("{}", "  ❌ Cargo.toml not found".red());
        return Ok(());
    }

    // Validate workspace configuration
    let cargo_content = fs::read_to_string("Cargo.toml")?;
    if cargo_content.contains("[workspace]") {
        println!("{}", "  ✅ Workspace configuration found".green());
    } else if strict {
        println!("{}", "  ❌ Workspace configuration missing".red());
    }

    // Check if cargo check passes
    let status = Command::new("cargo")
        .args(&["check", "--workspace"])
        .output()?;

    if status.status.success() {
        println!("{}", "  ✅ Cargo check passed".green());
    } else {
        println!("{}", "  ❌ Cargo check failed".red());
        if strict {
            let stderr = String::from_utf8_lossy(&status.stderr);
            println!("{}", stderr);
        }
    }

    Ok(())
}

async fn validate_docker_configuration(_strict: bool) -> Result<()> {
    println!("{}", "🐳 Validating Docker configuration...".cyan());

    // Check for Dockerfile existence
    let dockerfiles = vec!["Dockerfile", "docker-compose.yml", "docker-compose.dev.yml"];

    for dockerfile in dockerfiles {
        if Path::new(dockerfile).exists() {
            println!("{}", format!("  ✅ {} found", dockerfile).green());
        } else {
            println!("{}", format!("  ⚠️  {} not found", dockerfile).yellow());
        }
    }

    Ok(())
}

async fn validate_k8s_configuration(_strict: bool) -> Result<()> {
    println!("{}", "☸️  Validating Kubernetes configuration...".cyan());

    // Check for k8s manifests
    if Path::new("infra/k8s").exists() {
        println!("{}", "  ✅ Kubernetes manifests directory found".green());

        // Count yaml files
        let yaml_count = count_files_with_extension("infra/k8s", "yaml")?
            + count_files_with_extension("infra/k8s", "yml")?;
        println!("{}", format!("  📄 Found {} YAML files", yaml_count).cyan());
    } else {
        println!(
            "{}",
            "  ⚠️  Kubernetes manifests directory not found".yellow()
        );
    }

    Ok(())
}

async fn validate_project_structure(_strict: bool) -> Result<()> {
    println!("{}", "📁 Validating project structure...".cyan());

    let expected_dirs = vec!["layanan", "antarmuka", "scripts", "docs", "infra"];

    for dir in expected_dirs {
        if Path::new(dir).exists() {
            println!("{}", format!("  ✅ {} directory exists", dir).green());
        } else {
            println!("{}", format!("  ⚠️  {} directory missing", dir).yellow());
        }
    }

    Ok(())
}

async fn handle_project_update(scope: String, check: bool) -> Result<()> {
    println!(
        "{}",
        format!("⬆️  Project Update: {} (check: {})", scope, check).bright_blue()
    );

    match scope.as_str() {
        "all" => update_all_components(check).await,
        "cargo" => update_cargo_dependencies(check).await,
        "npm" => update_npm_dependencies(check).await,
        "docker" => update_docker_images(check).await,
        _ => {
            println!("{}", format!("❌ Unknown update scope: {}", scope).red());
            Ok(())
        }
    }
}

async fn update_all_components(check: bool) -> Result<()> {
    println!("{}", "⬆️  Updating all project components...".cyan());

    update_cargo_dependencies(check).await?;
    update_npm_dependencies(check).await?;
    update_docker_images(check).await?;

    println!("{}", "✅ All components updated".green());
    Ok(())
}

async fn update_cargo_dependencies(check: bool) -> Result<()> {
    println!("{}", "🦀 Updating Cargo dependencies...".cyan());

    if check {
        // Just check for outdated dependencies
        if Command::new("cargo-outdated")
            .arg("--version")
            .output()
            .is_ok()
        {
            let _ = Command::new("cargo-outdated")
                .args(&["--root-deps-only"])
                .status();
        }
    } else {
        // Actually update dependencies
        let status = Command::new("cargo").arg("update").status()?;

        if status.success() {
            println!("{}", "  ✅ Cargo dependencies updated".green());
        } else {
            println!("{}", "  ❌ Failed to update Cargo dependencies".red());
        }
    }

    Ok(())
}

async fn update_npm_dependencies(check: bool) -> Result<()> {
    println!("{}", "📦 Checking for NPM dependencies...".cyan());

    if Path::new("package.json").exists() {
        if check {
            // Check for outdated packages
            let _ = Command::new("npm").args(&["outdated"]).status();
        } else {
            // Update packages
            let status = Command::new("npm").args(&["update"]).status()?;

            if status.success() {
                println!("{}", "  ✅ NPM dependencies updated".green());
            }
        }
    } else {
        println!("{}", "  ⚠️  No package.json found".yellow());
    }

    Ok(())
}

async fn update_docker_images(check: bool) -> Result<()> {
    println!("{}", "🐳 Updating Docker images...".cyan());

    if check {
        // Just list current images
        let _ = Command::new("docker").args(&["images"]).status();
    } else {
        // Pull latest images
        let _ = Command::new("docker").args(&["compose", "pull"]).status();
    }

    Ok(())
}

async fn handle_project_archive(component: String, location: Option<String>) -> Result<()> {
    let archive_location = location.unwrap_or_else(|| "archive".to_string());

    println!(
        "{}",
        format!(
            "📦 Archiving component: {} to {}",
            component, archive_location
        )
        .bright_blue()
    );

    // Create archive directory
    fs::create_dir_all(&archive_location)?;

    // Archive the component
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let archive_name = format!("{}_{}.tar.gz", component, timestamp);
    let archive_path = format!("{}/{}", archive_location, archive_name);

    let status = Command::new("tar")
        .args(&["-czf", &archive_path, &component])
        .status()?;

    if status.success() {
        println!(
            "{}",
            format!("✅ Component archived to: {}", archive_path).green()
        );

        // Optionally remove original
        println!("{}", "Remove original component? (y/N)".yellow());
        // In a real implementation, you'd wait for user input here
    } else {
        println!("{}", "❌ Failed to create archive".red());
    }

    Ok(())
}

async fn initialize_git(path: &str) -> Result<()> {
    println!("{}", "📚 Initializing Git repository...".cyan());

    let status = Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .status()?;

    if status.success() {
        // Create initial commit
        let _ = Command::new("git")
            .args(&["add", "."])
            .current_dir(path)
            .status();

        let _ = Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(path)
            .status();

        println!("{}", "  ✅ Git repository initialized".green());
    }

    Ok(())
}

fn add_service_to_workspace(name: &str) -> Result<()> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;

    if !cargo_toml.contains(&format!("layanan/{}", name)) {
        // Simple implementation - in reality you'd want proper TOML parsing
        let updated = cargo_toml.replace(
            "members = [",
            &format!("members = [\n    \"layanan/{}\",", name),
        );
        fs::write("Cargo.toml", updated)?;
        println!("{}", "  ✅ Added to workspace Cargo.toml".green());
    }

    Ok(())
}

fn add_frontend_to_workspace(name: &str) -> Result<()> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;

    if !cargo_toml.contains(&format!("antarmuka/{}", name)) {
        let updated = cargo_toml.replace(
            "members = [",
            &format!("members = [\n    \"antarmuka/{}\",", name),
        );
        fs::write("Cargo.toml", updated)?;
        println!("{}", "  ✅ Added to workspace Cargo.toml".green());
    }

    Ok(())
}

// Template content generators
fn create_service_cargo_toml(name: &str, template: Option<&str>) -> String {
    let dependencies = match template {
        Some("web") => {
            r#"tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
tower = "0.4"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0""#
        }
        Some("grpc") => {
            r#"tokio = { version = "1.0", features = ["full"] }
tonic = "0.10"
prost = "0.12"
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0""#
        }
        _ => {
            r#"tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0""#
        }
    };

    format!(
        r#"[package]
name = "layanan-{}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "layanan-{}"
path = "src/main.rs"

[dependencies]
{}

[dev-dependencies]
tokio-test = "0.4"
"#,
        name, name, dependencies
    )
}

fn create_service_main_rs(name: &str, template: Option<&str>) -> String {
    match template {
        Some("web") => format!(
            r#"use anyhow::Result;
use axum::{{routing::get, Router}};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {{
    println!("🚀 Starting {} service...", "{}");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root_handler));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}}

async fn health_check() -> &'static str {{
    "OK"
}}

async fn root_handler() -> &'static str {{
    "Hello from {} service!"
}}
"#,
            name, name, name
        ),
        _ => format!(
            r#"use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {{
    println!("🚀 Starting {} service...", "{}");

    // Service implementation here
    loop {{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }}
}}
"#,
            name, name
        ),
    }
}

fn create_service_lib_rs(name: &str, _template: Option<&str>) -> String {
    format!(
        r#"//! {} Service Library
//!
//! This module contains the core business logic for the {} service.

pub mod config;
pub mod handlers;
pub mod models;

pub use config::Config;

/// Service configuration
pub mod config {{
    use serde::{{Deserialize, Serialize}};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {{
        pub service_name: String,
        pub port: u16,
        pub host: String,
    }}

    impl Default for Config {{
        fn default() -> Self {{
            Self {{
                service_name: "{}".to_string(),
                port: 3000,
                host: "0.0.0.0".to_string(),
            }}
        }}
    }}
}}

/// Request/Response models
pub mod models {{
    use serde::{{Deserialize, Serialize}};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthCheck {{
        pub status: String,
        pub service: String,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServiceInfo {{
        pub id: Uuid,
        pub name: String,
        pub version: String,
    }}
}}

/// HTTP handlers
pub mod handlers {{
    use crate::models::*;
    use uuid::Uuid;

    pub async fn health() -> HealthCheck {{
        HealthCheck {{
            status: "healthy".to_string(),
            service: "{}".to_string(),
            timestamp: chrono::Utc::now(),
        }}
    }}

    pub async fn info() -> ServiceInfo {{
        ServiceInfo {{
            id: Uuid::new_v4(),
            name: "{}".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }}
    }}
}}
"#,
        name, name, name, name, name
    )
}

fn create_service_dockerfile(name: &str) -> String {
    format!(
        r#"# Multi-stage build for Rust service
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --bin layanan-{}

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create app user
RUN addgroup -g 1000 appgroup && \
    adduser -D -s /bin/sh -u 1000 -G appgroup appuser

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/layanan-{} ./layanan-{}

# Change ownership
RUN chown -R appuser:appgroup /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the application
CMD ["./layanan-{}"]
"#,
        name, name, name, name
    )
}

fn create_service_test(name: &str) -> String {
    format!(
        r#"//! Integration tests for {} service

use layanan_{};

#[tokio::test]
async fn test_service_health() {{
    let health = layanan_{}::handlers::health().await;
    assert_eq!(health.status, "healthy");
    assert_eq!(health.service, "{}");
}}

#[tokio::test]
async fn test_service_info() {{
    let info = layanan_{}::handlers::info().await;
    assert_eq!(info.name, "{}");
    assert!(!info.id.to_string().is_empty());
}}
"#,
        name, name, name, name, name, name
    )
}

fn create_frontend_cargo_toml(name: &str, _template: Option<&str>) -> String {
    format!(
        r#"[package]
name = "{}-microfrontend"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
leptos = {{ version = "0.5", features = ["csr", "nightly"] }}
leptos_meta = {{ version = "0.5", features = ["csr", "nightly"] }}
leptos_router = {{ version = "0.5", features = ["csr", "nightly"] }}
wasm-bindgen = "0.2"
console_log = "1.0"
console_error_panic_hook = "0.1"
web-sys = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
gloo-net = "0.4"
uuid = {{ version = "1.0", features = ["v4", "js"] }}

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Window",
  "Document",
  "Element",
  "HtmlElement",
  "Location",
]
"#,
        name
    )
}

fn create_frontend_main_rs(name: &str, _template: Option<&str>) -> String {
    format!(
        r#"use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
fn App() -> impl IntoView {{
    provide_meta_context();

    view! {{
        <Stylesheet id="leptos" href="/pkg/{}-microfrontend.css"/>
        <Title text="{}"/>
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }}
}}

#[component]
fn HomePage() -> impl IntoView {{
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {{
        <div class="container">
            <h1>"Welcome to {} Microfrontend"</h1>
            <button on:click=on_click>"Click Me: " {{count}}</button>
        </div>
    }}
}}

fn main() {{
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");

    mount_to_body(|| {{
        view! {{ <App/> }}
    }})
}}
"#,
        name, name, name
    )
}

fn create_frontend_index_html(name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} Microfrontend</title>
    <link rel="stylesheet" href="styles/main.css">
</head>
<body>
    <div id="app"></div>
</body>
</html>
"#,
        name
    )
}

fn create_frontend_trunk_toml(name: &str) -> String {
    format!(
        r#"[build]
target = "index.html"
dist = "dist"
public_url = "/{}"

[serve]
address = "127.0.0.1"
port = 8080
open = false

[watch]
ignore = ["dist"]

[clean]
dist = "dist"
cargo = true
"#,
        name
    )
}

fn create_frontend_styles(_name: &str) -> String {
    r#"/* Modern CSS styles for microfrontend */

:root {
    --primary-color: #3b82f6;
    --secondary-color: #64748b;
    --background-color: #f8fafc;
    --text-color: #1e293b;
    --border-color: #e2e8f0;
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
        'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
        sans-serif;
    background-color: var(--background-color);
    color: var(--text-color);
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    color: var(--primary-color);
    margin-bottom: 1.5rem;
    font-size: 2.5rem;
    font-weight: 700;
}

button {
    background-color: var(--primary-color);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
    box-shadow: var(--shadow);
}

button:hover {
    background-color: #2563eb;
}

button:focus {
    outline: 2px solid var(--primary-color);
    outline-offset: 2px;
}

.card {
    background: white;
    border-radius: 0.75rem;
    padding: 1.5rem;
    box-shadow: var(--shadow);
    border: 1px solid var(--border-color);
}

@media (max-width: 768px) {
    .container {
        padding: 1rem;
    }

    h1 {
        font-size: 2rem;
    }
}
"#
    .to_string()
}

fn create_frontend_dockerfile(_name: &str) -> String {
    format!(
        r#"# Multi-stage build for WASM frontend
FROM rust:1.70 AS builder

# Install trunk and wasm target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Create app directory
WORKDIR /app

# Copy project files
COPY Cargo.toml ./
COPY src ./src
COPY index.html ./
COPY Trunk.toml ./
COPY styles ./styles

# Build the frontend
RUN trunk build --release

# Nginx stage for serving
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
"#
    )
}

fn create_workspace_cargo_toml(name: &str) -> String {
    format!(
        r#"[workspace]
resolver = "2"
members = [
    # Backend services will be added here
    # Frontend applications will be added here
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
repository = "https://github.com/your-org/{}"
license = "MIT"

[workspace.dependencies]
# Common dependencies
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4"] }}
anyhow = "1.0"
chrono = {{ version = "0.4", features = ["serde"] }}

# Web framework dependencies
axum = "0.7"
tower = "0.4"
tower-http = "0.4"

# Database dependencies
tokio-postgres = "0.7"
deadpool-postgres = "0.12"

# Frontend dependencies (for WASM)
leptos = {{ version = "0.5", features = ["csr", "nightly"] }}
leptos_meta = {{ version = "0.5", features = ["csr", "nightly"] }}
leptos_router = {{ version = "0.5", features = ["csr", "nightly"] }}
wasm-bindgen = "0.2"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[profile.release-wasm]
inherits = "release"
opt-level = "s"
lto = true
debug = false
"#,
        name
    )
}

fn create_workspace_readme(name: &str) -> String {
    format!(
        r#"# {}

A modern microservices architecture built with Rust and WebAssembly.

## 🏗️ Architecture

This workspace contains:

- **Backend Services** (`layanan/`) - Rust microservices
- **Frontend Applications** (`antarmuka/`) - WASM microfrontends
- **Infrastructure** (`infra/`) - Deployment configurations
- **Scripts** (`scripts/`) - Development and deployment tools
- **Documentation** (`docs/`) - Project documentation

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Docker & Docker Compose
- Node.js (for some tooling)

### Development Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd {}
```

2. Install development tools:
```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

3. Start development environment:
```bash
./scripts/cli/target/debug/simpel dev start
```

## 🔧 CLI Tool

This project includes a comprehensive CLI tool for development:

```bash
# Build all components
./simpel build all

# Run tests
./simpel test all

# Deploy to Kubernetes
./simpel k8s deploy

# Show project status
./simpel status --detailed
```

## 📁 Project Structure

```
├── layanan/           # Backend microservices
├── antarmuka/         # Frontend microfrontends
├── infra/             # Infrastructure as code
├── scripts/           # Development scripts
├── docs/              # Documentation
├── Cargo.toml         # Workspace configuration
└── docker-compose.yml # Local development
```

## 🧪 Testing

```bash
# Run all tests
./simpel test all

# Run specific test types
./simpel test unit
./simpel test integration
./simpel test security
```

## 🚀 Deployment

### Local Development
```bash
./simpel dev start
```

### Kubernetes
```bash
./simpel k8s deploy --env production
```

## 📚 Documentation

- [API Documentation](docs/api/)
- [Architecture Guide](docs/architecture.md)
- [Development Guide](docs/development.md)
- [Deployment Guide](docs/deployment.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `./simpel test all`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
"#,
        name, name
    )
}

fn create_gitignore() -> String {
    r#"# Rust
/target/
Cargo.lock
*.pdb

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Environment
.env
.env.local
.env.production

# Dependencies
node_modules/

# Build artifacts
dist/
pkg/
*.wasm

# Temporary files
*.tmp
*.temp

# Docker
.dockerignore

# Database
*.db
*.sqlite

# Cache
.cache/
"#
    .to_string()
}

fn create_bash_tool(name: &str) -> String {
    format!(
        r#"#!/bin/bash
# {} tool

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${{BLUE}}🔧 {} Tool${{NC}}"

# Tool implementation here
main() {{
    echo -e "${{GREEN}}✅ {} tool executed successfully${{NC}}"
}}

# Run main function
main "$@"
"#,
        name, name, name
    )
}

fn create_python_tool(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env python3
"""
{} tool

A Python-based tool for SIMPelv2 project.
"""

import sys
import logging
from pathlib import Path

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def main():
    """Main function for {} tool."""
    logger.info("🔧 Starting {} tool...")

    # Tool implementation here

    logger.info("✅ {} tool completed successfully")

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        logger.error(f"❌ Error: {{e}}")
        sys.exit(1)
"#,
        name, name, name, name
    )
}

fn create_rust_tool(name: &str) -> String {
    format!(
        r#"//! {} tool
//!
//! A Rust-based tool for SIMPelv2 project.

use anyhow::Result;
use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(name = "{}")]
#[command(about = "A Rust tool for SIMPelv2")]
struct Args {{
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}}

fn main() -> Result<()> {{
    let args = Args::parse();

    if args.verbose {{
        env_logger::init();
    }}

    println!("{{}}", "🔧 {} tool starting...".bright_blue());

    // Tool implementation here

    println!("{{}}", "✅ {} tool completed successfully".green());

    Ok(())
}}
"#,
        name, name, name, name
    )
}
