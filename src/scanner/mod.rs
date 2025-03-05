mod collector;
mod scanner;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

use crate::mission_scanner::database::MissionDatabase;
use crate::mission_scanner::extractor::types::MissionExtractionResult;

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
}

impl<'a> MissionScanner<'a> {
    /// Create a new mission scanner
    pub fn new(
        input_dir: &'a Path,
        cache_dir: &'a Path,
        threads: usize,
    ) -> Self {
        let db_path = cache_dir.join("mission_db.json");
        let db = Arc::new(Mutex::new(
            MissionDatabase::load_or_create(&db_path).unwrap_or_else(|_| {
                warn!("Failed to load mission database, creating a new one");
                MissionDatabase::new()
            })
        ));
        
        Self {
            input_dir,
            cache_dir,
            threads,
            db,
        }
    }
    
    /// Scan and extract mission files
    pub async fn scan_and_extract(&self) -> Result<Vec<MissionExtractionResult>> {
        scanner::scan_and_extract(
            self.input_dir,
            self.cache_dir,
            self.threads,
            &self.db,
        ).await
    }
    
    /// Get the mission database
    pub fn get_database(&self) -> Arc<Mutex<MissionDatabase>> {
        self.db.clone()
    }
} 