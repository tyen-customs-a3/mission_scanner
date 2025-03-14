use std::path::PathBuf;
use anyhow::Result;
use mission_scanner::{
    scan_mission,
    MissionScannerConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Example 1: Scan a single mission
    println!("\n=== Scanning a single mission ===");
    let mission_dir = PathBuf::from("path/to/your/mission");
    let config = MissionScannerConfig::default();
    
    match scan_mission(&mission_dir, num_cpus::get(), &config).await {
        Ok(result) => {
            println!("Mission: {}", result.mission_name);
            println!("Found {} SQF files", result.sqf_files.len());
            println!("Found {} CPP/HPP files", result.cpp_files.len());
            println!("Found {} class dependencies", result.class_dependencies.len());
            
            // Print some example dependencies
            println!("\nExample dependencies:");
            for dep in result.class_dependencies.iter().take(5) {
                println!("  - {} ({:?})", dep.class_name, dep.reference_type);
            }
        }
        Err(e) => println!("Error scanning mission: {}", e),
    }

    // Example 2: Scan multiple missions with custom configuration
    println!("\n=== Scanning multiple missions ===");
    let missions_dir = PathBuf::from("path/to/your/missions");
    let mut config = MissionScannerConfig::default();
    config.file_extensions = vec!["sqm".to_string(), "sqf".to_string()]; // Only scan SQM and SQF files
    config.max_threads = 4; // Use 4 threads for parallel processing

    // Scan each mission directory
    let entries = std::fs::read_dir(&missions_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            match scan_mission(&path, config.max_threads, &config).await {
                Ok(result) => {
                    println!("\nMission: {}", result.mission_name);
                    println!("  SQF files: {}", result.sqf_files.len());
                    println!("  Dependencies: {}", result.class_dependencies.len());
                    
                    // Count reference types
                    let mut ref_types = std::collections::HashMap::new();
                    for dep in &result.class_dependencies {
                        *ref_types.entry(&dep.reference_type).or_insert(0) += 1;
                    }
                    
                    println!("  Reference types:");
                    for (ref_type, count) in ref_types {
                        println!("    - {:?}: {}", ref_type, count);
                    }
                }
                Err(e) => println!("Error scanning mission {}: {}", path.display(), e),
            }
        }
    }

    Ok(())
} 