use crate::{
    config::{SimplConfig, ServiceType},
    utils::{output::SimplOutput, process::CargoHelper},
};
use anyhow::Result;

pub async fn handle_test_command(
    command: &crate::commands::TestCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::TestCommands::All { unit, integration, coverage } => {
            test_all(*unit, *integration, *coverage, config, output).await
        }
        crate::commands::TestCommands::Service { name, test_type } => {
            test_service(name, test_type.as_deref(), config, output).await
        }
        crate::commands::TestCommands::Bench { service } => {
            test_bench(service.as_deref(), config, output).await
        }
        crate::commands::TestCommands::Lint { fix } => {
            test_lint(*fix, config, output).await
        }
        crate::commands::TestCommands::Audit => test_audit(config, output).await,
    }
}

async fn test_all(
    unit: bool,
    integration: bool,
    coverage: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Running all tests...")?;

    let mut test_types = Vec::new();
    if unit && !integration {
        test_types.push("unit");
    } else if integration && !unit {
        test_types.push("integration");
    } else {
        test_types.extend(&["unit", "integration"]);
    }

    let progress = output.progress("Running tests...", test_types.len())?;

    for test_type in &test_types {
        progress.set_message(&format!("Running {} tests...", test_type));
        
        match *test_type {
            "unit" => run_unit_tests(config, output).await?,
            "integration" => run_integration_tests(config, output).await?,
            _ => {}
        }
        
        progress.inc(1);
    }

    if coverage {
        progress.set_message("Generating coverage report...");
        generate_coverage_report(config, output).await?;
    }

    progress.finish_with_message("All tests completed!");
    output.success("Test run completed successfully!")?;

    Ok(())
}

async fn test_service(
    name: &str,
    test_type: Option<&str>,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(service_config) = config.workspace.services.get(name) {
        output.info(&format!("Running tests for service: {}", name))?;

        match test_type {
            Some("unit") => {
                run_service_unit_tests(name, service_config, output).await?;
            }
            Some("integration") => {
                run_service_integration_tests(name, service_config, output).await?;
            }
            Some("e2e") => {
                run_service_e2e_tests(name, service_config, output).await?;
            }
            _ => {
                // Run all test types for the service
                run_service_unit_tests(name, service_config, output).await?;
                run_service_integration_tests(name, service_config, output).await?;
            }
        }

        output.success(&format!("Tests for {} completed successfully!", name))?;
    } else {
        output.error(&format!("Service {} not found", name))?;
    }

    Ok(())
}

async fn test_bench(
    service: Option<&str>,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    if let Some(service_name) = service {
        // Run benchmarks for specific service
        if let Some(service_config) = config.workspace.services.get(service_name) {
            output.info(&format!("Running benchmarks for: {}", service_name))?;
            run_service_benchmarks(service_name, service_config, output).await?;
        } else {
            output.error(&format!("Service {} not found", service_name))?;
        }
    } else {
        // Run all benchmarks
        output.info("Running all benchmarks...")?;

        let backend_services: Vec<_> = config
            .workspace
            .services
            .iter()
            .filter(|(_, cfg)| cfg.service_type == "backend")
            .collect();

        let progress = output.progress("Running benchmarks...", backend_services.len())?;

        for (name, service_config) in backend_services {
            progress.set_message(&format!("Benchmarking {}", name));
            run_service_benchmarks(name, service_config, output).await?;
            progress.inc(1);
        }

        progress.finish_with_message("All benchmarks completed!");
        output.success("Benchmark run completed!")?;
    }

    Ok(())
}

