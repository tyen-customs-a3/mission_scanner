//! HPP Parser for scanning config files
//! 
//! This module provides functionality for parsing HPP files and extracting class information.

use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::collections::HashMap;
use std::io;

use hemtt_workspace::{reporting::{Processed, Output, Token, Symbol}, position::{Position, LineCol}, WorkspacePath, Error as WorkspaceError};
use hemtt_preprocessor::Processor;

mod models;
mod analyzer;

// Re-export public types
pub use models::{ClassKind, ClassReference, ClassContext, ItemKind, ItemReference};

/// Error type for HPP parsing operations
#[derive(Debug)]
pub enum Error {
    /// IO error during file operations
    IoError(io::Error),
    /// Error related to workspace operations
    WorkspaceError(WorkspaceError),
    /// Error during preprocessing
    PreprocessorError(String),
    /// Error parsing the preprocessed syntax
    AnalyzerError(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<WorkspaceError> for Error {
    fn from(err: WorkspaceError) -> Self {
        Error::WorkspaceError(err)
    }
}

/// Represents a class found in an HPP file
#[derive(Debug, Clone)]
pub struct ClassFound {
    /// The class name/ID
    pub class_name: String,
    /// The type of class (class, struct, etc)
    pub kind: ClassKind,
    /// The context where this class was found (scope/conditions)
    pub context: String,
}

/// Parse an HPP file and extract all class definitions.
///
/// # Arguments
/// * `file_path` - Path to the HPP file to parse
/// * `workspace` - Optional workspace path for enhanced preprocessor configuration
///
/// # Returns
/// * `Result<Vec<ClassReference>, Error>` - List of found classes or error
pub fn parse_file(file_path: &Path, workspace: Option<&WorkspacePath>) -> Result<Vec<ClassReference>, Error> {
    let content = fs::read_to_string(file_path)?;
    
    // Create processed context with file info
    let processed = Processed::new(
        vec![Output::Direct(Arc::new(Token::new(
            Symbol::Word(content.to_string()),
            Position::new(
                LineCol(0, (1, 0)),
                LineCol(content.len(), (1, content.len())),
                WorkspacePath::slim(&file_path.to_path_buf())?,
            )
        )))],
        HashMap::new(),
        vec![],
        false,
    )?;

    // Analyze the processed content
    let result = analyzer::analyze_hpp(&processed)
        .map_err(|e| Error::AnalyzerError(e.to_string()))?;

    // Convert ClassContext to ClassReference
    Ok(result.classes.into_iter()
        .map(|ctx| ClassReference {
            class_id: ctx.class_name,
            kind: ClassKind::Class, // Default to Class kind for now
            items: ctx.items,
            parent_class: ctx.parent_class,
        })
        .collect())
}

// Re-export analyzer for convenience
pub use analyzer::analyze_hpp;