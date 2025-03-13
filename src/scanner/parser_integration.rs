use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};
use hemtt_workspace::WorkspacePath;

use parser_hpp::{parse_file as parser_hpp_file, HppValue};
use parser_sqm::{parse_sqm as parse_sqm_file, extract_class_dependencies};
use parser_sqf::{parse_file as parse_sqf_file, ItemKind};

use crate::types::ClassDependency;
use crate::types::ReferenceType;

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
/// * `workspace` - Optional workspace path for enhanced parsing configuration
/// 
/// # Returns
/// 
/// * `Ok(Vec<ClassDependency>)` - List of class dependencies found in the file
/// * `Err` - If file reading or parsing fails
pub fn parse_file(file_path: &Path) -> Result<Vec<ClassDependency>> {
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

    result
}

/// Parse a loadout file and extract equipment information
pub fn parse_hpp(file_path: &Path) -> Result<Vec<ClassDependency>> {
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
            dependencies.push(ClassDependency {
                class_name: parent,
                reference_type: ReferenceType::Inheritance,
                context: format!("loadout:class:{}", file_path.display())
            });
        }
        
        // Add each item as a direct dependency
        for property in class.properties {
            if let HppValue::Array(items) = property.value {
                for item in items {
                    dependencies.push(ClassDependency {
                        class_name: item,
                        reference_type: ReferenceType::Direct,
                        context: format!("loadout:item:{}", file_path.display())
                    });
                }
            }
        }
    }
    
    debug!("Total of {} dependencies found in loadout file", dependencies.len());
    Ok(dependencies)
}

/// Parse a SQM file and extract class references
pub fn parse_sqm(file_path: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Starting SQM file parse: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQM file: {}", e))?;
    
    debug!("Read {} bytes from SQM file", content.len());
    
    let mut dependencies = Vec::new();
    
    // First extract addons array
    if let Some(addons_start) = content.find("addons[] = {") {
        let addons_end = content[addons_start..].find("};")
            .map(|end| addons_start + end + 2)
            .unwrap_or(content.len());
        
        let addons_str = &content[addons_start..addons_end];
        let addons_list = addons_str.split('{').nth(1)
            .and_then(|s| s.split('}').next())
            .unwrap_or("");
        
        debug!("Found addons section: {}", addons_str);
        
        for addon in addons_list.split(',') {
            let addon = addon.trim().trim_matches('"');
            if !addon.is_empty() {
                debug!("Found addon dependency: {}", addon);
                dependencies.push(ClassDependency {
                    class_name: addon.to_string(),
                    reference_type: ReferenceType::Direct,
                    context: format!("sqm:addons[]:{}", file_path.display())
                });
            }
        }
    }
    
    // Then parse inventory classes
    match parse_sqm_file(&content) {
        Ok((_, inventory_classes)) => {
            debug!("Successfully parsed {} inventory classes from SQM", inventory_classes.len());
            for ic in &inventory_classes {
                debug!("Found inventory class with parent: {}", ic.parent_class);
                for ref_class in &ic.references {
                    debug!("Found class reference: {}", ref_class.name);
                }
            }
            dependencies.extend(inventory_classes.into_iter()
                .flat_map(|ic| {
                    let mut deps = Vec::new();
                    deps.push(ClassDependency {
                        class_name: ic.parent_class,
                        reference_type: ReferenceType::Inheritance,
                        context: format!("sqm:class:{}", file_path.display())
                    });
                    deps.extend(ic.references.into_iter().map(|ref_class| ClassDependency {
                        class_name: ref_class.name,
                        reference_type: ReferenceType::Direct,
                        context: format!("sqm:inventory:{}", file_path.display())
                    }));
                    deps
                }));
        },
        Err(e) => warn!("Failed to parse inventory classes in SQM file: {:?}", e),
    }
    
    // Also get any additional class dependencies
    let class_deps = extract_class_dependencies(&content);
    debug!("Found {} additional class dependencies", class_deps.len());
    for class_name in &class_deps {
        debug!("Found additional class dependency: {}", class_name);
    }
    dependencies.extend(class_deps.into_iter()
        .map(|class_name| ClassDependency {
            class_name,
            reference_type: ReferenceType::Direct,
            context: format!("sqm:class:{}", file_path.display())
        }));
    
    debug!("Total of {} dependencies found in SQM file", dependencies.len());
    Ok(dependencies)
}

/// Wrapper around the SQF parser that converts its output to our format
pub fn parse_sqf(file_path: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Starting SQF file parse: {}", file_path.display());
    
    // Use the parser_sqf crate's API
    let items = parse_sqf_file(file_path,)
        .map_err(|e| anyhow!("Failed to parse SQF file: {:?}", e))?;
    
    debug!("Found {} items in SQF file", items.len());
    for item in &items {
        debug!("Found SQF item: {} ({:?})", item.class_name, item.kind);
    }
    
    // Convert to our format
    let dependencies: Vec<ClassDependency> = items.into_iter()
        .map(|item| ClassDependency {
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