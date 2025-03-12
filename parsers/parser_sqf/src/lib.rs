//! SQF Parser for scanning mission files
//! 
//! This module provides functionality to parse SQF files and extract item references
//! from various commands like addBackpack, addWeapon, etc.

// Declare modules
mod models;
mod analyzer;

use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;
use std::io;

use hemtt_sqf::{Statements, StringWrapper};
use hemtt_sqf::parser::{run as parse_sqf, database::Database, ParserError};
use hemtt_sqf::parser::lexer::{strip_comments, strip_noop};
use hemtt_workspace::{reporting::{Processed, Code, Definition, Output, Token, Symbol}, position::{Position, LineCol}, WorkspacePath, Workspace, LayerType, Error as WorkspaceError};
use hemtt_common::config::PDriveOption;
use hemtt_sqf::Error as SqfError;
use hemtt_preprocessor::Processor;
use crate::analyzer::Analyzer;
use crate::models::{ItemReference, ItemContext};

// Re-export public types
pub use models::ItemKind;

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

/// Represents a reference to an item found in SQF code
#[derive(Debug, Clone)]
pub struct ItemFound {
    /// The item's class name/ID
    pub class_name: String,
    /// The type of item (weapon, magazine, etc)
    pub kind: ItemKind,
    /// The context where this item was found (scope/conditions)
    pub context: String,
}

/// Parse an SQF file and extract all item references.
///
/// # Arguments
/// * `file_path` - Path to the SQF file to parse
/// * `workspace` - Optional workspace path for enhanced database configuration
///
/// # Returns
/// * `Result<Vec<ItemFound>, Error>` - List of found items or error
pub fn parse_file(file_path: &Path, workspace: Option<&WorkspacePath>) -> Result<Vec<ItemFound>, Error> {
    let content = fs::read_to_string(file_path)?;
    
    // Create database with workspace if available
    let database = if let Some(workspace) = workspace {
        Database::a3_with_workspace(workspace, false)?
    } else {
        Database::a3(false)
    };

    // Create processed context with file info
    let processed = Processed::new(
        vec![Output::Direct(Arc::new(Token::new(
            Symbol::Word(content.to_string()),
            Position::new(
                LineCol(0, (1, 0)),
                LineCol(content.len(), (1, content.len())),
                workspace.cloned().unwrap_or_else(|| WorkspacePath::slim(&file_path.to_path_buf()).unwrap()),
            )
        )))],
        HashMap::new(),
        vec![],
        false,
    )?;

    // Parse and analyze
    let statements = parse_sqf(&database, &processed)
        .map_err(Error::ParserError)?;

    let result = analyzer::analyze_sqf(&statements)
        .map_err(|e| Error::UnparseableSyntax(e))?;
    
    // Convert internal types to public API types
    let items = result.items.into_iter()
        .map(|item| ItemFound {
            class_name: item.item.item_id,
            kind: item.item.kind,
            context: item.scope,
        })
        .collect();
    
    Ok(items)
}


// Re-export analyzer for convenience
pub use analyzer::analyze_sqf;