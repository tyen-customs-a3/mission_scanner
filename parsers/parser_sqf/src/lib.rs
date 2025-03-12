//! SQF Parser for scanning mission files
//! 
//! This module provides functionality to parse SQF files and extract item references
//! from various commands like addBackpack, addWeapon, etc.

use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::collections::HashMap;
use hemtt_sqf::{Statements, StringWrapper};
use hemtt_sqf::parser::{run as parse_sqf, database::Database, ParserError};
use hemtt_sqf::parser::lexer::{strip_comments, strip_noop};
use hemtt_workspace::{reporting::{Processed, Code, Definition, Output, Token, Symbol}, position::{Position, LineCol}, WorkspacePath, Workspace, LayerType, Error as WorkspaceError};
use hemtt_common::config::PDriveOption;
use hemtt_sqf::Error as SqfError;
use hemtt_preprocessor::Processor;
use crate::analyzer::Analyzer;

pub mod models;
pub mod analyzer;

// Re-export the public interface
pub use crate::models::{ItemReference, ItemKind, ItemContext, AnalysisResult};
pub use crate::analyzer::analyze_sqf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parser error: {0}")]
    ParserError(#[from] ParserError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("SQF error: {0}")]
    SqfError(#[from] SqfError),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Unparseable syntax: {0}")]
    UnparseableSyntax(String),
    #[error("Preprocessor error: {0}")]
    PreprocessorError(String),
}

impl Error {
    pub fn codes(&self) -> Vec<Arc<dyn Code>> {
        match self {
            Error::ParserError(e) => e.codes().to_vec(),
            _ => Vec::new()
        }
    }
}

/// Parse and analyze SQF content to extract item references.
/// This is the main entry point for the parser.
///
/// # Arguments
/// * `content` - The SQF code to parse
/// * `workspace` - Optional workspace path for enhanced database configuration
/// * `file_path` - Optional file path for error reporting and context
///
/// # Returns
/// * `Result<Vec<ItemContext>, Error>` - List of found items or error
pub fn parse_sqf_content(
    content: &str, 
    workspace: Option<&WorkspacePath>,
    file_path: Option<&Path>,
) -> Result<Vec<ItemContext>, Error> {
    // Create database with workspace if available
    let database = if let Some(workspace) = workspace {
        Database::a3_with_workspace(workspace, false)?
    } else {
        Database::a3(false)
    };

    // Create processed context with file info
    let file_path = file_path.map(|p| p.to_path_buf()).unwrap_or_else(|| Path::new("test.sqf").to_path_buf());
    let workspace_path = if let Some(ws) = workspace {
        ws.clone()
    } else {
        WorkspacePath::slim(&file_path)?
    };

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

    let result = analyze_sqf(&statements)
        .map_err(|e| Error::UnparseableSyntax(e))?;
    
    Ok(result.items)
}

/// Convenience function to scan a file for item references
///
/// # Arguments
/// * `path` - Path to the SQF file to scan
/// * `workspace` - Optional workspace path for enhanced database configuration
///
/// # Returns
/// * `Result<Vec<ItemContext>, Error>` - List of found items or error
pub fn scan_file(path: &Path, workspace: Option<&WorkspacePath>) -> Result<Vec<ItemContext>, Error> {
    let content = fs::read_to_string(path)?;
    parse_sqf_content(&content, workspace, Some(path))
}