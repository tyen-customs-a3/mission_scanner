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

use crate::types::SkipReason;
use super::types::{MissionExtractionResult, MissionInfo};

/// Extract mission files
pub fn extract_missions(
    cache_dir: &Path,
    threads: usize,
    mission_files: &[PathBuf],
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
            }
        }
    }

    Ok(extraction_results)
}

/// Extract a single mission file
pub fn extract_single_mission(cache_dir: &Path, pbo_path: &Path) -> Result<MissionExtractionResult> {
    info!("Extracting mission: {}", pbo_path.display());
    
    let start_time = Instant::now();
    
    // Create a unique output directory for this mission
    let mission_name = pbo_path.file_stem()
        .ok_or_else(|| anyhow!("Invalid PBO file name"))?
        .to_string_lossy()
        .to_string();
    
    let output_dir = cache_dir.join(&mission_name);
    
    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }
    
    // Extract the PBO file
    let api = PboApi::new(30);
    
    // Configure extraction options
    let mut options = ExtractOptions::default();
    options.verbose = true;
    options.warnings_as_errors = false;
    options.no_pause = true;
    
    // Extract the PBO file
    api.extract_with_options(pbo_path, &output_dir, options)?;
    
    // Find mission.sqm file
    let sqm_file = find_file_by_extension(&output_dir, "sqm");
    
    // Find all SQF script files
    let sqf_files = find_files_by_extension(&output_dir, "sqf");
    
    // Find all CPP/HPP config files
    let mut cpp_files = find_files_by_extension(&output_dir, "cpp");
    cpp_files.extend(find_files_by_extension(&output_dir, "hpp"));
    
    // Create extraction result
    let result = MissionExtractionResult {
        mission_name,
        pbo_path: pbo_path.to_path_buf(),
        extracted_path: output_dir,
        sqm_file,
        sqf_files,
        cpp_files,
    };
    
    info!("Extracted mission in {} ms", start_time.elapsed().as_millis());
    
    Ok(result)
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

/// Find a file with a specific extension in a directory
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

/// Find all files with a specific extension in a directory
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