use std::path::PathBuf;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use std::fs;

use super::*;
use crate::types::{MissionScannerConfig, MissionExtractionResult};
use crate::scanner::parse_file;

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
    let test_dir = get_test_data_dir();
    let mission_files = collector::collect_mission_files(&test_dir)?;
    
    // Find test_mission_1
    let mission1 = mission_files.iter()
        .find(|m| m.mission_name == "test_mission_1")
        .expect("test_mission_1 should exist");
    
    let mut dependencies = Vec::new();
    
    // Process SQM file if available
    if let Some(sqm_path) = &mission1.sqm_file {
        if let Ok(deps) = parse_file(sqm_path) {
            dependencies.extend(deps);
        }
    }
    
    // Process SQF files
    for sqf_path in &mission1.sqf_files {
        if let Ok(deps) = parse_file(sqf_path) {
            dependencies.extend(deps);
        }
    }
    
    // Process CPP/HPP files
    for cpp_path in &mission1.cpp_files {
        if let Ok(deps) = parse_file(cpp_path) {
            dependencies.extend(deps);
        }
    }

    // Expected addon dependencies from mission.sqm
    // Expected equipment items from mission.sqm
    let mission_sqm_items = [
        // Unit inventory weapons
        "rhs_weap_mg42",
        "rhsusf_weap_glock17g4",
        // Unit inventory magazines
        "rhsgref_50Rnd_792x57_SmE_drum",
        "rhsusf_mag_17Rnd_9x19_JHP",
        // Unit inventory equipment
        "TC_U_aegis_guerilla_garb_m81_sudan",
        "rhsusf_spcs_ocp_saw",
        "pca_eagle_a3_od",
        "simc_pasgt_m81",
        // Unit inventory items
        "ACRE_BF888S",
        "ACE_fieldDressing",
        "ACE_packingBandage",
        "ACE_tourniquet",
        "ACE_epinephrine",
        "ACE_morphine",
        "ACE_splint",
        "ItemMap",
        "ItemCompass",
        "ItemWatch"
    ];

    // Expected items from enemy_loadout.hpp
    let enemy_loadout_items = [
        // Uniforms
        "rhs_uniform_emr_patchless",
        "rhs_uniform_gorka_r_g",
        // Vests
        "rhs_6b23_6sh116",
        "rhs_6b23_6sh116_vog",
        "rhs_6b23_digi_6sh92",
        "rhs_6b23_digi_6sh92_spetsnaz",
        // Backpacks
        "rhs_assault_umbts",
        // Headgear
        "rhs_6b27m_digi",
        "rhs_6b47",
        "rhs_6b7_1m",
        "rhs_6b7_1m_emr",
        // Weapons
        "rhs_weap_ak74m",
        "rhs_weap_ak74m_gp25",
        "rhs_weap_pkp",
        "rhs_weap_rpg26",
        "rhs_weap_rpg7",
        // Weapon attachments
        "rhs_acc_1p63",
        "rhs_acc_ekp1",
        "rhs_acc_pgo7v3",
        // Magazines and grenades
        "rhs_30Rnd_545x39_7N10_AK",
        "rhs_VOG25",
        "rhs_GRD40_White",
        "rhs_100Rnd_762x54mmR",
        "rhs_rpg7_PG7V_mag",
        "rhs_rpg7_PG7VL_mag"
    ];

    // Expected items from player_loadout.hpp
    let player_loadout_items = [
        // Uniforms
        "U_I_L_Uniform_01_tshirt_sport_F",
        "U_I_L_Uniform_01_tshirt_skull_F",
        "CUP_I_B_PMC_Unit_19",
        "CUP_I_B_PMC_Unit_12",
        // Vests
        "V_HarnessOGL_brn",
        "usm_vest_lbe_gr",
        "usm_vest_lbe_machinegunner",
        // Backpacks
        "aegis_carryall_blk",
        "B_AssaultPack_mcamo",
        "B_TacticalPack_mcamo",
        // Weapons
        "rhs_weap_m1garand_sa43",
        "CUP_lmg_PKM",
        "rhs_weap_m79",
        "rhs_weap_m82a1",
        "rhs_weap_akms",
        // Sidearms
        "rhs_weap_makarov_pm",
        "CUP_hgun_TEC9",
        // Magazines
        "rhsusf_mag_10Rnd_STD_50BMG_M33",
        "rhsusf_mag_10Rnd_STD_50BMG_mk211",
        "CUP_32Rnd_9x19_TEC9",
        "rhs_30Rnd_762x39mm_bakelite",
        "rhs_30Rnd_762x39mm_bakelite_tracer"
    ];

    // Expected items from arsenal.sqf
    let arsenal_items = [
        // Uniforms
        "Tarkov_Uniforms_1",
        "Tarkov_Uniforms_2",
        // Vests
        "V_PlateCarrier2_blk",
        "V_PlateCarrier1_blk",
        // Helmets
        "H_HelmetSpecB_blk",
        "H_HelmetSpecB_snakeskin",
        // Backpacks
        "rhs_tortila_black",
        // NVGs
        "rhsusf_ANPVS_15",
        // Radios
        "ACRE_PRC343",
        "ACRE_PRC148",
        "ACRE_PRC152",
        "ACRE_PRC117F",
        // ACE Items
        "ACE_Flashlight_XL50",
        "ACE_MapTools",
        "ACE_RangeCard",
        // Equipment
        "ItemCompass",
        "ItemMap",
        "ItemWatch",
        "Binocular",
        "Rangefinder",
        // Grenades and Flares
        "ACE_40mm_Flare_green",
        "ACE_40mm_Flare_red",
        "UGL_FlareGreen_F",
        "UGL_FlareRed_F",
        "1Rnd_SmokeBlue_Grenade_shell",
        "SmokeShellBlue",
        "SmokeShellGreen"
    ];

    // Get all class names from dependencies
    let found_classes: std::collections::HashSet<_> = dependencies.iter()
        .map(|dep| dep.class_name.as_str())
        .collect();

    // Verify all items from each file are found in dependencies
    let mut missing_items = Vec::new();

    // Check mission.sqm items
    for item in &mission_sqm_items {
        if !found_classes.contains(*item) {
            missing_items.push(format!("mission.sqm item: {}", item));
        }
    }

    // Check enemy loadout items
    for item in &enemy_loadout_items {
        if !found_classes.contains(*item) {
            missing_items.push(format!("enemy_loadout.hpp: {}", item));
        }
    }

    // Check player loadout items
    for item in &player_loadout_items {
        if !found_classes.contains(*item) {
            missing_items.push(format!("player_loadout.hpp: {}", item));
        }
    }

    // Check arsenal items
    for item in &arsenal_items {
        if !found_classes.contains(*item) {
            missing_items.push(format!("arsenal.sqf: {}", item));
        }
    }

    // If any items are missing, fail the test with details
    assert!(missing_items.is_empty(), 
        "Missing dependencies:\n{}", missing_items.join("\n"));

    // Print some debug info about what we found
    debug!("Found {} total dependencies:", dependencies.len());
    let mut by_type: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
    for dep in &dependencies {
        by_type.entry(&dep.reference_type).or_default().push(&dep.class_name);
    }
    for (ref_type, classes) in &by_type {
        debug!("- {:?}: {} items", ref_type, classes.len());
    }

    Ok(())
} 