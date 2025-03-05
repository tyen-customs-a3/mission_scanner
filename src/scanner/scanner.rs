use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::task;
use futures::future::join_all;
use walkdir::WalkDir;

use crate::mission_scanner::database::MissionDatabase;
use crate::mission_scanner::database::operations::has_mission_changed;
use crate::mission_scanner::extractor::types::MissionExtractionResult;
use crate::mission_scanner::extractor::extractor;
use crate::mission_scanner::types::SkipReason;
use super::collector;

/// Scan and extract mission files
pub async fn scan_and_extract(
    input_dir: &Path,
    cache_dir: &Path,
    threads: usize,
    db: &Arc<Mutex<MissionDatabase>>
) -> Result<Vec<MissionExtractionResult>> {
    info!("Scanning for mission files in {}", input_dir.display());
    
    // Collect mission files
    let mission_files = collector::collect_mission_files(input_dir)?;
    
    if mission_files.is_empty() {
        warn!("No mission files found in {}", input_dir.display());
        return Ok(Vec::new());
    }
    
    info!("Found {} mission files", mission_files.len());
    
    // Create progress bar
    let progress = ProgressBar::new(mission_files.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-")
    );
    progress.set_message("Scanning mission files");
    
    // First, check for cached results
    let cached_results = collect_cached_results(&mission_files, cache_dir, db)?;
    
    if !cached_results.is_empty() {
        info!("Found {} cached mission results", cached_results.len());
        progress.finish_with_message(format!("Processed {} mission files", cached_results.len()));
        return Ok(cached_results);
    }
    
    // Scan mission files
    let scan_results = scan_mission_files(&mission_files, progress.clone(), db)?;
    
    if scan_results.is_empty() {
        warn!("No mission files were successfully scanned");
        progress.finish_with_message("No mission files were successfully scanned");
        return Ok(Vec::new());
    }
    
    info!("Scanned {} mission files", scan_results.len());
    
    // Extract mission files
    let mut extraction_results = Vec::new();
    let mut handles = Vec::new();
    
    // Create a thread pool
    let thread_count = std::cmp::min(threads, scan_results.len());
    info!("Using {} threads for extraction", thread_count);
    
    // Create a progress bar for extraction
    let extract_progress = ProgressBar::new(scan_results.len() as u64);
    extract_progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-")
    );
    extract_progress.set_message("Extracting mission files");
    
    // Process missions in parallel
    for scan_result in scan_results {
        let cache_dir = cache_dir.to_path_buf();
        let db_clone = db.clone();
        let extract_progress_clone = extract_progress.clone();
        
        let handle = task::spawn(async move {
            let result = match extractor::extract_single_mission(&cache_dir, &scan_result) {
                Ok(result) => {
                    info!("Extracted mission: {}", scan_result.path.display());
                    Some(result)
                },
                Err(e) => {
                    warn!("Failed to extract mission {}: {}", scan_result.path.display(), e);
                    let mut db = db_clone.lock().unwrap();
                    db.update_mission_with_reason(
                        &scan_result.path,
                        &scan_result.hash,
                        true,
                        SkipReason::ExtractionFailed,
                    );
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
    
    // Collect successful results
    for result in results {
        if let Ok(Some(extraction_result)) = result {
            extraction_results.push(extraction_result);
        }
    }
    
    extract_progress.finish_with_message(format!("Extracted {} mission files", extraction_results.len()));
    
    // Save the database
    let db_path = cache_dir.join("mission_db.json");
    if let Err(e) = db.lock().unwrap().save(&db_path) {
        warn!("Failed to save mission database: {}", e);
    }
    
    Ok(extraction_results)
}

/// Scan mission files
fn scan_mission_files(
    mission_files: &[PathBuf],
    progress: ProgressBar,
    db: &Arc<Mutex<MissionDatabase>>
) -> Result<Vec<MissionScanResult>> {
    let mut scan_results = Vec::new();
    
    for path in mission_files {
        // Calculate hash of the mission file
        let hash = match calculate_file_hash(path) {
            Ok(hash) => hash,
            Err(e) => {
                warn!("Failed to calculate hash for {}: {}", path.display(), e);
                progress.inc(1);
                continue;
            }
        };
        
        // Check if the mission has changed
        let has_changed = {
            let db = db.lock().unwrap();
            has_mission_changed(&db, path, &hash)
        };
        
        if !has_changed {
            debug!("Mission unchanged: {}", path.display());
            let mut db = db.lock().unwrap();
            db.update_mission_with_reason(
                path,
                &hash,
                false,
                SkipReason::Unchanged,
            );
            progress.inc(1);
            continue;
        }
        
        // Create scan result
        let mission_name = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            
        let scan_result = MissionScanResult {
            mission_name,
            path: path.clone(),
            hash,
        };
        
        scan_results.push(scan_result);
        progress.inc(1);
    }
    
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