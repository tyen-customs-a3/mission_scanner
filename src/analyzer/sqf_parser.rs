use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use regex::Regex;
use lazy_static::lazy_static;

use crate::mission_scanner::analyzer::types::{ClassDependency, ReferenceType};

/// Parser for SQF files
pub struct SqfClassParser {
    /// Regular expressions for finding class references
    class_regexes: Vec<(Regex, ReferenceType)>,
}

/// Equipment in SQF file
#[derive(Debug, Clone)]
pub struct Equipment {
    /// Name of the equipment class
    pub class_name: String,
    /// Line number in the source file
    pub line_number: usize,
    /// Context of the equipment
    pub context: String,
}

impl SqfClassParser {
    /// Create a new SQF class parser
    pub fn new() -> Self {
        let mut class_regexes = Vec::new();
        
        // Add regular expressions for finding class references
        class_regexes.push((
            Regex::new(r#"createVehicle\s*\[\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Direct,
        ));
        class_regexes.push((
            Regex::new(r#"createVehicle\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Direct,
        ));
        class_regexes.push((
            Regex::new(r#"typeOf\s*([A-Za-z0-9_]+)\s*==\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Direct,
        ));
        class_regexes.push((
            Regex::new(r#"isKindOf\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Parent,
        ));
        class_regexes.push((
            Regex::new(r#"addWeapon\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addMagazine\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addItem\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addBackpack\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addHeadgear\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addGoggles\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addPrimaryWeaponItem\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addSecondaryWeaponItem\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        class_regexes.push((
            Regex::new(r#"addHandgunItem\s*["']([^"']+)["']"#).unwrap(),
            ReferenceType::Component,
        ));
        
        Self {
            class_regexes,
        }
    }
    
    /// Parse an SQF file for class references
    pub fn parse_file(&self, file_path: &Path) -> Result<Vec<ClassDependency>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();
        
        for (line_number, line) in content.lines().enumerate() {
            for (regex, ref_type) in &self.class_regexes {
                for cap in regex.captures_iter(line) {
                    if let Some(class_name) = cap.get(1) {
                        let class_name = class_name.as_str().to_string();
                        if looks_like_classname(&class_name) {
                            dependencies.push(ClassDependency {
                                class_name,
                                source_file: file_path.to_path_buf(),
                                line_number: line_number + 1,
                                context: format!("SQF: {}", line.trim()),
                                reference_type: ref_type.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Extract equipment from SQF file
    pub fn extract_equipment(&self, file_path: &Path) -> Result<Vec<Equipment>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut equipment = Vec::new();
        
        // Regular expressions for finding equipment
        let weapon_regex = Regex::new(r#"addWeapon\s*["']([^"']+)["']"#).unwrap();
        let magazine_regex = Regex::new(r#"addMagazine\s*["']([^"']+)["']"#).unwrap();
        let item_regex = Regex::new(r#"addItem\s*["']([^"']+)["']"#).unwrap();
        let backpack_regex = Regex::new(r#"addBackpack\s*["']([^"']+)["']"#).unwrap();
        
        for (line_number, line) in content.lines().enumerate() {
            // Check for weapons
            for cap in weapon_regex.captures_iter(line) {
                if let Some(class_name) = cap.get(1) {
                    let class_name = class_name.as_str().to_string();
                    equipment.push(Equipment {
                        class_name,
                        line_number: line_number + 1,
                        context: "Weapon".to_string(),
                    });
                }
            }
            
            // Check for magazines
            for cap in magazine_regex.captures_iter(line) {
                if let Some(class_name) = cap.get(1) {
                    let class_name = class_name.as_str().to_string();
                    equipment.push(Equipment {
                        class_name,
                        line_number: line_number + 1,
                        context: "Magazine".to_string(),
                    });
                }
            }
            
            // Check for items
            for cap in item_regex.captures_iter(line) {
                if let Some(class_name) = cap.get(1) {
                    let class_name = class_name.as_str().to_string();
                    equipment.push(Equipment {
                        class_name,
                        line_number: line_number + 1,
                        context: "Item".to_string(),
                    });
                }
            }
            
            // Check for backpacks
            for cap in backpack_regex.captures_iter(line) {
                if let Some(class_name) = cap.get(1) {
                    let class_name = class_name.as_str().to_string();
                    equipment.push(Equipment {
                        class_name,
                        line_number: line_number + 1,
                        context: "Backpack".to_string(),
                    });
                }
            }
        }
        
        Ok(equipment)
    }
}

/// Analyze an SQF file for class dependencies
pub fn analyze_sqf_file(sqf_parser: &SqfClassParser, sqf_file: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Analyzing SQF file: {}", sqf_file.display());
    
    // Parse the file for class references
    let mut dependencies = sqf_parser.parse_file(sqf_file)?;
    
    // Extract equipment and add as dependencies
    let equipment = sqf_parser.extract_equipment(sqf_file)?;
    extract_dependencies_from_sqf_equipment(&equipment, sqf_file, &mut dependencies);
    
    Ok(dependencies)
}

/// Extract dependencies from SQF equipment
pub fn extract_dependencies_from_sqf_equipment(
    equipment: &[Equipment],
    file_path: &Path,
    dependencies: &mut Vec<ClassDependency>
) {
    for item in equipment {
        if looks_like_classname(&item.class_name) {
            dependencies.push(ClassDependency {
                class_name: item.class_name.clone(),
                source_file: file_path.to_path_buf(),
                line_number: item.line_number,
                context: format!("Equipment: {}", item.context),
                reference_type: ReferenceType::Component,
            });
        }
    }
}

/// Check if a string looks like a class name
pub fn looks_like_classname(name: &str) -> bool {
    lazy_static! {
        static ref CLASSNAME_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
    }
    
    // Check if the name matches the class name pattern
    if !CLASSNAME_REGEX.is_match(name) {
        return false;
    }
    
    // Check if the name is a common variable name
    let common_vars = ["this", "true", "false", "nil", "player", "vehicle", "group"];
    if common_vars.contains(&name) {
        return false;
    }
    
    true
} 