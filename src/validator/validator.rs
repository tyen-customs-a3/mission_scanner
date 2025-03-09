use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};

use parser_code::Equipment;
use super::ClassExistenceValidator;
use super::types::{ClassExistenceReport, MissionClassExistenceReport, MissingClassInfo, MissionDependencyResult, ClassDependency};

/// Load class database from memory
pub fn load_class_database_from_memory(
    validator: &mut ClassExistenceValidator,
    processed_classes: &[Equipment]
) -> Result<()> {
    info!("Loading class database from memory with {} classes", processed_classes.len());
    
    validator.processed_classes = processed_classes.to_vec();
    validator.db_loaded = true;
    
    info!("Loaded class database with {} classes", validator.processed_classes.len());
    
    Ok(())
}

/// Validate mission classes against the loaded database
/// Will return a report of missing classes
pub fn validate_mission_classes(
    validator: &ClassExistenceValidator,
    mission_results: &[MissionDependencyResult]
) -> Result<ClassExistenceReport> {
    info!("Validating mission classes");

    // Ensure class database is loaded
    if !validator.db_loaded {
        return Err(anyhow::anyhow!("Class database not loaded"));
    }

    // Track missing classes across all missions
    let mut all_missing_classes = HashMap::new();
    let mut all_class_count = 0;

    // Reports for individual missions
    let mut mission_reports = Vec::new();

    // Process each mission
    for mission_result in mission_results {
        // Track missing classes for this mission
        let mut missing_classes = HashMap::new();
        
        // Track class statistics
        let total_classes = mission_result.class_dependencies.len();
        let mut existing_classes = 0;
        
        // Check each class dependency
        for dependency in &mission_result.class_dependencies {
            all_class_count += 1;
            
            // Check if class exists
            if validator.class_exists(&dependency.class_name) {
                existing_classes += 1;
            } else {
                // Add to missing classes for this mission
                let entry = missing_classes
                    .entry(dependency.class_name.clone())
                    .or_insert_with(|| MissingClassInfo {
                        class_name: dependency.class_name.clone(),
                        reference_count: 0,
                        reference_locations: Vec::new(),
                        suggested_alternatives: Vec::new(),
                    });
                
                entry.reference_count += 1;
                entry.reference_locations.push(format!("{} ({:?})", dependency.context, dependency.reference_type));
                
                // Add to all missing classes
                let entry = all_missing_classes
                    .entry(dependency.class_name.clone())
                    .or_insert_with(|| MissingClassInfo {
                        class_name: dependency.class_name.clone(),
                        reference_count: 0,
                        reference_locations: Vec::new(),
                        suggested_alternatives: Vec::new(),
                    });
                
                entry.reference_count += 1;
                entry.reference_locations.push(
                    format!("{} in {} ({:?})", 
                            dependency.context, 
                            mission_result.mission_name,
                            dependency.reference_type)
                );
            }
        }
        
        // Add suggested alternatives for missing classes
        for (class_name, missing_info) in missing_classes.iter_mut() {
            missing_info.suggested_alternatives = validator.find_similar_classes(class_name);
        }
        
        // Calculate existence percentage
        let existence_percentage = if total_classes > 0 {
            (existing_classes as f64 / total_classes as f64) * 100.0
        } else {
            100.0 // If no classes are referenced, all classes exist
        };
        
        // Create mission report
        let mission_report = MissionClassExistenceReport {
            mission_name: mission_result.mission_name.clone(),
            total_classes,
            existing_classes,
            missing_classes: total_classes - existing_classes,
            existence_percentage,
            missing_class_list: missing_classes.into_values().collect(),
        };
        
        mission_reports.push(mission_report);
    }
    
    // Add suggested alternatives for all missing classes
    for (class_name, missing_info) in all_missing_classes.iter_mut() {
        missing_info.suggested_alternatives = validator.find_similar_classes(class_name);
    }
    
    // Calculate overall existence percentage
    let total_unique_classes = all_missing_classes.len() + validator.processed_classes.len();
    let existing_classes = validator.processed_classes.len();
    let missing_classes = all_missing_classes.len();
    
    let existence_percentage = if total_unique_classes > 0 {
        (existing_classes as f64 / total_unique_classes as f64) * 100.0
    } else {
        100.0 // If no classes are referenced, all classes exist
    };
    
    // Create overall report
    let report = ClassExistenceReport {
        total_missions: mission_results.len(),
        total_unique_classes,
        existing_classes,
        missing_classes,
        existence_percentage,
        mission_reports,
    };
    
    Ok(report)
}

/// Find similar classes to the given class name
pub fn find_similar_classes(
    validator: &ClassExistenceValidator,
    class_name: &str
) -> Vec<String> {
    if !validator.db_loaded {
        return Vec::new();
    }
    
    // Find classes with similar names (using Levenshtein distance)
    let mut similar_classes = Vec::new();
    
    for class in &validator.processed_classes {
        // Skip exact matches
        if class.class_name == class_name {
            continue;
        }
        
        // Calculate Levenshtein distance
        let distance = levenshtein_distance(&class_name.to_lowercase(), &class.class_name.to_lowercase());
        
        // Consider similar if distance is less than 25% of the class name length
        let threshold = (class_name.len() as f64 * 0.25).ceil() as usize;
        
        if distance <= threshold {
            similar_classes.push(class.class_name.clone());
        }
    }
    
    // Sort by Levenshtein distance (closest first)
    similar_classes.sort_by_key(|name| levenshtein_distance(&class_name.to_lowercase(), &name.to_lowercase()));
    
    // Limit to 5 suggestions
    similar_classes.truncate(5);
    
    similar_classes
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    let m = s1_chars.len();
    let n = s2_chars.len();
    
    // Handle empty strings
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    
    // Create distance matrix
    let mut matrix = vec![vec![0; n + 1]; m + 1];
    
    // Initialize first row and column
    for i in 0..=m {
        matrix[i][0] = i;
    }
    for j in 0..=n {
        matrix[0][j] = j;
    }
    
    // Fill the matrix
    for i in 1..=m {
        for j in 1..=n {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1       // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }
    
    matrix[m][n]
} 