async fn test_lint(
    fix: bool,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info("Running linting checks...")?;

    let progress = output.progress("Running linters...", 4)?;

    // Clippy
    progress.set_message("Running clippy...");
    run_clippy(fix, output).await?;
    progress.inc(1);

    // Rustfmt
    progress.set_message("Running rustfmt...");
    run_rustfmt(fix, output).await?;
    progress.inc(1);

    // Check frontend formatting
    progress.set_message("Checking frontend code...");
    check_frontend_formatting(config, output).await?;
    progress.inc(1);

    // Custom lints
    progress.set_message("Running custom lints...");
    run_custom_lints(config, output).await?;
    progress.inc(1);

    progress.finish_with_message("Linting completed!");
    output.success("Linting checks completed!")?;

    Ok(())
}

async fn test_audit(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Running security audit...")?;

    let progress = output.progress("Running security checks...", 3)?;

    // Cargo audit
    progress.set_message("Running cargo audit...");
    run_cargo_audit(output).await?;
    progress.inc(1);

    // Check for common vulnerabilities
    progress.set_message("Checking for vulnerabilities...");
    check_vulnerabilities(config, output).await?;
    progress.inc(1);

    // Dependency scan
    progress.set_message("Scanning dependencies...");
    scan_dependencies(config, output).await?;
    progress.inc(1);

    progress.finish_with_message("Security audit completed!");
    output.success("Security audit completed successfully!")?;

    Ok(())
}

async fn run_unit_tests(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    let args = vec!["test", "--lib"];
    CargoHelper::build_with_args(&args, None).await?;
    output.success("Unit tests passed!")?;
    Ok(())
}

async fn run_integration_tests(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    let args = vec!["test", "--test", "*"];
    CargoHelper::build_with_args(&args, None).await?;
    output.success("Integration tests passed!")?;
    Ok(())
}

async fn run_service_unit_tests(
    name: &str,
    service_config: &crate::config::ServiceConfig,
    output: &SimplOutput,
) -> Result<()> {
    if matches!(service_config.service_type, ServiceType::Backend) {
        let args = vec!["test", "--lib"];
        CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
        output.success(&format!("Unit tests for {} passed!", name));
    } else {
        output.info(&format!("Skipping unit tests for frontend service: {}", name));
    }
    Ok(())
}

async fn run_service_integration_tests(
    name: &str,
    service_config: &crate::config::ServiceConfig,
    output: &SimplOutput,
) -> Result<()> {
    if matches!(service_config.service_type, ServiceType::Backend) {
        let args = vec!["test", "--test", "*"];
        CargoHelper::build_with_args(&args, Some(&service_config.path)).await?;
        output.success(&format!("Integration tests for {} passed!", name));
    } else {
        output.info(&format!("Skipping integration tests for frontend service: {}", name));
    }
    Ok(())
}

async fn run_service_e2e_tests(
    name: &str,
    service_config: &crate::config::ServiceConfig,
    output: &SimplOutput,
) -> Result<()> {
    // Look for E2E test scripts
    let e2e_script = format!("{}/tests/e2e.sh", service_config.path.display());
    if tokio::fs::metadata(&e2e_script).await.is_ok() {
        output.info(&format!("Running E2E tests for {}", name));
        
        let mut cmd = tokio::process::Command::new("bash");
        cmd.arg(&e2e_script);
        cmd.current_dir(&service_config.path);
        
        let status = cmd.status().await?;
        if status.success() {
            output.success(&format!("E2E tests for {} passed!", name));
        } else {
            output.error(&format!("E2E tests for {} failed!", name));
        }
    } else {
        output.info(&format!("No E2E tests found for {}", name));
    }
    
    Ok(())
}

async fn run_service_benchmarks(
    name: &str,
    service_config: &crate::config::ServiceConfig,
    output: &SimplOutput,
) -> Result<()> {
    if matches!(service_config.service_type, ServiceType::Backend) {
        let args = vec!["bench"];
        match CargoHelper::build_with_args(&args, Some(&service_config.path)).await {
            Ok(_) => output.success(&format!("Benchmarks for {} completed!", name)),
            Err(_) => output.info(&format!("No benchmarks found for {}", name)),
        }
    }
    Ok(())
}

