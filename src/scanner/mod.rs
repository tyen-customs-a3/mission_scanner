mod collector;
mod scanner;
mod parser_integration;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

use crate::database::MissionDatabase;
use crate::extractor::types::MissionExtractionResult;
use crate::types::MissionScannerConfig;

// Re-export parsing functions for easier access
pub use parser_integration::{
    parse_loadout_file,
    parse_sqm_file,
    extract_sqm_dependencies,
    scan_sqf_file,
};

/// Scanner for mission files
pub struct MissionScanner<'a> {
    /// Directory containing mission files to scan
    input_dir: &'a Path,
    /// Directory for caching extraction results
    cache_dir: &'a Path,
    /// Number of parallel threads to use
    threads: usize,
    /// Database for storing scan results
    db: Arc<Mutex<MissionDatabase>>,
    /// Configuration options
    config: MissionScannerConfig,
}

impl<'a> MissionScanner<'a> {
    /// Create a new mission scanner
    pub fn new(
        input_dir: &'a Path,
        cache_dir: &'a Path,
        threads: usize,
    ) -> Self {
        // Load or create the database
        let db_path = cache_dir.join("mission_database.json");
        let db = match MissionDatabase::load_or_create(&db_path) {
            Ok(db) => {
                info!("Loaded mission database from {}", db_path.display());
                db
            },
            Err(e) => {
                warn!("Failed to load mission database, creating new one: {}", e);
                MissionDatabase::new()
            }
        };
        
        Self {
            input_dir,
            cache_dir,
            threads,
            db: Arc::new(Mutex::new(db)),
            config: MissionScannerConfig::default(),
        }
    }
    
    /// Create a new mission scanner with custom configuration
    pub fn with_config(
        input_dir: &'a Path,
        cache_dir: &'a Path,
        config: MissionScannerConfig,
    ) -> Self {
        // Load or create the database
        let db_path = cache_dir.join("mission_database.json");
        let db = match MissionDatabase::load_or_create(&db_path) {
            Ok(db) => {
                info!("Loaded mission database from {}", db_path.display());
                db
            },
            Err(e) => {
                warn!("Failed to load mission database, creating new one: {}", e);
                MissionDatabase::new()
            }
        };
        
        Self {
            input_dir,
            cache_dir,
            threads: config.max_threads,
            db: Arc::new(Mutex::new(db)),
            config,
        }
    }
    
    /// Scan and extract mission files
    pub async fn scan_and_extract(&self) -> Result<Vec<MissionExtractionResult>> {
        scanner::scan_and_extract_with_config(
            self.input_dir,
            self.cache_dir,
            self.threads,
            &self.db,
            &self.config,
        ).await
    }
    
    /// Get access to the mission database
    pub fn get_database(&self) -> Arc<Mutex<MissionDatabase>> {
        self.db.clone()
    }
    
    /// Save the database to disk
    pub fn save_database(&self) -> Result<()> {
        let db_path = self.cache_dir.join("mission_database.json");
        let db = self.db.lock().unwrap();
        db.save(&db_path)
    }
    
    /// Export scan results to a file
    pub fn export_results(&self, path: &Path, results: &[MissionExtractionResult]) -> Result<()> {
        let json = serde_json::to_string_pretty(results)?;
        std::fs::write(path, json)?;
        info!("Exported scan results to {}", path.display());
        Ok(())
    }
    
    /// Export extracted file metadata
    /// 
    /// Useful for analyzing what files were found in missions
    pub fn export_file_metadata(&self, path: &Path, results: &[MissionExtractionResult]) -> Result<()> {
        let mut file_data = Vec::new();
        
        for result in results {
            let mut files = Vec::new();
            
            // Add SQM file if available
            if let Some(sqm_path) = &result.sqm_file {
                files.push(serde_json::json!({
                    "type": "sqm",
                    "path": sqm_path.to_string_lossy()
                }));
            }
            
            // Add SQF files
            for sqf_path in &result.sqf_files {
                files.push(serde_json::json!({
                    "type": "sqf",
                    "path": sqf_path.to_string_lossy()
                }));
            }
            
            // Add CPP/HPP files
            for cpp_path in &result.cpp_files {
                let ext = cpp_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                files.push(serde_json::json!({
                    "type": ext,
                    "path": cpp_path.to_string_lossy()
                }));
            }
            
            file_data.push(serde_json::json!({
                "mission_name": result.mission_name,
                "pbo_path": result.pbo_path.to_string_lossy(),
                "extracted_path": result.extracted_path.to_string_lossy(),
                "file_count": files.len(),
                "files": files
            }));
        }
        
        let json = serde_json::to_string_pretty(&file_data)?;
        std::fs::write(path, json)?;
        info!("Exported file metadata to {}", path.display());
        Ok(())
    }
} 