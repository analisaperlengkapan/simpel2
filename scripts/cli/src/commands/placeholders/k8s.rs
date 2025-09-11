use anyhow::Result;

pub async fn run_k8s_command(command: String) -> Result<()> {
    println!("K8s command: {}", command);
    Ok(())
}
