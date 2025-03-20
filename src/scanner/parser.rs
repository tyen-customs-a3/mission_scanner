// Std imports
use std::fs;
use std::path::Path;

// External crate imports
use anyhow::{Result, anyhow};
use log::{debug, warn};
use parser_hpp::{parse_file as parser_hpp_file, HppValue};
use parser_sqf::{parse_file as parse_sqf_file, ItemKind};
use parser_sqm::extract_class_dependencies;

// Internal crate imports
use crate::types::{ClassReference, ReferenceType};

/// Parse any supported file type and extract class dependencies.
/// 
/// This function will automatically detect the file type based on its extension
/// and use the appropriate parser:
/// - .sqf files: Scanned for equipment references in SQF code
/// - .sqm files: Parsed for inventory classes and addon dependencies
/// - .cpp/.hpp/.ext files: Parsed for loadout configurations
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the file to parse
/// 
/// # Returns
/// 
/// * `Ok(Vec<ClassReference>)` - List of class references found in the file
/// * `Err` - If file reading or parsing fails
/// 
/// Note: Arma 3 class names are case-insensitive, but we preserve the original case
/// in the returned ClassReference objects. When comparing class names later,
/// they should be compared case-insensitively.
pub fn parse_file(file_path: &Path) -> Result<Vec<ClassReference>> {
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow!("File has no extension: {}", file_path.display()))?
        .to_lowercase();

    debug!("Starting to parse file: {} (type: {})", file_path.display(), extension);

    let result = match extension.as_str() {
        "sqf" => parse_sqf(file_path),
        "sqm" => parse_sqm(file_path),
        "cpp" | "hpp" | "ext" => parse_hpp(file_path),
        _ => Err(anyhow!("Unsupported file type: {}", extension))
    };

    match &result {
        Ok(deps) => debug!("Successfully parsed {} with {} dependencies", file_path.display(), deps.len()),
        Err(e) => warn!("Failed to parse {}: {}", file_path.display(), e),
    }

    // Filter out empty class names
    if let Ok(deps) = &result {
        if deps.iter().any(|d| d.class_name.is_empty()) {
            warn!("Found empty class names in file: {}", file_path.display());
        }
    }

    result
}

/// Parse a loadout file and extract equipment information
pub fn parse_hpp(file_path: &Path) -> Result<Vec<ClassReference>> {
    debug!("Starting loadout file parse: {}", file_path.display());
    
    // Parse using parser_hpp
    let classes = parser_hpp_file(file_path)
        .map_err(|e| anyhow!("Failed to parse loadout file: {:?}", e))?;
    
    debug!("Found {} classes in loadout file", classes.len());
    
    let mut dependencies = Vec::new();
    
    // Convert each class and its items to dependencies
    for class in classes {
        debug!("Processing class: {}", class.name);
        
        // Add parent class as inheritance dependency if it exists
        if let Some(parent) = class.parent {
            dependencies.push(ClassReference {
                class_name: parent,
                reference_type: ReferenceType::Inheritance,
                context: format!("loadout:class:{}", file_path.display())
            });
        }
        
        // Add items only from named arrays we care about
        for property in class.properties {
            if let HppValue::Array(items) = property.value {
                // Only process named array properties that typically contain equipment
                let property_name = property.name.to_lowercase();
                if is_equipment_array(&property_name) {
                    debug!("Processing equipment array: {}", property_name);
                    // Process each array item, stripping any extra quotes
                    for item in items {
                        // Skip empty items
                        let clean_item = item.trim().trim_matches('"');
                        if !clean_item.is_empty() {
                            dependencies.push(ClassReference {
                                class_name: clean_item.to_string(),
                                reference_type: ReferenceType::Direct,
                                context: format!("loadout:{}:{}", property_name, file_path.display())
                            });
                        }
                    }
                }
            }
        }
    }
    
    debug!("Total of {} dependencies found in loadout file", dependencies.len());
    Ok(dependencies)
}

/// Determine if a property name is an equipment array we should process
fn is_equipment_array(name: &str) -> bool {
    // List of known equipment array property names in loadout files
    const EQUIPMENT_ARRAYS: [&str; 17] = [
        "uniform", "vest", "backpack", "headgear", "goggles", "hmd",
        "primaryweapon", "secondaryweapon", "handgunweapon", "sidearmweapon",
        "scope", "bipod", "attachment", "silencer", "magazines", "items", "linkeditems",
        // Add any other relevant equipment array names here
    ];
    
    EQUIPMENT_ARRAYS.iter().any(|&array_name| name == array_name)
}

/// Parse a SQM file and extract class references
pub fn parse_sqm(file_path: &Path) -> Result<Vec<ClassReference>> {
    debug!("Starting SQM file parse: {}", file_path.display());
    
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQM file: {}", e))?;
    
    let classes = extract_class_dependencies(&content);
    
    let mut dependencies = Vec::new();
    for class in classes {
        dependencies.push(ClassReference {
            class_name: class,
            reference_type: ReferenceType::Direct,
            context: format!("sqm:{}", file_path.display())
        });
    }
    Ok(dependencies)
}

/// Wrapper around the SQF parser that converts its output to our format
pub fn parse_sqf(file_path: &Path) -> Result<Vec<ClassReference>> {
    debug!("Starting SQF file parse: {}", file_path.display());
    
    // Use the parser_sqf crate's API
    let items = parse_sqf_file(file_path,)
        .map_err(|e| anyhow!("Failed to parse SQF file: {:?}", e))?;
    
    debug!("Found {} items in SQF file", items.len());
    for item in &items {
        debug!("Found SQF item: {} ({:?})", item.class_name, item.kind);
    }
    
    // Convert to our format
    let dependencies: Vec<ClassReference> = items.into_iter()
        .map(|item| ClassReference {
            class_name: item.class_name,
            reference_type: match item.kind {
                ItemKind::Weapon => ReferenceType::Direct,
                ItemKind::Magazine => ReferenceType::Direct,
                ItemKind::Uniform => ReferenceType::Direct,
                ItemKind::Vest => ReferenceType::Direct,
                ItemKind::Backpack => ReferenceType::Direct,
                ItemKind::Item => ReferenceType::Variable,
            },
            context: format!("sqf:{}:{:?}", file_path.display(), item.kind)
        })
        .collect();
    
    debug!("Converted {} SQF items to dependencies", dependencies.len());
    Ok(dependencies)
}