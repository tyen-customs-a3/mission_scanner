//! SQF Parser for scanning mission files
//! 
//! This module provides functionality to parse SQF files and extract item references
//! from various commands like addBackpack, addWeapon, etc.

mod models;
mod workspace;
mod scanner;
mod extractors;

// Re-export the public interface
pub use models::{ItemReference, ItemKind};
pub use scanner::scan_sqf; 