pub mod types;
pub mod extractor;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use rayon::prelude::*;

use crate::types::SkipReason;

pub use types::MissionExtractionResult;
pub use extractor::extract_single_mission;

/// Extractor for mission files
pub struct MissionExtractor<'a> {
    /// Directory for caching extraction results
    cache_dir: &'a Path,
    /// Number of parallel threads to use
    threads: usize,
}

impl<'a> MissionExtractor<'a> {
    /// Create a new mission extractor
    pub fn new(cache_dir: &'a Path, threads: usize) -> Result<Self> {
        
        Ok(Self {
            cache_dir,
            threads,
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
            progress,
        )
    }
} 