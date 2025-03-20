mod models;
mod parser;
mod query;

use parser::parse_sqm;
use query::QueryEngine;

use std::collections::HashSet;

pub fn extract_class_dependencies(sqm_content: &str) -> HashSet<String> {
    match parse_sqm(sqm_content) {
        Ok(sqm) => {
            let query_engine = QueryEngine::new(&sqm);
            
            let mut all_dependencies = query_engine.extract_inventory_dependencies();
            let object_types = query_engine.extract_object_types();
            all_dependencies.extend(object_types);
            
            all_dependencies
        }
        Err(_) => HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_class_with_inventory() {
        let input = r#"
        class Mission {
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

    #[test]
    fn test_parse_with_comments_and_defines() {
        let input = r#"
        ////////////////////////////////////////////////////////////////////
        //DeRap: mission.sqm
        //Produced from mikero's Dos Tools Dll version 9.98
        
        #define _ARMA_
        
        class Mission {
            class Item1 {
                class Attributes {
                    class Inventory {
                        // This is a comment
                        name="test_weapon"; // Inline comment
                    };
                };
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains("test_weapon"));
    }
    
    #[test]
    fn test_paramedic_inventory() {
        let input = r#"
        class Mission {
            class Item1 {
                dataType = "Object";
                class PositionInfo {
                    position[] = {11376.814,17.579823,3333.3503};
                    angles[] = {0.0,4.887411,0.0};
                };
                side = "Civilian";
                flags = 1;
                class Attributes {
                    class Inventory {
                        class uniform {
                            typeName = "U_C_Paramedic_01_F";
                            isBackpack = 0;
                            class ItemCargo {
                                items = 1;
                                class Item0 {
                                    name = "FirstAidKit";
                                    count = 3;
                                };
                            };
                        };
                        class backpack {
                            typeName = "B_Messenger_Gray_Medical_F";
                            isBackpack = 1;
                            class ItemCargo {
                                items = 2;
                                class Item0 {
                                    name = "Medikit";
                                    count = 1;
                                };
                                class Item1 {
                                    name = "FirstAidKit";
                                    count = 7;
                                };
                            };
                        };
                        map = "ItemMap";
                        compass = "ItemCompass";
                        watch = "ItemWatch";
                        radio = "ItemRadio";
                        gps = "ItemGPS";
                    };
                };
                id = 1048;
                type = "C_Man_Paramedic_01_F";
                atlOffset = 3.5353775;
            };
        };"#;
        
        let dependencies = extract_class_dependencies(input);
        
        // Check for uniform
        assert!(dependencies.contains("U_C_Paramedic_01_F"), "Uniform not found");
        
        // Check for backpack and its contents
        assert!(dependencies.contains("B_Messenger_Gray_Medical_F"), "Backpack not found");
        assert!(dependencies.contains("Medikit"), "Medikit not found");
        assert!(dependencies.contains("FirstAidKit"), "FirstAidKit not found");
        
        // Check for equipment
        assert!(dependencies.contains("ItemMap"), "Map not found");
        assert!(dependencies.contains("ItemCompass"), "Compass not found");
        assert!(dependencies.contains("ItemWatch"), "Watch not found");
        assert!(dependencies.contains("ItemRadio"), "Radio not found");
        assert!(dependencies.contains("ItemGPS"), "GPS not found");
        
        // Check object type
        assert!(dependencies.contains("C_Man_Paramedic_01_F"), "Object type not found");
    }
}