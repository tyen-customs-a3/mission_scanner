use std::collections::HashSet;
use hemtt_sqm::{Class, SqmFile, Value};
use crate::models::{ClassExt, DependencyCollector};

/// Represents a query pattern to search for and extract data from SQM classes
#[derive(Debug, Clone)]
pub struct QueryPattern {
    /// The path to search for (e.g. "Inventory/primaryWeapon")
    path: Vec<String>,
    /// Properties to extract from matching classes
    properties: Vec<String>,
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

/// Extracts class dependencies from an SQM file using predefined patterns
pub struct DependencyExtractor<'a> {
    sqm_file: &'a SqmFile,
    patterns: Vec<QueryPattern>,
}

impl<'a> DependencyExtractor<'a> {
    /// Create a new dependency extractor with default patterns
    pub fn new(sqm_file: &'a SqmFile) -> Self {
        let patterns = vec![
            // Inventory direct properties
            QueryPattern::new("Inventory", &[
                "uniform", "vest", "backpack", "headgear",
                "map", "compass", "watch", "radio", "gps", "goggles"
            ]),
            
            // Primary weapon and magazines
            QueryPattern::new("Inventory/primaryWeapon", &["name", "muzzle"]),
            QueryPattern::new("Inventory/primaryWeapon/primaryMuzzleMag", &["name"]),
            
            // Secondary weapon and magazines
            QueryPattern::new("Inventory/secondaryWeapon", &["name", "muzzle"]),
            QueryPattern::new("Inventory/secondaryWeapon/primaryMuzzleMag", &["name"]),
            
            // Handgun weapon and magazines
            QueryPattern::new("Inventory/handgunWeapon", &["name", "muzzle"]),
            QueryPattern::new("Inventory/handgunWeapon/primaryMuzzleMag", &["name"]),
            
            // Container contents
            QueryPattern::new("Inventory/*/ItemCargo/Item*", &["name"]),
            QueryPattern::new("Inventory/*/MagazineCargo/Item*", &["name"]),
            
            // General object types
            QueryPattern::new("*", &["type"]),
        ];
        
        Self { sqm_file, patterns }
    }

    /// Extract all class dependencies from the SQM file
    pub fn extract_dependencies(&self) -> HashSet<String> {
        let mut collector = DependencyCollector::new();
        
        // Process all Mission classes
        for mission_class in self.get_mission_classes() {
            self.process_class(mission_class, &[], &mut collector);
        }
        
        collector.get_dependencies()
    }
    
    /// Process a class and its children recursively
    fn process_class(&self, class: &Class, current_path: &[String], collector: &mut DependencyCollector) {
        // Build the current class path
        let mut class_path = current_path.to_vec();
        class_path.push(class.name.clone());
        
        // Check each pattern against the current class
        for pattern in &self.patterns {
            if pattern.matches_path(&class_path) {
                // Extract properties defined in the pattern
                for prop_name in &pattern.properties {
                    if let Some(value) = class.get_property_string(prop_name) {
                        collector.add_dependency(value);
                    }
                }
            }
        }
        
        // Process child classes
        for (child_name, child_classes) in &class.classes {
            for child_class in child_classes {
                self.process_class(child_class, &class_path, collector);
            }
        }
    }
    
    /// Get all Mission classes from the SQM file
    fn get_mission_classes(&self) -> Vec<&Class> {
        self.sqm_file.classes.get("Mission")
            .map(|classes| classes.iter().collect())
            .unwrap_or_default()
    }
}