use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};

use parser_code::parse_loadout;
use parser_sqf::scan_equipment_references;
use parser_sqm::parse_sqm;
use parser_sqm::extract_class_dependencies;

/// Parse a loadout file and extract equipment information
pub fn parse_loadout_file(file_path: &Path) -> Result<Vec<parser_code::Equipment>> {
    debug!("Parsing loadout file: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read loadout file: {}", e))?;
    
    // Parse the loadout
    match parse_loadout(&content) {
        Ok((_, equipment)) => {
            debug!("Parsed {} equipment items from loadout file", equipment.len());
            Ok(equipment)
        },
        Err(e) => Err(anyhow!("Failed to parse loadout file: {:?}", e)),
    }
}

/// Parse a SQM file and extract class references
pub fn parse_sqm_file(file_path: &Path) -> Result<Vec<parser_sqm::InventoryClass>> {
    debug!("Parsing SQM file: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQM file: {}", e))?;
    
    // Parse the SQM
    match parse_sqm(&content) {
        Ok((_, inventory_classes)) => {
            debug!("Parsed {} inventory classes from SQM file", inventory_classes.len());
            Ok(inventory_classes)
        },
        Err(e) => Err(anyhow!("Failed to parse SQM file: {:?}", e)),
    }
}

/// Extract class dependencies from a SQM file
pub fn extract_sqm_dependencies(file_path: &Path) -> Result<std::collections::HashSet<String>> {
    debug!("Extracting class dependencies from SQM file: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQM file: {}", e))?;
    
    // Extract dependencies
    let dependencies = extract_class_dependencies(&content);
    debug!("Extracted {} class dependencies from SQM file", dependencies.len());
    
    Ok(dependencies)
}

/// Scan a SQF file for equipment references
pub fn scan_sqf_file(file_path: &Path) -> Result<std::collections::HashSet<parser_sqf::EquipmentReference>> {
    debug!("Scanning SQF file for equipment references: {}", file_path.display());
    
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read SQF file: {}", e))?;
    
    // Scan for equipment references
    let references = scan_equipment_references(&content);
    debug!("Found {} equipment references in SQF file", references.len());
    
    Ok(references)
} 