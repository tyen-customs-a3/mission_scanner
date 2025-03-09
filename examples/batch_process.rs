use std::path::Path;
use anyhow::Result;
use mission_scanner::{
    process_mission_directory,
    extract_mission_dependencies,
    MissionScannerConfig
};
use log::{info, warn, error, LevelFilter};
use env_logger::Builder;
use std::io::Write;
use std::time::Instant;

// This example demonstrates how to batch process a directory of mission files
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
    
    // Create configuration
    let config = MissionScannerConfig {
        max_threads: num_cpus::get(),
        force_rescan: false,
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
    
    // Process all missions in the directory
    info!("Starting mission batch processing");
    
    // Step 1: Scan and extract missions
    let stats = process_mission_directory(
        input_dir,
        cache_dir,
        output_dir,
        &config
    ).await?;
    
    info!("Mission scanning complete:");
    info!("  Total missions: {}", stats.total);
    info!("  Processed: {}", stats.processed);
    info!("  Failed: {}", stats.failed);
    info!("  Unchanged: {}", stats.unchanged);
    
    // Calculate processing time
    let duration = start_time.elapsed();
    info!("Processing completed in {:.2} seconds", duration.as_secs_f64());
    
    Ok(())
} 