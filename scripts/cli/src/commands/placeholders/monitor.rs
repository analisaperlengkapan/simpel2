use anyhow::Result;

pub async fn run_monitor_command(command: String) -> Result<()> {
    println!("Monitor command: {}", command);
    Ok(())
}
