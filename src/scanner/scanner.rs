use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::task;
use futures::future::join_all;
use walkdir::WalkDir;

use crate::database::MissionDatabase;
use crate::database::operations::has_mission_changed;
use crate::extractor::types::MissionExtractionResult;
use crate::extractor::extractor;
use crate::types::{SkipReason, MissionScannerConfig};
use super::collector;

/// Scan and extract mission files with configuration
pub async fn scan_and_extract_with_config(
    input_dir: &Path,
    cache_dir: &Path,
    threads: usize,
    db: &Arc<Mutex<MissionDatabase>>,
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
    let scan_results = scan_mission_files_with_config(&mission_files, scan_progress, db, config)?;
    
    if scan_results.is_empty() {
        info!("No new or changed mission files to process");
        return collect_cached_results(&mission_files, cache_dir, db);
    }
    
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
        let db_clone = db.clone();
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
    
    // Add previously processed missions from the database
    if !config.force_rescan {
        for path in &mission_files {
            if !extraction_results.iter().any(|er| er.pbo_path == *path) {
                if let Ok(cached_result) = load_cached_mission(cache_dir, path) {
                    debug!("Using cached result for {}", path.display());
                    extraction_results.push(cached_result);
                }
            }
        }
    }
    
    Ok(extraction_results)
}

/// Scan and extract mission files
pub async fn scan_and_extract(
    input_dir: &Path,
    cache_dir: &Path,
    threads: usize,
    db: &Arc<Mutex<MissionDatabase>>
) -> Result<Vec<MissionExtractionResult>> {
    // Use default config
    let config = MissionScannerConfig::default();
    scan_and_extract_with_config(input_dir, cache_dir, threads, db, &config).await
}

/// Scan mission files with configuration options
fn scan_mission_files_with_config(
    mission_files: &[PathBuf],
    progress: ProgressBar,
    db: &Arc<Mutex<MissionDatabase>>,
    config: &MissionScannerConfig
) -> Result<Vec<MissionScanResult>> {
    let mut scan_results = Vec::new();
    
    for path in mission_files {
        // Calculate hash of the mission file
        let hash = calculate_file_hash(path)?;
        
        // Check if the mission has changed if we're not forcing a rescan
        if !config.force_rescan && config.skip_unchanged {
            let has_changed = {
                let db_guard = db.lock().unwrap();
                has_mission_changed(&db_guard, path, &hash)?
            };
            
            if !has_changed.0 {
                debug!("Mission unchanged: {}", path.display());
                let mut db = db.lock().unwrap();
                db.update_mission_with_reason(
                    path,
                    &hash,
                    false,
                    has_changed.1.unwrap_or(SkipReason::Unchanged)
                );
                
                progress.inc(1);
                continue;
            }
        }
        
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

/// Collect cached results
fn collect_cached_results(
    mission_files: &[PathBuf],
    cache_dir: &Path,
    db: &Arc<Mutex<MissionDatabase>>
) -> Result<Vec<MissionExtractionResult>> {
    let mut cached_results = Vec::new();
    
    for path in mission_files {
        // Check if the mission is in the database
        let mission_info = {
            let db = db.lock().unwrap();
            db.get_mission_info(path).cloned()
        };
        
        if let Some(info) = mission_info {
            if info.processed {
                // Try to load the cached result
                match load_cached_mission(cache_dir, path) {
                    Ok(result) => {
                        cached_results.push(result);
                    },
                    Err(e) => {
                        warn!("Failed to load cached mission {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    Ok(cached_results)
}

/// Load a cached mission
fn load_cached_mission(cache_dir: &Path, mission_path: &Path) -> Result<MissionExtractionResult> {
    let mission_name = mission_path.file_stem()
        .ok_or_else(|| anyhow!("Invalid mission path"))?
        .to_string_lossy()
        .to_string();
        
    let extracted_path = cache_dir.join(&mission_name);
    
    if !extracted_path.exists() {
        return Err(anyhow!("Cached mission directory does not exist"));
    }
    
    // Find mission.sqm file
    let sqm_file = find_file_by_extension(&extracted_path, "sqm");
    
    // Find SQF files
    let sqf_files = find_files_by_extension(&extracted_path, "sqf");
    
    // Find CPP/HPP files
    let mut cpp_files = find_files_by_extension(&extracted_path, "cpp");
    cpp_files.extend(find_files_by_extension(&extracted_path, "hpp"));
    
    Ok(MissionExtractionResult {
        mission_name,
        pbo_path: mission_path.to_path_buf(),
        extracted_path,
        sqm_file,
        sqf_files,
        cpp_files,
    })
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