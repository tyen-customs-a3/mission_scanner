use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Information about a mission in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionInfo {
    /// Hash of the mission file
    pub hash: String,
    /// Whether the mission was processed successfully
    pub processed: bool,
    /// Timestamp of when the mission was processed
    pub timestamp: u64,
}

/// Database operation result
#[derive(Debug, Clone)]
pub struct DatabaseResult<T> {
    /// Whether the operation was successful
    pub success: bool,
    /// Result of the operation
    pub result: Option<T>,
    /// Error message if the operation failed
    pub error: Option<String>,
} 