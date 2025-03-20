use std::collections::HashSet;
use crate::models::{ Class, Property, PropertyValue};

pub struct QueryEngine<'a> {
    sqm: &'a Class,
}

impl<'a> QueryEngine<'a> {
    pub fn new(sqm: &'a Class) -> Self {
        Self { sqm }
    }

    pub fn find_classes(&self, name: &str) -> Vec<&Class> {
        self.sqm.find_classes(&|class| class.name == name)
    }

    pub fn find_properties(&self, name: &str) -> Vec<&Property> {
        self.sqm.find_properties(&|prop| prop.name == name)
    }

    pub fn find_classes_with_property(&self, property_name: &str, property_value: &str) -> Vec<&Class> {
        self.sqm.find_classes(&|class| {
            class.properties.get(property_name)
                .map(|prop| matches!(&prop.value, PropertyValue::String(s) if s == property_value))
                .unwrap_or(false)
        })
    }

    pub fn find_property_values(&self, class_name: &str, property_name: &str) -> Vec<String> {
        let mut values = Vec::new();
        
        for class in self.find_classes(class_name) {
            if let Some(prop) = class.properties.get(property_name) {
                match &prop.value {
                    PropertyValue::String(s) => values.push(s.clone()),
                    PropertyValue::Array(arr) => values.extend(arr.clone()),
                    _ => {}
                }
            }
        }
        
        values
    }

    pub fn extract_object_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        
        // Find all classes that have a type property
        let item_classes = self.sqm.find_classes(&|class| {
            class.properties.contains_key("type")
        });
        
        for class in item_classes {
            if let Some(prop) = class.properties.get("type") {
                if let PropertyValue::String(value) = &prop.value {
                    if !value.is_empty() {
                        types.insert(value.clone());
                    }
                }
            }
        }
        
