//! HPP file format parser module
//! 
//! This module provides parsers for Arma 3 HPP (header) files, which are used to define:
//! - Loadouts (equipment configurations)
//! - Medical item properties
//! - Other configuration data

pub mod models;
pub mod parsers;

pub use models::*;
pub use parsers::*;

// Re-export commonly used types
pub use models::{ItemReference, MedicalItemProperties};

// Re-export commonly used functions with more intuitive names
pub use parsers::{
    items_array as parse_items,
    magazines_array as parse_magazines,
    backpack_items_array as parse_backpack_items,
    medical_item_properties as parse_medical_item,
}; 