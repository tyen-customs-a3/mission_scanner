pub mod types;
mod parser;
mod sqf_parser;
mod sqm_parser;
mod cpp_parser;

pub use types::*;

use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{info, warn, error, debug};
use sqf_parser::SqfClassParser;

use crate::mission_scanner::extractor::types::MissionExtractionResult;

/// Analyzer for mission dependencies
pub struct MissionAnalyzer<'a> {
    /// Directory for caching extraction results
    cache_dir: &'a Path,
    /// Parser for SQF files
    sqf_parser: SqfClassParser,
}

impl<'a> MissionAnalyzer<'a> {
    /// Create a new mission analyzer
    pub fn new(cache_dir: &'a Path) -> Self {
        Self {
            cache_dir,
            sqf_parser: SqfClassParser::new(),
        }
    }
    
    /// Analyze mission dependencies
    pub fn analyze_missions(
        &self,
        extraction_results: &[MissionExtractionResult],
    ) -> Result<Vec<types::MissionDependencyResult>> {
        info!("Analyzing dependencies for {} missions", extraction_results.len());
        
        let mut results = Vec::new();
        
        for extraction in extraction_results {
            match self.analyze_single_mission(extraction) {
                Ok(result) => {
                    info!("Analyzed mission {} with {} dependencies", 
                        result.mission_name, 
                        result.class_dependencies.len()
                    );
                    results.push(result);
                },
                Err(e) => {
                    warn!("Failed to analyze mission {}: {}", extraction.mission_name, e);
                }
            }
        }
        
        info!("Analyzed dependencies for {} missions", results.len());
        
        Ok(results)
    }
    
    /// Analyze a single mission
    fn analyze_single_mission(&self, extraction: &MissionExtractionResult) -> Result<types::MissionDependencyResult> {
        parser::analyze_mission(&self.sqf_parser, extraction)
    }
} 