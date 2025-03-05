use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use cpp_parser::{Class, Value};

use crate::mission_scanner::analyzer::types::{ClassDependency, ReferenceType};
use super::sqf_parser::looks_like_classname;

/// Analyze CPP files for class dependencies
pub fn analyze_cpp_files(cpp_files: &[PathBuf]) -> Result<Vec<ClassDependency>> {
    let mut dependencies = Vec::new();
    
    for cpp_file in cpp_files {
        match analyze_single_cpp_file(cpp_file) {
            Ok(file_deps) => {
                info!("Found {} dependencies in CPP file {}", 
                    file_deps.len(), 
                    cpp_file.display()
                );
                dependencies.extend(file_deps);
            },
            Err(e) => {
                warn!("Failed to analyze CPP file {}: {}", cpp_file.display(), e);
            }
        }
    }
    
    Ok(dependencies)
}

/// Analyze a single CPP file for class dependencies
pub fn analyze_single_cpp_file(cpp_file: &Path) -> Result<Vec<ClassDependency>> {
    debug!("Analyzing CPP file: {}", cpp_file.display());
    
    // Read the file
    let content = std::fs::read_to_string(cpp_file)?;
    
    // Parse the file
    let classes = match cpp_parser::parse(&content) {
        Ok(classes) => classes,
        Err(e) => {
            warn!("Failed to parse CPP file {}: {}", cpp_file.display(), e);
            return Ok(Vec::new());
        }
    };
    
    // Extract dependencies
    let mut dependencies = Vec::new();
    process_parsed_classes(&classes, cpp_file, &mut dependencies, None);
    
    Ok(dependencies)
}

/// Process parsed classes to extract dependencies
fn process_parsed_classes(
    classes: &[Class],
    file_path: &Path,
    dependencies: &mut Vec<ClassDependency>,
    parent_context: Option<&str>
) {
    for class in classes {
        // Add the class itself as a definition
        dependencies.push(ClassDependency {
            class_name: class.name.clone(),
            source_file: file_path.to_path_buf(),
            line_number: class.line_number,
            context: format!("CPP Definition: {}", class.name),
            reference_type: ReferenceType::Definition,
        });
        
        // Check if the class has a parent
        if let Some(parent) = &class.parent {
            if looks_like_classname(parent) {
                dependencies.push(ClassDependency {
                    class_name: parent.clone(),
                    source_file: file_path.to_path_buf(),
                    line_number: class.line_number,
                    context: format!("CPP Parent: {} inherits from {}", class.name, parent),
                    reference_type: ReferenceType::Parent,
                });
            }
        }
        
        // Check properties for class references
        process_properties(
            &class.properties,
            &class.name,
            file_path,
            dependencies,
            class.line_number
        );
        
        // Process child classes
        let context = if let Some(parent) = parent_context {
            format!("{}::{}", parent, class.name)
        } else {
            class.name.clone()
        };
        
        process_parsed_classes(
            &class.children,
            file_path,
            dependencies,
            Some(&context)
        );
    }
}

/// Process properties to extract dependencies
fn process_properties(
    properties: &std::collections::HashMap<String, Value>,
    class_name: &str,
    file_path: &Path,
    dependencies: &mut Vec<ClassDependency>,
    line_number: usize
) {
    for (prop_name, value) in properties {
        match value {
            Value::String(val) => {
                if !val.is_empty() && looks_like_classname(val) {
                    dependencies.push(ClassDependency {
                        class_name: val.clone(),
                        source_file: file_path.to_path_buf(),
                        line_number,
                        context: format!("CPP Property: {} in {}", prop_name, class_name),
                        reference_type: ReferenceType::Component,
                    });
                }
            },
            Value::Array(vals) => {
                for (i, val) in vals.iter().enumerate() {
                    match val {
                        Value::String(val) => {
                            if !val.is_empty() && looks_like_classname(val) {
                                dependencies.push(ClassDependency {
                                    class_name: val.clone(),
                                    source_file: file_path.to_path_buf(),
                                    line_number,
                                    context: format!("CPP Array: {}[{}] in {}", prop_name, i, class_name),
                                    reference_type: ReferenceType::Component,
                                });
                            }
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
} 