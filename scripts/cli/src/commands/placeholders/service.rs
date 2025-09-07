use anyhow::Result;

pub async fn run_service_command(command: String) -> Result<()> {
    println!("Service command: {}", command);
    Ok(())
}
