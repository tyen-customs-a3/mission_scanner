use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    character::complete::{char, space0, space1, multispace0},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, opt, value},
    Parser,
};
use crate::parsers::sqf::models::ItemAddition;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse a for loop header
/// Example: for "_i" from 1 to 4 do {
pub fn for_loop_header(input: &str) -> IResult<&str, ()> {
    // Match the entire for loop header pattern more flexibly
    let (input, _) = tag("for").parse(input)?;
    let (input, _) = space1.parse(input)?;
    
    // Skip the variable name in quotes
    let (input, _) = char('"').parse(input)?;
    let (input, _) = take_until("\"").parse(input)?;
    let (input, _) = char('"').parse(input)?;
    
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("from").parse(input)?;
    
    // Skip everything until "do"
    let (input, _) = take_until("do").parse(input)?;
    let (input, _) = tag("do").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('{').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    Ok((input, ()))
}

/// Parse an item addition line
/// Examples:
/// - _unit addItemToUniform "ACE_fieldDressing";
/// - player addItem "ACE_morphine";
pub fn item_addition(input: &str) -> IResult<&str, ItemAddition> {
    let (input, _) = multispace0.parse(input)?;
    
    // Skip any "for" loop constructs
    let (input, _) = opt(for_loop_header).parse(input)?;
    
    // Parse the unit reference and function name
    let (input, _) = take_until(" ").parse(input)?;
    let (input, _) = space1.parse(input)?;
    
    // Parse the function name and determine container type
    let (input, container) = alt((
        value(Some("uniform".to_string()), tag("addItemToUniform")),
        value(Some("vest".to_string()), tag("addItemToVest")),
        value(Some("backpack".to_string()), tag("addItemToBackpack")),
        value(None, tag("addItem")),
        value(None, tag("addMagazine")),
        value(None, tag("addWeapon"))
    )).parse(input)?;
    
    // Parse the item name
    let (input, _) = space1.parse(input)?;
    let (input, _) = char('"').parse(input)?;
    let (input, item_name) = take_until("\"").parse(input)?;
    let (input, _) = char('"').parse(input)?;
    
    // Parse optional semicolon and for loop closing brace
    let (input, _) = opt(char(';')).parse(input)?;
    let (input, _) = opt(preceded(
        multispace0,
        char('}')
    )).parse(input)?;
    
    Ok((input, ItemAddition {
        item_name: item_name.to_string(),
        container,
        count: None,
    }))
}

/// Parse a line that might contain an item addition
/// Returns None if the line doesn't match a pattern we're interested in
pub fn parse_item_line(input: &str) -> IResult<&str, Option<ItemAddition>> {
    let trimmed = input.trim();
    
    // Skip empty lines or comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return Ok((input, None));
    }
    
    // Check for item addition patterns
    if trimmed.contains("addItem") || 
       trimmed.contains("addMagazine") || 
       trimmed.contains("addWeapon") {
        
        // Find the function name, skipping "player" prefix if present
        let mut parts = trimmed.split_whitespace();
        let function = if let Some(first) = parts.next() {
            if first == "player" {
                parts.next().unwrap_or("")
            } else {
                first
            }
        } else {
            ""
        };
        
        // Find the string literal
        if let Some(quote_start) = trimmed.find('"') {
            if let Ok((rest, item)) = string_literal(&trimmed[quote_start..]) {
                let container = if function.contains("ToUniform") {
                    Some("uniform".to_string())
                } else if function.contains("ToVest") {
                    Some("vest".to_string())
                } else if function.contains("ToBackpack") {
                    Some("backpack".to_string())
                } else {
                    None
                };
                
                return Ok((rest, Some(ItemAddition {
                    item_name: item.to_string(),
                    container,
                    count: None,
                })));
            }
        }
    }
    
    // No item addition found
    Ok((input, None))
}

/// Parse SQF content and extract all item additions
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_literal() {
        let input = r#""ACE_fieldDressing""#;
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(result, "ACE_fieldDressing");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_parse_item_addition() {
        let input = r#"_unit addItemToUniform "ACE_fieldDressing";"#;
        let items = parse_sqf_content(input);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
    }

    #[test]
    fn test_parse_for_loop_item_addition() {
        // This is a specific test for testing the for loop parser
        // Do not change this test input
        let input = r#"for "_i" from 1 to (ceil (random [0,1,4])) do {_unit addItemToUniform "ACE_fieldDressing"};"#;
        let items = parse_sqf_content(input);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
    }

    #[test]
    fn test_parse_sqf_content() {
        let input = r#"
        _unit addItemToUniform "ACE_fieldDressing";
        _unit addItemToVest "ACE_packingBandage";
        _unit addMagazine "rhs_mag_rgd5";
        "#;
        
        let items = parse_sqf_content(input);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
        assert_eq!(items[1].item_name, "ACE_packingBandage");
        assert_eq!(items[1].container, Some("vest".to_string()));
        assert_eq!(items[2].item_name, "rhs_mag_rgd5");
        assert_eq!(items[2].container, None);
    }
}
