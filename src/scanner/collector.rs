use std::path::{Path, PathBuf};
use std::collections::HashSet;
use anyhow::{Result, anyhow};
use log::{info, warn, debug};
use walkdir::WalkDir;

use crate::types::{MissionExtractionResult, MissionScannerConfig};

/// Check if a path is a mission directory
fn is_mission_directory(path: &Path) -> bool {
    path.is_dir() && path.join("mission.sqm").exists()
}

/// Find mission.sqm in a directory
fn find_mission_sqm(dir: &Path) -> Result<Option<PathBuf>> {
    let sqm_path = dir.join("mission.sqm");
    if sqm_path.exists() {
        Ok(Some(sqm_path))
    } else {
        Ok(None)
    }
}

/// Find all SQF files in a directory
fn find_sqf_files(dir: &Path) -> Result<Vec<PathBuf>> {
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
fn find_cpp_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut cpp_files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "cpp" || ext == "hpp" || ext == "ext" {
                    cpp_files.push(path.to_path_buf());
                }
            }
        }
    }
    Ok(cpp_files)
}

/// Collect mission files from a directory
pub fn collect_mission_files(dir: &Path) -> Result<Vec<MissionExtractionResult>> {
    collect_mission_files_with_config(dir, &MissionScannerConfig::default())
}

/// Collect mission files from a directory with configuration
pub fn collect_mission_files_with_config(dir: &Path, config: &MissionScannerConfig) -> Result<Vec<MissionExtractionResult>> {
    let mut results = Vec::new();
    
    // Walk the directory recursively if configured
    let walker = if config.recursive {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).max_depth(1)
    };
    
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
        let sqm_file = find_mission_sqm(path)?;
        
        // Find SQF files
        let sqf_files = find_sqf_files(path)?;
        
        // Find CPP/HPP files
        let cpp_files = find_cpp_files(path)?;
        
        results.push(MissionExtractionResult {
            mission_name,
            mission_dir: path.to_path_buf(),
            sqm_file,
            sqf_files,
            cpp_files,
            pbo_path: None, // No PBO path for directory scans
            class_dependencies: Vec::new(), // Initialize empty dependencies
        });
    }
    
    Ok(results)
} 