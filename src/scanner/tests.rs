use std::path::PathBuf;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use std::fs;

use super::*;
use crate::types::MissionScannerConfig;
use crate::extractor::types::MissionExtractionResult;

/// Helper function to get the test data directory
fn get_test_data_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("test_data")
}

/// Helper function to scan mission files for testing
fn scan_mission_files_with_config(
    mission_files: &[MissionExtractionResult],
    progress: ProgressBar,
    _config: &MissionScannerConfig
) -> Result<Vec<MissionExtractionResult>> {
    debug!("Scanning {} mission files", mission_files.len());
    for result in mission_files {
        debug!("Found mission file: {}", result.mission_name);
    }
    
    progress.finish_with_message(format!("Scanned {} missions", mission_files.len()));
    debug!("Completed scanning {} missions", mission_files.len());
    
    Ok(mission_files.to_vec())
}

#[test]
fn test_collect_mission_files() -> Result<()> {
    let test_dir = get_test_data_dir();
    debug!("Test directory: {}", test_dir.display());
    
    let mission_files = collector::collect_mission_files(&test_dir)?;
    debug!("Found {} mission files", mission_files.len());
    
    for result in &mission_files {
        debug!("Mission file: {}", result.mission_name);
    }
    
    assert!(!mission_files.is_empty(), "Should find mission files");
    assert!(mission_files.iter().any(|p| p.mission_name == "test_mission_1"), "Should find test_mission_1");
    assert!(mission_files.iter().any(|p| p.mission_name == "test_mission_2"), "Should find test_mission_2");
    
    Ok(())
}

#[test]
fn test_collect_mission_files_with_config() -> Result<()> {
    let test_dir = get_test_data_dir();
    debug!("Test directory: {}", test_dir.display());
    
    let config = MissionScannerConfig {
        recursive: true,
        ..Default::default()
    };
    
    let mission_files = collector::collect_mission_files_with_config(&test_dir, &config)?;
    debug!("Found {} mission files", mission_files.len());
    
    for result in &mission_files {
        debug!("Mission file: {}", result.mission_name);
    }
    
    assert!(!mission_files.is_empty(), "Should find mission files");
    assert!(mission_files.iter().any(|p| p.mission_name == "test_mission_1"), "Should find test_mission_1");
    assert!(mission_files.iter().any(|p| p.mission_name == "test_mission_2"), "Should find test_mission_2");
    
    Ok(())
}

