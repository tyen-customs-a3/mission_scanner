use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};

use parser_code::parse_loadout;
use parser_sqm::parse_sqm;
use parser_sqm::extract_class_dependencies;
use parser_sqf::{parse_file as parse_sqf_file, ItemKind};

use crate::types::ClassDependency;
use crate::types::ReferenceType;

/// Parse any supported file type and extract class dependencies.
/// 
/// This function will automatically detect the file type based on its extension
/// and use the appropriate parser:
/// - .sqf files: Scanned for equipment references in SQF code
/// - .sqm files: Parsed for inventory classes and addon dependencies
/// - .cpp/.hpp files: Parsed for loadout configurations
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the file to parse
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

    debug!("Parsing file: {} (type: {})", file_path.display(), extension);

    match extension.as_str() {
        "sqf" => parse_sqf_file_wrapper(file_path),
        "sqm" => parse_sqm_file(file_path),
        "cpp" | "hpp" => parse_loadout_file(file_path),
        _ => Err(anyhow!("Unsupported file type: {}", extension))
    }
}

/// Parse a loadout file and extract equipment information
pub fn parse_loadout_file(file_path: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Parsing loadout file: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read loadout file: {}", e))?;
    
    // Parse the loadout
    match parse_loadout(&content) {
        Ok((_, equipment)) => {
            debug!("Parsed {} equipment items from loadout file", equipment.len());
            let items = equipment.into_iter()
                .map(|e| ClassDependency {
                    class_name: e.class_name,
                    reference_type: ReferenceType::Direct,
                    context: format!("loadout:{}", file_path.display())
                })
                .collect();
            Ok(items)
        },
        Err(e) => Err(anyhow!("Failed to parse loadout file: {:?}", e)),
    }
}

/// Parse a SQM file and extract class references
pub fn parse_sqm_file(file_path: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Parsing SQM file: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQM file: {}", e))?;
    
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
        
        for addon in addons_list.split(',') {
            let addon = addon.trim().trim_matches('"');
            if !addon.is_empty() {
                dependencies.push(ClassDependency {
                    class_name: addon.to_string(),
                    reference_type: ReferenceType::Direct,
                    context: format!("sqm:addons[]:{}", file_path.display())
                });
            }
        }
    }
    
    // Then parse inventory classes
    match parse_sqm(&content) {
        Ok((_, inventory_classes)) => {
            debug!("Parsed {} inventory classes from SQM file", inventory_classes.len());
            dependencies.extend(inventory_classes.into_iter()
                .flat_map(|ic| {
                    let mut deps = Vec::new();
                    // Add parent class as inheritance reference
                    deps.push(ClassDependency {
                        class_name: ic.parent_class,
                        reference_type: ReferenceType::Inheritance,
                        context: format!("sqm:class:{}", file_path.display())
                    });
                    // Add all direct references
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
    dependencies.extend(class_deps.into_iter()
        .map(|class_name| ClassDependency {
            class_name,
            reference_type: ReferenceType::Direct,
            context: format!("sqm:class:{}", file_path.display())
        }));
    
    debug!("Extracted {} total class dependencies from SQM file", dependencies.len());
    Ok(dependencies)
}

/// Wrapper around the SQF parser that converts its output to our format
fn parse_sqf_file_wrapper(file_path: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Scanning SQF file for equipment references: {}", file_path.display());
    
    // Use the parser_sqf crate's API
    let items = parse_sqf_file(file_path, None)
        .map_err(|e| anyhow!("Failed to parse SQF file: {:?}", e))?;
    
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
    
    debug!("Found {} item references in SQF file", dependencies.len());
    Ok(dependencies)
} 