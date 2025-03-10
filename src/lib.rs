pub mod extractor;
pub mod scanner;
pub mod utils;
pub mod types;

// Re-export main types and functions for easier access
pub use extractor::{MissionExtractor, types::MissionExtractionResult};
pub use scanner::{MissionScanner, parse_loadout_file, parse_sqm_file, extract_sqm_dependencies, scan_sqf_file};
pub use types::{MissionScanResult, MissionScanStats, SkipReason, MissionScannerConfig, MissionDependencyResult, ClassDependency, ReferenceType};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use log::{info, warn, error};
use std::fs;

/// Process all mission files in a directory
/// 
/// This function handles the complete workflow:
/// 1. Scans for mission files
/// 2. Extracts mission files
/// 3. Analyzes dependencies
/// 
/// # Parameters
/// * `input_dir` - Directory containing mission files
/// * `cache_dir` - Directory for caching extracted files
/// * `output_dir` - Directory for output reports
/// * `config` - Configuration options for scanning
/// 
/// # Returns
/// A list of mission scan results
pub async fn process_mission_directory(
    input_dir: &Path,
    cache_dir: &Path,
    output_dir: &Path,
    config: &MissionScannerConfig,
) -> Result<MissionScanStats> {
    // Verify input directory exists
    if !input_dir.exists() {
        return Err(anyhow!("Input directory does not exist: {}", input_dir.display()));
    }
    
    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        info!("Creating cache directory: {}", cache_dir.display());
        fs::create_dir_all(cache_dir)?;
    }
    
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        info!("Creating output directory: {}", output_dir.display());
        fs::create_dir_all(output_dir)?;
    }
    
    // Create a mission scanner
    let scanner = scanner::MissionScanner::new(
        input_dir,
        cache_dir,
        config.max_threads
    );
    
    // Scan and extract mission files
    info!("Scanning and extracting mission files");
    let extraction_results = scanner.scan_and_extract().await?;
    
    if extraction_results.is_empty() {
        warn!("No mission files found or extracted");
        return Ok(MissionScanStats {
            total: 0,
            processed: 0,
            failed: 0,
            unchanged: 0,
        });
    }
    
    info!("Extracted {} mission files", extraction_results.len());
    
    // TODO: Implement dependency analysis and validation
    return Ok(MissionScanStats {
        total: extraction_results.len(),
        processed: extraction_results.len(),
        failed: 0,
        unchanged: 0,
    })
}

/// Extract dependency information from missions
/// 
/// This function analyzes the extracted mission files and produces
/// a list of all classes that each mission depends on.
/// 
/// # Parameters
/// * `cache_dir` - Directory containing extracted mission files
/// * `missions` - List of mission extraction results to analyze
/// 
/// # Returns
/// A list of mission dependency results
pub fn extract_mission_dependencies(
    cache_dir: &Path,
    missions: &[MissionExtractionResult],
) -> Result<Vec<MissionDependencyResult>> {
    let mut results = Vec::new();
    
    for mission in missions {
        let mut dependencies = Vec::new();
        
        // Process SQM file if available
        if let Some(sqm_path) = &mission.sqm_file {
            let sqm_deps = extract_sqm_dependencies(sqm_path)?;
            for class_name in sqm_deps {
                dependencies.push(ClassDependency {
                    class_name,
                    reference_type: ReferenceType::Direct,
                    context: format!("SQM file: {}", sqm_path.file_name().unwrap_or_default().to_string_lossy()),
                });
            }
        }
        
        // Process SQF files
        for sqf_path in &mission.sqf_files {
            if let Ok(references) = scan_sqf_file(sqf_path) {
                for reference in references {
                    dependencies.push(ClassDependency {
                        class_name: reference.class_name,
                        reference_type: ReferenceType::Variable,
                        context: format!("SQF file: {}", sqf_path.file_name().unwrap_or_default().to_string_lossy()),
                    });
                }
            }
        }
        
        // Process CPP/HPP files
        for cpp_path in &mission.cpp_files {
            if let Ok(equipment) = parse_loadout_file(cpp_path) {
                for equip in equipment {
                    dependencies.push(ClassDependency {
                        class_name: equip.class_name,
                        reference_type: ReferenceType::Direct,
                        context: format!("CPP file: {}", cpp_path.file_name().unwrap_or_default().to_string_lossy()),
                    });
                    
                    if let Some(parent) = equip.parent_class {
                        dependencies.push(ClassDependency {
                            class_name: parent,
                            reference_type: ReferenceType::Inheritance,
                            context: format!("CPP file: {}", cpp_path.file_name().unwrap_or_default().to_string_lossy()),
                        });
                    }
                }
            }
        }
        
        results.push(MissionDependencyResult {
            mission_name: mission.mission_name.clone(),
            mission_path: mission.pbo_path.clone(),
            class_dependencies: dependencies,
        });
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio;

    fn setup_test_dirs() -> (TempDir, TempDir) {
        let cache_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();
        (cache_dir, output_dir)
    }

    #[tokio::test]
    async fn test_process_mission_directory() -> Result<()> {
        let (cache_dir, output_dir) = setup_test_dirs();
        
        let config = MissionScannerConfig {
            max_threads: 4,
            force_rescan: false,
            skip_unchanged: true,
            file_extensions: vec!["pbo".to_string()],
            recursive: true,
        };

        // Test with test_mission_1 and test_mission_2 directories
        let input_dir = Path::new("test_data");
        let stats = process_mission_directory(
            input_dir,
            cache_dir.path(),
            output_dir.path(),
            &config
        ).await?;

        assert!(stats.total > 0, "Should find at least one mission");
        assert!(stats.processed > 0, "Should process at least one mission");
        assert_eq!(stats.failed, 0, "Should not have any failed missions");

        Ok(())
    }

    #[test]
    fn test_extract_mission_dependencies() -> Result<()> {
        let (cache_dir, _) = setup_test_dirs();

        // Create test mission extraction results
        let missions = vec![
            MissionExtractionResult {
                mission_name: "test_mission_1".to_string(),
                pbo_path: PathBuf::from("test_data/test_mission_1.pbo"),
                extracted_path: PathBuf::from("test_data/test_mission_1"),
                sqm_file: Some(PathBuf::from("test_data/test_mission_1/mission.sqm")),
                sqf_files: vec![PathBuf::from("test_data/test_mission_1/init.sqf")],
                cpp_files: vec![PathBuf::from("test_data/test_mission_1/description.ext")],
            },
            MissionExtractionResult {
                mission_name: "test_mission_2".to_string(),
                pbo_path: PathBuf::from("test_data/test_mission_2.pbo"),
                extracted_path: PathBuf::from("test_data/test_mission_2"),
                sqm_file: Some(PathBuf::from("test_data/test_mission_2/mission.sqm")),
                sqf_files: vec![],
                cpp_files: vec![],
            }
        ];

        let results = extract_mission_dependencies(cache_dir.path(), &missions)?;

        assert_eq!(results.len(), 2, "Should have results for both missions");
        assert_eq!(results[0].mission_name, "test_mission_1");
        assert_eq!(results[1].mission_name, "test_mission_2");

        Ok(())
    }
}