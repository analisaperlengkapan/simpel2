use crate::K8sCommands;
use anyhow::Result;
use colored::Colorize;
use tokio::process::Command as AsyncCommand;

/// Handle K8s operations
pub async fn handle_k8s(action: K8sCommands) -> Result<()> {
    match action {
        K8sCommands::Deploy { env, force } => handle_k8s_deploy(env, force).await,
        K8sCommands::Manage {
            action,
            resource,
            name,
        } => handle_k8s_manage(action, resource, name).await,
        K8sCommands::Status { detailed } => handle_k8s_status(detailed).await,
        K8sCommands::Cleanup { force, scope } => handle_k8s_cleanup(force, scope).await,
        K8sCommands::Logs { pod, follow } => handle_k8s_logs(pod, follow).await,
        K8sCommands::Port {
            service,
            local_port,
            remote_port,
        } => handle_k8s_port(service, local_port, remote_port).await,
    }
}

async fn handle_k8s_deploy(environment: String, force: bool) -> Result<()> {
    println!("{}", "🚀 K8s Deployment".bright_blue());
    println!("Environment: {}", environment.cyan());

    if force {
        println!("{}", "⚠️  Force deployment mode enabled".yellow());
    }

    run_kubectl_command(&["apply", "-f", &format!("infra/k8s/{}", environment)]).await
}

async fn handle_k8s_manage(action: String, resource: String, name: Option<String>) -> Result<()> {
    println!(
        "{}",
        format!("⚙️  K8s Management: {} {}", action, resource).bright_blue()
    );

    match action.as_str() {
        "scale" => {
            if let Some(resource_name) = name {
                let resource_path = format!("{}/{}", resource, resource_name);
                run_kubectl_command(&["scale", &resource_path, "--replicas=1"]).await
            } else {
                println!("{}", "❌ Resource name required for scaling".red());
                Ok(())
            }
        }
        _ => {
            println!(
                "{}",
                format!("❌ Unknown management action: {}", action).red()
            );
            Ok(())
        }
    }
}

async fn handle_k8s_status(detailed: bool) -> Result<()> {
    println!("{}", "📊 K8s Status".bright_blue());

    if detailed {
        run_kubectl_command(&["get", "all", "-n", "simpelv2"]).await
    } else {
        run_kubectl_command(&["get", "pods", "-n", "simpelv2"]).await
    }
}

async fn handle_k8s_cleanup(force: bool, scope: String) -> Result<()> {
    println!("{}", format!("🧹 K8s Cleanup: {}", scope).bright_blue());

    if force {
        println!("{}", "⚠️  Force cleanup mode enabled".yellow());
    }

    match scope.as_str() {
        "namespace" => run_kubectl_command(&["delete", "namespace", "simpelv2"]).await,
        "all" => run_kubectl_command(&["delete", "all", "--all", "-n", "simpelv2"]).await,
        _ => {
            println!("{}", format!("❌ Unknown cleanup scope: {}", scope).red());
            Ok(())
        }
    }
}

async fn handle_k8s_logs(pod: String, follow: bool) -> Result<()> {
    println!("{}", format!("📝 K8s Logs for pod: {}", pod).bright_blue());

    if follow {
        run_kubectl_command(&["logs", "-f", &pod, "-n", "simpelv2"]).await
    } else {
        run_kubectl_command(&["logs", &pod, "-n", "simpelv2"]).await
    }
}

async fn handle_k8s_port(service: String, local_port: u16, remote_port: u16) -> Result<()> {
    println!(
        "{}",
        format!(
            "🔗 Port Forward: {}:{} -> {}",
            service, local_port, remote_port
        )
        .bright_blue()
    );

    let port_spec = format!("{}:{}", local_port, remote_port);
    let service_spec = format!("service/{}", service);

    run_kubectl_command(&["port-forward", &service_spec, &port_spec, "-n", "simpelv2"]).await
}

async fn run_kubectl_command(args: &[&str]) -> Result<()> {
    let mut cmd_args = vec!["kubectl"];
    cmd_args.extend_from_slice(args);

    let output = AsyncCommand::new("microk8s")
        .args(&cmd_args)
        .output()
        .await?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!(
            "{}",
            format!(
                "❌ Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .red()
        );
    }

    Ok(())
}
