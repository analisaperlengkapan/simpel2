use anyhow::Result;

pub async fn run_security_command(command: String) -> Result<()> {
    println!("Security command: {}", command);
    Ok(())
}
