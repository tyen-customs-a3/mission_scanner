//! SQF Parser for scanning mission files
//! 
//! This module provides functionality to parse SQF files and extract class references
//! from mission scripts based on how they are used in functions.

// Declare modules
mod models;
mod evaluator;
mod array_handler;

use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::collections::HashMap;
use std::io;
use hemtt_sqf::parser::{run as parse_sqf, database::Database, ParserError};
use hemtt_sqf::Error as SqfError;

use hemtt_workspace::{reporting::{Processed, Output, Token, Symbol}, position::{Position, LineCol}, WorkspacePath, Error as WorkspaceError};

// Export our public types
pub use models::{ClassReference, UsageContext};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParserError(ParserError),
    WorkspaceError(WorkspaceError),
    UnparseableSyntax(String),
    SqfError(SqfError),
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

impl From<SqfError> for Error {
    fn from(err: SqfError) -> Self {
        Error::SqfError(err)
    }
}

/// Parse an SQF file and extract all class references by analyzing function usage.
///
/// # Arguments
/// * `file_path` - Path to the SQF file to parse
///
/// # Returns
/// * `Result<Vec<ClassReference>, Error>` - List of found class references or error
pub fn parse_file(file_path: &Path) -> Result<Vec<ClassReference>, Error> {
    // First do a quick scan with buffered reading
    let file = fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    
    if !evaluator::Evaluator::should_evaluate(reader) {
        return Ok(Vec::new());
    }
    
    // If we found a match, now read the whole file for full parsing
    let content = fs::read_to_string(file_path)?;
    
    // Create a workspace path for the file
    let workspace_path = WorkspacePath::slim_file(file_path)?;
    
    // Create database with workspace
    let database = Database::a3_with_workspace(&workspace_path, false)?;

    // Create processed context with file info
    let processed = Processed::new(
        vec![Output::Direct(Arc::new(Token::new(
            Symbol::Word(content.to_string()),
            Position::new(
                LineCol(0, (1, 0)),
                LineCol(content.len(), (1, content.len())),
                workspace_path,
            )
        )))],
        HashMap::new(),
        vec![],
        false,
    )?;

    // Parse and analyze
    let statements = parse_sqf(&database, &processed)
        .map_err(Error::ParserError)?;

    // Use the evaluator to extract class references
    evaluator::evaluate_sqf(&statements)
        .map_err(|e| Error::UnparseableSyntax(e))
        .map(|result| result.references)
}

// Re-export evaluator for convenience
pub use evaluator::evaluate_sqf;