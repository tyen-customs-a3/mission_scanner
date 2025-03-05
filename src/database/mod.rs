mod types;
mod operations;

pub use types::*;
pub use operations::*;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::{info, warn, error};
use serde::{Serialize, Deserialize};

use crate::mission_scanner::types::{MissionScanResult, MissionScanStats, SkipReason};

/// Database for storing mission scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDatabase {
    /// Map of mission paths to scan results
    missions: HashMap<String, MissionScanResult>,
}

/// Statistics about the mission database
#[derive(Debug, Clone)]
pub struct MissionDatabaseStats {
    /// Total number of missions in the database
    pub total: usize,
    /// Number of missions that were processed successfully
    pub processed: usize,
    /// Number of missions that failed to process
    pub failed: usize,
}

impl MissionDatabase {
    /// Create a new empty mission database
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
        }
    }
    
    /// Load a mission database from disk or create a new one if it doesn't exist
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            info!("Loading mission database from {}", path.display());
            let file = std::fs::File::open(path)?;
            let db: Self = serde_json::from_reader(file)?;
            info!("Loaded mission database with {} entries", db.missions.len());
            Ok(db)
        } else {
            info!("Creating new mission database");
            Ok(Self::new())
        }
    }
    
    /// Save the mission database to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        info!("Saving mission database to {}", path.display());
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        info!("Saved mission database with {} entries", self.missions.len());
        Ok(())
    }
    
    /// Get information about a mission
    pub fn get_mission_info(&self, path: &Path) -> Option<&MissionScanResult> {
        self.missions.get(&path.to_string_lossy().to_string())
    }
    
    /// Update information about a mission
    pub fn update_mission(&mut self, mission: MissionScanResult) {
        let path_str = mission.mission_path.to_string_lossy().to_string();
        self.missions.insert(path_str, mission);
    }
    
    /// Update a mission with a skip reason
    pub fn update_mission_with_reason(
        &mut self,
        path: &Path,
        hash: &str,
        failed: bool,
        reason: SkipReason,
    ) {
        let path_str = path.to_string_lossy().to_string();
        let mission_name = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            
        let mission = MissionScanResult {
            mission_name,
            mission_path: path.to_path_buf(),
            hash: hash.to_string(),
            processed: !failed,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.missions.insert(path_str, mission);
    }
    
    /// Get statistics about the mission database
    pub fn get_stats(&self) -> MissionDatabaseStats {
        let total = self.missions.len();
        let processed = self.missions.values()
            .filter(|m| m.processed)
            .count();
        let failed = total - processed;
        
        MissionDatabaseStats {
            total,
            processed,
            failed,
        }
    }
} 