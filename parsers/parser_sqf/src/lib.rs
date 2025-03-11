//! SQF Parser for scanning mission files
//! 
//! This module provides functionality to parse SQF files and extract item references
//! from various commands like addBackpack, addWeapon, etc.

use std::path::Path;
use std::fs;
use std::io;

mod models;
mod workspace;
mod scanner;
mod extractors;

// Re-export the public interface
pub use models::{ItemReference, ItemKind};
pub use scanner::scan_sqf;

/// Scans a SQF file for equipment references.
/// 
/// This function reads the specified file and extracts all equipment references from it,
/// such as weapons, backpacks, items, etc.
///
/// # Arguments
///
/// * `file_path` - Path to the SQF file to scan
///
/// # Returns
///
/// * `Ok(Vec<ItemReference>)` - List of found item references if successful
/// * `Err(String)` - Error message if the scan fails
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use parser_sqf::scan_sqf_file;
///
/// let items = scan_sqf_file(Path::new("mission/loadouts/arsenal.sqf"));
/// match items {
///     Ok(found_items) => {
///         for item in found_items {
///             println!("Found item: {} of type {:?}", item.item_id, item.kind);
///         }
///     },
///     Err(e) => eprintln!("Failed to scan file: {}", e),
/// }
/// ```
pub fn scan_sqf_file(file_path: &Path) -> Result<Vec<ItemReference>, String> {
    // Read the file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => format!("File not found: {}", file_path.display()),
            io::ErrorKind::PermissionDenied => format!("Permission denied reading file: {}", file_path.display()),
            _ => format!("Error reading file {}: {}", file_path.display(), e),
        })?;

    // Convert the Path to a PathBuf for the scanner
    let path_buf = file_path.to_path_buf();
    
    // Scan the file contents
    scan_sqf(&content, &path_buf)
} 