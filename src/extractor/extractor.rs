use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;
use rayon::prelude::*;
use pbo_tools::core::api::{PboApi, PboApiOps};
use pbo_tools::extract::ExtractOptions;

use crate::mission_scanner::database::MissionDatabase;
use crate::mission_scanner::types::SkipReason;
use super::types::{MissionExtractionResult, MissionInfo};

/// Extract mission files
pub fn extract_missions(
    cache_dir: &Path,
    threads: usize,
    mission_files: &[PathBuf],
    db: Arc<Mutex<MissionDatabase>>,
    progress: ProgressBar,
) -> Result<Vec<MissionExtractionResult>> {
    info!("Extracting {} mission files", mission_files.len());
    
    // Create a thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;
    
    // Process missions in parallel
    let results: Vec<Result<MissionExtractionResult>> = pool.install(|| {
        mission_files.par_iter()
            .map(|path| {
                let result = extract_single_mission(cache_dir, path);
                progress.inc(1);
                result
            })
            .collect()
    });
    
    // Collect successful results
    let mut extraction_results = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(extraction_result) => {
                extraction_results.push(extraction_result);
            },
            Err(e) => {
                let path = &mission_files[i];
                warn!("Failed to extract mission {}: {}", path.display(), e);
                
                // Calculate hash of the mission file
                let hash = match calculate_file_hash(path) {
                    Ok(hash) => hash,
                    Err(e) => {
                        warn!("Failed to calculate hash for {}: {}", path.display(), e);
                        continue;
                    }
                };
                
                // Update the database
                let mut db = db.lock().unwrap();
                db.update_mission_with_reason(
                    path,
                    &hash,
                    true,
                    SkipReason::ExtractionFailed,
                );
            }
        }
    }
    
    // Save the database
    let db_path = cache_dir.join("mission_db.json");
    if let Err(e) = db.lock().unwrap().save(&db_path) {
        warn!("Failed to save mission database: {}", e);
    }
    
    Ok(extraction_results)
}

/// Extract a single mission
pub fn extract_single_mission(cache_dir: &Path, pbo_path: &Path) -> Result<MissionExtractionResult> {
    info!("Extracting mission: {}", pbo_path.display());
    
    let start_time = Instant::now();
    
    // Get the mission name from the PBO filename
    let mission_name = pbo_path.file_stem()
        .ok_or_else(|| anyhow!("Invalid PBO path"))?
        .to_string_lossy()
        .to_string();
    
    // Create the output directory
    let output_dir = cache_dir.join(&mission_name);
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }
    
    // Extract the PBO file
    let api = PboApi::new(Some(30))?;
    
    // Configure extraction options
    let mut options = ExtractOptions::default();
    options.verbose = true;
    options.treat_warnings_as_errors = false;
    options.pause_on_error = false;
    
    // Extract the PBO file
    api.extract_with_options(pbo_path, &output_dir, options)?;
    
    // Check if the extraction was successful
    let extracted_files_on_disk = WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
        
    if extracted_files_on_disk == 0 {
        return Err(anyhow!("No files were extracted from the PBO"));
    }
    
    // Find mission.sqm file
    let sqm_file = find_file_by_extension(&output_dir, "sqm");
    
    // Find SQF files
    let sqf_files = find_files_by_extension(&output_dir, "sqf");
    
    // Find CPP/HPP files
    let mut cpp_files = find_files_by_extension(&output_dir, "cpp");
    cpp_files.extend(find_files_by_extension(&output_dir, "hpp"));
    
    let extraction_time = start_time.elapsed().as_millis() as u64;
    
    info!("Extracted mission {} in {}ms", mission_name, extraction_time);
    info!("  SQM file: {}", sqm_file.as_ref().map_or("None".to_string(), |p| p.display().to_string()));
    info!("  SQF files: {}", sqf_files.len());
    info!("  CPP/HPP files: {}", cpp_files.len());
    
    Ok(MissionExtractionResult {
        mission_name,
        pbo_path: pbo_path.to_path_buf(),
        extracted_path: output_dir,
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