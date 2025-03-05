use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

/// Dependency on a class in a mission file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDependency {
    /// Name of the class
    pub class_name: String,
    /// Path to the source file
    pub source_file: PathBuf,
    /// Line number in the source file
    pub line_number: usize,
    /// Context of the dependency
    pub context: String,
    /// Type of reference
    pub reference_type: ReferenceType,
}

/// Type of reference to a class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceType {
    /// Class is directly referenced (e.g., in createVehicle)
    Direct,
    /// Class is defined in mission file
    Definition,
    /// Class is a parent class referenced in an inheritance relationship
    Parent,
    /// Class is referenced as a component/property
    Component,
}

/// Result of analyzing a mission's dependencies
#[derive(Debug, Clone)]
pub struct MissionDependencyResult {
    /// Name of the mission
    pub mission_name: String,
    /// Path to the original PBO file
    pub pbo_path: PathBuf,
    /// List of class dependencies
    pub class_dependencies: Vec<ClassDependency>,
    /// Set of unique class names
    pub unique_class_names: HashSet<String>,
}

impl std::fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceType::Direct => write!(f, "Direct"),
            ReferenceType::Definition => write!(f, "Definition"),
            ReferenceType::Parent => write!(f, "Parent"),
            ReferenceType::Component => write!(f, "Component"),
        }
    }
} 