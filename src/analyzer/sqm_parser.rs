use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use cpp_parser::{Class, Value};

use crate::mission_scanner::analyzer::types::{ClassDependency, ReferenceType};
use super::sqf_parser::looks_like_classname;

/// SQM class
#[derive(Debug, Clone)]
pub struct SqmClass {
    /// Name of the class
    pub name: String,
    /// Parent class
    pub parent: Option<String>,
    /// Properties of the class
    pub properties: std::collections::HashMap<String, Value>,
    /// Child classes
    pub children: Vec<SqmClass>,
}

/// Analyze an SQM file for class dependencies
pub fn analyze_sqm_file(sqm_file: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Analyzing SQM file: {}", sqm_file.display());
    
    // Read the file
    let content = std::fs::read_to_string(sqm_file)?;
    
    // Parse the file
    let classes = match cpp_parser::parse(&content) {
        Ok(classes) => classes,
        Err(e) => {
            warn!("Failed to parse SQM file {}: {}", sqm_file.display(), e);
            return Ok(Vec::new());
        }
    };
    
    // Extract dependencies
    let mut dependencies = Vec::new();
    extract_dependencies_from_sqm_classes(&classes, sqm_file, &mut dependencies);
    
    Ok(dependencies)
}

/// Extract dependencies from SQM classes
pub fn extract_dependencies_from_sqm_classes(
    classes: &[Class],
    file_path: &Path,
    dependencies: &mut Vec<ClassDependency>
) {
    for class in classes {
        // Check if the class has a parent
        if let Some(parent) = &class.parent {
            if looks_like_classname(parent) {
                dependencies.push(ClassDependency {
                    class_name: parent.clone(),
                    source_file: file_path.to_path_buf(),
                    line_number: class.line_number,
                    context: format!("SQM Parent: {} inherits from {}", class.name, parent),
                    reference_type: ReferenceType::Parent,
                });
            }
        }
        
        // Check properties for class references
        for (prop_name, value) in &class.properties {
            match value {
                Value::String(val) => {
                    if looks_like_classname(val) {
                        dependencies.push(ClassDependency {
                            class_name: val.clone(),
                            source_file: file_path.to_path_buf(),
                            line_number: class.line_number,
                            context: format!("SQM Property: {} in {}", prop_name, class.name),
                            reference_type: ReferenceType::Component,
                        });
                    }
                },
                Value::Array(vals) => {
                    for (i, val) in vals.iter().enumerate() {
                        if let Value::String(val) = val {
                            if looks_like_classname(val) {
                                dependencies.push(ClassDependency {
                                    class_name: val.clone(),
                                    source_file: file_path.to_path_buf(),
                                    line_number: class.line_number,
                                    context: format!("SQM Array: {}[{}] in {}", prop_name, i, class.name),
                                    reference_type: ReferenceType::Component,
                                });
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        // Process child classes
        extract_dependencies_from_sqm_classes(&class.children, file_path, dependencies);
    }
} 