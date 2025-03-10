use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, debug};
use walkdir::WalkDir;

use crate::types::{MissionExtractionResult, MissionScannerConfig};

/// Process a mission directory and collect its files
fn process_mission_directory(mission_dir: &Path, sqm_path: PathBuf) -> MissionExtractionResult {
    debug!("Processing mission directory: {}", mission_dir.display());
    
    // Get mission name from directory
    let mission_name = mission_dir.file_name()
        .map_or("unknown".to_string(), |name| name.to_string_lossy().to_string());
    
    // Find all relevant files
    let mut sqf_files = Vec::new();
    let mut cpp_files = Vec::new();
    
    for file_entry in WalkDir::new(mission_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {
        
        let file_path = file_entry.path().to_owned();
        
        // Special case for description.ext
        if file_path.file_name().map_or(false, |name| name == "description.ext") {
            debug!("Found description.ext: {}", file_path.display());
            cpp_files.push(file_path);
            continue;
        }
        
        // Process other files by extension
        if let Some(ext) = file_path.extension() {
            match ext.to_string_lossy().as_ref() {
                "sqf" => {
                    debug!("Found SQF file: {}", file_path.display());
                    sqf_files.push(file_path);
                },
                "cpp" | "hpp" => {
                    debug!("Found CPP/HPP file: {}", file_path.display());
                    cpp_files.push(file_path);
                },
                _ => {}
            }
        }
    }
    
    debug!("Mission directory summary:");
    debug!("  Name: {}", mission_name);
    debug!("  SQM file: {}", sqm_path.display());
    debug!("  SQF files: {}", sqf_files.len());
    debug!("  CPP files: {}", cpp_files.len());
    
    MissionExtractionResult {
        mission_name,
        pbo_path: mission_dir.to_path_buf(),
        mission_dir: mission_dir.to_path_buf(),
        sqm_file: Some(sqm_path),
        sqf_files,
        cpp_files,
    }
}

/// Collect mission files from a directory
pub fn collect_mission_files(input_dir: &Path) -> Result<Vec<MissionExtractionResult>> {
    // Use default config with recursion enabled
    let config = MissionScannerConfig {
        recursive: true,
        ..Default::default()
    };
    
    collect_mission_files_with_config(input_dir, &config)
}

/// Collect mission files from a directory with configuration options
pub fn collect_mission_files_with_config(input_dir: &Path, config: &MissionScannerConfig) -> Result<Vec<MissionExtractionResult>> {
    info!("Collecting mission files from {} with config", input_dir.display());
    debug!("Configuration: {:?}", config);
    
    let mut mission_results = Vec::new();
    
    // Configure the walker based on recursion setting
    let walker = if config.recursive {
        WalkDir::new(input_dir)
    } else {
        WalkDir::new(input_dir).max_depth(1)
    };
    
    // Walk through the directory and find mission.sqm files
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Check if the file is a mission.sqm file
        if path.is_file() && path.file_name().map_or(false, |name| name == "mission.sqm") {
            debug!("Found mission.sqm: {}", path.display());
            
            // The parent directory is the mission directory
            if let Some(mission_dir) = path.parent() {
                let result = process_mission_directory(mission_dir, path.to_path_buf());
                mission_results.push(result);
            } else {
                warn!("Found mission.sqm without parent directory: {}", path.display());
            }
        }
    }
    
    info!("Found {} mission directories", mission_results.len());
    
    Ok(mission_results)
} 