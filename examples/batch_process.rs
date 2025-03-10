use std::path::Path;
use anyhow::Result;
use log::{info, warn, error};
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use mission_scanner::types::MissionScannerConfig;

// This example demonstrates how to batch process a directory of mission files
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_dir>", args[0]);
        std::process::exit(1);
    }
    
    let input_dir = Path::new(&args[1]);
    
    // Create configuration
    let config = MissionScannerConfig {
        max_threads: 4,
        force_rescan: false,
        skip_unchanged: true,
        file_extensions: vec!["sqm".to_string(), "sqf".to_string(), "cpp".to_string(), "hpp".to_string()],
        recursive: true,
    };
    
    // Process mission files
    info!("Starting mission processing");
    
    let stats = mission_scanner::process_mission_directory(
        input_dir,
        &config,
    ).await?;
    
    info!("Processing complete:");
    info!("  Total missions: {}", stats.total);
    info!("  Processed: {}", stats.processed);
    info!("  Failed: {}", stats.failed);
    info!("  Unchanged: {}", stats.unchanged);
    
    Ok(())
} 