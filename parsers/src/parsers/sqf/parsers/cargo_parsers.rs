use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, space0, space1},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, opt},
    Parser,
};
use crate::parsers::sqf::models::CargoOperation;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse a variable name
pub fn variable_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('_').parse(input)?;
    let (input, name) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;
    Ok((input, name))
}

/// Parse a clear cargo operation
/// Example: clearWeaponCargoGlobal _hmmvw;
pub fn clear_cargo(input: &str) -> IResult<&str, CargoOperation> {
    let (input, _) = tag("clear").parse(input)?;
    let (input, cargo_type) = alt((
        map(tag("WeaponCargoGlobal"), |_| "weapon"),
        map(tag("MagazineCargoGlobal"), |_| "magazine"),
        map(tag("ItemCargoGlobal"), |_| "item"),
        map(tag("BackpackCargoGlobal"), |_| "backpack"),
    )).parse(input)?;
    
    let (input, _) = space1.parse(input)?;
    let (input, vehicle) = variable_name(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, CargoOperation::Clear {
        vehicle: vehicle.to_string(),
        cargo_type: cargo_type.to_string(),
    }))
}

/// Parse an ACE cargo load operation
/// Example: ["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;
pub fn ace_cargo_load(input: &str) -> IResult<&str, CargoOperation> {
    let (input, _) = char('[').parse(input)?;
    let (input, item) = preceded(space0, string_literal).parse(input)?;
    let (input, _) = preceded(space0, char(',')).parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, vehicle) = variable_name(input)?;
    let (input, _) = preceded(space0, char(']')).parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("call").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, function) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, CargoOperation::Load {
        item: item.to_string(),
        vehicle: vehicle.to_string(),
        function: function.to_string(),
    }))
}

/// Parse a line that might contain a cargo operation
pub fn parse_cargo_line(input: &str) -> Option<CargoOperation> {
    let trimmed = input.trim();
    
    // Skip empty lines or comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }
    
    // Try to parse clear cargo operations
    if trimmed.starts_with("clear") {
        if let Ok((_, operation)) = clear_cargo(trimmed) {
            return Some(operation);
        }
    }
    
    // Try to parse ACE cargo load operations
    if trimmed.starts_with("[") && trimmed.contains("ace_cargo_fnc_loadItem") {
        if let Ok((_, operation)) = ace_cargo_load(trimmed) {
            return Some(operation);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_cargo() {
        let input = "clearWeaponCargoGlobal _hmmvw;";
        let (_, operation) = clear_cargo(input).unwrap();
        match operation {
            CargoOperation::Clear { vehicle, cargo_type } => {
                assert_eq!(vehicle, "hmmvw");
                assert_eq!(cargo_type, "weapon");
            },
            _ => panic!("Expected Clear operation"),
        }
    }

    #[test]
    fn test_ace_cargo_load() {
        let input = r#"["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;"#;
        let (_, operation) = ace_cargo_load(input).unwrap();
        match operation {
            CargoOperation::Load { item, vehicle, function } => {
                assert_eq!(item, "Land_CanisterFuel_Red_F");
                assert_eq!(vehicle, "hmmvw");
                assert_eq!(function, "ace_cargo_fnc_loadItem");
            },
            _ => panic!("Expected Load operation"),
        }
    }

    #[test]
    fn test_parse_cargo_line() {
        let input = "clearWeaponCargoGlobal _hmmvw;";
        let operation = parse_cargo_line(input).unwrap();
        match operation {
            CargoOperation::Clear { vehicle, cargo_type } => {
                assert_eq!(vehicle, "hmmvw");
                assert_eq!(cargo_type, "weapon");
            },
            _ => panic!("Expected Clear operation"),
        }

        let input = r#"["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;"#;
        let operation = parse_cargo_line(input).unwrap();
        match operation {
            CargoOperation::Load { item, vehicle, function } => {
                assert_eq!(item, "Land_CanisterFuel_Red_F");
                assert_eq!(vehicle, "hmmvw");
                assert_eq!(function, "ace_cargo_fnc_loadItem");
            },
            _ => panic!("Expected Load operation"),
        }
    }
} 