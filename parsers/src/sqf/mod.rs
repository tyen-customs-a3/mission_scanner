//! SQF script parser module
//! 
//! This module provides parsers for Arma 3 SQF script files, which handle:
//! - Inventory management (adding/removing items)
//! - Vehicle cargo operations
//! - Random number generation
//! - Other script operations

pub mod models;
pub mod parsers;

pub use models::*;
pub use parsers::*;

// Re-export commonly used types
pub use models::{ItemAddition, CargoOperation, RandomRange};

// Re-export commonly used functions with more intuitive names
pub use parsers::{
    parse_sqf_content as parse_inventory_changes,
    parse_cargo_line as parse_cargo_operation,
    parse_random_line as parse_random_range,
}; 