pub mod types;
pub mod analyzer;
pub mod extractor;
pub mod scanner;
pub mod validator;
pub mod database;
pub mod utils;

// Re-export main types and functions for easier access
pub use types::*;
pub use analyzer::{
    MissionAnalyzer,
    types::{ClassDependency, MissionDependencyResult, ReferenceType},
};
pub use extractor::{
    MissionExtractor,
    types::MissionExtractionResult,
};
pub use scanner::MissionScanner;
pub use validator::{
    ClassExistenceValidator,
    types::{ClassExistenceReport, MissionClassExistenceReport, MissingClassInfo},
};
pub use database::{
    MissionDatabase,
    MissionDatabaseStats,
};

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
pub async fn scan_missions(config: MissionScanConfig<'_>) -> Result<Vec<MissionDependencyResult>> {
    info!("Starting mission scanning with configuration:");
    info!("  Input directory: {}", config.input_dir.display());
    info!("  Cache directory: {}", config.cache_dir.display());
    info!("  Output directory: {}", config.output_dir.display());
    info!("  Threads: {}", config.threads);
    
    // Verify input directory exists
    if !config.input_dir.exists() {
        return Err(anyhow::anyhow!("Input directory does not exist: {}", config.input_dir.display()));
    }
    
    // Create cache directory if it doesn't exist
    if !config.cache_dir.exists() {
        info!("Creating cache directory: {}", config.cache_dir.display());
        std::fs::create_dir_all(config.cache_dir)?;
    }
    
    // Create output directory if it doesn't exist
    if !config.output_dir.exists() {
        info!("Creating output directory: {}", config.output_dir.display());
        std::fs::create_dir_all(config.output_dir)?;
    }
    
    // Create a mission scanner
    let mission_scanner = scanner::MissionScanner::new(
        config.input_dir,
        config.cache_dir,
        config.threads
    );
    
    // Scan and extract mission files
    info!("Scanning and extracting mission files");
    let extraction_results = mission_scanner.scan_and_extract().await?;
    
    if extraction_results.is_empty() {
        warn!("No mission files found or extracted");
        return Ok(Vec::new());
    }
    
    info!("Extracted {} mission files", extraction_results.len());
    
    // Analyze mission dependencies
    let dependency_analyzer = analyzer::MissionAnalyzer::new(config.cache_dir);
    let mission_results = dependency_analyzer.analyze_missions(&extraction_results)?;
    
    info!("Analyzed dependencies for {} missions", mission_results.len());
    
    Ok(mission_results)
}

/// Validate classes against mission dependencies
pub fn validate_mission_classes(
    mission_results: &[MissionDependencyResult],
    processed_classes: &[crate::code_scanner::class::types::ProcessedClass]
) -> Result<ClassExistenceReport> {
    info!("Validating classes against mission dependencies");
    
    // Create a class validator
    let mut class_validator = validator::ClassExistenceValidator::new();
    
    // Load class database from memory
    class_validator.load_class_database_from_memory(processed_classes)?;
    
    // Validate mission classes
    let class_existence_report = class_validator.validate_mission_classes(mission_results)?;
    
    info!("Validated {} missions with {} unique classes", 
        class_existence_report.total_missions,
        class_existence_report.total_unique_classes
    );
    
    Ok(class_existence_report)
} 