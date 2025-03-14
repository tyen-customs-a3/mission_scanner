#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::{PathBuf, Path};
    use std::io::Write;
    use parser_sqf::{parse_file, ItemKind};
    use hemtt_workspace::{Workspace, LayerType, WorkspacePath};
    use hemtt_common::config::PDriveOption;
    use log::debug;
    use std::fs::File;
    use tempfile::tempdir;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_arsenal_file_parsing() {
        init();

        // Get the test file path
        let test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("example_data")
            .join("arsenal.sqf");
        
        debug!("Test file path: {:?}", test_file_path);
        assert!(test_file_path.exists(), "Test file does not exist at {:?}", test_file_path);
        
        // Parse the file using the workspace file
        let result = parse_file(&test_file_path)
            .expect("Failed to parse arsenal.sqf");

        // Verify items that should have specific types based on how they're used
        let weapons = vec![
            "rhs_weap_hk416d145",
            "rhs_weap_m16a4_imod",
            "rhs_weap_m4a1_m320",
            "rhs_weap_M136"
        ];
        for weapon in weapons {
            assert!(
                result.iter().any(|item| item.class_name == weapon && item.kind == ItemKind::Weapon),
                "Expected weapon '{}' not found or has wrong type",
                weapon
            );
        }

        let magazines = vec![
            "rhs_mag_30Rnd_556x45_M855A1_Stanag",
            "rhsusf_200Rnd_556x45_M855_mixed_soft_pouch"
        ];
        for magazine in magazines {
            assert!(
                result.iter().any(|item| item.class_name == magazine && item.kind == ItemKind::Magazine),
                "Expected magazine '{}' not found or has wrong type",
                magazine
            );
        }

        // Check uniform
        assert!(
            result.iter().any(|item| item.class_name == "Tarkov_Uniforms_1" && item.kind == ItemKind::Uniform),
            "Expected uniform not found or has wrong type"
        );

        // Check vest
        assert!(
            result.iter().any(|item| item.class_name == "V_PlateCarrier2_blk" && item.kind == ItemKind::Vest),
            "Expected vest not found or has wrong type"
        );

        // Check backpack
        assert!(
            result.iter().any(|item| item.class_name == "rhsusf_spcs_ocp_saw" && item.kind == ItemKind::Backpack),
            "Expected backpack not found or has wrong type"
        );

        // All other items should be found but with ItemKind::Item
        let generic_items = vec![
            "rhsusf_acc_eotech_552",
            "rhsusf_acc_compm4",
            "rhsusf_acc_grip1",
            "rhsusf_acc_grip2",
            "rhsusf_acc_grip3",
            "rhsusf_acc_grip4",
            "rhsusf_acc_grip4_bipod",
            "rhsusf_acc_saw_lw_bipod",
            "ACE_HandFlare_Green",
            "ACE_HandFlare_Red",
            "ACE_HandFlare_White",
            "ACE_HandFlare_Yellow",
            "1Rnd_HE_Grenade_shell",
            "1Rnd_Smoke_Grenade_shell",
            "HandGrenade",
            "SmokeShell"
        ];
        for item in generic_items {
            assert!(
                result.iter().any(|i| i.class_name == item && i.kind == ItemKind::Item),
                "Expected generic item '{}' not found or has wrong type",
                item
            );
        }
    }

    #[test]
    fn test_getvariable_not_treated_as_item() {
        init();

        // Create a temporary directory for our test file
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_file_path = temp_dir.path().join("test_getvariable.sqf");
        
        // Create test content
        let test_content = r#"
        private _unitRole = _unit getVariable ["tmf_assignGear_role", nil];
        private _unitRoleCommon = ["rm","rm_lat","rm_mat", "medic", "engineer"];
        private _unitRoleForceBackpack = ["rm_mat", "medic", "engineer"];
        "#;
        
        // Write the test content to the file
        let mut file = File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content.as_bytes()).expect("Failed to write test content");
        
        // Parse the file
        let result = parse_file(&test_file_path).expect("Failed to parse test_getvariable.sqf");
        
        // Verify that "tmf_assignGear_role" is not treated as an item
        assert!(
            !result.iter().any(|item| item.class_name == "tmf_assignGear_role"),
            "getVariable parameter 'tmf_assignGear_role' should not be treated as an item"
        );
        
        // Verify that array elements are still found as items
        // These should be found because they're in an array assignment
        let expected_items = vec!["rm", "rm_lat", "rm_mat", "medic", "engineer"];
        for item in expected_items {
            assert!(
                result.iter().any(|i| i.class_name == item),
                "Expected array item '{}' not found",
                item
            );
        }
    }
} 