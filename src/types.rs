use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Default file extensions to scan
pub const DEFAULT_FILE_EXTENSIONS: &[&str] = &["sqm", "sqf", "cpp", "hpp"];

/// Configuration for mission scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Directory containing mission files to scan
    pub input_dir: PathBuf,
    /// Directory for caching extraction results
    pub cache_dir: PathBuf,
    /// Directory for output reports
    pub output_dir: PathBuf,
    /// Number of parallel threads to use (defaults to number of CPU cores)
    pub threads: Option<usize>,
    /// File extensions to scan (defaults to ["sqm", "sqf", "cpp", "hpp"])
    pub file_extensions: Option<Vec<String>>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            cache_dir: PathBuf::new(),
            output_dir: PathBuf::new(),
            threads: Some(num_cpus::get()),
            file_extensions: Some(DEFAULT_FILE_EXTENSIONS.iter().map(|&s| s.to_string()).collect()),
        }
    }
}

/// Configuration for the mission scanner implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionScannerConfig {
    /// Maximum number of threads to use for scanning
    pub max_threads: usize,
    /// Extract only specific file extensions (empty = all)
    pub file_extensions: Vec<String>,
}

impl Default for MissionScannerConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get(),
            file_extensions: DEFAULT_FILE_EXTENSIONS.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

/// Result of extracting mission files
#[derive(Debug, Clone)]
pub struct MissionFileResults {
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
}

/// Result of analyzing mission dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionResults {
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
    /// List of class dependencies
    pub class_dependencies: Vec<ClassReference>,
}

/// Class dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassReference {
    /// Name of the class
    /// Note: Arma 3 class names are case-insensitive. When comparing class names,
    /// they should be converted to lowercase first.
    pub class_name: String,
    /// Type of reference
    pub reference_type: ReferenceType,
    /// Context where the class is referenced
    pub context: String,
    /// Source file
    pub source_file: PathBuf,
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

/// Represents the source of an inventory item reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassSource {
    /// Found in a SQF script file
    Script {
        /// Path to the script file
        file_path: String,
    },
    /// Found in mission.sqm
    Mission {
        /// Class or section where item was found
        context: String,
    },
    /// Found in a config file
    Code {
        /// Path to the code file
        file_path: String,
        /// Class or section where item was found
        class: String,
    },
}

impl std::fmt::Display for ClassSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassSource::Script { file_path } => write!(f, "Script: {}", file_path),
            ClassSource::Mission { context } => write!(f, "Mission: {}", context),
            ClassSource::Code { file_path, class } => write!(f, "Code: {} in {}", class, file_path),
        }
    }
} 