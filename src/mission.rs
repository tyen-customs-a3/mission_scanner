use std::path::Path;
use anyhow::Result;
use log::{info, warn};

/// Configuration for the mission scanning process
#[derive(Debug, Clone)]
pub struct MissionScanConfig<'a> {
    /// Directory containing mission files to scan
    pub input_dir: &'a Path,
    /// Directory for caching extraction results
    pub cache_dir: &'a Path,
    /// Directory for output reports
    pub output_dir: &'a Path,
    /// Number of parallel threads to use
    pub threads: usize,
}

/// Main entry point for mission scanning functionality
pub async fn scan_missions(config: MissionScanConfig<'_>) -> Result<Vec<crate::types::MissionDependencyResult>> {
    info!("Starting mission scanning with configuration:");
    info!("  Input directory: {}", config.input_dir.display());
    info!("  Cache directory: {}", config.cache_dir.display());
    info!("  Output directory: {}", config.output_dir.display());
    info!("  Threads: {}", config.threads);
    
    // Verify directories exist or create them
    verify_or_create_directories(&config)?;
    
    // Create a mission scanner with default configuration
    let scanner_config = crate::types::MissionScannerConfig {
        max_threads: config.threads,
        force_rescan: false,
        skip_unchanged: true,
        file_extensions: vec!["sqm".to_string(), "sqf".to_string(), "cpp".to_string(), "hpp".to_string()],
        recursive: true,
    };
    
    let mission_scanner = crate::scanner::MissionScanner::with_config(
        config.input_dir,
        scanner_config,
    );
    
    // Scan and extract mission files
    info!("Scanning and extracting mission files");
    let extraction_results = mission_scanner.scan().await?;
    
    if extraction_results.is_empty() {
        warn!("No mission files found or extracted");
        return Ok(Vec::new());
    }
    
    info!("Found {} mission files", extraction_results.len());
    
    // Extract dependencies from the missions
    let dependency_results = crate::scanner::extract_mission_dependencies(&extraction_results)?;
    
    info!("Analyzed dependencies for {} missions", dependency_results.len());
    
    Ok(dependency_results)
}

/// Verify that required directories exist or create them
fn verify_or_create_directories(config: &MissionScanConfig<'_>) -> Result<()> {
    // Verify input directory exists
    if !config.input_dir.exists() {
        return Err(anyhow::anyhow!("Input directory does not exist: {}", config.input_dir.display()));
    }
    
    // Create cache directory if it doesn't exist
    if !config.cache_dir.exists() {
        info!("Creating cache directory: {}", config.cache_dir.display());
        std::fs::create_dir_all(config.cache_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {} - {}", config.cache_dir.display(), e))?;
    }
    
    // Create output directory if it doesn't exist
    if !config.output_dir.exists() {
        info!("Creating output directory: {}", config.output_dir.display());
        std::fs::create_dir_all(config.output_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create output directory: {} - {}", config.output_dir.display(), e))?;
    }
    
    // Verify directories are writable
    let test_file_cache = config.cache_dir.join(".test_write");
    let test_file_output = config.output_dir.join(".test_write");
    
    std::fs::write(&test_file_cache, "test")
        .map_err(|e| anyhow::anyhow!("Cache directory is not writable: {} - {}", config.cache_dir.display(), e))?;
    std::fs::write(&test_file_output, "test")
        .map_err(|e| anyhow::anyhow!("Output directory is not writable: {} - {}", config.output_dir.display(), e))?;
        
    // Clean up test files
    let _ = std::fs::remove_file(test_file_cache);
    let _ = std::fs::remove_file(test_file_output);
    
    Ok(())
} 