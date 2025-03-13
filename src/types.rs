use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Result of extracting mission files
#[derive(Debug, Clone)]
pub struct MissionExtractionResult {
    /// Name of the mission
    pub mission_name: String,
    /// Path to the mission directory
    pub mission_dir: PathBuf,
    /// Path to the mission.sqm file if it exists
    pub sqm_file: Option<PathBuf>,
    /// List of SQF files in the mission
    pub sqf_files: Vec<PathBuf>,
    /// List of CPP/HPP files in the mission
    pub cpp_files: Vec<PathBuf>,
    /// Path to the PBO file if this is from a PBO
    pub pbo_path: Option<PathBuf>,
    /// List of class dependencies found in the mission
    pub class_dependencies: Vec<ClassDependency>,
}

/// Result of analyzing mission dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDependencyResult {
    /// Name of the mission
    pub mission_name: String,
    /// Path to the mission file
    pub mission_path: PathBuf,
    /// List of class dependencies
    pub class_dependencies: Vec<ClassDependency>,
}

/// Class dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDependency {
    /// Name of the class
    pub class_name: String,
    /// Type of reference
    pub reference_type: ReferenceType,
    /// Context where the class is referenced
    pub context: String,
}

/// Type of reference to a class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReferenceType {
    /// Direct reference to a class
    Direct,
    /// Inheritance from a parent class
    Inheritance,
    /// Reference through a variable
    Variable,
}

/// Result of the mission scanning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionScanResult {
    /// Name of the mission
    pub mission_name: String,
    /// Path to the original mission file
    pub mission_path: PathBuf,
    /// Hash of the mission file
    pub hash: String,
    /// Whether the mission was processed successfully
    pub processed: bool,
    /// Timestamp of when the mission was processed
    pub timestamp: u64,
}

/// Statistics about the mission scanning process
#[derive(Debug, Clone)]
pub struct MissionScanStats {
    /// Total number of missions found
    pub total: usize,
    /// Number of missions processed
    pub processed: usize,
    /// Number of missions that failed to process
    pub failed: usize,
    /// Number of missions that were unchanged since last scan
    pub unchanged: usize,
}

/// Reason why a mission was skipped during scanning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    /// Mission was unchanged since last scan
    Unchanged,
    /// Mission extraction failed
    ExtractionFailed,
    /// Mission analysis failed
    AnalysisFailed,
    /// Mission was empty
    Empty,
    /// Other reason (with description)
    Other(String),
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::Unchanged => write!(f, "Unchanged"),
            SkipReason::ExtractionFailed => write!(f, "Extraction failed"),
            SkipReason::AnalysisFailed => write!(f, "Analysis failed"),
            SkipReason::Empty => write!(f, "Empty"),
            SkipReason::Other(reason) => write!(f, "Other: {}", reason),
        }
    }
}

/// Configuration for the mission scanning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionScannerConfig {
    /// Maximum number of threads to use for scanning
    pub max_threads: usize,
    /// Whether to force rescanning of unchanged missions
    pub force_rescan: bool,
    /// Skip extraction if PBO hash hasn't changed (uses database)
    pub skip_unchanged: bool,
    /// Extract only specific file extensions (empty = all)
    pub file_extensions: Vec<String>,
    /// Recursively scan subdirectories
    pub recursive: bool,
}

impl Default for MissionScannerConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get(),
            force_rescan: false,
            skip_unchanged: true,
            file_extensions: vec!["sqm".to_string(), "sqf".to_string(), "cpp".to_string(), "hpp".to_string()],
            recursive: true,
        }
    }
} 