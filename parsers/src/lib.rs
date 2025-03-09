//! Mission Scanner Parser Library
//! 
//! This library provides parsers for various Arma 3 file formats:
//! - HPP (Header files) - For loadouts and medical item configurations
//! - SQF (Script files) - For inventory management and vehicle cargo operations
//! - SQM (Mission files) - For mission configuration and layout
//! 
//! # Examples
//! 
//! ```rust
//! use mission_scanner_parsers::hpp::parse_items;
//! use mission_scanner_parsers::sqf::parse_inventory_changes;
//! 
//! // Parse a loadout from HPP
//! let hpp_content = r#"
//!     {
//!         "ACRE_PRC343",
//!         LIST_10("ACE_fieldDressing")
//!     }
//! "#;
//! let items = parse_items(hpp_content).unwrap();
//! assert_eq!(items[0].item_name, "ACRE_PRC343");
//! 
//! // Parse inventory changes from SQF
//! let sqf_content = r#"
//!     _unit addItem "ACE_fieldDressing";
//!     _unit addItemToVest "ACE_morphine";
//! "#;
//! let changes = parse_inventory_changes(sqf_content);
//! ```
//! 
//! # Feature Flags
//! 
//! - `serde` - Enables serialization/deserialization support (enabled by default)

pub mod hpp;
pub mod sqf;

// Re-export commonly used types
pub use hpp::models::{ItemReference, MedicalItemProperties};
pub use sqf::models::{ItemAddition, CargoOperation, RandomRange};

// Re-export main parsing functions with intuitive names
pub use hpp::parsers::{
    items_array as parse_items,
    magazines_array as parse_magazines,
    backpack_items_array as parse_backpack_items,
    medical_item_properties as parse_medical_item,
};

pub use sqf::parsers::{
    parse_sqf_content as parse_inventory_changes,
    parse_cargo_line as parse_cargo_operation,
    parse_random_line as parse_random_range,
};

/// Error type for parsing operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse input: {0}")]
    ParseError(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

/// Result type for parsing operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience function to parse a complete loadout
pub fn parse_loadout(input: &str) -> Result<Vec<ItemReference>> {
    hpp::parsers::items_array(input)
        .map(|(_, items)| items)
        .map_err(|e| Error::ParseError(e.to_string()))
} 