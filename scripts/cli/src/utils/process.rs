use anyhow::Result;
use std::process::{Command, Stdio};
use crate::utils::output::{debug, info};
use std::path::Path;

pub struct CargoHelper;

impl CargoHelper {
    pub async fn build(
        &self,
        package: Option<&str>,
        features: &[String],
        release: bool,
        working_dir: Option<&Path>,
    ) -> Result<()> {
        cargo_build(package, features, release, working_dir).await
    }
    
    pub async fn test(
        &self,
        package: Option<&str>,
        test_name: Option<&str>,
        working_dir: Option<&Path>,
    ) -> Result<()> {
        cargo_test(package, test_name, working_dir).await
    }
    
    pub async fn clippy(&self, working_dir: Option<&Path>) -> Result<()> {
        cargo_clippy(working_dir).await
    }
    
    pub async fn build_with_args(args: &[&str], working_dir: Option<&Path>) -> Result<()> {
        run_command_streaming("cargo", args, working_dir).await
    }
}

pub struct DockerHelper;

impl DockerHelper {
    pub async fn build(
        &self,
        dockerfile: &str,
        tag: &str,
        context: &Path,
    ) -> Result<()> {
        docker_build(dockerfile, tag, context).await
    }
    
    pub async fn push(&self, tag: &str) -> Result<()> {
        docker_push(tag).await
    }
    
    pub async fn run(
        &self,
        image: &str,
        ports: &[(u16, u16)],
        env: &[(&str, &str)],
    ) -> Result<()> {
        docker_run(image, ports, env).await
    }
}

pub struct KubectlHelper;

impl KubectlHelper {
    pub async fn apply(&self, manifest: &Path) -> Result<()> {
        kubectl_apply(manifest).await
    }
    
    pub async fn delete(&self, manifest: &Path) -> Result<()> {
        kubectl_delete(manifest).await
    }
    
    pub async fn get_pods(&self, namespace: Option<&str>) -> Result<String> {
        kubectl_get_pods(namespace).await
    }
    
    pub async fn get_services(&self, namespace: Option<&str>) -> Result<String> {
        kubectl_get_services(namespace).await
    }
}

pub struct ProcessResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn run_command(cmd: &str, args: &[&str], working_dir: Option<&str>) -> Result<ProcessResult> {
    debug(&format!("Running: {} {}", cmd, args.join(" ")));
    
    let mut command = Command::new(cmd);
    command.args(args);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    let output = command.output()?;
    
    let result = ProcessResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    };
    
    debug(&format!("Command finished with exit code: {}", result.exit_code));
    
    Ok(result)
}

pub async fn run_command_streaming(cmd: &str, args: &[&str], working_dir: Option<&str>) -> Result<bool> {
    info(&format!("Running: {} {}", cmd, args.join(" ")));
    
    let mut command = Command::new(cmd);
    command.args(args);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    
    let status = command.status()?;
    Ok(status.success())
}

pub async fn cargo_build(package: Option<&str>, release: bool, features: &[&str], working_dir: Option<&str>) -> Result<bool> {
    let mut args = vec!["build"];
    
    if release {
        args.push("--release");
    }
    
    if let Some(pkg) = package {
        args.extend(&["--package", pkg]);
    }
    
    if !features.is_empty() {
        let feature_str = features.join(",");
        args.push("--features");
        args.push(&feature_str);
    }
    
    run_command_streaming("cargo", &args, working_dir).await
}

pub async fn cargo_test(package: Option<&str>, working_dir: Option<&str>) -> Result<bool> {
    let mut args = vec!["test"];
    
    if let Some(pkg) = package {
        args.extend(&["--package", pkg]);
    }
    
    run_command_streaming("cargo", &args, working_dir).await
}

pub async fn cargo_check(package: Option<&str>, working_dir: Option<&str>) -> Result<bool> {
    let mut args = vec!["check"];
    
    if let Some(pkg) = package {
        args.extend(&["--package", pkg]);
    }
    
    run_command_streaming("cargo", &args, working_dir).await
}

pub async fn docker_build(dockerfile: &str, tag: &str, context: &str) -> Result<bool> {
    let args = vec![
        "build",
        "-f", dockerfile,
        "-t", tag,
        context
    ];
    
    run_command_streaming("docker", &args, None).await
}

pub async fn docker_push(tag: &str) -> Result<bool> {
    let args = vec!["push", tag];
    run_command_streaming("docker", &args, None).await
}

pub async fn kubectl_apply(file: &str, namespace: Option<&str>) -> Result<bool> {
    let mut args = vec!["apply", "-f", file];
    
    if let Some(ns) = namespace {
        args.extend(&["-n", ns]);
    }
    
    run_command_streaming("kubectl", &args, None).await
}

pub async fn kubectl_delete(resource: &str, name: &str, namespace: Option<&str>) -> Result<bool> {
    let mut args = vec!["delete", resource, name];
    
    if let Some(ns) = namespace {
        args.extend(&["-n", ns]);
    }
    
    run_command_streaming("kubectl", &args, None).await
}

pub async fn trunk_build(config: Option<&str>, working_dir: Option<&str>) -> Result<bool> {
    let mut args = vec!["build"];
    
    if let Some(cfg) = config {
        args.extend(&["--config", cfg]);
    }
    
    run_command_streaming("trunk", &args, working_dir).await
}

pub async fn trunk_serve(config: Option<&str>, port: Option<u16>, working_dir: Option<&str>) -> Result<bool> {
    let mut args = vec!["serve"];
    
    if let Some(cfg) = config {
        args.extend(&["--config", cfg]);
    }
    
    if let Some(p) = port {
        let port_str = p.to_string();
        args.push("--port");
        args.push(&port_str);
    }
    
    run_command_streaming("trunk", &args, working_dir).await
}
