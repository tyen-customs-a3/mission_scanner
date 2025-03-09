mod cargo_parsers;
mod random_parsers;

pub use cargo_parsers::*;
pub use random_parsers::*;

use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    character::complete::{char, space0, space1},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, opt},
    error::Error,
    Parser,
};

use crate::sqf::models::ItemAddition;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse an item addition line
/// Examples:
/// - _unit addItemToUniform "ACE_fieldDressing";
/// - player addItem "ACE_morphine";
pub fn parse_sqf_content(input: &str) -> Vec<ItemAddition> {
    let mut items = Vec::new();
    
    // Split the input into lines
    for line in input.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines or comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        // Find all item additions in the line
        if let Some(add_pos) = trimmed.find("add") {
            // Find the string literal after the add command
            let after_add = &trimmed[add_pos..];
            if let Some(quote_start) = after_add.find('"') {
                if let Some(quote_end) = after_add[quote_start+1..].find('"') {
                    let item_name = &after_add[quote_start+1..quote_start+1+quote_end];
                    
                    // Determine container type based on the command
                    let container = if after_add.starts_with("addItemToUniform") {
                        Some("uniform".to_string())
                    } else if after_add.starts_with("addItemToVest") {
                        Some("vest".to_string())
                    } else if after_add.starts_with("addItemToBackpack") {
                        Some("backpack".to_string())
                    } else {
                        None
                    };
                    
                    items.push(ItemAddition {
                        item_name: item_name.to_string(),
                        container,
                        count: None,
                    });
                }
            }
        }
    }
    
    items
}

pub use cargo_parsers::parse_cargo_line;
pub use random_parsers::parse_random_line; 