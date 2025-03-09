use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error, debug};
use walkdir::WalkDir;

/// Find a file with a specific extension
pub fn find_file_by_extension(dir: &Path, extension: &str) -> Option<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_type().is_file() && 
            e.path().extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == extension)
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
}

/// Find all files with a specific extension
pub fn find_files_by_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && 
            e.path().extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == extension)
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Count files in a directory
pub fn count_files_in_directory(dir: &Path) -> Result<usize> {
    let count = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
    
    Ok(count)
}

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Create a directory if it doesn't exist
pub fn create_dir_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    
    Ok(())
} 