use std::path::PathBuf;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use pretty_assertions::assert_eq;
use test_log::test;

use super::*;
use crate::types::{MissionScannerConfig, MissionExtractionResult, ReferenceType};

use env_logger;

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

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
    assert!(mission1.mission_dir.ends_with("test_mission_1"), "Extracted path should be correct");
    
    // Verify loadouts directory exists and contains files
    let loadouts_dir = mission1.mission_dir.join("loadouts");
    assert!(loadouts_dir.exists(), "loadouts directory should exist");
    
    // Verify briefing directory exists
    let briefing_dir = mission1.mission_dir.join("briefing");
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
fn test_mission_class_dependencies() -> Result<()> {
    init();
    let test_dir = get_test_data_dir();
    println!("Test directory: {}", test_dir.display());
    
    // First collect the mission files using the collector
    let mission_files = collector::collect_mission_files(&test_dir)?;
    println!("Found {} mission files", mission_files.len());
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    println!("Processing test_mission_1");
    println!("Mission directory: {}", mission1.mission_dir.display());
    println!("SQM file: {:?}", mission1.sqm_file);
    println!("SQF files: {:?}", mission1.sqf_files);
    println!("CPP files: {:?}", mission1.cpp_files);
    
    // Extract dependencies using the scanner's extract_mission_dependencies
    let results = extract_mission_dependencies(&[mission1.clone()])?;
    println!("Extracted {} mission results", results.len());
    
    // Get the dependencies for test_mission_1
    let mission_deps = results.iter()
        .find(|r| r.mission_name == "test_mission_1")
        .expect("Should have dependencies for test_mission_1");
    
    println!("Found {} dependencies for test_mission_1", mission_deps.class_dependencies.len());
    
    // Get all class names from dependencies
    let found_classes: std::collections::HashSet<_> = mission_deps.class_dependencies.iter()
        .map(|dep| dep.class_name.as_str())
        .collect();
    
    println!("Found classes:");
    for class in &found_classes {
        println!("  - {}", class);
    }
    
    // Expected items from each file
    let expected_items = [
        // Basic items that should be found in mission.sqm
        "ItemMap", "ItemCompass", "ItemWatch",
        
        // Common equipment items
        "rhs_weap_mg42", "rhsusf_weap_glock17g4",
        "rhsgref_50Rnd_792x57_SmE_drum", "rhsusf_mag_17Rnd_9x19_JHP",
        "TC_U_aegis_guerilla_garb_m81_sudan", "rhsusf_spcs_ocp_saw",
        "pca_eagle_a3_od", "simc_pasgt_m81",
        
        // Common items from loadouts
        "ACRE_BF888S", "ACE_fieldDressing", "ACE_packingBandage",
        "ACE_tourniquet", "ACE_epinephrine", "ACE_morphine", "ACE_splint",
        
        // Enemy loadout items
        "rhs_uniform_emr_patchless", "rhs_uniform_gorka_r_g",
        "rhs_6b23_6sh116", "rhs_6b23_6sh116_vog",
        "rhs_weap_ak74m", "rhs_weap_ak74m_gp25", "rhs_weap_pkp",
        "rhs_weap_rpg26", "rhs_weap_rpg7",
        "rhs_30Rnd_545x39_7N10_AK", "rhs_VOG25", "rhs_GRD40_White",
        
        // Player loadout items
        "U_I_L_Uniform_01_tshirt_sport_F", "U_I_L_Uniform_01_tshirt_skull_F",
        "CUP_I_B_PMC_Unit_19", "CUP_I_B_PMC_Unit_12",
        "rhs_weap_m1garand_sa43", "CUP_lmg_PKM", "rhs_weap_m79",
        "rhs_weap_m82a1", "rhs_weap_akms",
        
        // Arsenal items
        "Tarkov_Uniforms_1", "Tarkov_Uniforms_2",
        "V_PlateCarrier2_blk", "V_PlateCarrier1_blk",
        "H_HelmetSpecB_blk", "H_HelmetSpecB_snakeskin",
        "rhsusf_ANPVS_15", "ACRE_PRC343", "ACRE_PRC148",
        "ACE_Flashlight_XL50", "ACE_MapTools", "ACE_RangeCard"
    ];
    
    // Verify all expected items are found in dependencies
    let mut missing_items = Vec::new();
    
    for &item in &expected_items {
        if !found_classes.contains(item) {
            missing_items.push(format!("Missing item: {}", item));
        }
    }
    
    // If any items are missing, fail the test with details
    assert!(missing_items.is_empty(), 
        "Missing dependencies:\n{}", missing_items.join("\n"));
    
    // Also verify we found some common reference types
    let reference_types: std::collections::HashSet<_> = mission_deps.class_dependencies.iter()
        .map(|dep| &dep.reference_type)
        .collect();
    
    println!("Found reference types:");
    for ref_type in &reference_types {
        println!("  - {:?}", ref_type);
    }
    
    assert!(reference_types.contains(&ReferenceType::Direct), "Should find direct references");
    assert!(reference_types.contains(&ReferenceType::Variable), "Should find variable references");
    
    Ok(())
} 