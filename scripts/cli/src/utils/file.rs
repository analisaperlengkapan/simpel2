use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

pub fn read_to_string(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn write_string(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

pub fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path;
    
    loop {
        if current.join("Cargo.toml").exists() 
            && current.join("layanan").exists() 
            && current.join("antarmuka").exists() {
            return Some(current.to_path_buf());
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}
