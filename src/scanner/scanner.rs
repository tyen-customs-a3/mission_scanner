use std::path::Path;

use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use rayon::prelude::*;

use crate::types::{MissionScannerConfig, MissionResults};
use super::{collector, parser};

/// Scan a single mission directory with configuration
pub async fn scan_mission(
    mission_dir: &Path,
    threads: usize,
    config: &MissionScannerConfig
) -> Result<MissionResults> {
    info!("Scanning mission directory: {}", mission_dir.display());
    debug!("Using {} threads", threads);
    debug!("Configuration: {:?}", config);
    
    // Verify mission directory exists and is readable
    if !mission_dir.exists() {
        return Err(anyhow!("Mission directory does not exist: {}", mission_dir.display()));
    }
    
    if let Err(e) = std::fs::read_dir(mission_dir) {
        return Err(anyhow!("Mission directory is not readable: {} - {}", mission_dir.display(), e));
    }
    
    // Get mission name from directory
    let mission_name = mission_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid mission directory name"))?
        .to_string();
    
    // Find mission files
    let sqm_file = collector::find_mission_file(mission_dir)?;
    let sqf_files = collector::find_script_files(mission_dir, &config.file_extensions)?;
    let cpp_files = collector::find_code_files(mission_dir, &config.file_extensions)?;
    
    if sqm_file.is_none() && sqf_files.is_empty() && cpp_files.is_empty() {
        warn!("No mission files found in {}", mission_dir.display());
        return Ok(MissionResults {
            mission_name,
            mission_dir: mission_dir.to_path_buf(),
            sqm_file: None,
            sqf_files: Vec::new(),
            cpp_files: Vec::new(),
            class_dependencies: Vec::new(),
        });
    }
    
    info!("Found mission files: {} SQM, {} SQF, {} CPP/HPP", 
        if sqm_file.is_some() { 1 } else { 0 },
        sqf_files.len(),
        cpp_files.len());
    
    let mut dependencies = Vec::new();
    
    // Process mission.sqm if present
    if let Some(sqm_file) = &sqm_file {
        debug!("Processing mission.sqm: {}", sqm_file.display());
        match parser::parse_file(sqm_file) {
            Ok(mut deps) => {
                debug!("Found {} dependencies in SQM file", deps.len());
                dependencies.append(&mut deps);
            },
            Err(e) => warn!("Failed to parse SQM file {}: {}", sqm_file.display(), e),
        }
    }
    
    // Process SQF files in parallel
    let sqf_deps: Vec<_> = sqf_files.par_iter()
        .flat_map(|file| {
            debug!("Processing SQF file: {}", file.display());
            parser::parse_file(file).unwrap_or_default()
        })
        .collect();
    dependencies.extend(sqf_deps);
    
    // Process CPP/HPP files in parallel
    let cpp_deps: Vec<_> = cpp_files.par_iter()
        .flat_map(|file| {
            debug!("Processing CPP/HPP file: {}", file.display());
            parser::parse_file(file).unwrap_or_default()
        })
        .collect();
    dependencies.extend(cpp_deps);
    
    debug!("Total of {} dependencies found for mission {}", 
        dependencies.len(), mission_name);
    
    // Log unique class names found
    let unique_classes: std::collections::HashSet<_> = dependencies.iter()
        .map(|d| d.class_name.as_str())
        .collect();
    
    debug!("Unique class names found in {}:", mission_name);
    for class in &unique_classes {
        debug!("  - {}", class);
    }
    
    Ok(MissionResults {
        mission_name,
        mission_dir: mission_dir.to_path_buf(),
        sqm_file,
        sqf_files,
        cpp_files,
        class_dependencies: dependencies,
    })
}