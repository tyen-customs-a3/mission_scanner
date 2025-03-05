use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};

use crate::mission_scanner::analyzer::types::MissionDependencyResult;
use crate::code_scanner::class::types::ProcessedClass;
use super::ClassExistenceValidator;
use super::types::{ClassExistenceReport, MissionClassExistenceReport, MissingClassInfo};

/// Load class database from memory
pub fn load_class_database_from_memory(
    validator: &mut ClassExistenceValidator,
    processed_classes: &[ProcessedClass]
) -> Result<()> {
    info!("Loading class database from memory with {} classes", processed_classes.len());
    
    validator.processed_classes = processed_classes.to_vec();
    validator.db_loaded = true;
    
    info!("Loaded class database with {} classes", validator.processed_classes.len());
    
    Ok(())
}

/// Validate mission classes
pub fn validate_mission_classes(
    validator: &ClassExistenceValidator,
    mission_results: &[MissionDependencyResult]
) -> Result<ClassExistenceReport> {
    if !validator.db_loaded {
        return Err(anyhow!("Class database not loaded"));
    }
    
    info!("Validating classes for {} missions", mission_results.len());
    
    let mut mission_reports = Vec::new();
    let mut all_classes = HashSet::new();
    let mut existing_classes = HashSet::new();
    let mut missing_classes = HashSet::new();
    
    for mission in mission_results {
        let mission_name = &mission.mission_name;
        info!("Validating classes for mission: {}", mission_name);
        
        let mut mission_existing_classes = HashSet::new();
        let mut mission_missing_classes = HashMap::new();
        
        // Check each class dependency
        for dep in &mission.class_dependencies {
            let class_name = &dep.class_name;
            all_classes.insert(class_name.clone());
            
            if validator.class_exists(class_name) {
                mission_existing_classes.insert(class_name.clone());
                existing_classes.insert(class_name.clone());
            } else {
                missing_classes.insert(class_name.clone());
                
                // Add to mission missing classes
                let entry = mission_missing_classes.entry(class_name.clone())
                    .or_insert_with(|| {
                        MissingClassInfo {
                            class_name: class_name.clone(),
                            reference_count: 0,
                            reference_locations: Vec::new(),
                            suggested_alternatives: Vec::new(),
                        }
                    });
                
                entry.reference_count += 1;
                entry.reference_locations.push(format!(
                    "{}:{} ({})",
                    dep.source_file.display(),
                    dep.line_number,
                    dep.context
                ));
            }
        }
        
        // Find similar classes for missing classes
        for missing_class in mission_missing_classes.values_mut() {
            missing_class.suggested_alternatives = validator.find_similar_classes(&missing_class.class_name);
        }
        
        // Create mission report
        let total_classes = mission.unique_class_names.len();
        let existing_count = mission_existing_classes.len();
        let missing_count = mission_missing_classes.len();
        let existence_percentage = if total_classes > 0 {
            (existing_count as f64 / total_classes as f64) * 100.0
        } else {
            100.0
        };
        
        let mission_report = MissionClassExistenceReport {
            mission_name: mission_name.clone(),
            total_classes,
            existing_classes: existing_count,
            missing_classes: missing_count,
            existence_percentage,
            missing_class_list: mission_missing_classes.into_values().collect(),
        };
        
        info!("Mission {} has {}% class existence ({}/{} classes exist)",
            mission_name,
            existence_percentage,
            existing_count,
            total_classes
        );
        
        mission_reports.push(mission_report);
    }
    
    // Create overall report
    let total_unique_classes = all_classes.len();
    let existing_count = existing_classes.len();
    let missing_count = missing_classes.len();
    let existence_percentage = if total_unique_classes > 0 {
        (existing_count as f64 / total_unique_classes as f64) * 100.0
    } else {
        100.0
    };
    
    let report = ClassExistenceReport {
        total_missions: mission_results.len(),
        total_unique_classes,
        existing_classes: existing_count,
        missing_classes: missing_count,
        existence_percentage,
        mission_reports,
    };
    
    info!("Overall class existence: {}% ({}/{} classes exist)",
        existence_percentage,
        existing_count,
        total_unique_classes
    );
    
    Ok(report)
}

/// Find similar classes
pub fn find_similar_classes(
    validator: &ClassExistenceValidator,
    class_name: &str
) -> Vec<String> {
    if !validator.db_loaded {
        warn!("Class database not loaded");
        return Vec::new();
    }
    
    let mut similar_classes = Vec::new();
    let mut similarities = Vec::new();
    
    // Find classes with similar names
    for class in &validator.processed_classes {
        let distance = levenshtein_distance(class_name, &class.name);
        let max_len = std::cmp::max(class_name.len(), class.name.len());
        let similarity = if max_len > 0 {
            1.0 - (distance as f64 / max_len as f64)
        } else {
            0.0
        };
        
        if similarity > 0.7 {
            similarities.push((class.name.clone(), similarity));
        }
    }
    
    // Sort by similarity (descending)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Take top 5 similar classes
    for (class_name, _) in similarities.into_iter().take(5) {
        similar_classes.push(class_name);
    }
    
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