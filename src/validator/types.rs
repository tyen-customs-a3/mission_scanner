use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Information about a missing class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingClassInfo {
    /// Name of the missing class
    pub class_name: String,
    /// Number of references to the class
    pub reference_count: usize,
    /// Locations where the class is referenced
    pub reference_locations: Vec<String>,
    /// Suggested alternative classes
    pub suggested_alternatives: Vec<String>,
}

/// Report on class existence for a mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionClassExistenceReport {
    /// Name of the mission
    pub mission_name: String,
    /// Total number of classes referenced in the mission
    pub total_classes: usize,
    /// Number of classes that exist
    pub existing_classes: usize,
    /// Number of classes that are missing
    pub missing_classes: usize,
    /// Percentage of classes that exist
    pub existence_percentage: f64,
    /// List of missing classes
    pub missing_class_list: Vec<MissingClassInfo>,
}

/// Report on class existence for all missions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassExistenceReport {
    /// Total number of missions
    pub total_missions: usize,
    /// Total number of unique classes referenced across all missions
    pub total_unique_classes: usize,
    /// Number of classes that exist
    pub existing_classes: usize,
    /// Number of classes that are missing
    pub missing_classes: usize,
    /// Percentage of classes that exist
    pub existence_percentage: f64,
    /// Reports for individual missions
    pub mission_reports: Vec<MissionClassExistenceReport>,
} 