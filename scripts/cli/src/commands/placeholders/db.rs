use anyhow::Result;

pub async fn run_db_command(command: String) -> Result<()> {
    println!("Database command: {}", command);
    Ok(())
}
