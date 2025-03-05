pub mod types;
mod validator;

pub use types::*;
pub use validator::*;

use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use anyhow::Result;
use log::{info, warn, error, debug};

use crate::mission_scanner::analyzer::types::MissionDependencyResult;
use crate::code_scanner::class::types::ProcessedClass;

/// Validator for class existence in missions
pub struct ClassExistenceValidator {
    /// Processed classes for searching
    processed_classes: Vec<ProcessedClass>,
    /// Flag indicating if the class database has been loaded
    db_loaded: bool,
}

impl ClassExistenceValidator {
    /// Create a new class existence validator
    pub fn new() -> Self {
        Self {
            processed_classes: Vec::new(),
            db_loaded: false,
        }
    }
    
    /// Load class database from memory
    pub fn load_class_database_from_memory(&mut self, processed_classes: &[ProcessedClass]) -> Result<()> {
        validator::load_class_database_from_memory(self, processed_classes)
    }
    
    /// Validate mission classes
    pub fn validate_mission_classes(&self, mission_results: &[MissionDependencyResult]) -> Result<types::ClassExistenceReport> {
        validator::validate_mission_classes(self, mission_results)
    }
    
    /// Check if a class exists
    pub fn class_exists(&self, class_name: &str) -> bool {
        if !self.db_loaded {
            warn!("Class database not loaded");
            return false;
        }
        
        self.processed_classes.iter()
            .any(|c| c.name == class_name)
    }
    
    /// Find similar classes
    pub fn find_similar_classes(&self, class_name: &str) -> Vec<String> {
        validator::find_similar_classes(self, class_name)
    }
} 