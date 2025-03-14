use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use walkdir::WalkDir;

use crate::types::MissionFileResults;

/// Check if a path is a mission directory
fn is_mission_directory(path: &Path) -> bool {
    path.is_dir() && path.join("mission.sqm").exists()
}

/// Find mission.sqm in a directory
pub fn find_mission_file(dir: &Path) -> Result<Option<PathBuf>> {
    let sqm_path = dir.join("mission.sqm");
    if sqm_path.exists() {
        Ok(Some(sqm_path))
    } else {
        Ok(None)
    }
}

/// Find all SQF files in a directory
pub fn find_script_files(dir: &Path, allowed_extensions: &[String]) -> Result<Vec<PathBuf>> {
    if !allowed_extensions.contains(&"sqf".to_string()) {
        return Ok(Vec::new());
    }

    let mut sqf_files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "sqf") {
            sqf_files.push(path.to_path_buf());
        }
    }
    Ok(sqf_files)
}

/// Find all CPP/HPP files in a directory
pub fn find_code_files(dir: &Path, allowed_extensions: &[String]) -> Result<Vec<PathBuf>> {
    // Check if any code file extensions are allowed
    let has_code_extensions = allowed_extensions.iter().any(|ext| 
        ext == "cpp" || ext == "hpp" || ext == "ext"
    );
    if !has_code_extensions {
        return Ok(Vec::new());
    }

    let mut cpp_files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if allowed_extensions.contains(&ext.to_string()) {
                    cpp_files.push(path.to_path_buf());
                }
            }
        }
    }
    Ok(cpp_files)
}

/// Collect mission files from a directory with configuration
pub fn collect_mission_files(dir: &Path) -> Result<Vec<MissionFileResults>> {
    let mut results = Vec::new();
    
    let walker = WalkDir::new(dir);

    // Track unique mission names to avoid duplicates
    let mut seen_missions = HashSet::new();
    
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip non-mission directories
        if !is_mission_directory(path) {
            continue;
        }
        
        // Get mission name from directory name
        let mission_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid mission directory name"))?
            .to_string();
        
        // Skip if we've seen this mission name before
        if !seen_missions.insert(mission_name.clone()) {
            continue;
        }
        
        // Find mission.sqm
        let mission_file = find_mission_file(path)?;
        
        // Find SQF files
        let script_files = find_script_files(path, &["sqf".to_string()])?;
        
        // Find CPP/HPP files
        let code_files = find_code_files(path, &["cpp".to_string(), "hpp".to_string(), "ext".to_string()])?;
        
        results.push(MissionFileResults {
            mission_name,
            mission_dir: path.to_path_buf(),
            sqm_file: mission_file,
            sqf_files: script_files,
            cpp_files: code_files,
        });
    }
    
    Ok(results)
} 