use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// Result of extracting a mission
#[derive(Debug, Clone)]
pub struct MissionExtractionResult {
    /// Name of the mission (derived from PBO filename)
    pub mission_name: String,
    /// Path to the original PBO file
    pub pbo_path: PathBuf,
    /// Path to the extracted mission directory
    pub extracted_path: PathBuf,
    /// Path to the mission.sqm file if found
    pub sqm_file: Option<PathBuf>,
    /// Paths to all SQF script files
    pub sqf_files: Vec<PathBuf>,
    /// Paths to all CPP/HPP config files
    pub cpp_files: Vec<PathBuf>,
}

/// Information about a mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionInfo {
    /// Hash of the mission file
    pub hash: String,
    /// Whether the extraction failed
    pub failed: bool,
    /// Time taken to extract the mission (in milliseconds)
    pub extraction_time: u64,
}

/// Statistics about mission extraction
#[derive(Debug, Clone)]
pub struct MissionExtractionStats {
    /// Total number of missions
    pub total: usize,
    /// Number of missions processed
    pub processed: usize,
    /// Number of missions that failed to extract
    pub failed: usize,
    /// Number of missions that were unchanged since last extraction
    pub unchanged: usize,
} 