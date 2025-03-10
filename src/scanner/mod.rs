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
pub use parser_integration::{
    parse_loadout_file,
    parse_sqm_file,
    extract_sqm_dependencies,
    scan_sqf_file,
};

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
            let sqm_deps = extract_sqm_dependencies(sqm_path)?;
            for class_name in sqm_deps {
                dependencies.push(crate::types::ClassDependency {
                    class_name,
                    reference_type: crate::types::ReferenceType::Direct,
                    context: format!("SQM file: {}", sqm_path.file_name().unwrap_or_default().to_string_lossy()),
                });
            }
        }
        
        // Process SQF files
        for sqf_path in &mission.sqf_files {
            if let Ok(references) = scan_sqf_file(sqf_path) {
                for reference in references {
                    dependencies.push(crate::types::ClassDependency {
                        class_name: reference.class_name.clone(),
                        reference_type: crate::types::ReferenceType::Variable,
                        context: format!("SQF file: {}", sqf_path.file_name().unwrap_or_default().to_string_lossy()),
                    });
                }
            }
        }
        
        // Process CPP/HPP files
        for cpp_path in &mission.cpp_files {
            if let Ok(equipment) = parse_loadout_file(cpp_path) {
                for equip in equipment {
                    dependencies.push(crate::types::ClassDependency {
                        class_name: equip.class_name,
                        reference_type: crate::types::ReferenceType::Direct,
                        context: format!("CPP file: {}", cpp_path.file_name().unwrap_or_default().to_string_lossy()),
                    });
                    
                    if let Some(parent) = equip.parent_class {
                        dependencies.push(crate::types::ClassDependency {
                            class_name: parent,
                            reference_type: crate::types::ReferenceType::Inheritance,
                            context: format!("CPP file: {}", cpp_path.file_name().unwrap_or_default().to_string_lossy()),
                        });
                    }
                }
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