#[test]
fn test_scan_mission_files() -> Result<()> {
    let test_dir = get_test_data_dir();
    debug!("Test directory: {}", test_dir.display());
    
    let mission_files = collector::collect_mission_files(&test_dir)?;
    debug!("Found {} mission files", mission_files.len());
    
    // Set up progress bar for testing
    let progress = ProgressBar::new(mission_files.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    // Scan mission files
    let scan_results = scan_mission_files_with_config(&mission_files, progress, &MissionScannerConfig::default())?;
    debug!("Scanned {} missions", scan_results.len());
    
    assert!(!scan_results.is_empty(), "Should find mission files");
    assert!(scan_results.iter().any(|p| p.mission_name == "test_mission_1"), "Should scan test_mission_1");
    assert!(scan_results.iter().any(|p| p.mission_name == "test_mission_2"), "Should scan test_mission_2");
    
    Ok(())
}

#[test]
fn test_scan_mission_files_with_config() -> Result<()> {
    let test_dir = get_test_data_dir();
    debug!("Test directory: {}", test_dir.display());
    
    let config = MissionScannerConfig {
        recursive: true,
        ..Default::default()
    };
    
    let mission_files = collector::collect_mission_files_with_config(&test_dir, &config)?;
    debug!("Found {} mission files", mission_files.len());
    
    // Set up progress bar for testing
    let progress = ProgressBar::new(mission_files.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    // Scan mission files
    let scan_results = scan_mission_files_with_config(&mission_files, progress, &config)?;
    debug!("Scanned {} missions", scan_results.len());
    
    assert!(!scan_results.is_empty(), "Should find mission files");
    assert!(scan_results.iter().any(|p| p.mission_name == "test_mission_1"), "Should scan test_mission_1");
    assert!(scan_results.iter().any(|p| p.mission_name == "test_mission_2"), "Should scan test_mission_2");
    
    Ok(())
}

#[test]
fn test_mission_file_contents() -> Result<()> {
    let test_dir = get_test_data_dir();
    let mission_files = collector::collect_mission_files(&test_dir)?;
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    // Verify mission.sqm exists
    assert!(mission1.sqm_file.is_some(), "mission.sqm should exist");
    assert!(mission1.sqm_file.as_ref().unwrap().ends_with("mission.sqm"), "mission.sqm path should be correct");
    
    // Verify SQF files
    assert!(mission1.sqf_files.iter().any(|p| p.ends_with("init.sqf")), "init.sqf should exist");
    assert!(mission1.sqf_files.iter().any(|p| p.ends_with("initplayerlocal.sqf")), "initplayerlocal.sqf should exist");
    assert!(mission1.sqf_files.iter().any(|p| p.ends_with("initserver.sqf")), "initserver.sqf should exist");
    
    // Verify CPP/HPP files
    assert!(mission1.cpp_files.iter().any(|p| p.ends_with("description.ext")), "description.ext should exist");
    
    // Find test_mission_2
    let mission2 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_2")
        .expect("test_mission_2 should exist");
    
    // Verify mission.sqm exists
    assert!(mission2.sqm_file.is_some(), "mission.sqm should exist");
    assert!(mission2.sqm_file.as_ref().unwrap().ends_with("mission.sqm"), "mission.sqm path should be correct");
    
    Ok(())
}

#[test]
fn test_mission_file_structure() -> Result<()> {
    let test_dir = get_test_data_dir();
    let mission_files = collector::collect_mission_files(&test_dir)?;
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    // Verify directory structure
    assert!(mission1.extracted_path.ends_with("test_mission_1"), "Extracted path should be correct");
    
    // Verify loadouts directory exists and contains files
    let loadouts_dir = mission1.extracted_path.join("loadouts");
    assert!(loadouts_dir.exists(), "loadouts directory should exist");
    
    // Verify briefing directory exists
    let briefing_dir = mission1.extracted_path.join("briefing");
    assert!(briefing_dir.exists(), "briefing directory should exist");
    
    // Count total number of files
    let total_files = mission1.sqf_files.len() + mission1.cpp_files.len() + 1; // +1 for mission.sqm
    assert!(total_files > 0, "Should have found multiple mission files");
    
    // Verify file extensions
    for file in &mission1.sqf_files {
        assert!(file.extension().unwrap() == "sqf", "SQF files should have .sqf extension");
    }
    
    for file in &mission1.cpp_files {
        let ext = file.extension().unwrap();
        assert!(ext == "cpp" || ext == "hpp" || ext == "ext", "CPP files should have .cpp, .hpp, or .ext extension");
    }
    
    Ok(())
}

#[test]
fn test_mission_class_contents() -> Result<()> {
    use crate::scanner::parse_sqm_file;
    use crate::scanner::parse_loadout_file;
    use crate::scanner::scan_sqf_file;
    use parser_code::Equipment;
    use parser_sqm::InventoryClass;
    
    let test_dir = get_test_data_dir();
    let mission_files = collector::collect_mission_files(&test_dir)?;
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    // Test enemy loadout file classes
    let loadout_path = mission1.extracted_path.join("loadouts").join("enemy_loadout.hpp");
    let loadout_classes = parse_loadout_file(&loadout_path)?;
    
    // Verify base class exists
    let base_class = loadout_classes.iter()
        .find(|e| e.class_name == "baseMan")
        .expect("Should find baseMan class");
    
    // Verify base class equipment
    let uniform_options = base_class.properties.get("uniform").expect("Should have uniform options");
    assert!(uniform_options.contains(&"rhs_uniform_emr_patchless".to_string()), "Should find EMR uniform");
    assert!(uniform_options.contains(&"rhs_uniform_gorka_r_g".to_string()), "Should find Gorka uniform");
    
    let vest_options = base_class.properties.get("vest").expect("Should have vest options");
    assert!(vest_options.contains(&"rhs_6b23_6sh116".to_string()), "Should find 6b23 vest");
    assert!(vest_options.contains(&"rhs_6b23_digi_6sh92".to_string()), "Should find digi vest");
    
    let headgear_options = base_class.properties.get("headgear").expect("Should have headgear options");
    assert!(headgear_options.contains(&"rhs_6b27m_digi".to_string()), "Should find 6b27m helmet");
    assert!(headgear_options.contains(&"rhs_6b47".to_string()), "Should find 6b47 helmet");
    
    // Verify role classes exist and their equipment
    let rifleman = loadout_classes.iter()
        .find(|e| e.class_name == "r")
        .expect("Should find rifleman class");
    let primary_weapon = rifleman.properties.get("primaryWeapon").expect("Should have primary weapon");
    assert!(primary_weapon.contains(&"rhs_weap_ak74m".to_string()), "Should find AK-74M");
    
    let grenadier = loadout_classes.iter()
        .find(|e| e.class_name == "g")
        .expect("Should find grenadier class");
    let grenadier_weapon = grenadier.properties.get("primaryWeapon").expect("Should have primary weapon");
    assert!(grenadier_weapon.contains(&"rhs_weap_ak74m_gp25".to_string()), "Should find AK-74M GP-25");
    
    let mg = loadout_classes.iter()
        .find(|e| e.class_name == "mg")
        .expect("Should find machine gunner class");
    let mg_weapon = mg.properties.get("primaryWeapon").expect("Should have primary weapon");
    assert!(mg_weapon.contains(&"rhs_weap_pkp".to_string()), "Should find PKP");
    
    // Test arsenal.sqf contents
    let arsenal_path = mission1.extracted_path.join("loadouts").join("arsenal.sqf");
    let content = fs::read_to_string(&arsenal_path)?;
    
    // Check for specific equipment in arsenal
    assert!(content.contains("\"Tarkov_Uniforms_1\""), "Should find Tarkov uniform");
    assert!(content.contains("\"V_PlateCarrier2_blk\""), "Should find plate carrier");
    assert!(content.contains("\"H_HelmetSpecB_blk\""), "Should find spec ops helmet");
    assert!(content.contains("\"rhs_weap_hk416d145\""), "Should find HK416");
    
    Ok(())
}

#[test]
fn test_mission_file_dependencies() -> Result<()> {
    use crate::scanner::extract_sqm_dependencies;
    use crate::scanner::parse_loadout_file;
    use parser_code::Equipment;
    
    let test_dir = get_test_data_dir();
    let mission_files = collector::collect_mission_files(&test_dir)?;
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    // Test mission.sqm dependencies
    let sqm_path = mission1.sqm_file.as_ref().unwrap();
    let content = fs::read_to_string(sqm_path)?;
    
    // Find and verify addons array
    if let Some(addons_start) = content.find("addons[] = {") {
        let addons_end = content[addons_start..].find("};").unwrap() + addons_start + 2;
        let addons_str = &content[addons_start..addons_end];
        
        // Extract addon names
        let mut dependencies = std::collections::HashSet::new();
        let addons_list = addons_str.split('{').nth(1).unwrap().split('}').next().unwrap();
        for addon in addons_list.split(',') {
            let addon = addon.trim().trim_matches('"');
            if !addon.is_empty() {
                dependencies.insert(addon.to_string());
            }
        }
        
        // Verify core dependencies
        assert!(dependencies.contains("A3_Characters_F"), "Should find A3_Characters_F");
        assert!(dependencies.contains("A3_Characters_F_Mark"), "Should find A3_Characters_F_Mark");
        
        // Verify RHS dependencies
        assert!(dependencies.contains("rhsgref_c_troops"), "Should find RHS troops");
        assert!(dependencies.contains("rhsgref_c_vehicles_ret"), "Should find RHS vehicles");
        
        // Verify CUP dependencies
        assert!(dependencies.contains("CUP_Misc_e_Config"), "Should find CUP misc config");
        assert!(dependencies.contains("CUP_WheeledVehicles_Kamaz"), "Should find CUP Kamaz");
        assert!(dependencies.contains("CUP_AirVehicles_Mi8"), "Should find CUP Mi-8");
        assert!(dependencies.contains("CUP_Creatures_Military_Russia"), "Should find CUP Russian military");
        
        // Verify structure dependencies
        assert!(dependencies.contains("A3_Structures_F_Walls"), "Should find walls");
        assert!(dependencies.contains("A3_Structures_F_Civ_Market"), "Should find market structures");
        assert!(dependencies.contains("A3_Structures_F_EPA_Mil_Scrapyard"), "Should find scrapyard");
        assert!(dependencies.contains("A3_Structures_F_Exp_Military_Fortifications"), "Should find fortifications");
    } else {
        panic!("Could not find addons array in mission.sqm");
    }
    
    // Test enemy loadout dependencies
    let loadout_path = mission1.extracted_path.join("loadouts").join("enemy_loadout.hpp");
    let loadout_deps = parse_loadout_file(&loadout_path)?;
    
    // Verify equipment dependencies through properties
    let base_class = loadout_deps.iter()
        .find(|e| e.class_name == "baseMan")
        .expect("Should find baseMan class");
    
    // Check uniform dependencies
    let uniform_deps = base_class.properties.get("uniform").expect("Should have uniform dependencies");
    assert!(uniform_deps.contains(&"rhs_uniform_emr_patchless".to_string()), "Should find EMR uniform");
    assert!(uniform_deps.contains(&"rhs_uniform_gorka_r_g".to_string()), "Should find Gorka uniform");
    
    // Check vest dependencies
    let vest_deps = base_class.properties.get("vest").expect("Should have vest dependencies");
    assert!(vest_deps.contains(&"rhs_6b23_6sh116".to_string()), "Should find 6b23 vest");
    assert!(vest_deps.contains(&"rhs_6b23_digi_6sh92".to_string()), "Should find digi vest");
    
    // Check weapon dependencies from role classes
    let rifleman = loadout_deps.iter()
        .find(|e| e.class_name == "r")
        .expect("Should find rifleman class");
    let primary_weapon = rifleman.properties.get("primaryWeapon").expect("Should have primary weapon");
    assert!(primary_weapon.contains(&"rhs_weap_ak74m".to_string()), "Should find AK-74M");
    
    let mg = loadout_deps.iter()
        .find(|e| e.class_name == "mg")
        .expect("Should find machine gunner class");
    let mg_weapon = mg.properties.get("primaryWeapon").expect("Should have primary weapon");
    assert!(mg_weapon.contains(&"rhs_weap_pkp".to_string()), "Should find PKP");
    
    let rpg = loadout_deps.iter()
        .find(|e| e.class_name == "rrpg")
        .expect("Should find RPG class");
    let rpg_weapon = rpg.properties.get("secondaryWeapon").expect("Should have secondary weapon");
    assert!(rpg_weapon.contains(&"rhs_weap_rpg7".to_string()), "Should find RPG-7");
    
    Ok(())
} 