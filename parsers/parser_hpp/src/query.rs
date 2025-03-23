use std::collections::{HashMap, HashSet};
use crate::{HppClass, HppValue};

/// Represents a query pattern to search for and extract data from HPP classes
#[derive(Debug, Clone)]
pub struct QueryPattern {
    /// The path to search for (e.g. "baseMan/uniform")
    path: Vec<String>,
    /// Properties to extract from matching classes
    properties: HashSet<String>,
}

impl QueryPattern {
    /// Create a new query pattern
    pub fn new(path: &str, properties: &[&str]) -> Self {
        Self {
            path: path.split('/').map(String::from).collect(),
            properties: properties.iter().map(|&s| s.to_string()).collect(),
        }
    }

    /// Check if a class matches this pattern's path
    fn matches_path(&self, class_path: &[String]) -> bool {
        if class_path.len() < self.path.len() {
            return false;
        }
        
        // Check if the end of the class_path matches our pattern path
        let start_idx = class_path.len() - self.path.len();
        class_path[start_idx..].iter().zip(&self.path)
            .all(|(a, b)| b == "*" || a == b)
    }
}

/// Extracts class dependencies from an HPP file using predefined patterns
pub struct DependencyExtractor {
    classes: Vec<HppClass>,
    patterns: Vec<QueryPattern>,
    /// Cache of property names to look for
    property_names: HashSet<String>,
}

impl DependencyExtractor {
    /// Create a new dependency extractor with default patterns for loadout files
    pub fn new(classes: Vec<HppClass>) -> Self {
        let patterns = vec![
            // Base equipment
            QueryPattern::new("*", &[
                "uniform", "vest", "backpack", "headgear",
                "goggles", "hmd", "faces", "insignias"
            ]),
            
            // Weapons and attachments
            QueryPattern::new("*", &[
                "primaryWeapon", "scope", "bipod", "attachment", "silencer",
                "secondaryWeapon", "secondaryAttachments",
                "sidearmWeapon", "sidearmAttachments"
            ]),
            
            // Items and magazines
            QueryPattern::new("*", &[
                "magazines", "items", "linkedItems", "backpackItems"
            ]),

            // Nested class properties
            QueryPattern::new("*/primaryWeapon", &["name"]),
            QueryPattern::new("*/secondaryWeapon", &["name"]),
            QueryPattern::new("*/sidearmWeapon", &["name"]),
        ];

        // Build property name cache
        let mut property_names = HashSet::new();
        for pattern in &patterns {
            property_names.extend(pattern.properties.iter().cloned());
        }

        Self { 
            classes,
            patterns,
            property_names,
        }
    }

    /// Extract all class dependencies from the HPP classes
    pub fn extract_dependencies(&self) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        
        for class in &self.classes {
            // Build property index for fast lookup
            let property_index: HashMap<_, _> = class.properties.iter()
                .filter(|p| self.property_names.contains(&p.name))
                .map(|p| (&p.name, &p.value))
                .collect();
            
            self.process_class(class, &[], &property_index, &mut dependencies);
        }
        
        dependencies
    }
    
    /// Process a class and its properties recursively
    fn process_class(
        &self,
        class: &HppClass,
        current_path: &[String],
        property_index: &HashMap<&String, &HppValue>,
        dependencies: &mut HashSet<String>
    ) {
        // Build the current class path
        let mut class_path = current_path.to_vec();
        class_path.push(class.name.clone());
        
        // Check each pattern against the current class
        for pattern in &self.patterns {
            if pattern.matches_path(&class_path) {
                // Extract properties defined in the pattern
                for prop_name in &pattern.properties {
                    if let Some(value) = property_index.get(prop_name) {
                        match value {
                            HppValue::String(s) => {
                                dependencies.insert(s.to_string());
                            }
                            HppValue::Array(arr) => {
                                dependencies.extend(arr.iter().cloned());
                            }
                            HppValue::Class(nested_class) => {
                                // For nested classes, process them with the current path
                                let nested_property_index: HashMap<_, _> = nested_class.properties.iter()
                                    .filter(|p| self.property_names.contains(&p.name))
                                    .map(|p| (&p.name, &p.value))
                                    .collect();
                                self.process_class(nested_class, &class_path, &nested_property_index, dependencies);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Process nested classes in properties
        for prop in &class.properties {
            if let HppValue::Class(nested_class) = &prop.value {
                let nested_property_index: HashMap<_, _> = nested_class.properties.iter()
                    .filter(|p| self.property_names.contains(&p.name))
                    .map(|p| (&p.name, &p.value))
                    .collect();
                self.process_class(nested_class, &class_path, &nested_property_index, dependencies);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HppProperty;

    #[test]
    fn test_simple_extraction() {
        let class = HppClass {
            name: "baseMan".to_string(),
            parent: None,
            properties: vec![
                HppProperty {
                    name: "uniform".to_string(),
                    value: HppValue::Array(vec!["test_uniform".to_string()]),
                },
                HppProperty {
                    name: "vest".to_string(),
                    value: HppValue::Array(vec!["test_vest".to_string()]),
                },
            ],
        };

        let extractor = DependencyExtractor::new(vec![class]);
        let dependencies = extractor.extract_dependencies();

        assert!(dependencies.contains("test_uniform"));
        assert!(dependencies.contains("test_vest"));
    }

    #[test]
    fn test_nested_extraction() {
        let nested_class = HppClass {
            name: "primaryWeapon".to_string(),
            parent: None,
            properties: vec![
                HppProperty {
                    name: "name".to_string(),
                    value: HppValue::String("test_rifle".to_string()),
                },
            ],
        };

        let class = HppClass {
            name: "rifleman".to_string(),
            parent: Some("baseMan".to_string()),
            properties: vec![
                HppProperty {
                    name: "primaryWeapon".to_string(),
                    value: HppValue::Class(nested_class),
                },
            ],
        };

        let extractor = DependencyExtractor::new(vec![class]);
        let dependencies = extractor.extract_dependencies();

        assert!(dependencies.contains("test_rifle"), "Dependencies: {:?}", dependencies);
    }

    #[test]
    fn test_path_matching() {
        let pattern = QueryPattern::new("*/primaryWeapon", &["name"]);
        assert!(pattern.matches_path(&["rifleman".to_string(), "primaryWeapon".to_string()]));
        assert!(pattern.matches_path(&["baseMan".to_string(), "inventory".to_string(), "primaryWeapon".to_string()]));
        assert!(!pattern.matches_path(&["rifleman".to_string()]));
        assert!(!pattern.matches_path(&["primaryWeapon".to_string(), "magazine".to_string()]));
    }
} 