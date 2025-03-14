#[cfg(test)]
mod tests {
    use parser_sqm::extract_class_dependencies;

    #[test]
    fn test_parse_class_with_inventory() {
        let input = r#"class Item1 {
            dataType="Object";
            class Attributes {
                skill=1;
                name="B_C_AR";
                description="Automatic Rifleman";
                isPlayable=1;
                class Inventory {
                    class primaryWeapon {
                        name="rhs_weap_mg42";
                        firemode="rhs_weap_mg42:manual";
                        class primaryMuzzleMag {
                            name="rhsgref_50Rnd_792x57_SmE_drum";
                            ammoLeft=50;
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains("rhs_weap_mg42"));
        assert!(dependencies.contains("rhsgref_50Rnd_792x57_SmE_drum"));
    }

    #[test]
    fn test_parse_real_mission_file() {
        let mission_content = std::fs::read_to_string("tests/fixtures/example_mission.sqm")
            .expect("Unable to read example mission file");
        
        let dependencies = extract_class_dependencies(&mission_content);
        assert!(!dependencies.is_empty());
        
        // Check for some expected classes from the example
        assert!(dependencies.contains("rhs_weap_mg42"));
        assert!(dependencies.contains("rhsgref_50Rnd_792x57_SmE_drum"));
        assert!(dependencies.contains("rhsusf_weap_glock17g4"));
        assert!(dependencies.contains("rhsusf_mag_17Rnd_9x19_JHP"));
        assert!(dependencies.contains("TC_U_aegis_guerilla_garb_m81_sudan"));
        assert!(dependencies.contains("pca_eagle_a3_od"));
        assert!(dependencies.contains("simc_pasgt_m81"));
    }
    
    #[test]
    fn test_parse_nested_class_with_inventory() {
        let input = r#"
        class Mission {
            class Entities {
                class Item1 {
                    class Attributes {
                        class Inventory {
                            class primaryWeapon {
                                name="test_weapon";
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains("test_weapon"));
    }

    #[test]
    fn test_parse_multiple_inventories() {
        let input = r#"
        class Mission {
            class Entities {
                class Item1 {
                    class Attributes {
                        class Inventory {
                            class primaryWeapon {
                                name="weapon1";
                            };
                        };
                    };
                };
                class Item2 {
                    class Attributes {
                        class Inventory {
                            class primaryWeapon {
                                name="weapon2";
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains("weapon1"));
        assert!(dependencies.contains("weapon2"));
    }

    #[test]
    fn test_parse_mixed_hierarchy() {
        let input = r#"
        class Mission {
            class Item1 {
                class Attributes {
                    class Inventory {
                        name="direct_weapon";
                    };
                };
            };
            class Entities {
                class Item2 {
                    class Attributes {
                        class Inventory {
                            name="nested_weapon";
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains("direct_weapon"));
        assert!(dependencies.contains("nested_weapon"));
    }

    #[test]
    fn test_parse_direct_assignments() {
        let input = r#"
        class Mission {
            class Item1 {
                class Attributes {
                    class Inventory {
                        headgear="test_helmet";
                        uniform="test_uniform";
                        vest="test_vest";
                        backpack="test_backpack";
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 4);
        assert!(dependencies.contains("test_helmet"));
        assert!(dependencies.contains("test_uniform"));
        assert!(dependencies.contains("test_vest"));
        assert!(dependencies.contains("test_backpack"));
    }
}
