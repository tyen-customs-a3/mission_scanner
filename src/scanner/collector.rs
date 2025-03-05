use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, debug};
use walkdir::WalkDir;

/// Collect mission files from a directory
pub fn collect_mission_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    info!("Collecting mission files from {}", input_dir.display());
    
    let mut mission_files = Vec::new();
    
    // Walk through the directory and find PBO files
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Check if the file is a PBO file
        if path.is_file() && 
           path.extension().map(|ext| ext.to_string_lossy().to_lowercase() == "pbo").unwrap_or(false) {
            // Check if the file name contains "mission" or ends with "_co.pbo"
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            if file_name.contains("mission") || file_name.ends_with("_co.pbo") {
                debug!("Found mission file: {}", path.display());
                mission_files.push(path.to_path_buf());
            }
        }
    }
    
    info!("Found {} mission files", mission_files.len());
    
    Ok(mission_files)
} 