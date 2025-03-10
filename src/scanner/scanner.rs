use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::task;
use futures::future::join_all;
use walkdir::WalkDir;

use crate::types::{MissionExtractionResult, MissionScannerConfig};
use super::collector;
use crate::utils::{find_file_by_extension, find_files_by_extension};

/// Scan mission files with configuration
pub async fn scan_with_config(
    input_dir: &Path,
    threads: usize,
    config: &MissionScannerConfig
) -> Result<Vec<MissionExtractionResult>> {
    info!("Scanning for mission files in {} with configuration", input_dir.display());
    debug!("Using {} threads", threads);
    debug!("Configuration: {:?}", config);
    
    // Verify input directory exists and is readable
    if !input_dir.exists() {
        return Err(anyhow!("Input directory does not exist: {}", input_dir.display()));
    }
    
    if let Err(e) = std::fs::read_dir(input_dir) {
        return Err(anyhow!("Input directory is not readable: {} - {}", input_dir.display(), e));
    }
    
    // Collect mission files
    let mission_files = if config.recursive {
        collector::collect_mission_files_with_config(input_dir, config)?
    } else {
        collector::collect_mission_files(input_dir)?
    };
    
    if mission_files.is_empty() {
        warn!("No mission files found in {}", input_dir.display());
        return Ok(Vec::new());
    }
    
    info!("Found {} mission files", mission_files.len());
    
    // Set up progress bars
    let multi_progress = MultiProgress::new();
    let scan_progress = multi_progress.add(ProgressBar::new(mission_files.len() as u64));
    scan_progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    scan_progress.set_message("Scanning mission files");
    
    // Log each mission file found
    for mission in &mission_files {
        debug!("Found mission: {} at {}", mission.mission_name, mission.pbo_path.display());
        if let Some(sqm) = &mission.sqm_file {
            debug!("  SQM file: {}", sqm.display());
        }
        debug!("  SQF files: {}", mission.sqf_files.len());
        debug!("  CPP files: {}", mission.cpp_files.len());
    }
    
    scan_progress.finish_with_message(format!("Scanned {} missions", mission_files.len()));
    
    Ok(mission_files)
}

/// Scan mission files with default configuration
pub async fn scan(
    input_dir: &Path,
    threads: usize,
) -> Result<Vec<MissionExtractionResult>> {
    // Use default config
    let config = MissionScannerConfig::default();
    scan_with_config(input_dir, threads, &config).await
}

/// Scan mission files with configuration options
fn scan_mission_files_with_config(
    mission_files: &[PathBuf],
    progress: ProgressBar,
    config: &MissionScannerConfig
) -> Result<Vec<MissionScanResult>> {
    let mut scan_results = Vec::new();
    
    for path in mission_files {
        // Mission needs to be processed
        let mission_name = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let scan_result = MissionScanResult {
            mission_name,
            path: path.to_path_buf(),
        };
        
        scan_results.push(scan_result);
        progress.inc(1);
    }
    
    progress.finish_with_message(format!("Scanned {} missions", mission_files.len()));
    
    Ok(scan_results)
}

/// Mission scan result
#[derive(Debug, Clone)]
struct MissionScanResult {
    /// Name of the mission
    mission_name: String,
    /// Path to the mission file
    path: PathBuf,
} 