        types
    }

    pub fn extract_inventory_dependencies(&self) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        
        // Find all classes that might contain inventory items
        let inventory_classes = self.sqm.find_classes(&|class| {
            class.name == "Inventory" || 
            class.name == "primaryWeapon" || 
            class.name == "secondaryWeapon" || 
            class.name == "handgunWeapon" ||
            class.name == "primaryMuzzleMag" ||
            class.name == "secondaryMuzzleMag" ||
            class.name == "handgunMuzzleMag"
        });
        
        // Equipment property names to look for
        let equipment_props = [
            "uniform", "vest", "backpack", "headgear", "weapon", "name",
            "primaryWeapon", "secondaryWeapon", "handgunWeapon",
            "primaryMuzzleMag", "secondaryMuzzleMag", "handgunMuzzleMag",
            "map", "compass", "watch", "radio", "gps", "goggles"
        ];
        
        for class in inventory_classes {
            // Check direct equipment properties
            for prop_name in &equipment_props {
                if let Some(prop) = class.properties.get(*prop_name) {
                    if let PropertyValue::String(value) = &prop.value {
                        if !value.is_empty() && !value.contains(":") {
                            dependencies.insert(value.clone());
                        }
                    }
                }
            }
            
            // Check array properties that might contain items
            for prop in class.properties.values() {
                if let PropertyValue::Array(items) = &prop.value {
                    dependencies.extend(items.iter().filter(|s| !s.is_empty()).cloned());
                }
            }
            
            // Check inventory item classes for typeName property (uniform, backpack, etc.)
            for nested in &class.nested_classes {
                // Check for typeName property that indicates equipment type
                if let Some(prop) = nested.properties.get("typeName") {
                    if let PropertyValue::String(value) = &prop.value {
                        if !value.is_empty() && !value.contains(":") {
                            dependencies.insert(value.clone());
                        }
                    }
                }
                
                // Also process ItemCargo classes to find items inside containers
                let item_cargo_classes = nested.find_classes(&|c| c.name == "ItemCargo");
                for cargo_class in item_cargo_classes {
                    let item_classes = cargo_class.find_classes(&|c| c.name.starts_with("Item"));
                    for item_class in item_classes {
                        if let Some(prop) = item_class.properties.get("name") {
                            if let PropertyValue::String(value) = &prop.value {
                                if !value.is_empty() && !value.contains(":") {
                                    dependencies.insert(value.clone());
                                }
                            }
                        }
                    }
                }
                
                // Continue to recursively check nested classes for 'name' props
                if let Some(prop) = nested.properties.get("name") {
                    if let PropertyValue::String(value) = &prop.value {
                        if !value.is_empty() && !value.contains(":") {
                            dependencies.insert(value.clone());
                        }
                    }
                }
            }
        }
        
        dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Class, Property, PropertyValue};

    fn create_test_sqm() -> Class {
        let mut sqm = Class::new("Mission".to_string());
        
        // Create a test class structure similar to real mission files
        let mut item = Class::new("Item1".to_string());
        
        // Add type property for item types test
        item.properties.insert("type".to_string(), Property {
            name: "type".to_string(),
            value: PropertyValue::String("Land_Rugbyball_01_F".to_string()),
        });
        
        let mut attributes = Class::new("Attributes".to_string());
        let mut inventory = Class::new("Inventory".to_string());
        
        // Add primary weapon class
        let mut primary_weapon = Class::new("primaryWeapon".to_string());
        primary_weapon.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("rhs_weap_mg42".to_string()),
        });
        primary_weapon.properties.insert("firemode".to_string(), Property {
            name: "firemode".to_string(),
            value: PropertyValue::String("rhs_weap_mg42:manual".to_string()),
        });
        
        // Add primary magazine class
        let mut primary_mag = Class::new("primaryMuzzleMag".to_string());
        primary_mag.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("rhsgref_50Rnd_792x57_SmE_drum".to_string()),
        });
        primary_mag.properties.insert("ammoLeft".to_string(), Property {
            name: "ammoLeft".to_string(),
            value: PropertyValue::Number(50.0),
        });
        
        // Add handgun class
        let mut handgun = Class::new("handgunWeapon".to_string());
        handgun.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("rhsusf_weap_glock17g4".to_string()),
        });
        
        // Add handgun magazine
        let mut handgun_mag = Class::new("handgunMuzzleMag".to_string());
        handgun_mag.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("rhsusf_mag_17Rnd_9x19_JHP".to_string()),
        });
        
        // Add uniform and other equipment
        inventory.properties.insert("uniform".to_string(), Property {
            name: "uniform".to_string(),
            value: PropertyValue::String("TC_U_aegis_guerilla_garb_m81_sudan".to_string()),
        });
        inventory.properties.insert("vest".to_string(), Property {
            name: "vest".to_string(),
            value: PropertyValue::String("pca_eagle_a3_od".to_string()),
        });
        inventory.properties.insert("headgear".to_string(), Property {
            name: "headgear".to_string(),
            value: PropertyValue::String("simc_pasgt_m81".to_string()),
        });
        
        // Build the class hierarchy
        primary_weapon.nested_classes.push(primary_mag);
        handgun.nested_classes.push(handgun_mag);
        inventory.nested_classes.push(primary_weapon);
        inventory.nested_classes.push(handgun);
        attributes.nested_classes.push(inventory);
        item.nested_classes.push(attributes);
        sqm.nested_classes.push(item);
        
        sqm
    }

    fn create_test_sqm_with_typename() -> Class {
        let mut sqm = Class::new("Mission".to_string());
        let mut item = Class::new("Item1".to_string());
        
        // Add type property for object type
        item.properties.insert("type".to_string(), Property {
            name: "type".to_string(),
            value: PropertyValue::String("C_Man_Paramedic_01_F".to_string()),
        });
        
        let mut attributes = Class::new("Attributes".to_string());
        let mut inventory = Class::new("Inventory".to_string());
        
        // Add direct properties
        inventory.properties.insert("map".to_string(), Property {
            name: "map".to_string(),
            value: PropertyValue::String("ItemMap".to_string()),
        });
        
        inventory.properties.insert("compass".to_string(), Property {
            name: "compass".to_string(),
            value: PropertyValue::String("ItemCompass".to_string()),
        });
        
        // Add uniform class with typeName
        let mut uniform = Class::new("uniform".to_string());
        uniform.properties.insert("typeName".to_string(), Property {
            name: "typeName".to_string(),
            value: PropertyValue::String("U_C_Paramedic_01_F".to_string()),
        });
        
        // Add ItemCargo to uniform
        let mut item_cargo = Class::new("ItemCargo".to_string());
        let mut item0 = Class::new("Item0".to_string());
        item0.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("FirstAidKit".to_string()),
        });
        item_cargo.nested_classes.push(item0);
        uniform.nested_classes.push(item_cargo);
        
        // Add backpack class with typeName
        let mut backpack = Class::new("backpack".to_string());
        backpack.properties.insert("typeName".to_string(), Property {
            name: "typeName".to_string(),
            value: PropertyValue::String("B_Messenger_Gray_Medical_F".to_string()),
        });
        
        // Add ItemCargo to backpack
        let mut backpack_cargo = Class::new("ItemCargo".to_string());
        let mut backpack_item0 = Class::new("Item0".to_string());
        backpack_item0.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("Medikit".to_string()),
        });
        backpack_cargo.nested_classes.push(backpack_item0);
        backpack.nested_classes.push(backpack_cargo);
        
        // Build the class hierarchy
        inventory.nested_classes.push(uniform);
        inventory.nested_classes.push(backpack);
        attributes.nested_classes.push(inventory);
        item.nested_classes.push(attributes);
        sqm.nested_classes.push(item);
        
        sqm
    }

    #[test]
    fn test_find_classes() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let inventory_classes = query.find_classes("Inventory");
        assert_eq!(inventory_classes.len(), 1);
        assert_eq!(inventory_classes[0].name, "Inventory");
    }

    #[test]
    fn test_find_properties() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let weapon_props = query.find_properties("name");
        assert!(!weapon_props.is_empty());
        assert!(weapon_props.iter().any(|p| matches!(&p.value, PropertyValue::String(s) if s == "rhs_weap_mg42")));
    }

    #[test]
    fn test_find_classes_with_property() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let classes = query.find_classes_with_property("name", "rhs_weap_mg42");
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "primaryWeapon");
    }

    #[test]
    fn test_find_property_values() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let values = query.find_property_values("primaryWeapon", "name");
        assert_eq!(values, vec!["rhs_weap_mg42"]);
    }

    #[test]
    fn test_extract_item_types() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let types = query.extract_object_types();
        assert!(types.contains("Land_Rugbyball_01_F"));
    }

    #[test]
    fn test_extract_inventory_dependencies() {
        let sqm = create_test_sqm();
        let query = QueryEngine::new(&sqm);
        
        let dependencies = query.extract_inventory_dependencies();
        assert!(dependencies.contains("rhs_weap_mg42"));
        assert!(dependencies.contains("rhsgref_50Rnd_792x57_SmE_drum"));
        assert!(dependencies.contains("rhsusf_weap_glock17g4"));
        assert!(dependencies.contains("rhsusf_mag_17Rnd_9x19_JHP"));
        assert!(dependencies.contains("TC_U_aegis_guerilla_garb_m81_sudan"));
        assert!(dependencies.contains("pca_eagle_a3_od"));
        assert!(dependencies.contains("simc_pasgt_m81"));
    }
    
    #[test]
    fn test_extract_typename_properties() {
        let sqm = create_test_sqm_with_typename();
        let query = QueryEngine::new(&sqm);
        
        let dependencies = query.extract_inventory_dependencies();
        
        // Check for uniform with typeName
        assert!(dependencies.contains("U_C_Paramedic_01_F"), "Uniform not found");
        
        // Check for backpack with typeName
        assert!(dependencies.contains("B_Messenger_Gray_Medical_F"), "Backpack not found");
        
        // Check for items in containers
        assert!(dependencies.contains("FirstAidKit"), "FirstAidKit not found");
        assert!(dependencies.contains("Medikit"), "Medikit not found");
        
        // Check for direct properties
        assert!(dependencies.contains("ItemMap"), "Map not found");
        assert!(dependencies.contains("ItemCompass"), "Compass not found");
    }
    
    #[test]
    fn test_paramedic_example() {
        use crate::parser::parse_sqm;
        
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
        
        match parse_sqm(input) {
            Ok(sqm) => {
                let query = QueryEngine::new(&sqm);
                let dependencies = query.extract_inventory_dependencies();
                let object_types = query.extract_object_types();
                
                let mut all_dependencies = dependencies;
                all_dependencies.extend(object_types);
                
                // Check for uniform
                assert!(all_dependencies.contains("U_C_Paramedic_01_F"), "Uniform not found");
                
                // Check for backpack and its contents
                assert!(all_dependencies.contains("B_Messenger_Gray_Medical_F"), "Backpack not found");
                assert!(all_dependencies.contains("Medikit"), "Medikit not found");
                assert!(all_dependencies.contains("FirstAidKit"), "FirstAidKit not found");
                
                // Check for equipment
                assert!(all_dependencies.contains("ItemMap"), "Map not found");
                assert!(all_dependencies.contains("ItemCompass"), "Compass not found");
                assert!(all_dependencies.contains("ItemWatch"), "Watch not found");
                assert!(all_dependencies.contains("ItemRadio"), "Radio not found");
                assert!(all_dependencies.contains("ItemGPS"), "GPS not found");
                
                // Check object type
                assert!(all_dependencies.contains("C_Man_Paramedic_01_F"), "Object type not found");
            },
            Err(_) => panic!("Failed to parse SQM")
        }
    }
} 