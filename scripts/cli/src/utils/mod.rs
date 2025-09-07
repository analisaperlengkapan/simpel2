pub mod output;
pub mod process;
pub mod file;
pub mod template;

use anyhow::Result;
use std::path::Path;

pub fn detect_workspace_root() -> Result<std::path::PathBuf> {
    let mut current = std::env::current_dir()?;
    
    loop {
        // Look for Cargo.toml with workspace definition
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }
        
        // Look for simpel.toml
        let simpel_toml = current.join("simpel.toml");
        if simpel_toml.exists() {
            return Ok(current);
        }
        
        // Look for specific SIMPelv2 structure
        let layanan_dir = current.join("layanan");
        let antarmuka_dir = current.join("antarmuka");
        if layanan_dir.exists() && antarmuka_dir.exists() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    
    // Default to current directory
    Ok(std::env::current_dir()?)
}

pub fn is_simpelv2_workspace(path: &Path) -> bool {
    let layanan = path.join("layanan");
    let antarmuka = path.join("antarmuka");
    let cargo_toml = path.join("Cargo.toml");
    
    layanan.exists() && antarmuka.exists() && cargo_toml.exists()
}
