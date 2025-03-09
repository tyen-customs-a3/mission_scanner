mod collector;
mod scanner;
mod parser_integration;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

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
        
        Self {
            input_dir,
            cache_dir,
            threads,
            config: MissionScannerConfig::default(),
        }
    }
    
    /// Create a new mission scanner with custom configuration
    pub fn with_config(
        input_dir: &'a Path,
        cache_dir: &'a Path,
        config: MissionScannerConfig,
    ) -> Self {
        
        Self {
            input_dir,
            cache_dir,
            threads: config.max_threads,
            config,
        }
    }
    
    /// Scan and extract mission files
    pub async fn scan_and_extract(&self) -> Result<Vec<MissionExtractionResult>> {
        scanner::scan_and_extract_with_config(
            self.input_dir,
            self.cache_dir,
            self.threads,
            &self.config,
        ).await
    }
} 