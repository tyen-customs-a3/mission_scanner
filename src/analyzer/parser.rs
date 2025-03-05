use std::path::{Path, PathBuf};
use std::collections::HashSet;
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};

use crate::mission_scanner::extractor::types::MissionExtractionResult;
use crate::mission_scanner::analyzer::types::{ClassDependency, MissionDependencyResult, ReferenceType};
use super::sqf_parser::{SqfClassParser, analyze_sqf_file};
use super::sqm_parser::analyze_sqm_file;
use super::cpp_parser::analyze_cpp_files;

/// Analyze a mission's dependencies
pub fn analyze_mission(
    sqf_parser: &SqfClassParser,
    extraction: &MissionExtractionResult
) -> Result<MissionDependencyResult> {
    info!("Analyzing mission: {}", extraction.mission_name);
    
    let mut class_dependencies = Vec::new();
    
    // Analyze SQM file if present
    if let Some(sqm_file) = &extraction.sqm_file {
        debug!("Analyzing SQM file: {}", sqm_file.display());
        match analyze_sqm_file(sqm_file) {
            Ok(dependencies) => {
                info!("Found {} dependencies in SQM file", dependencies.len());
                class_dependencies.extend(dependencies);
            },
            Err(e) => {
                warn!("Failed to analyze SQM file {}: {}", sqm_file.display(), e);
            }
        }
    }
    
    // Analyze SQF files
    for sqf_file in &extraction.sqf_files {
        debug!("Analyzing SQF file: {}", sqf_file.display());
        match analyze_sqf_file(sqf_parser, sqf_file) {
            Ok(dependencies) => {
                info!("Found {} dependencies in SQF file {}", 
                    dependencies.len(), 
                    sqf_file.display()
                );
                class_dependencies.extend(dependencies);
            },
            Err(e) => {
                warn!("Failed to analyze SQF file {}: {}", sqf_file.display(), e);
            }
        }
    }
    
    // Analyze CPP/HPP files
    if !extraction.cpp_files.is_empty() {
        debug!("Analyzing {} CPP/HPP files", extraction.cpp_files.len());
        match analyze_cpp_files(&extraction.cpp_files) {
            Ok(dependencies) => {
                info!("Found {} dependencies in CPP/HPP files", dependencies.len());
                class_dependencies.extend(dependencies);
            },
            Err(e) => {
                warn!("Failed to analyze CPP/HPP files: {}", e);
            }
        }
    }
    
    // Collect unique class names
    let unique_class_names: HashSet<String> = class_dependencies.iter()
        .map(|dep| dep.class_name.clone())
        .collect();
    
    info!("Found {} dependencies with {} unique classes in mission {}", 
        class_dependencies.len(),
        unique_class_names.len(),
        extraction.mission_name
    );
    
    Ok(MissionDependencyResult {
        mission_name: extraction.mission_name.clone(),
        pbo_path: extraction.pbo_path.clone(),
        class_dependencies,
        unique_class_names,
    })
} 