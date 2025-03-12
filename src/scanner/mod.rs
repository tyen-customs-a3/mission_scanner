#[cfg(test)]
mod tests;

mod collector;
mod scanner;
mod parser_integration;

use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

use crate::types::{MissionExtractionResult, MissionScannerConfig};

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

/// Extract dependency information from missions
pub fn extract_mission_dependencies(
    missions: &[MissionExtractionResult],
) -> Result<Vec<crate::types::MissionDependencyResult>> {
    let mut results = Vec::new();
    
    for mission in missions {
        let mut dependencies = Vec::new();
        
        // Process SQM file if available
        if let Some(sqm_path) = &mission.sqm_file {
            if let Ok(deps) = parse_file(sqm_path) {
                dependencies.extend(deps);
            }
        }
        
        // Process SQF files
        for sqf_path in &mission.sqf_files {
            if let Ok(deps) = parse_file(sqf_path) {
                dependencies.extend(deps);
            }
        }
        
        // Process CPP/HPP files
        for cpp_path in &mission.cpp_files {
            if let Ok(deps) = parse_file(cpp_path) {
                dependencies.extend(deps);
            }
        }
        
        results.push(crate::types::MissionDependencyResult {
            mission_name: mission.mission_name.clone(),
            mission_path: mission.pbo_path.clone(),
            class_dependencies: dependencies,
        });
    }
    
    Ok(results)
} 