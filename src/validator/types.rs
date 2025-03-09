use std::path::PathBuf;
use serde::{Serialize, Deserialize};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceType {
    /// Direct reference to a class
    Direct,
    /// Inheritance from a parent class
    Inheritance,
    /// Reference through a variable
    Variable,
}