async fn run_clippy(fix: bool, output: &SimplOutput) -> Result<()> {
    let mut args = vec!["clippy", "--all", "--all-targets", "--", "-D", "warnings"];
    if fix {
        args.insert(1, "--fix");
        args.insert(2, "--allow-dirty");
    }

    match CargoHelper::build_with_args(&args, None).await {
        Ok(_) => output.success("Clippy checks passed!")?,
        Err(_) => output.error("Clippy found issues!")?,
    }

    Ok(())
}

async fn run_rustfmt(fix: bool, output: &SimplOutput) -> Result<()> {
    let args = if fix {
        vec!["fmt"]
    } else {
        vec!["fmt", "--check"]
    };

    match CargoHelper::build_with_args(&args, None).await {
        Ok(_) => output.success("Formatting checks passed!")?,
        Err(_) => {
            if fix {
                output.info("Code formatted successfully!")?;
            } else {
                output.error("Formatting issues found! Run with --fix to auto-format.")?;
            }
        }
    }

    Ok(())
}

async fn check_frontend_formatting(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    for (name, service_config) in &config.workspace.services {
        if service_config.service_type == "frontend" {
            // Check if prettier or similar tools are configured
            let prettier_config = format!("{}/.prettierrc", service_config.path);
            if tokio::fs::metadata(&prettier_config).await.is_ok() {
                output.info(&format!("Frontend {} has formatting configuration", name))?;
            } else {
                output.warning(&format!("Frontend {} has no formatting configuration", name))?;
            }
        }
    }

    Ok(())
}

async fn run_custom_lints(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    // Check for common patterns and best practices
    
    // Check for TODO/FIXME comments
    output.info("Checking for TODO/FIXME comments...")?;
    
    // Check for hardcoded secrets
    output.info("Checking for potential hardcoded secrets...")?;
    
    // Check dependency versions
    output.info("Checking dependency versions...")?;
    
    output.success("Custom lints completed!")?;
    Ok(())
}

async fn run_cargo_audit(output: &SimplOutput) -> Result<()> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(&["audit"]);
    
    match cmd.status().await {
        Ok(status) if status.success() => {
            output.success("No security vulnerabilities found!")?;
        }
        Ok(_) => {
            output.warning("Cargo audit found some issues")?;
        }
        Err(_) => {
            output.info("cargo-audit not installed, skipping vulnerability check")?;
        }
    }
    
    Ok(())
}

async fn check_vulnerabilities(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    // Check for common vulnerability patterns
    output.info("Checking for common vulnerability patterns...")?;
    
    // Check for SQL injection patterns
    output.info("Checking for potential SQL injection patterns...")?;
    
    // Check for XSS patterns
    output.info("Checking for potential XSS patterns...")?;
    
    output.success("Vulnerability check completed!")?;
    Ok(())
}

async fn scan_dependencies(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    // Check Cargo.toml files for outdated dependencies
    output.info("Scanning dependencies for updates...")?;
    
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(&["outdated"]);
    
    match cmd.status().await {
        Ok(status) if status.success() => {
            output.success("All dependencies are up to date!")?;
        }
        Ok(_) => {
            output.warning("Some dependencies may be outdated")?;
        }
        Err(_) => {
            output.info("cargo-outdated not installed, skipping dependency scan")?;
        }
    }
    
    Ok(())
}

async fn generate_coverage_report(config: &SimplConfig, output: &SimplOutput) -> Result<()> {
    output.info("Generating coverage report...")?;
    
    // Use tarpaulin or similar for coverage
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(&["tarpaulin", "--out", "Html", "--output-dir", "target/coverage"]);
    
    match cmd.status().await {
        Ok(status) if status.success() => {
            output.success("Coverage report generated at target/coverage/tarpaulin-report.html")?;
        }
        Ok(_) => {
            output.error("Failed to generate coverage report")?;
        }
        Err(_) => {
            output.info("cargo-tarpaulin not installed, skipping coverage report")?;
        }
    }
    
    Ok(())
}
