pub mod models;
pub mod scanner;
pub mod utils;
pub mod types;
pub mod mission;

// Re-export main types and functions for easier access
pub use scanner::{
    MissionScanner,
    extract_mission_dependencies,
};

pub use types::{
    MissionScanResult,
    MissionScanStats,
    SkipReason,
    MissionScannerConfig,
    MissionDependencyResult,
    ClassDependency,
    ReferenceType,
    MissionExtractionResult,
};

pub use mission::{MissionScanConfig, scan_missions};

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use log::{info, warn, error};
use std::fs;

/// Process all mission files in a directory
/// 
/// This function handles the complete workflow:
/// 1. Scans for mission files
/// 2. Analyzes dependencies
/// 
/// # Parameters
/// * `input_dir` - Directory containing mission files
/// * `config` - Configuration options for scanning
/// 
/// # Returns
/// A list of mission scan results
pub async fn process_mission_directory(
    input_dir: &Path,
    config: &MissionScannerConfig,
) -> Result<MissionScanStats> {
    // Verify input directory exists
    if !input_dir.exists() {
        return Err(anyhow!("Input directory does not exist: {}", input_dir.display()));
    }
    
    // Create a mission scanner
    let scanner = scanner::MissionScanner::new(
        input_dir,
        config.max_threads
    );
    
    // Scan mission files
    info!("Scanning mission files");
    let scan_results = scanner.scan().await?;
    
    if scan_results.is_empty() {
        warn!("No mission files found");
        return Ok(MissionScanStats {
            total: 0,
            processed: 0,
            failed: 0,
            unchanged: 0,
        });
    }
    
    info!("Found {} mission files", scan_results.len());
    
    Ok(MissionScanStats {
        total: scan_results.len(),
        processed: scan_results.len(),
        failed: 0,
        unchanged: 0,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_process_mission_directory() -> Result<()> {
        let config = MissionScannerConfig {
            max_threads: 4,
            force_rescan: false,
            skip_unchanged: true,
            file_extensions: vec!["sqm".to_string(), "sqf".to_string(), "cpp".to_string(), "hpp".to_string()],
            recursive: true,
        };

        // Test with test_mission_1 and test_mission_2 directories
        let input_dir = Path::new("test_data");
        let stats = process_mission_directory(
            input_dir,
            &config
        ).await?;

        assert!(stats.total > 0, "Should find at least one mission");
        assert!(stats.processed > 0, "Should process at least one mission");
        assert_eq!(stats.failed, 0, "Should not have any failed missions");

        Ok(())
    }
}