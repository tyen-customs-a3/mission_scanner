#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::PathBuf;
    use parser_sqf::{parse_file, ItemKind};

    #[test]
    fn test_arsenal_file_parsing() {
        // Read the arsenal.sqf file path
        let test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("example_data")
            .join("arsenal.sqf");
        
        // Parse the file using the public interface
        let result = parse_file(&test_file_path, None).expect("Failed to parse arsenal.sqf");

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
} 