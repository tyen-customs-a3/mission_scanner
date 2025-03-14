use std::path::PathBuf;
use anyhow::Result;
use log::debug;

use mission_scanner::{
    scan_mission,
    MissionScannerConfig,
    ReferenceType,
};

use env_logger;

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

fn get_test_data_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("tests").join("fixtures")
}

#[tokio::test]
async fn test_scan_single_mission() -> Result<()> {
    let test_dir = get_test_data_dir().join("test_mission_1");
    debug!("Test directory: {}", test_dir.display());
    
    let config = MissionScannerConfig::default();
    let result = scan_mission(&test_dir, num_cpus::get(), &config).await?;
    
    assert_eq!(result.mission_name, "test_mission_1");
    assert!(result.sqm_file.is_some(), "Should find mission.sqm");
    assert!(!result.sqf_files.is_empty(), "Should find SQF files");
    assert!(!result.cpp_files.is_empty(), "Should find CPP/HPP files");
    
    Ok(())
}

#[tokio::test]
async fn test_scan_mission_with_config() -> Result<()> {
    let test_dir = get_test_data_dir().join("test_mission_1");
    debug!("Test directory: {}", test_dir.display());
    
    let mut config = MissionScannerConfig::default();
    config.file_extensions = vec!["sqm".to_string(), "sqf".to_string()];
    let result = scan_mission(&test_dir, num_cpus::get(), &config).await?;
    
    assert_eq!(result.mission_name, "test_mission_1");
    assert!(result.sqm_file.is_some(), "Should find mission.sqm");
    assert!(!result.sqf_files.is_empty(), "Should find SQF files");
    assert!(result.cpp_files.is_empty(), "Should not find CPP/HPP files");
    
    Ok(())
}

#[tokio::test]
async fn test_mission_class_dependencies() -> Result<()> {
    init();
    let test_dir = get_test_data_dir().join("test_mission_1");
    println!("Test directory: {}", test_dir.display());
    
    let config = MissionScannerConfig::default();
    let result = scan_mission(&test_dir, num_cpus::get(), &config).await?;
    println!("Found {} dependencies", result.class_dependencies.len());
    
    // Get all class names from dependencies
    let found_classes: std::collections::HashSet<_> = result.class_dependencies.iter()
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
    let reference_types: std::collections::HashSet<_> = result.class_dependencies.iter()
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