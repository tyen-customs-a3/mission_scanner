use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::task;
use futures::future::join_all;
use walkdir::WalkDir;

use crate::extractor::types::MissionExtractionResult;
use crate::extractor::extractor;
use crate::types::{SkipReason, MissionScannerConfig};
use super::collector;

/// Scan and extract mission files with configuration
pub async fn scan_and_extract_with_config(
    input_dir: &Path,
    cache_dir: &Path,
    threads: usize,
    config: &MissionScannerConfig
) -> Result<Vec<MissionExtractionResult>> {
    info!("Scanning for mission files in {} with configuration", input_dir.display());
    
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
    
    // Filter missions that need to be processed based on config
    let scan_results = scan_mission_files_with_config(&mission_files, scan_progress, config)?;
    
    info!("Processing {} missions", scan_results.len());
    
    // Extract mission files in parallel
    let extract_progress = multi_progress.add(ProgressBar::new(scan_results.len() as u64));
    extract_progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    extract_progress.set_message("Extracting mission files");
    
    let mut handles = Vec::new();
    let mut unchanged_count = 0;
    
    // Process missions in parallel
    for scan_result in scan_results {
        let cache_dir = cache_dir.to_path_buf();
        let extract_progress_clone = extract_progress.clone();
        let config_clone = config.clone();
        
        let handle = task::spawn(async move {
            let result = match extractor::extract_single_mission(&cache_dir, &scan_result.path) {
                Ok(result) => {
                    info!("Extracted mission: {}", scan_result.path.display());
                    Some(result)
                },
                Err(e) => {
                    error!("Failed to extract mission {}: {}", scan_result.path.display(), e);
                    None
                }
            };
            
            extract_progress_clone.inc(1);
            result
        });
        
        handles.push(handle);
    }
    
    // Wait for all extraction tasks to complete
    let results = join_all(handles).await;
    
    // Process results
    let mut extraction_results = Vec::new();
    
    for result in results {
        if let Ok(Some(extraction_result)) = result {
            extraction_results.push(extraction_result);
        }
    }
    
    info!("Extracted {} mission files", extraction_results.len());
    extract_progress.finish_with_message(format!("Extracted {} missions", extraction_results.len()));
    
    Ok(extraction_results)
}

/// Scan and extract mission files
pub async fn scan_and_extract(
    input_dir: &Path,
    cache_dir: &Path,
    threads: usize,
) -> Result<Vec<MissionExtractionResult>> {
    // Use default config
    let config = MissionScannerConfig::default();
    scan_and_extract_with_config(input_dir, cache_dir, threads, &config).await
}

/// Scan mission files with configuration options
fn scan_mission_files_with_config(
    mission_files: &[PathBuf],
    progress: ProgressBar,
    config: &MissionScannerConfig
) -> Result<Vec<MissionScanResult>> {
    let mut scan_results = Vec::new();
    
    for path in mission_files {
        // Calculate hash of the mission file
        let hash = calculate_file_hash(path)?;
        
        // Mission needs to be processed
        let mission_name = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let scan_result = MissionScanResult {
            mission_name,
            path: path.to_path_buf(),
            hash,
        };
        
        scan_results.push(scan_result);
        progress.inc(1);
    }
    
    progress.finish_with_message(format!("Scanned {} missions", mission_files.len()));
    
    Ok(scan_results)
}

/// Calculate hash of a file
fn calculate_file_hash(path: &Path) -> Result<String> {
    use std::fs::File;
    use std::io::Read;
    use sha2::{Sha256, Digest};
    
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = hasher.finalize();
    
    Ok(format!("{:x}", hash))
}

/// Find a file with a specific extension
fn find_file_by_extension(dir: &Path, extension: &str) -> Option<PathBuf> {
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
fn find_files_by_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
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

/// Mission scan result
#[derive(Debug, Clone)]
struct MissionScanResult {
    /// Name of the mission
    mission_name: String,
    /// Path to the mission file
    path: PathBuf,
    /// Hash of the mission file
    hash: String,
} 