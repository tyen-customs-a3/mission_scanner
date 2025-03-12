#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::PathBuf;
    use parser_hpp::{parse_file, ClassKind, ItemKind};
    use env_logger;
    use log::debug;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_loadout_file_parsing() {
        init();
        debug!("Starting loadout.hpp parsing test");

        // Get the absolute path to the test file
        let test_file_path = std::env::current_dir()
            .unwrap()
            .join("tests")
            .join("fixtures")
            .join("loadout.hpp")
            .canonicalize()
            .expect("Failed to get absolute path to loadout.hpp");
        
        debug!("Test file path: {:?}", test_file_path);
        assert!(test_file_path.exists(), "Test file does not exist at {:?}", test_file_path);
        
        // Parse the file using the public interface
        let result = parse_file(&test_file_path, None).expect("Failed to parse loadout.hpp");

        debug!("Found {} classes", result.len());
        for class in &result {
            debug!("Class: {} (parent: {:?})", class.class_id, class.parent_class);
            for item in &class.items {
                debug!("  - Item: {} ({:?}) x{}", item.class_id, item.kind, item.count);
            }
        }

        // Verify base classes are found with correct inheritance
        let inheritance_map = vec![
            ("baseMan", None),
            ("rm", Some("baseMan")),
            ("ar", Some("rm")),
            ("aar", Some("rm")),
            ("rm_lat", Some("rm")),
            ("gren", Some("rm")),
            ("tl", Some("rm")),
            ("sl", Some("tl")),
            ("co", Some("sl")),
            ("rm_fa", Some("rm")),
            ("cls", Some("rm_fa"))
        ];

        for (class_name, parent) in inheritance_map {
            let class = result.iter()
                .find(|c| c.class_id == class_name)
                .unwrap_or_else(|| panic!("Expected class '{}' not found", class_name));
            
            assert_eq!(class.kind, ClassKind::Class);
            assert_eq!(class.parent_class.as_deref(), parent, 
                "Class '{}' should have parent '{:?}' but has '{:?}'", 
                class_name, parent, class.parent_class);
        }

        // Check specific class contents
        let base_man = result.iter().find(|c| c.class_id == "baseMan").unwrap();
        assert!(base_man.items.iter().any(|i| i.class_id == "ItemWatch" && i.kind == ItemKind::Item));
        assert!(base_man.items.iter().any(|i| i.class_id == "ItemMap" && i.kind == ItemKind::Item));
        assert!(base_man.items.iter().any(|i| i.class_id == "ItemCompass" && i.kind == ItemKind::Item));

        // Check rifleman (rm) class
        let rm = result.iter().find(|c| c.class_id == "rm").unwrap();
        let rm_uniforms = vec!["usp_g3c_kp_mx_aor2", "usp_g3c_rs_kp_mx_aor2", "usp_g3c_rs2_kp_mx_aor2"];
        for uniform in rm_uniforms {
            assert!(rm.items.iter().any(|i| i.class_id == uniform && i.kind == ItemKind::Uniform),
                "Uniform '{}' not found in rm class", uniform);
        }
        assert!(rm.items.iter().any(|i| i.class_id == "s4_lbt_comms_aor2" && i.kind == ItemKind::Vest));
        assert!(rm.items.iter().any(|i| i.class_id == "bear_eagleaiii_aor2" && i.kind == ItemKind::Backpack));
        assert!(rm.items.iter().any(|i| i.class_id == "rhs_weap_m4a1_blockII_KAC" && i.kind == ItemKind::Weapon));
        assert!(rm.items.iter().any(|i| i.class_id == "rhsusf_acc_g33_xps3" && i.kind == ItemKind::Attachment));
        assert!(rm.items.iter().any(|i| i.class_id == "rhsusf_acc_rvg_blk" && i.kind == ItemKind::Attachment));

        // Check automatic rifleman (ar) class
        let ar = result.iter().find(|c| c.class_id == "ar").unwrap();
        assert!(ar.items.iter().any(|i| i.class_id == "rhs_weap_m249_light_S" && i.kind == ItemKind::Weapon));
        assert!(ar.items.iter().any(|i| i.class_id == "rhsusf_weap_glock17g4" && i.kind == ItemKind::Weapon));
        assert!(ar.items.iter().any(|i| i.class_id == "rhsusf_200Rnd_556x45_mixed_soft_pouch" && i.kind == ItemKind::Magazine));

        // Check LAT rifleman
        let rm_lat = result.iter().find(|c| c.class_id == "rm_lat").unwrap();
        assert!(rm_lat.items.iter().any(|i| i.class_id == "rhs_weap_m72a7" && i.kind == ItemKind::Weapon));

        // Check grenadier
        let gren = result.iter().find(|c| c.class_id == "gren").unwrap();
        assert!(gren.items.iter().any(|i| i.class_id == "bear_weap_m4a1_blockII_m203" && i.kind == ItemKind::Weapon));
        assert!(gren.items.iter().any(|i| i.class_id == "rhs_mag_M441_HE" && i.kind == ItemKind::Magazine));
        assert!(gren.items.iter().any(|i| i.class_id == "rhs_mag_M433_HEDP" && i.kind == ItemKind::Magazine));

        // Check squad leader
        let sl = result.iter().find(|c| c.class_id == "sl").unwrap();
        assert!(sl.items.iter().any(|i| i.class_id == "rhs_weap_mk18_m320" && i.kind == ItemKind::Weapon));
        assert!(sl.items.iter().any(|i| i.class_id == "rhsusf_weap_glock17g4" && i.kind == ItemKind::Weapon));
        assert!(sl.items.iter().any(|i| i.class_id == "ACRE_PRC148" && i.kind == ItemKind::Item));

        // Check medic classes
        let rm_fa = result.iter().find(|c| c.class_id == "rm_fa").unwrap();
        let cls = result.iter().find(|c| c.class_id == "cls").unwrap();

        // Check medical items in CLS class
        let medical_items = vec![
            "ACE_elasticBandage",
            "ACE_packingBandage",
            "ACE_fieldDressing",
            "ACE_epinephrine",
            "ACE_morphine",
            "ACE_bloodIV",
            "ACE_splint",
            "ACE_tourniquet",
            "ACE_surgicalKit"
        ];
        for item in medical_items {
            assert!(cls.items.iter().any(|i| i.class_id == item && i.kind == ItemKind::Item),
                "Medical item '{}' not found in CLS class", item);
        }

        // Verify item counts
        let cls_bandages = cls.items.iter()
            .find(|i| i.class_id == "ACE_elasticBandage" && i.kind == ItemKind::Item)
            .expect("ACE_elasticBandage not found in CLS class");
        assert_eq!(cls_bandages.count, 30, "CLS should have 30 elastic bandages");

        let rm_magazines = rm.items.iter()
            .find(|i| i.class_id == "rhs_mag_30Rnd_556x45_M855A1_PMAG" && i.kind == ItemKind::Magazine)
            .expect("PMAG magazines not found in rifleman class");
        assert_eq!(rm_magazines.count, 13, "Rifleman should have 13 PMAG magazines");
    }

    #[test]
    fn test_basic_class_parsing() {
        init();
        let test_file = create_test_file(r#"
            class Soldier {
                displayName = "Basic Soldier";
                role = "Rifleman";
                uniform[] = {"U_B_CombatUniform_mcam"};
                vest[] = {"V_PlateCarrier1_rgr"};
                backpack[] = {"B_AssaultPack_mcamo"};
            };
        "#);

        let classes = parse_file(&test_file, None).unwrap();
        assert_eq!(classes.len(), 1);
        
        let soldier = &classes[0];
        assert_eq!(soldier.class_id, "Soldier");
        assert_eq!(soldier.kind, ClassKind::Class);
        
        // Check items
        assert!(soldier.items.iter().any(|item| item.class_id == "U_B_CombatUniform_mcam" && item.kind == ItemKind::Uniform));
        assert!(soldier.items.iter().any(|item| item.class_id == "V_PlateCarrier1_rgr" && item.kind == ItemKind::Vest));
        assert!(soldier.items.iter().any(|item| item.class_id == "B_AssaultPack_mcamo" && item.kind == ItemKind::Backpack));
    }

    #[test]
    fn test_class_inheritance() {
        init();
        let test_file = create_test_file(r#"
            class Soldier {
                displayName = "Basic Soldier";
                role = "Rifleman";
                uniform[] = {"U_B_CombatUniform_mcam"};
            };

            class Medic: Soldier {
                displayName = "Combat Medic";
                role = "Medic";
                backpack[] = {"B_Kitbag_mcamo"};
                items[] = {
                    "ACE_fieldDressing",
                    LIST_2("ACE_morphine"),
                    "ACE_epinephrine"
                };
            };

            class SquadLeader: Soldier {
                displayName = "Squad Leader";
                role = "Leader";
                vest[] = {"V_PlateCarrierGL_rgr"};
                binocular[] = {"ACE_Vector"};
            };
        "#);

        let classes = parse_file(&test_file, None).unwrap();
        assert_eq!(classes.len(), 3);
        
        let medic = classes.iter().find(|c| c.class_id == "Medic").unwrap();
        assert_eq!(medic.kind, ClassKind::Class);
        assert_eq!(medic.parent_class.as_deref(), Some("Soldier"));
        
        // Check medic items
        assert!(medic.items.iter().any(|item| item.class_id == "B_Kitbag_mcamo" && item.kind == ItemKind::Backpack));
        assert!(medic.items.iter().any(|item| item.class_id == "ACE_fieldDressing" && item.kind == ItemKind::Item));
        assert!(medic.items.iter().any(|item| item.class_id == "ACE_morphine" && item.kind == ItemKind::Item && item.count == 2));
        assert!(medic.items.iter().any(|item| item.class_id == "ACE_epinephrine" && item.kind == ItemKind::Item));
        
        let leader = classes.iter().find(|c| c.class_id == "SquadLeader").unwrap();
        assert_eq!(leader.kind, ClassKind::Class);
        assert_eq!(leader.parent_class.as_deref(), Some("Soldier"));
        
        // Check leader items
        assert!(leader.items.iter().any(|item| item.class_id == "V_PlateCarrierGL_rgr" && item.kind == ItemKind::Vest));
        assert!(leader.items.iter().any(|item| item.class_id == "ACE_Vector" && item.kind == ItemKind::Binocular));
    }

    #[test]
    fn test_array_assignments() {
        init();
        let test_file = create_test_file(r#"
            class Loadout {
                weapons[] = {
                    "arifle_MX_F",
                    "hgun_P07_F"
                };
                magazines[] = {
                    "30Rnd_65x39_caseless_mag",
                    "16Rnd_9x21_Mag"
                };
            };
        "#);

        let classes = parse_file(&test_file, None).unwrap();
        assert_eq!(classes.len(), 1);
        
        let loadout = &classes[0];
        assert_eq!(loadout.class_id, "Loadout");
        
        // Check weapons and magazines
        assert!(loadout.items.iter().any(|item| item.class_id == "arifle_MX_F" && item.kind == ItemKind::Weapon));
        assert!(loadout.items.iter().any(|item| item.class_id == "hgun_P07_F" && item.kind == ItemKind::Weapon));
        assert!(loadout.items.iter().any(|item| item.class_id == "30Rnd_65x39_caseless_mag" && item.kind == ItemKind::Magazine));
        assert!(loadout.items.iter().any(|item| item.class_id == "16Rnd_9x21_Mag" && item.kind == ItemKind::Magazine));
    }

    #[test]
    fn test_nested_classes() {
        init();
        let test_file = create_test_file(r#"
            class CfgVehicles {
                class Soldier_Base_F {
                    scope = 1;
                    side = 1;
                };
                
                class B_Soldier_base_F: Soldier_Base_F {
                    scope = 1;
                    faction = "BLU_F";
                };
                
                class B_Soldier_F: B_Soldier_base_F {
                    scope = 2;
                    displayName = "Rifleman";
                };
            };
        "#);

        let classes = parse_file(&test_file, None).unwrap();
        assert_eq!(classes.len(), 4); // Including CfgVehicles
        
        assert!(classes.iter().any(|c| c.class_id == "CfgVehicles"));
        assert!(classes.iter().any(|c| c.class_id == "Soldier_Base_F"));
        
        let base_soldier = classes.iter().find(|c| c.class_id == "B_Soldier_base_F").unwrap();
        assert_eq!(base_soldier.parent_class.as_deref(), Some("Soldier_Base_F"));
        
        let soldier = classes.iter().find(|c| c.class_id == "B_Soldier_F").unwrap();
        assert_eq!(soldier.parent_class.as_deref(), Some("B_Soldier_base_F"));
    }

    #[test]
    fn test_macro_handling() {
        init();
        let test_file = create_test_file(r#"
            #define MACRO_LOADOUT(role) \
                class ##role { \
                    displayName = QUOTE(role); \
                }

            class CfgLoadouts {
                MACRO_LOADOUT(Rifleman);
                MACRO_LOADOUT(Medic);
                MACRO_LOADOUT(Engineer);
            };
        "#);

        let classes = parse_file(&test_file, None).unwrap();
        assert!(classes.len() >= 1); // At minimum CfgLoadouts should be found
    }

    /// Helper function to create a temporary test file
    fn create_test_file(content: &str) -> PathBuf {
        use std::fs::File;
        use std::io::Write;
        
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.hpp");
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        file_path
    }
} 