#[cfg(test)]
mod tests;

mod collector;
mod scanner;
mod parser_integration;

use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error, debug};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use hemtt_workspace::WorkspacePath;

use crate::types::{MissionExtractionResult, MissionScannerConfig, ClassDependency, MissionDependencyResult};

// Re-export parsing functions for easier access
pub use parser_integration::parse_file;

// Re-export scanner functionality
pub use scanner::{scan_with_config, scan};

// Re-export collector functionality
pub use collector::{collect_mission_files, collect_mission_files_with_config};

/// Scanner for mission files
pub struct MissionScanner<'a> {
    /// Directory containing mission files to scan
    input_dir: &'a Path,
    /// Number of parallel threads to use
    threads: usize,
    /// Configuration options
    config: MissionScannerConfig,
}

impl<'a> MissionScanner<'a> {
    /// Create a new mission scanner
    pub fn new(
        input_dir: &'a Path,
        threads: usize,
    ) -> Self {
        Self {
            input_dir,
            threads,
            config: MissionScannerConfig::default(),
        }
    }
    
    /// Create a new mission scanner with custom configuration
    pub fn with_config(
        input_dir: &'a Path,
        config: MissionScannerConfig,
    ) -> Self {
        Self {
            input_dir,
            threads: config.max_threads,
            config,
        }
    }
    
    /// Scan mission files
    pub async fn scan(&self) -> Result<Vec<MissionExtractionResult>> {
        scanner::scan_with_config(
            self.input_dir,
            self.threads,
            &self.config,
        ).await
    }
}

/// Convert MissionExtractionResult to MissionDependencyResult
fn convert_to_dependency_result(result: MissionExtractionResult) -> MissionDependencyResult {
    MissionDependencyResult {
        mission_name: result.mission_name,
        mission_path: result.mission_dir,
        class_dependencies: result.class_dependencies,
    }
}

/// Extract class dependencies from mission files with a provided workspace path
///
/// # Arguments
/// * `missions` - List of missions to process
/// * `workspace` - Workspace path for enhanced parsing configuration
///
/// # Returns
/// * `Result<Vec<MissionDependencyResult>>` - List of results with dependencies
pub fn extract_mission_dependencies(
    missions: &[MissionExtractionResult],
) -> Result<Vec<MissionDependencyResult>> {
    let mut results = Vec::new();
    
    for mission in missions {
        debug!("Processing mission: {}", mission.mission_name);
        let mut dependencies = Vec::new();
        
        // Process mission.sqm if present
        if let Some(sqm_file) = &mission.sqm_file {
            debug!("Processing mission.sqm: {}", sqm_file.display());
            match parser_integration::parse_file(sqm_file) {
                Ok(mut deps) => {
                    debug!("Found {} dependencies in SQM file", deps.len());
                    dependencies.append(&mut deps);
                },
                Err(e) => warn!("Failed to parse SQM file {}: {}", sqm_file.display(), e),
            }
        }
        
        // Process SQF files
        debug!("Processing {} SQF files", mission.sqf_files.len());
        for file in &mission.sqf_files {
            debug!("Processing SQF file: {}", file.display());
            match parser_integration::parse_file(file) {
                Ok(mut deps) => dependencies.append(&mut deps),
                Err(e) => warn!("Failed to parse SQF file {}: {}", file.display(), e),
            }
        }
        
        // Process CPP/HPP files
        debug!("Processing {} CPP/HPP files", mission.cpp_files.len());
        for file in &mission.cpp_files {
            debug!("Processing CPP/HPP file: {}", file.display());
            match parser_integration::parse_file(file) {
                Ok(mut deps) => dependencies.append(&mut deps),
                Err(e) => warn!("Failed to parse CPP/HPP file {}: {}", file.display(), e),
            }
        }
        
        debug!("Total of {} dependencies found for mission {}", 
            dependencies.len(), mission.mission_name);
        
        // Log unique class names found
        let unique_classes: std::collections::HashSet<_> = dependencies.iter()
            .map(|d| d.class_name.as_str())
            .collect();
        
        debug!("Unique class names found in {}:", mission.mission_name);
        for class in &unique_classes {
            debug!("  - {}", class);
        }
        
        // Convert to MissionDependencyResult
        results.push(MissionDependencyResult {
            mission_name: mission.mission_name.clone(),
            mission_path: mission.mission_dir.clone(),
            class_dependencies: dependencies,
        });
    }
    
    debug!("Completed dependency extraction for {} missions", missions.len());
    Ok(results)
}