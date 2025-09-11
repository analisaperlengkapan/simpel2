use anyhow::Result;

pub async fn run_docker_command(command: String) -> Result<()> {
    println!("Docker command: {}", command);
    Ok(())
}
