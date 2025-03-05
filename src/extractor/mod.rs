pub mod types;
mod extractor;

pub use types::*;
pub use extractor::*;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};

use crate::mission_scanner::database::MissionDatabase;

/// Extractor for mission files
pub struct MissionExtractor<'a> {
    /// Directory for caching extraction results
    cache_dir: &'a Path,
    /// Number of parallel threads to use
    threads: usize,
    /// Database for storing extraction results
    db: Arc<Mutex<MissionDatabase>>,
}

impl<'a> MissionExtractor<'a> {
    /// Create a new mission extractor
    pub fn new(cache_dir: &'a Path, threads: usize) -> Result<Self> {
        let db_path = cache_dir.join("mission_db.json");
        let db = Arc::new(Mutex::new(
            MissionDatabase::load_or_create(&db_path).unwrap_or_else(|_| {
                warn!("Failed to load mission database, creating a new one");
                MissionDatabase::new()
            })
        ));
        
        Ok(Self {
            cache_dir,
            threads,
            db,
        })
    }
    
    /// Extract mission files
    pub fn extract_missions(
        &self,
        mission_files: &[PathBuf],
        progress: ProgressBar,
    ) -> Result<Vec<types::MissionExtractionResult>> {
        extractor::extract_missions(
            self.cache_dir,
            self.threads,
            mission_files,
            self.db.clone(),
            progress,
        )
    }
    
    /// Get the mission database
    pub fn get_database(&self) -> Arc<Mutex<MissionDatabase>> {
        self.db.clone()
    }
} 