#[cfg(test)]
mod tests {
    use parser_sqm::extract_class_dependencies;

    #[test]
    fn test_parse_class_with_inventory() {
        let input = r#"class Mission {
            class Item1 {
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
    fn test_parse_nested_inventory_with_cargo() {
        let input = r#"
        class Mission {
            class Entities {
                class Item1 {
                    class Attributes {
                        class Inventory {
                            class uniform {
                                typeName = "test_uniform";
                                class ItemCargo {
                                    items = 2;
                                    class Item0 {
                                        name = "test_item1";
                                        count = 1;
                                    };
                                    class Item1 {
                                        name = "test_item2";
                                        count = 2;
                                    };
                                };
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains("test_uniform"));
        assert!(dependencies.contains("test_item1"));
        assert!(dependencies.contains("test_item2"));
    }

    #[test]
    fn test_parse_equipment_properties() {
        let input = r#"
        class Mission {
            class Item1 {
                class Attributes {
                    class Inventory {
                        uniform = "test_uniform";
                        vest = "test_vest";
                        backpack = "test_backpack";
                        headgear = "test_helmet";
                        map = "test_map";
                        compass = "test_compass";
                        watch = "test_watch";
                        radio = "test_radio";
                        gps = "test_gps";
                        goggles = "test_goggles";
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 10);
        assert!(dependencies.contains("test_uniform"));
        assert!(dependencies.contains("test_vest"));
        assert!(dependencies.contains("test_backpack"));
        assert!(dependencies.contains("test_helmet"));
        assert!(dependencies.contains("test_map"));
        assert!(dependencies.contains("test_compass"));
        assert!(dependencies.contains("test_watch"));
        assert!(dependencies.contains("test_radio"));
        assert!(dependencies.contains("test_gps"));
        assert!(dependencies.contains("test_goggles"));
    }

    #[test]
    fn test_parse_weapon_magazines() {
        let input = r#"
        class Mission {
            class Item1 {
                class Attributes {
                    class Inventory {
                        class primaryWeapon {
                            name = "test_rifle";
                            muzzle = "test_muzzle";
                            class primaryMuzzleMag {
                                name = "test_mag";
                            };
                        };
                        class secondaryWeapon {
                            name = "test_launcher";
                            class primaryMuzzleMag {
                                name = "test_rocket";
                            };
                        };
                        class handgunWeapon {
                            name = "test_pistol";
                            class primaryMuzzleMag {
                                name = "test_pistol_mag";
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 7);
        assert!(dependencies.contains("test_rifle"));
        assert!(dependencies.contains("test_muzzle"));
        assert!(dependencies.contains("test_mag"));
        assert!(dependencies.contains("test_launcher"));
        assert!(dependencies.contains("test_rocket"));
        assert!(dependencies.contains("test_pistol"));
        assert!(dependencies.contains("test_pistol_mag"));
    }

    #[test]
    fn test_deep_nested_structure() {
        let input = r#"class Mission {
            class Item0 {
                class Attributes {
                    class Inventory {
                        class Item1 {
                            class Attributes {
                                class Inventory {
                                    class Item2 {
                                        class Attributes {
                                            class Inventory {
                                                uniform = "test_uniform";
                                                class primaryWeapon {
                                                    name = "test_rifle";
                                                    class primaryMuzzleMag {
                                                        name = "test_mag";
                                                    };
                                                };
                                            };
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains("test_uniform"));
        assert!(dependencies.contains("test_rifle"));
        assert!(dependencies.contains("test_mag"));
    }

    #[test]
    fn test_wide_structure() {
        let input = r#"class Mission {
            class Item0 {
                class Attributes {
                    class Inventory {
                        uniform = "test_uniform_0";
                        class primaryWeapon {
                            name = "test_rifle_0";
                        };
                    };
                };
            };
            class Item1 {
                class Attributes {
                    class Inventory {
                        uniform = "test_uniform_1";
                        class primaryWeapon {
                            name = "test_rifle_1";
                        };
                    };
                };
            };
            class Item2 {
                class Attributes {
                    class Inventory {
                        uniform = "test_uniform_2";
                        class primaryWeapon {
                            name = "test_rifle_2";
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 6);
        for i in 0..3 {
            assert!(dependencies.contains(&format!("test_uniform_{}", i)));
            assert!(dependencies.contains(&format!("test_rifle_{}", i)));
        }
    }

    #[test]
    fn test_mixed_structure() {
        let input = r#"class Mission {
            class Deep0 {
                class Attributes {
                    class Inventory {
                        class Deep1 {
                            class Attributes {
                                class Inventory {
                                    class Item0 {
                                        uniform = "test_uniform_0";
                                        class primaryWeapon {
                                            name = "test_rifle_0";
                                        };
                                    };
                                    class Item1 {
                                        uniform = "test_uniform_1";
                                        class primaryWeapon {
                                            name = "test_rifle_1";
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 4);
        for i in 0..2 {
            assert!(dependencies.contains(&format!("test_uniform_{}", i)));
            assert!(dependencies.contains(&format!("test_rifle_{}", i)));
        }
    }
}
