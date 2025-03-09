use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error};
use std::collections::HashMap;

use crate::types::SkipReason;
use super::MissionDatabase;

/// Save the mission database to disk
pub fn save_database(db: &MissionDatabase, path: &Path) -> Result<()> {
    db.save(path)
}

/// Load a mission database from disk or create a new one if it doesn't exist
pub fn load_database(path: &Path) -> Result<MissionDatabase> {
    MissionDatabase::load_or_create(path)
}

/// Check if a mission has changed since last scan
pub fn has_mission_changed(
    db: &MissionDatabase,
    mission_path: &Path,
    hash: &str
) -> Result<(bool, Option<SkipReason>)> {
    let mission_key = mission_path.to_string_lossy().to_string();
    
    // Check if mission is in database
    if let Some(mission) = db.get_mission_info(mission_path) {
        // Check if hash has changed
        if mission.hash == hash {
            return Ok((false, Some(SkipReason::Unchanged)));
        }
    }
    
    Ok((true, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::types::MissionScanResult;
    
    #[test]
    fn test_save_and_load() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db.json");
        
        // Create a test database
        let mut db = MissionDatabase::new();
        
        // Add a test mission
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
        
        Ok(())
    }
    
    #[test]
    fn test_has_mission_changed() -> Result<()> {
        // Create a test database
        let mut db = MissionDatabase::new();
        
        // Add a test mission
        let mission = MissionScanResult {
            mission_name: "test_mission".to_string(),
            mission_path: PathBuf::from("/path/to/test_mission.pbo"),
            hash: "test_hash".to_string(),
            processed: true,
            timestamp: 123456789,
        };
        
        db.update_mission(mission);
        
        // Check if the mission has changed
        let (changed, reason) = has_mission_changed(&db, &PathBuf::from("/path/to/test_mission.pbo"), "test_hash")?;
        assert!(!changed);
        assert_eq!(reason, Some(SkipReason::Unchanged));
        
        let (changed, reason) = has_mission_changed(&db, &PathBuf::from("/path/to/test_mission.pbo"), "new_hash")?;
        assert!(changed);
        assert_eq!(reason, None);
        
        let (changed, reason) = has_mission_changed(&db, &PathBuf::from("/path/to/other_mission.pbo"), "test_hash")?;
        assert!(changed);
        assert_eq!(reason, None);
        
        Ok(())
    }
} 