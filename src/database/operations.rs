use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error};

use super::types::*;
use super::MissionDatabase;
use crate::mission_scanner::types::SkipReason;

/// Save the mission database to disk
pub fn save_database(db: &MissionDatabase, path: &Path) -> Result<()> {
    db.save(path)
}

/// Load a mission database from disk or create a new one if it doesn't exist
pub fn load_database(path: &Path) -> Result<MissionDatabase> {
    MissionDatabase::load_or_create(path)
}

/// Check if a mission has changed since the last scan
pub fn has_mission_changed(db: &MissionDatabase, path: &Path, hash: &str) -> bool {
    match db.get_mission_info(path) {
        Some(info) => info.hash != hash,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::mission_scanner::types::MissionScanResult;
    
    #[test]
    fn test_save_and_load() -> Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_db.json");
        
        // Create a new database
        let mut db = MissionDatabase::new();
        
        // Add a mission
        let mission = MissionScanResult {
            mission_name: "test_mission".to_string(),
            mission_path: PathBuf::from("/path/to/test_mission.pbo"),
            hash: "test_hash".to_string(),
            processed: true,
            timestamp: 123456789,
        };
        db.update_mission(mission);
        
        // Save the database
        save_database(&db, &db_path)?;
        
        // Load the database
        let loaded_db = load_database(&db_path)?;
        
        // Check that the mission is in the loaded database
        let loaded_mission = loaded_db.get_mission_info(&PathBuf::from("/path/to/test_mission.pbo"));
        assert!(loaded_mission.is_some());
        let loaded_mission = loaded_mission.unwrap();
        assert_eq!(loaded_mission.hash, "test_hash");
        assert_eq!(loaded_mission.processed, true);
        assert_eq!(loaded_mission.timestamp, 123456789);
        
        Ok(())
    }
    
    #[test]
    fn test_has_mission_changed() -> Result<()> {
        let mut db = MissionDatabase::new();
        
        // Add a mission
        let mission = MissionScanResult {
            mission_name: "test_mission".to_string(),
            mission_path: PathBuf::from("/path/to/test_mission.pbo"),
            hash: "test_hash".to_string(),
            processed: true,
            timestamp: 123456789,
        };
        db.update_mission(mission);
        
        // Check if the mission has changed
        assert!(!has_mission_changed(&db, &PathBuf::from("/path/to/test_mission.pbo"), "test_hash"));
        assert!(has_mission_changed(&db, &PathBuf::from("/path/to/test_mission.pbo"), "new_hash"));
        assert!(has_mission_changed(&db, &PathBuf::from("/path/to/other_mission.pbo"), "test_hash"));
        
        Ok(())
    }
} 