use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }
    Ok(())
}

pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

pub fn write_file_content(path: &Path, content: &str) -> Result<()> {
    ensure_directory_exists(path)?;
    fs::write(path, content).with_context(|| format!("Failed to write file: {:?}", path))
}

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn remove_file(path: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("Failed to remove file: {:?}", path))
}

pub fn move_file(src: &Path, dst: &Path) -> Result<()> {
    ensure_directory_exists(dst)?;
    fs::rename(src, dst).with_context(|| format!("Failed to move file from {:?} to {:?}", src, dst))
}
