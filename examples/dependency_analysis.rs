use std::path::Path;
use anyhow::Result;
use mission_scanner::{
    MissionScanner,
    extract_mission_dependencies,
    validate_mission_dependencies,
    MissionScannerConfig,
    parse_loadout_file
};
use log::{info, warn, error, LevelFilter};
use env_logger::Builder;
use std::io::Write;
use std::time::Instant;
use serde_json;

// This example demonstrates how to extract and analyze dependencies from mission files
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    
    // Define paths
    let input_dir = Path::new("path/to/missions");
    let cache_dir = Path::new("path/to/cache");
    let output_dir = Path::new("path/to/output");
    let class_db_file = Path::new("path/to/classdb.cpp");
    
    // Create configuration
    let config = MissionScannerConfig {
        max_threads: num_cpus::get(),
        force_rescan: false,
        skip_validation: false,
        skip_unchanged: true,
        file_extensions: vec![
            "sqm".to_string(),
            "sqf".to_string(),
            "cpp".to_string(),
            "hpp".to_string()
        ],
        recursive: true,
    };
    
    // Start timing
    let start_time = Instant::now();
    
    // Step 1: Create a scanner and scan missions
    info!("Creating mission scanner");
    let scanner = MissionScanner::with_config(
        input_dir,
        cache_dir,
        config.clone()
    );
    
    info!("Scanning and extracting missions");
    let extraction_results = scanner.scan_and_extract().await?;
    info!("Extracted {} mission files", extraction_results.len());
    
    // Step 2: Extract dependencies from the missions
    info!("Extracting mission dependencies");
    let dependencies = extract_mission_dependencies(cache_dir, &extraction_results)?;
    info!("Extracted dependencies for {} missions", dependencies.len());
    
    // Step 3: Load class database for validation
    info!("Loading class database from {}", class_db_file.display());
    let class_db = if class_db_file.exists() {
        match parse_loadout_file(class_db_file) {
            Ok(classes) => {
                info!("Loaded {} classes from database", classes.len());
                classes
            },
            Err(e) => {
                warn!("Failed to load class database: {}", e);
                Vec::new()
            }
        }
    } else {
        warn!("Class database file not found");
        Vec::new()
    };
    
    // Step 4: Validate dependencies against class database
    if !class_db.is_empty() {
        info!("Validating mission dependencies against class database");
        let validation_report = validate_mission_dependencies(&dependencies, &class_db)?;
        
        info!("Validation complete:");
        info!("  Total missions: {}", validation_report.total_missions);
        info!("  Total unique classes: {}", validation_report.total_unique_classes);
        info!("  Existing classes: {}", validation_report.existing_classes);
        info!("  Missing classes: {}", validation_report.missing_classes);
        info!("  Existence percentage: {:.2}%", validation_report.existence_percentage);
        
        // Export validation report
        let report_path = output_dir.join("validation_report.json");
        let json = serde_json::to_string_pretty(&validation_report)?;
        std::fs::write(&report_path, json)?;
        info!("Exported validation report to {}", report_path.display());
    }
    
    // Step 5: Export dependency information
    let dependency_path = output_dir.join("dependencies.json");
    let dependency_json = serde_json::to_string_pretty(&dependencies)?;
    std::fs::write(&dependency_path, dependency_json)?;
    info!("Exported dependency information to {}", dependency_path.display());
    
    // Step 6: Export metadata about extracted files
    scanner.export_file_metadata(&output_dir.join("file_metadata.json"), &extraction_results)?;
    
    // Step 7: Save database
    scanner.save_database()?;
    
    // Calculate processing time
    let duration = start_time.elapsed();
    info!("Processing completed in {:.2} seconds", duration.as_secs_f64());
    
    Ok(())
} 